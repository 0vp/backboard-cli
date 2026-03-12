use crate::agent::events::EventSink;
use crate::backboard::models::{FunctionDefinition, ToolCall, ToolDefinition, ToolOutput};
use crate::runtime::todos::TodoStore;
use crate::tools::builtins;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct ExecutionContext {
    pub workspace_root: PathBuf,
    pub command_timeout: Duration,
    pub jina_api_key: Option<String>,
    pub execute_allowlist: Vec<String>,
    pub todo_store: TodoStore,
    pub event_sink: Arc<dyn EventSink>,
}

#[derive(Clone, Debug)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Clone)]
pub struct ToolRegistry {
    specs: HashMap<String, ToolSpec>,
}

#[derive(Clone, Debug)]
pub struct ToolExecution {
    pub output: ToolOutput,
    pub is_finish: bool,
    pub finish_summary: Option<String>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let specs = builtins::specs()
            .into_iter()
            .map(|s| (s.name.clone(), s))
            .collect();
        Self { specs }
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.specs
            .values()
            .map(|s| ToolDefinition {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: s.name.clone(),
                    description: s.description.clone(),
                    parameters: s.parameters.clone(),
                },
            })
            .collect()
    }

    pub async fn execute(&self, call: &ToolCall, mut ctx: ExecutionContext) -> ToolExecution {
        let arguments = match call.arguments_map() {
            Ok(v) => v,
            Err(err) => return self.error_output(call, anyhow!("invalid tool arguments: {err}")),
        };

        let result = match builtins::dispatch(&call.function.name, arguments, &mut ctx).await {
            Ok(result) => result,
            Err(err) => return self.error_output(call, err),
        };

        if call.function.name == "finish" {
            let summary = result
                .get("summary")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let output = ToolOutput {
                tool_call_id: call.id.clone(),
                output: json!({ "ok": true }).to_string(),
            };
            return ToolExecution {
                output,
                is_finish: true,
                finish_summary: Some(summary),
            };
        }

        if call.function.name == "message" || call.function.name.starts_with("todo_") {
            return ToolExecution {
                output: ToolOutput {
                    tool_call_id: call.id.clone(),
                    output: json!({ "ok": true, "keep_alive": true }).to_string(),
                },
                is_finish: false,
                finish_summary: None,
            };
        }

        ToolExecution {
            output: ToolOutput {
                tool_call_id: call.id.clone(),
                output: result.to_string(),
            },
            is_finish: false,
            finish_summary: None,
        }
    }

    fn error_output(&self, call: &ToolCall, error: anyhow::Error) -> ToolExecution {
        ToolExecution {
            output: ToolOutput {
                tool_call_id: call.id.clone(),
                output: json!({ "ok": false, "error": error.to_string() }).to_string(),
            },
            is_finish: false,
            finish_summary: None,
        }
    }

    pub fn ensure_allowed(&self, name: &str) -> Result<()> {
        if self.specs.contains_key(name) {
            return Ok(());
        }
        Err(anyhow!("tool \"{name}\" is not allowlisted"))
    }
}

#[cfg(test)]
mod tests {
    use super::{ExecutionContext, ToolRegistry};
    use crate::agent::events::{AgentEvent, EventSink};
    use crate::backboard::models::{ToolCall, ToolCallFunction};
    use crate::runtime::todos::TodoStore;
    use serde_json::json;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::tempdir;

    struct NoopSink;
    impl EventSink for NoopSink {
        fn emit(&self, _event: AgentEvent) {}
    }

    #[tokio::test]
    async fn message_returns_keep_alive_ok_only() {
        let dir = tempdir().expect("tempdir");
        let registry = ToolRegistry::new();
        let call = ToolCall {
            id: "tc-1".to_string(),
            function: ToolCallFunction {
                name: "message".to_string(),
                arguments: Some("{\"content\":\"hello\"}".to_string()),
                parsed_arguments: None,
            },
        };
        let ctx = ExecutionContext {
            workspace_root: dir.path().to_path_buf(),
            command_timeout: Duration::from_secs(1),
            jina_api_key: None,
            execute_allowlist: vec![],
            todo_store: TodoStore::default(),
            event_sink: Arc::new(NoopSink),
        };

        let result = registry.execute(&call, ctx).await;
        let output: serde_json::Value =
            serde_json::from_str(&result.output.output).expect("valid json");
        assert_eq!(output, json!({ "ok": true, "keep_alive": true }));
        assert!(!result.is_finish);
    }
}
