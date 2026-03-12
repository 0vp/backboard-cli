use crate::agent::events::{AgentEvent, EventKind, EventSink};
use crate::agent::prompts::PromptStore;
use crate::agent::session::AgentSession;
use crate::backboard::client::BackboardClient;
use crate::backboard::models::{
    AddMessageRequest, CreateAssistantRequest, MessageResponse, ToolCall, STATUS_CANCELLED,
    STATUS_COMPLETED, STATUS_FAILED, STATUS_REQUIRES_ACTION,
};
use crate::config::Config;
use crate::runtime::logging::SessionLogger;
use crate::runtime::todos::TodoStore;
use crate::tools::registry::{ExecutionContext, ToolExecution, ToolRegistry};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use futures::stream::{FuturesUnordered, StreamExt};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

pub struct AgentRunner {
    client: BackboardClient,
    config: Config,
    prompts: PromptStore,
    tools: ToolRegistry,
    todos: TodoStore,
    logger: Arc<SessionLogger>,
    sink: Arc<dyn EventSink>,
    session: Arc<Mutex<Option<AgentSession>>>,
    assistant_id: Arc<Mutex<Option<String>>>,
    model_selection: Arc<Mutex<ModelSelection>>,
}

#[derive(Clone, Debug)]
struct ModelSelection {
    provider: String,
    model: String,
}

impl AgentRunner {
    pub fn new(
        client: BackboardClient,
        config: Config,
        prompts: PromptStore,
        tools: ToolRegistry,
        todos: TodoStore,
        logger: Arc<SessionLogger>,
        sink: Arc<dyn EventSink>,
    ) -> Self {
        let default_provider = config.llm_provider.clone();
        let default_model = config.model_name.clone();
        Self {
            client,
            config,
            prompts,
            tools,
            todos,
            logger,
            sink,
            session: Arc::new(Mutex::new(None)),
            assistant_id: Arc::new(Mutex::new(None)),
            model_selection: Arc::new(Mutex::new(ModelSelection {
                provider: default_provider,
                model: default_model,
            })),
        }
    }

    pub async fn run_prompt(&self, run_id: &str, prompt: &str) -> Result<String> {
        self.log(
            "run_prompt_start",
            json!({
                "run_id": run_id,
                "prompt": prompt,
                "prompt_chars": prompt.chars().count(),
            }),
        );
        let mut recovery_attempts = 0_usize;
        loop {
            match self.run_prompt_once(run_id, prompt).await {
                Ok(summary) => {
                    self.log(
                        "run_prompt_completed",
                        json!({
                            "run_id": run_id,
                            "summary": &summary,
                            "recovery_attempts": recovery_attempts,
                        }),
                    );
                    return Ok(summary);
                }
                Err(err) if recovery_attempts < 3 && is_missing_tool_result_error(&err) => {
                    recovery_attempts += 1;
                    *self.session.lock().expect("session mutex poisoned") = None;
                    if recovery_attempts >= 2 {
                        *self
                            .assistant_id
                            .lock()
                            .expect("assistant_id mutex poisoned") = None;
                    }
                    self.emit(
                        EventKind::Status,
                        "detected tool-call mismatch; retrying prompt on a fresh session"
                            .to_string(),
                        Some(json!({
                            "run_id": run_id,
                            "recovery": "thread_reset",
                            "attempt": recovery_attempts,
                            "reset_assistant": recovery_attempts >= 2,
                        })),
                    );
                    self.log_error("run_prompt_recoverable_error", &err);
                    continue;
                }
                Err(err) => {
                    self.log_error("run_prompt_failed", &err);
                    return Err(err);
                }
            }
        }
    }

