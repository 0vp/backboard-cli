use crate::agent::events::{AgentEvent, EventKind, EventSink};
use crate::agent::prompts::PromptStore;
use crate::agent::session::AgentSession;
use crate::backboard::client::BackboardClient;
use crate::backboard::models::{
    AddMessageRequest, CreateAssistantRequest, MessageResponse, ToolCall, STATUS_CANCELLED,
    STATUS_COMPLETED, STATUS_FAILED, STATUS_REQUIRES_ACTION,
};
use crate::config::Config;
use crate::runtime::todos::TodoStore;
use crate::tools::registry::{ExecutionContext, ToolExecution, ToolRegistry};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use futures::stream::{FuturesUnordered, StreamExt};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

pub struct AgentRunner {
    client: BackboardClient,
    config: Config,
    prompts: PromptStore,
    tools: ToolRegistry,
    todos: TodoStore,
    sink: Arc<dyn EventSink>,
    session: Arc<Mutex<Option<AgentSession>>>,
}

impl AgentRunner {
    pub fn new(
        client: BackboardClient,
        config: Config,
        prompts: PromptStore,
        tools: ToolRegistry,
        todos: TodoStore,
        sink: Arc<dyn EventSink>,
    ) -> Self {
        Self {
            client,
            config,
            prompts,
            tools,
            todos,
            sink,
            session: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn run_prompt(&self, run_id: &str, prompt: &str) -> Result<String> {
        let session = self.ensure_session().await?;
        self.emit(
            EventKind::Status,
            format!(
                "run={} assistant={} thread={}",
                run_id, session.assistant_id, session.thread_id
            ),
            Some(json!({ "run_id": run_id })),
        );

        let mut response = self
            .add_message_with_retry(AddMessageRequest {
                thread_id: session.thread_id.clone(),
                content: prompt.to_string(),
                llm_provider: self.config.llm_provider.clone(),
                model_name: self.config.model_name.clone(),
                memory: self.config.memory_mode.clone(),
                web_search: self.config.web_search_mode.clone(),
                stream: false,
                send_to_llm: "true".to_string(),
            })
            .await?;

        let mut finish_summary: Option<String> = None;
        for iteration in 1..=self.config.max_iterations {
            let status = normalize_status(response.status.as_deref());
            let content = response.content.clone().unwrap_or_default();
            self.emit(
                EventKind::Status,
                if content.trim().is_empty() {
                    format!("iteration {iteration}: status={status}")
                } else {
                    format!("iteration {iteration}: status={status} {content}")
                },
                Some(json!({ "iteration": iteration, "status": status })),
            );

            match status.as_str() {
                STATUS_REQUIRES_ACTION => {
                    let calls = response.tool_calls.clone().unwrap_or_default();
                    if calls.is_empty() {
                        return Err(anyhow!("REQUIRES_ACTION without tool calls"));
                    }

                    let outputs = self.execute_tools(calls, &mut finish_summary).await?;
                    let run_id = response
                        .run_id
                        .clone()
                        .context("missing run_id in REQUIRES_ACTION response")?;
                    response = self
                        .submit_tool_outputs_with_retry(&session.thread_id, &run_id, outputs)
                        .await?;
                }
                STATUS_COMPLETED => {
                    return Ok(first_non_empty(
                        finish_summary.clone(),
                        response.content.clone(),
                        response.message.clone(),
                    ));
                }
                STATUS_FAILED | STATUS_CANCELLED => {
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
                    if response.tool_calls.as_ref().is_none_or(|v| v.is_empty())
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

        for (index, call) in calls.into_iter().enumerate() {
            self.tools.ensure_allowed(&call.function.name)?;
            self.emit(
                EventKind::ToolQueued,
                format!("queued tool {}/{}: {}", index + 1, total, call.function.name),
                Some(json!({ "tool": call.function.name, "tool_call_id": call.id, "state": "queued" })),
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
                    metadata: Some(json!({ "tool": call_clone.function.name, "tool_call_id": call_clone.id, "state": "running" })),
                });
                let result = tools.execute(&call_clone, ctx).await;
                (index, call_clone, result)
            });
        }

        let mut gathered: Vec<(usize, ToolExecution)> = Vec::new();
        while let Some((idx, call, outcome)) = tasks.next().await {
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

        gathered.sort_by_key(|(idx, _)| *idx);
        Ok(gathered.into_iter().map(|(_, item)| item.output).collect())
    }

    async fn ensure_session(&self) -> Result<AgentSession> {
        if let Some(existing) = self.session.lock().expect("session mutex poisoned").clone() {
            return Ok(existing);
        }

        let assistant = self
            .client
            .create_assistant(CreateAssistantRequest {
                name: "backboard-coding-agent".to_string(),
                system_prompt: Some(self.prompts.coder_prompt().to_string()),
                tools: self.tools.definitions(),
            })
            .await
            .context("failed to create assistant")?;

        let thread = self
            .client
            .create_thread(&assistant.assistant_id)
            .await
            .context("failed to create thread")?;

        let session = AgentSession {
            assistant_id: assistant.assistant_id,
            thread_id: thread.thread_id,
        };
        *self.session.lock().expect("session mutex poisoned") = Some(session.clone());
        Ok(session)
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
}

fn normalize_status(value: Option<&str>) -> String {
    value.unwrap_or_default().trim().to_uppercase()
}

fn parse_ok(output: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(output)
        .ok()
        .and_then(|v| v.get("ok").and_then(|v| v.as_bool()))
        .unwrap_or(false)
}

fn first_non_empty(a: Option<String>, b: Option<String>, c: Option<String>) -> String {
    [a, b, c]
        .into_iter()
        .flatten()
        .find(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "No summary provided".to_string())
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
