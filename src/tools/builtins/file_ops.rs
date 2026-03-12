use crate::tools::builtins::utils::{
    get_bool, get_optional_usize, get_usize, paginate, resolve_path, resolve_path_for_create,
};
use crate::tools::registry::ExecutionContext;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

pub async fn read(args: Value, ctx: &ExecutionContext) -> Result<Value> {
    let path_arg = args.get("path").and_then(Value::as_str);
    let path = resolve_path(&ctx.workspace_root, path_arg)?;

    let bytes = tokio::fs::read(&path)
        .await
        .with_context(|| format!("failed to read {}", path.display()))?;

    let offset = get_optional_usize(&args, "offset")
        .unwrap_or(0)
        .min(bytes.len());
    let limit = get_usize(&args, "limit_bytes", 20_000);
    let end = offset.saturating_add(limit).min(bytes.len());

    let content = String::from_utf8_lossy(&bytes[offset..end]).to_string();
    Ok(json!({
        "path": path.display().to_string(),
        "offset": offset,
        "limit_bytes": limit,
        "returned_bytes": end.saturating_sub(offset),
        "total_bytes": bytes.len(),
        "has_more": end < bytes.len(),
        "content": content
    }))
}

pub async fn ls(args: Value, ctx: &ExecutionContext) -> Result<Value> {
    let path_arg = args.get("path").and_then(Value::as_str);
    let dir = resolve_path(&ctx.workspace_root, path_arg)?;

    let mut entries = tokio::fs::read_dir(&dir)
        .await
        .with_context(|| format!("failed to read directory {}", dir.display()))?;

    let mut names = Vec::<String>::new();
    while let Some(entry) = entries.next_entry().await? {
        let metadata = entry.metadata().await?;
        let mut name = entry.file_name().to_string_lossy().to_string();
        if metadata.is_dir() {
            name.push('/');
        }
        names.push(name);
    }
    names.sort();

    let offset = get_optional_usize(&args, "offset").unwrap_or(0);
    let limit = get_usize(&args, "limit", 200);
    let (paged, has_more) = paginate(&names, offset, limit);

    Ok(json!({
        "path": dir.display().to_string(),
        "offset": offset,
        "limit": limit,
        "total_entries": names.len(),
        "has_more": has_more,
        "entries": paged
    }))
}

pub async fn create(args: Value, ctx: &ExecutionContext) -> Result<Value> {
    let path_arg = args
        .get("path")
        .and_then(Value::as_str)
        .context("path is required")?;
    let path = resolve_path_for_create(&ctx.workspace_root, Some(path_arg))?;

    let overwrite = get_bool(&args, "overwrite", false);
    if path.exists() && !overwrite {
        bail!(
            "file already exists: {} (set overwrite=true to replace)",
            path.display()
        );
    }

    let content = args
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    tokio::fs::write(&path, &content)
        .await
        .with_context(|| format!("failed to write {}", path.display()))?;

    Ok(json!({
        "path": path.display().to_string(),
        "bytes_written": content.len(),
        "overwritten": overwrite
    }))
}

pub async fn edit(args: Value, ctx: &ExecutionContext) -> Result<Value> {
    let path_arg = args
        .get("path")
        .and_then(Value::as_str)
        .context("path is required")?;
    let path = resolve_path(&ctx.workspace_root, Some(path_arg))?;

    let old_text = args
        .get("old_text")
        .and_then(Value::as_str)
        .context("old_text is required")?;
    if old_text.is_empty() {
        bail!("old_text cannot be empty");
    }

    let new_text = args
        .get("new_text")
        .and_then(Value::as_str)
        .context("new_text is required")?;
    let replace_all = get_bool(&args, "replace_all", false);

    let source = tokio::fs::read_to_string(&path)
        .await
        .with_context(|| format!("failed to read {}", path.display()))?;

    let occurrences = source.matches(old_text).count();
    if occurrences == 0 {
        bail!("old_text not found in {}", path.display());
    }

    let updated = if replace_all {
        source.replace(old_text, new_text)
    } else {
        source.replacen(old_text, new_text, 1)
    };

    tokio::fs::write(&path, updated)
        .await
        .with_context(|| format!("failed to write {}", path.display()))?;

    Ok(json!({
        "path": path.display().to_string(),
        "occurrences_found": occurrences,
        "occurrences_replaced": if replace_all { occurrences } else { 1 },
        "replace_all": replace_all
    }))
}

#[cfg(test)]
mod tests {
    use super::{create, edit, read};
    use crate::agent::events::{AgentEvent, EventSink};
    use crate::runtime::todos::TodoStore;
    use crate::tools::registry::ExecutionContext;
    use serde_json::json;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::tempdir;

    struct NoopSink;
    impl EventSink for NoopSink {
        fn emit(&self, _event: AgentEvent) {}
    }

    #[tokio::test]
    async fn read_supports_offset() {
        let dir = tempdir().expect("tempdir");
        let file = dir.path().join("sample.txt");
        tokio::fs::write(&file, "abcdef").await.expect("write file");

        let ctx = ExecutionContext {
            workspace_root: dir.path().to_path_buf(),
            command_timeout: Duration::from_secs(1),
            jina_api_key: None,
            execute_allowlist: vec![],
            todo_store: TodoStore::default(),
            event_sink: Arc::new(NoopSink),
        };

        let result = read(
            json!({"path":"sample.txt", "offset":2, "limit_bytes":2}),
            &ctx,
        )
        .await
        .expect("read result");
        assert_eq!(result["content"], "cd");
        assert_eq!(result["has_more"], true);
    }

    #[tokio::test]
    async fn create_and_edit_file() {
        let dir = tempdir().expect("tempdir");

        let ctx = ExecutionContext {
            workspace_root: dir.path().to_path_buf(),
            command_timeout: Duration::from_secs(1),
            jina_api_key: None,
            execute_allowlist: vec![],
            todo_store: TodoStore::default(),
            event_sink: Arc::new(NoopSink),
        };

        let create_out = create(
            json!({"path":"note.txt", "content":"hello world", "overwrite": false}),
            &ctx,
        )
        .await
        .expect("create result");
        assert_eq!(create_out["bytes_written"], 11);

        let edit_out = edit(
            json!({"path":"note.txt", "old_text":"hello", "new_text":"hi", "replace_all": false}),
            &ctx,
        )
        .await
        .expect("edit result");
        assert_eq!(edit_out["occurrences_replaced"], 1);

        let final_text = tokio::fs::read_to_string(dir.path().join("note.txt"))
            .await
            .expect("read file");
        assert_eq!(final_text, "hi world");
    }
}