    async fn run_prompt_once(&self, run_id: &str, prompt: &str) -> Result<String> {
        let session = self.ensure_session().await?;
        let (provider, model) = self.current_model();
        self.log(
            "run_prompt_session",
            json!({
                "run_id": run_id,
                "assistant_id": &session.assistant_id,
                "thread_id": &session.thread_id,
                "provider": &provider,
                "model": &model,
            }),
        );
        self.emit(
            EventKind::Status,
            format!(
                "run={} assistant={} thread={} model={}/{}",
                run_id, session.assistant_id, session.thread_id, provider, model
            ),
            Some(json!({ "run_id": run_id })),
        );

        let add_message_request = AddMessageRequest {
            thread_id: session.thread_id.clone(),
            content: prompt.to_string(),
            llm_provider: provider,
            model_name: model,
            memory: self.config.memory_mode.clone(),
            web_search: self.config.web_search_mode.clone(),
            stream: false,
            send_to_llm: "true".to_string(),
        };

        self.log(
            "add_message_request",
            json!({
                "thread_id": &add_message_request.thread_id,
                "content": &add_message_request.content,
                "llm_provider": &add_message_request.llm_provider,
                "model_name": &add_message_request.model_name,
                "memory": &add_message_request.memory,
                "web_search": &add_message_request.web_search,
                "stream": add_message_request.stream,
                "send_to_llm": &add_message_request.send_to_llm,
            }),
        );

        let mut response = match self.add_message_with_retry(add_message_request).await {
            Ok(value) => value,
            Err(err) => {
                self.log_error("add_message_failed", &err);
                return Err(err);
            }
        };
        self.log("add_message_response", message_response_snapshot(&response));

        let mut finish_summary: Option<String> = None;
        for iteration in 1..=self.config.max_iterations {
            let status = normalize_status(response.status.as_deref());
            let content = response.content.clone().unwrap_or_default();
            self.emit(
                EventKind::Status,
                status_message(iteration, &status, &content),
                Some(json!({ "iteration": iteration, "status": status })),
            );

            match status.as_str() {
                STATUS_REQUIRES_ACTION => {
                    let calls = response.tool_calls.clone().unwrap_or_default();
                    if calls.is_empty() {
                        return Err(anyhow!("REQUIRES_ACTION without tool calls"));
                    }

                    self.log(
                        "requires_action_tool_calls",
                        json!({
                            "run_id": &response.run_id,
                            "status": &response.status,
                            "tool_call_count": calls.len(),
                            "tool_call_ids": calls.iter().map(|call| call.id.as_str()).collect::<Vec<_>>(),
                        }),
                    );

                    let outputs = self.execute_tools(calls, &mut finish_summary).await?;
                    let run_id = response
                        .run_id
                        .clone()
                        .context("missing run_id in REQUIRES_ACTION response")?;
                    self.log(
                        "submit_tool_outputs_request",
                        json!({
                            "thread_id": &session.thread_id,
                            "run_id": &run_id,
                            "tool_output_count": outputs.len(),
                            "tool_outputs": outputs_snapshot(&outputs),
                        }),
                    );

                    response = match self
                        .submit_tool_outputs_with_retry(&session.thread_id, &run_id, outputs)
                        .await
                    {
                        Ok(value) => value,
                        Err(err) => {
                            self.log_error("submit_tool_outputs_failed", &err);
                            return Err(err);
                        }
                    };
                    self.log(
                        "submit_tool_outputs_response",
                        message_response_snapshot(&response),
                    );
                }
                STATUS_COMPLETED => {
                    return Ok(first_non_empty(
                        finish_summary.clone(),
                        response.content.clone(),
                        response.message.clone(),
                    ));
                }
                STATUS_FAILED | STATUS_CANCELLED => {
                    self.log(
                        "run_failed_status",
                        json!({
                            "status": &status,
                            "content": &response.content,
                            "message": &response.message,
                            "run_id": &response.run_id,
                        }),
                    );
                    if finish_summary.is_some() {
                        return Ok(first_non_empty(
                            finish_summary.clone(),
                            response.content.clone(),
                            response.message.clone(),
                        ));
                    }
                    return Err(anyhow!(
                        "agent run failed with status={} message={}",
                        status,
                        response.content.unwrap_or_default()
                    ));
                }
                _ => {
                    if status.is_empty()
                        && response.tool_calls.as_ref().is_none_or(|v| v.is_empty())
                        && !content.trim().is_empty()
                    {
                        return Ok(first_non_empty(
                            finish_summary.clone(),
                            Some(content),
                            response.message.clone(),
                        ));
                    }
                    sleep(Duration::from_millis(200)).await;
                }
            }
        }

        Err(anyhow!(
            "agent exceeded max iterations ({})",
            self.config.max_iterations
        ))
    }

