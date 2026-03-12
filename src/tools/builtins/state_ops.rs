use crate::agent::events::{AgentEvent, EventKind};
use crate::runtime::todos::{TodoPriority, TodoStatus};
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
    let status = parse_status_optional(&args, "status")?.unwrap_or(TodoStatus::Pending);
    let priority = parse_priority_create(&args, "priority")?;
    let item = ctx.todo_store.create(title, status, priority);
    Ok(todo_json(item))
}

pub async fn todo_update(args: Value, ctx: &ExecutionContext) -> Result<Value> {
    let id = get_string(&args, "id").unwrap_or_default();
    if id.trim().is_empty() {
        bail!("id is required");
    }

    let title = get_string(&args, "title")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let status = parse_status_optional(&args, "status")?;
    let priority = parse_priority_update(&args, "priority")?;

    if title.is_none() && status.is_none() && priority.is_none() {
        bail!("provide at least one of title, status, or priority");
    }

    let item = ctx
        .todo_store
        .update(&id, title, status, priority)
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
        "status": item.status.as_str(),
        "priority": item.priority.map(|value| value.as_str()),
        "created_at": item.created_at,
        "completed_at": item.completed_at,
    })
}

fn parse_status_optional(args: &Value, key: &str) -> Result<Option<TodoStatus>> {
    let Some(raw) = get_string(args, key) else {
        return Ok(None);
    };

    let value = raw.trim();
    if value.is_empty() {
        return Ok(None);
    }

    match value.to_ascii_lowercase().as_str() {
        "pending" => Ok(Some(TodoStatus::Pending)),
        "in_progress" => Ok(Some(TodoStatus::InProgress)),
        "completed" => Ok(Some(TodoStatus::Completed)),
        _ => bail!("invalid status: {value}"),
    }
}

fn parse_priority_create(args: &Value, key: &str) -> Result<Option<TodoPriority>> {
    let Some(raw) = get_string(args, key) else {
        return Ok(None);
    };

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    parse_priority_str(trimmed).map(Some)
}

fn parse_priority_update(args: &Value, key: &str) -> Result<Option<Option<TodoPriority>>> {
    let Some(value) = args.get(key) else {
        return Ok(None);
    };

    match value {
        Value::Null => Ok(Some(None)),
        Value::String(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Ok(Some(None));
            }
            Ok(Some(Some(parse_priority_str(trimmed)?)))
        }
        _ => bail!("priority must be a string or null"),
    }
}

fn parse_priority_str(value: &str) -> Result<TodoPriority> {
    match value.to_ascii_lowercase().as_str() {
        "low" => Ok(TodoPriority::Low),
        "medium" => Ok(TodoPriority::Medium),
        "high" => Ok(TodoPriority::High),
        _ => bail!("invalid priority: {value}"),
    }
}
