use crate::tools::builtins::utils::{get_optional_usize, get_usize, paginate, resolve_path};
use crate::tools::registry::ExecutionContext;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use tokio::process::Command;
use tokio::time::timeout;

const MAX_SCAN_MATCHES: usize = 2_000;

pub async fn glob(args: Value, ctx: &ExecutionContext) -> Result<Value> {
    let pattern = args
        .get("pattern")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .context("pattern is required")?;

    let base_arg = args.get("path").and_then(Value::as_str);
    let base_dir = resolve_path(&ctx.workspace_root, base_arg)?;

    let glob_pattern = if std::path::Path::new(pattern).is_absolute() {
        pattern.to_string()
    } else {
        format!("{}/{}", base_dir.display(), pattern)
    };

    let mut matches = Vec::<String>::new();
    for entry in glob::glob(&glob_pattern)? {
        let path = entry?;
        if path.exists() {
            matches.push(path.display().to_string());
        }
    }
    matches.sort();

    let offset = get_optional_usize(&args, "offset").unwrap_or(0);
    let limit = get_usize(&args, "limit", 200);
    let (paged, has_more) = paginate(&matches, offset, limit);

    Ok(json!({
        "pattern": glob_pattern,
        "offset": offset,
        "limit": limit,
        "total_matches": matches.len(),
        "has_more": has_more,
        "matches": paged
    }))
}

pub async fn grep(args: Value, ctx: &ExecutionContext) -> Result<Value> {
    let pattern = args
        .get("pattern")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .context("pattern is required")?;

    let base_arg = args.get("path").and_then(Value::as_str);
    let base_path = resolve_path(&ctx.workspace_root, base_arg)?;

    let output = timeout(
        ctx.command_timeout,
        Command::new("rg")
            .arg("--json")
            .arg("-n")
            .arg("--glob")
            .arg("!.git/**")
            .arg("--glob")
            .arg("!node_modules/**")
            .arg("--glob")
            .arg("!target/**")
            .arg("--glob")
            .arg("!.factory/**")
            .arg("--")
            .arg(pattern)
            .arg(base_path.as_os_str())
            .current_dir(&ctx.workspace_root)
            .output(),
    )
    .await;

    let output = match output {
        Ok(result) => result.context("failed to execute rg")?,
        Err(_) => {
            bail!(
                "grep command timed out after {}s",
                ctx.command_timeout.as_secs()
            )
        }
    };

    let status = output.status.code().unwrap_or(-1);
    if status != 0 && status != 1 {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        bail!("rg failed (exit {status}): {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut matches = Vec::<Value>::new();
    for raw in stdout.lines() {
        let Ok(event) = serde_json::from_str::<Value>(raw) else {
            continue;
        };

        if event.get("type").and_then(Value::as_str) != Some("match") {
            continue;
        }

        let path = event
            .pointer("/data/path/text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let line_number = event
            .pointer("/data/line_number")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        let content = event
            .pointer("/data/lines/text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim_end_matches('\n')
            .to_string();

        matches.push(json!({
            "path": path,
            "line": line_number,
            "content": content
        }));

        if matches.len() >= MAX_SCAN_MATCHES {
            break;
        }
    }

    let offset = get_optional_usize(&args, "offset").unwrap_or(0);
    let limit = get_usize(&args, "limit", 200);
    let (paged, has_more) = paginate(&matches, offset, limit);

    Ok(json!({
        "pattern": pattern,
        "path": base_path.display().to_string(),
        "offset": offset,
        "limit": limit,
        "total_matches": matches.len(),
        "scan_capped": matches.len() >= MAX_SCAN_MATCHES,
        "has_more": has_more,
        "matches": paged
    }))
}