    async fn execute_tools(
        &self,
        calls: Vec<ToolCall>,
        finish_summary: &mut Option<String>,
    ) -> Result<Vec<crate::backboard::models::ToolOutput>> {
        let mut tasks = FuturesUnordered::new();
        let total = calls.len();
        let mut expected_call_ids: Vec<String> = Vec::with_capacity(total);

        for (index, call) in calls.into_iter().enumerate() {
            self.tools.ensure_allowed(&call.function.name)?;
            let arguments = tool_arguments_preview(&call);
            expected_call_ids.push(call.id.clone());
            self.emit(
                EventKind::ToolQueued,
                format!(
                    "queued tool {}/{}: {}",
                    index + 1,
                    total,
                    call.function.name
                ),
                Some(json!({
                    "tool": call.function.name,
                    "tool_call_id": call.id,
                    "tool_index": index + 1,
                    "tool_total": total,
                    "arguments": arguments,
                    "state": "queued"
                })),
            );

            let call_clone = call.clone();
            let ctx = ExecutionContext {
                workspace_root: self.config.workspace_root.clone(),
                command_timeout: self.config.command_timeout,
                jina_api_key: self.config.jina_api_key.clone(),
                execute_allowlist: self.config.execute_allowlist.clone(),
                todo_store: self.todos.clone(),
                event_sink: self.sink.clone(),
            };
            let tools = self.tools.clone();
            let sink = self.sink.clone();

            tasks.push(async move {
                sink.emit(AgentEvent {
                    kind: EventKind::ToolRunning,
                    message: format!("running tool: {}", call_clone.function.name),
                    timestamp: Utc::now(),
                    metadata: Some(json!({
                        "tool": call_clone.function.name,
                        "tool_call_id": call_clone.id,
                        "arguments": tool_arguments_preview(&call_clone),
                        "state": "running"
                    })),
                });
                let result = tools.execute(&call_clone, ctx).await;
                (index, call_clone, result)
            });
        }

        let mut gathered: Vec<(usize, ToolExecution)> = Vec::new();
        while let Some((idx, call, mut outcome)) = tasks.next().await {
            let raw_output = outcome.output.output.clone();
            self.log(
                "tool_execution_result_raw",
                json!({
                    "tool": &call.function.name,
                    "tool_call_id": &call.id,
                    "raw_output": raw_output,
                }),
            );
            outcome.output.output = compact_tool_output(&outcome.output.output, 8_000);
            let ok = parse_ok(&outcome.output.output);
            self.emit(
                EventKind::ToolResult,
                format!(
                    "tool {} {}",
                    call.function.name,
                    if ok { "ok" } else { "error" }
                ),
                Some(json!({
                    "tool": call.function.name,
                    "tool_call_id": call.id,
                    "ok": ok,
                    "arguments": tool_arguments_preview(&call),
                    "error_code": extract_error_code(&outcome.output.output),
                    "output": outcome.output.output,
                })),
            );
            if outcome.is_finish {
                *finish_summary = outcome.finish_summary.clone();
                self.emit(
                    EventKind::Finished,
                    "finish tool received".to_string(),
                    None,
                );
            }
            gathered.push((idx, outcome));
        }

        let mut outputs: Vec<Option<crate::backboard::models::ToolOutput>> = vec![None; total];
        for (idx, item) in gathered {
            outputs[idx] = Some(item.output);
        }

        Ok(outputs
            .into_iter()
            .enumerate()
            .map(|(idx, item)| {
                item.unwrap_or_else(|| crate::backboard::models::ToolOutput {
                    tool_call_id: expected_call_ids
                        .get(idx)
                        .cloned()
                        .unwrap_or_else(|| format!("missing-{idx}")),
                    output: json!({
                        "ok": false,
                        "error": "tool execution missing output for tool call"
                    })
                    .to_string(),
                })
            })
            .collect())
    }

