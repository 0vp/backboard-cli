mod command_ops;
mod file_ops;
mod search_ops;
mod state_ops;
mod utils;
mod web_ops;

use crate::tools::registry::{ExecutionContext, ToolSpec};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};

pub fn specs() -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            name: "read".to_string(),
            description: "Read file contents from workspace with offset support".to_string(),
            parameters: schema(
                json!({
                    "path": { "type": "string", "description": "Absolute or workspace-relative path" },
                    "offset": { "type": "integer", "description": "Byte offset to start reading from", "default": 0 },
                    "limit_bytes": { "type": "integer", "description": "Maximum bytes to return", "default": 20000 }
                }),
                &["path"],
            ),
        },
        ToolSpec {
            name: "ls".to_string(),
            description: "List directory entries".to_string(),
            parameters: schema(
                json!({
                    "path": { "type": "string", "description": "Directory path, defaults to workspace root" },
                    "offset": { "type": "integer", "default": 0 },
                    "limit": { "type": "integer", "default": 200 }
                }),
                &[],
            ),
        },
        ToolSpec {
            name: "glob".to_string(),
            description: "Find files by glob pattern".to_string(),
            parameters: schema(
                json!({
                    "pattern": { "type": "string" },
                    "path": { "type": "string", "description": "Base path" },
                    "offset": { "type": "integer", "default": 0 },
                    "limit": { "type": "integer", "default": 200 }
                }),
                &["pattern"],
            ),
        },
        ToolSpec {
            name: "grep".to_string(),
            description: "Search files with regex pattern".to_string(),
            parameters: schema(
                json!({
                    "pattern": { "type": "string" },
                    "path": { "type": "string", "description": "File or directory path" },
                    "offset": { "type": "integer", "default": 0 },
                    "limit": { "type": "integer", "default": 200 }
                }),
                &["pattern"],
            ),
        },
        ToolSpec {
            name: "execute".to_string(),
            description: "Execute allowlisted shell command in workspace".to_string(),
            parameters: schema(json!({ "command": { "type": "string" } }), &["command"]),
        },
        ToolSpec {
            name: "websearch".to_string(),
            description: "Search web with Jina".to_string(),
            parameters: schema(
                json!({
                    "query": { "type": "string" },
                    "max_bytes": { "type": "integer", "default": 30000 }
                }),
                &["query"],
            ),
        },
        ToolSpec {
            name: "web_fetch".to_string(),
            description: "Fetch webpage markdown via Jina reader".to_string(),
            parameters: schema(
                json!({
                    "url": { "type": "string" },
                    "max_bytes": { "type": "integer", "default": 40000 }
                }),
                &["url"],
            ),
        },
        ToolSpec {
            name: "message".to_string(),
            description: "Send progress message to user".to_string(),
            parameters: schema(json!({ "content": { "type": "string" } }), &["content"]),
        },
        ToolSpec {
            name: "todo_create".to_string(),
            description: "Create todo item".to_string(),
            parameters: schema(json!({ "title": { "type": "string" } }), &["title"]),
        },
        ToolSpec {
            name: "todo_update".to_string(),
            description: "Update todo title".to_string(),
            parameters: schema(
                json!({
                    "id": { "type": "string" },
                    "title": { "type": "string" }
                }),
                &["id", "title"],
            ),
        },
        ToolSpec {
            name: "todo_delete".to_string(),
            description: "Delete todo item".to_string(),
            parameters: schema(json!({ "id": { "type": "string" } }), &["id"]),
        },
        ToolSpec {
            name: "todo_list".to_string(),
            description: "List todos".to_string(),
            parameters: schema(json!({}), &[]),
        },
        ToolSpec {
            name: "todo_complete".to_string(),
            description: "Mark todo completed".to_string(),
            parameters: schema(json!({ "id": { "type": "string" } }), &["id"]),
        },
        ToolSpec {
            name: "finish".to_string(),
            description: "Mark run complete and provide summary".to_string(),
            parameters: schema(json!({ "summary": { "type": "string" } }), &["summary"]),
        },
    ]
}

pub async fn dispatch(name: &str, args: Value, ctx: &mut ExecutionContext) -> Result<Value> {
    match name {
        "read" => file_ops::read(args, ctx).await,
        "ls" => file_ops::ls(args, ctx).await,
        "glob" => search_ops::glob(args, ctx).await,
        "grep" => search_ops::grep(args, ctx).await,
        "execute" => command_ops::execute(args, ctx).await,
        "websearch" => web_ops::websearch(args, ctx).await,
        "web_fetch" => web_ops::web_fetch(args, ctx).await,
        "message" => state_ops::message(args, ctx).await,
        "todo_create" => state_ops::todo_create(args, ctx).await,
        "todo_update" => state_ops::todo_update(args, ctx).await,
        "todo_delete" => state_ops::todo_delete(args, ctx).await,
        "todo_list" => state_ops::todo_list(args, ctx).await,
        "todo_complete" => state_ops::todo_complete(args, ctx).await,
        "finish" => state_ops::finish(args, ctx).await,
        other => Err(anyhow!("tool \"{other}\" is not allowlisted")),
    }
}

fn schema(properties: Value, required: &[&str]) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
    })
}
