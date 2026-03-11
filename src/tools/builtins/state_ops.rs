use crate::agent::events::{AgentEvent, EventKind};
use crate::tools::builtins::utils::get_string;
use crate::tools::registry::ExecutionContext;
use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde_json::{json, Value};

pub async fn message(args: Value, ctx: &ExecutionContext) -> Result<Value> {
    let content = get_string(&args, "content").unwrap_or_default();
    if content.trim().is_empty() {
        bail!("content is required");
    }

    ctx.event_sink.emit(AgentEvent {
        kind: EventKind::Status,
        message: content.clone(),
        timestamp: Utc::now(),
        metadata: Some(json!({ "source": "message_tool" })),
    });

    Ok(json!({ "ack": true, "content": content }))
}

pub async fn todo_create(args: Value, ctx: &ExecutionContext) -> Result<Value> {
    let title = get_string(&args, "title").unwrap_or_default();
    if title.trim().is_empty() {
        bail!("title is required");
    }
    let item = ctx.todo_store.create(title);
    Ok(todo_json(item))
}

pub async fn todo_update(args: Value, ctx: &ExecutionContext) -> Result<Value> {
    let id = get_string(&args, "id").unwrap_or_default();
    let title = get_string(&args, "title").unwrap_or_default();
    if id.trim().is_empty() || title.trim().is_empty() {
        bail!("id and title are required");
    }
    let item = ctx
        .todo_store
        .update(&id, title)
        .context("todo not found")?;
    Ok(todo_json(item))
}

pub async fn todo_delete(args: Value, ctx: &ExecutionContext) -> Result<Value> {
    let id = get_string(&args, "id").unwrap_or_default();
    if id.trim().is_empty() {
        bail!("id is required");
    }
    if !ctx.todo_store.delete(&id) {
        bail!("todo not found");
    }
    Ok(json!({ "deleted": id }))
}

pub async fn todo_list(_: Value, ctx: &ExecutionContext) -> Result<Value> {
    let items = ctx
        .todo_store
        .list()
        .into_iter()
        .map(todo_json)
        .collect::<Vec<_>>();
    Ok(json!(items))
}

pub async fn todo_complete(args: Value, ctx: &ExecutionContext) -> Result<Value> {
    let id = get_string(&args, "id").unwrap_or_default();
    if id.trim().is_empty() {
        bail!("id is required");
    }
    let item = ctx.todo_store.complete(&id).context("todo not found")?;
    Ok(todo_json(item))
}

pub async fn finish(args: Value, _ctx: &ExecutionContext) -> Result<Value> {
    let summary = get_string(&args, "summary").unwrap_or_else(|| args.to_string());
    Ok(json!({ "summary": summary }))
}

fn todo_json(item: crate::runtime::todos::TodoItem) -> Value {
    json!({
        "id": item.id,
        "title": item.title,
        "completed": item.completed,
        "created_at": item.created_at,
        "completed_at": item.completed_at,
    })
}