    async fn ensure_session(&self) -> Result<AgentSession> {
        if let Some(existing) = self.session.lock().expect("session mutex poisoned").clone() {
            return Ok(existing);
        }

        let cached_assistant_id = {
            self.assistant_id
                .lock()
                .expect("assistant_id mutex poisoned")
                .clone()
        };

        let assistant_id = if let Some(existing) = cached_assistant_id {
            existing
        } else {
            let assistant = self.create_assistant_with_retry().await?;
            let assistant_id = assistant.assistant_id;
            *self
                .assistant_id
                .lock()
                .expect("assistant_id mutex poisoned") = Some(assistant_id.clone());
            assistant_id
        };

        let thread = self
            .create_thread_with_retry(&assistant_id)
            .await
            .context("failed to create thread")?;

        let session = AgentSession {
            assistant_id,
            thread_id: thread.thread_id,
        };
        *self.session.lock().expect("session mutex poisoned") = Some(session.clone());
        Ok(session)
    }

    async fn create_assistant_with_retry(&self) -> Result<crate::backboard::models::Assistant> {
        retry_async(3, || {
            self.client.create_assistant(CreateAssistantRequest {
                name: "backboard-coding-agent".to_string(),
                system_prompt: Some(self.prompts.coder_prompt().to_string()),
                tools: self.tools.definitions(),
            })
        })
        .await
        .context("failed to create assistant")
    }

    async fn create_thread_with_retry(
        &self,
        assistant_id: &str,
    ) -> Result<crate::backboard::models::Thread> {
        let assistant_id = assistant_id.to_string();
        retry_async(3, || self.client.create_thread(&assistant_id))
            .await
            .context("failed to create thread")
    }

    async fn add_message_with_retry(&self, request: AddMessageRequest) -> Result<MessageResponse> {
        retry_async(3, || self.client.add_message(request.clone())).await
    }

    async fn submit_tool_outputs_with_retry(
        &self,
        thread_id: &str,
        run_id: &str,
        outputs: Vec<crate::backboard::models::ToolOutput>,
    ) -> Result<MessageResponse> {
        let thread_id = thread_id.to_string();
        let run_id = run_id.to_string();
        retry_async(3, || {
            self.client
                .submit_tool_outputs(&thread_id, &run_id, outputs.clone())
        })
        .await
    }

    fn emit(
        &self,
        kind: EventKind,
        message: impl Into<String>,
        metadata: Option<serde_json::Value>,
    ) {
        self.sink.emit(AgentEvent {
            kind,
            message: message.into(),
            timestamp: Utc::now(),
            metadata,
        });
    }

    fn log(&self, event: &str, payload: Value) {
        self.logger.log(event, payload);
    }

    fn log_error(&self, event: &str, err: &anyhow::Error) {
        self.logger.log_error(event, err);
    }

    pub fn clear_session(&self) {
        *self.session.lock().expect("session mutex poisoned") = None;
        self.emit(
            EventKind::Status,
            "conversation cleared; next prompt starts a new thread on the same assistant"
                .to_string(),
            None,
        );
    }

    pub fn set_model(&self, provider: impl Into<String>, model: impl Into<String>) {
        let mut guard = self
            .model_selection
            .lock()
            .expect("model_selection mutex poisoned");
        guard.provider = provider.into();
        guard.model = model.into();
    }

    pub fn current_model(&self) -> (String, String) {
        let guard = self
            .model_selection
            .lock()
            .expect("model_selection mutex poisoned");
        (guard.provider.clone(), guard.model.clone())
    }
}

fn normalize_status(value: Option<&str>) -> String {
    value.unwrap_or_default().trim().to_uppercase()
}

fn parse_ok(output: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(output) else {
        return false;
    };

    if let Some(ok) = value.get("ok").and_then(Value::as_bool) {
        return ok;
    }

    value.get("error").is_none()
}

fn first_non_empty(a: Option<String>, b: Option<String>, c: Option<String>) -> String {
    [a, b, c]
        .into_iter()
        .flatten()
        .find(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "No summary provided".to_string())
}

fn status_message(iteration: usize, status: &str, content: &str) -> String {
    match status {
        STATUS_REQUIRES_ACTION | STATUS_COMPLETED | STATUS_FAILED | STATUS_CANCELLED => {
            format!("iteration {iteration}: status={status}")
        }
        _ => {
            if content.trim().is_empty() {
                format!("iteration {iteration}: status={status}")
            } else {
                format!(
                    "iteration {iteration}: status={status} {}",
                    truncate_for_status(content, 180)
                )
            }
        }
    }
}

