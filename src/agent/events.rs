use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct AgentEvent {
    pub kind: EventKind,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<Value>,
}

#[derive(Clone, Debug)]
pub enum EventKind {
    Status,
    ToolQueued,
    ToolRunning,
    ToolResult,
    Finished,
}

pub trait EventSink: Send + Sync {
    fn emit(&self, event: AgentEvent);
}
