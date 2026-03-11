use crate::tools::builtins::utils::{get_optional_usize, get_usize, paginate, resolve_path};
use crate::tools::registry::ExecutionContext;
use anyhow::{Context, Result};
use regex::Regex;
use serde_json::{json, Value};
use std::fs::File;
use std::io::{BufRead, BufReader};
use walkdir::WalkDir;

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

    let regex = Regex::new(pattern).with_context(|| format!("invalid regex pattern: {pattern}"))?;
    let base_arg = args.get("path").and_then(Value::as_str);
    let base_path = resolve_path(&ctx.workspace_root, base_arg)?;

    let mut matches = Vec::<Value>::new();
    for entry in WalkDir::new(&base_path).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if should_skip(path) || !entry.file_type().is_file() {
            continue;
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => continue,
        };
        let reader = BufReader::new(file);
        for (idx, line_result) in reader.lines().enumerate() {
            let line = match line_result {
                Ok(line) => line,
                Err(_) => continue,
            };
            if regex.is_match(&line) {
                matches.push(json!({
                    "path": path.display().to_string(),
                    "line": idx + 1,
                    "content": line
                }));
                if matches.len() >= MAX_SCAN_MATCHES {
                    break;
                }
            }
        }

        if matches.len() >= MAX_SCAN_MATCHES {
            break;
        }
    }

    let offset = get_optional_usize(&args, "offset").unwrap_or(0);
    let limit = get_usize(&args, "limit", 200);
    let (paged, has_more) = paginate(&matches, offset, limit);

    Ok(json!({
        "pattern": pattern,
        "offset": offset,
        "limit": limit,
        "total_matches": matches.len(),
        "scan_capped": matches.len() >= MAX_SCAN_MATCHES,
        "has_more": has_more,
        "matches": paged
    }))
}

fn should_skip(path: &std::path::Path) -> bool {
    path.components().any(|component| {
        matches!(
            component.as_os_str().to_string_lossy().as_ref(),
            ".git" | "node_modules" | "target" | ".factory"
        )
    })
}
