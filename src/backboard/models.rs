use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const STATUS_REQUIRES_ACTION: &str = "REQUIRES_ACTION";
pub const STATUS_COMPLETED: &str = "COMPLETED";
pub const STATUS_FAILED: &str = "FAILED";
pub const STATUS_CANCELLED: &str = "CANCELLED";

#[derive(Clone, Debug, Serialize)]
pub struct CreateAssistantRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tools: Vec<ToolDefinition>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDefinition,
}

#[derive(Clone, Debug, Serialize)]
pub struct FunctionDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub parameters: Value,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Assistant {
    pub assistant_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Thread {
    pub thread_id: String,
}

#[derive(Clone, Debug)]
pub struct AddMessageRequest {
    pub thread_id: String,
    pub content: String,
    pub llm_provider: String,
    pub model_name: String,
    pub memory: String,
    pub web_search: String,
    pub stream: bool,
    pub send_to_llm: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MessageResponse {
    pub message: Option<String>,
    pub content: Option<String>,
    pub status: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub run_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub function: ToolCallFunction,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: Option<String>,
    pub parsed_arguments: Option<Value>,
}

impl ToolCall {
    pub fn arguments_map(&self) -> Result<Value> {
        if let Some(v) = &self.function.parsed_arguments {
            return Ok(v.clone());
        }

        if let Some(raw) = &self.function.arguments {
            let parsed: Value =
                serde_json::from_str(raw).context("failed to parse tool arguments JSON")?;
            return Ok(parsed);
        }

        Ok(Value::Object(Default::default()))
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SubmitToolOutputsRequest {
    pub tool_outputs: Vec<ToolOutput>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ToolOutput {
    pub tool_call_id: String,
    pub output: String,
}