fn truncate_for_status(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let prefix: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{prefix}...")
    } else {
        prefix
    }
}

fn tool_arguments_preview(call: &ToolCall) -> String {
    match call.arguments_map() {
        Ok(Value::Object(map)) if map.is_empty() => "{}".to_string(),
        Ok(value) => truncate_for_status(&value.to_string(), 260),
        Err(_) => call
            .function
            .arguments
            .as_deref()
            .map(|raw| truncate_for_status(raw, 260))
            .unwrap_or_else(|| "{}".to_string()),
    }
}

fn compact_tool_output(raw: &str, max_chars: usize) -> String {
    if raw.chars().count() <= max_chars {
        return raw.to_string();
    }

    let preview_len = max_chars.saturating_sub(220).max(200);
    let preview: String = raw.chars().take(preview_len).collect();
    let parsed_ok = parse_ok(raw);

    json!({
        "ok": parsed_ok,
        "truncated": true,
        "total_chars": raw.chars().count(),
        "preview": format!("{}...", preview),
    })
    .to_string()
}

fn extract_error_code(output: &str) -> Option<u16> {
    let parsed = serde_json::from_str::<Value>(output).ok()?;
    if parsed.get("ok").and_then(Value::as_bool).unwrap_or(true) {
        return None;
    }

    if let Some(code) = parsed
        .pointer("/status_code")
        .and_then(Value::as_u64)
        .and_then(|v| u16::try_from(v).ok())
    {
        return Some(code);
    }

    if let Some(code) = parsed
        .pointer("/result/status_code")
        .and_then(Value::as_u64)
        .and_then(|v| u16::try_from(v).ok())
    {
        return Some(code);
    }

    parsed
        .get("error")
        .and_then(Value::as_str)
        .and_then(parse_status_code_from_text)
}

fn outputs_snapshot(outputs: &[crate::backboard::models::ToolOutput]) -> Value {
    Value::Array(
        outputs
            .iter()
            .map(|item| {
                json!({
                    "tool_call_id": &item.tool_call_id,
                    "output": &item.output,
                })
            })
            .collect(),
    )
}

fn message_response_snapshot(response: &MessageResponse) -> Value {
    let tool_calls = response.tool_calls.as_ref().map(|calls| {
        calls
            .iter()
            .map(|call| {
                json!({
                    "id": &call.id,
                    "function": {
                        "name": &call.function.name,
                        "arguments": &call.function.arguments,
                        "parsed_arguments": &call.function.parsed_arguments,
                    }
                })
            })
            .collect::<Vec<_>>()
    });

    json!({
        "status": &response.status,
        "run_id": &response.run_id,
        "message": &response.message,
        "content": &response.content,
        "tool_calls": tool_calls,
    })
}

fn parse_status_code_from_text(input: &str) -> Option<u16> {
    let bytes = input.as_bytes();
    for window in bytes.windows(3) {
        if window.iter().all(u8::is_ascii_digit) && (window[0] == b'4' || window[0] == b'5') {
            if let Ok(raw) = std::str::from_utf8(window) {
                if let Ok(code) = raw.parse::<u16>() {
                    return Some(code);
                }
            }
        }
    }
    None
}

async fn retry_async<F, Fut, T>(attempts: usize, mut f: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_error: Option<anyhow::Error> = None;
    for attempt in 1..=attempts {
        match f().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                if !is_transient(&err) || attempt == attempts {
                    return Err(err);
                }
                last_error = Some(err);
                sleep(Duration::from_millis(700 * attempt as u64)).await;
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow!("retry failed")))
}

fn is_transient(err: &anyhow::Error) -> bool {
    let value = err.to_string().to_lowercase();
    [
        "429",
        "500",
        "502",
        "503",
        "504",
        "timeout",
        "temporarily",
        "connection reset",
        "broken pipe",
        "eof",
    ]
    .iter()
    .any(|marker| value.contains(marker))
}

fn is_missing_tool_result_error(err: &anyhow::Error) -> bool {
    let value = err.to_string().to_lowercase();
    value.contains("tool_use")
        && value.contains("tool_result")
        && value.contains("immediately after")
}
