use crate::tools::registry::ExecutionContext;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use tokio::process::Command;
use tokio::time::timeout;

const MAX_OUTPUT_BYTES: usize = 20_000;

pub async fn execute(args: Value, ctx: &ExecutionContext) -> Result<Value> {
    let command = args
        .get("command")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .context("command is required")?;

    let tokens = shell_words::split(command).context("failed to parse command")?;
    let first = tokens.first().map(String::as_str).unwrap_or_default();
    if first.is_empty() {
        bail!("command is empty")
    }

    if !ctx
        .execute_allowlist
        .iter()
        .any(|entry| first == entry || first.starts_with(entry))
    {
        bail!("command blocked by allowlist policy: {first}")
    }

    let outcome = timeout(
        ctx.command_timeout,
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(&ctx.workspace_root)
            .output(),
    )
    .await;
    let output = match outcome {
        Ok(result) => result.context("failed to execute command")?,
        Err(_) => {
            return Ok(json!({
                "command": command,
                "timed_out": true,
                "timeout_secs": ctx.command_timeout.as_secs(),
                "stdout": "",
                "stderr": "command timed out"
            }));
        }
    };

    let stdout = truncate_bytes(output.stdout, MAX_OUTPUT_BYTES);
    let stderr = truncate_bytes(output.stderr, MAX_OUTPUT_BYTES);

    Ok(json!({
        "command": command,
        "timed_out": false,
        "exit_code": output.status.code(),
        "success": output.status.success(),
        "stdout": String::from_utf8_lossy(&stdout.bytes).to_string(),
        "stderr": String::from_utf8_lossy(&stderr.bytes).to_string(),
        "stdout_truncated": stdout.truncated,
        "stderr_truncated": stderr.truncated
    }))
}

struct Truncated {
    bytes: Vec<u8>,
    truncated: bool,
}

fn truncate_bytes(mut bytes: Vec<u8>, max: usize) -> Truncated {
    if bytes.len() <= max {
        return Truncated {
            bytes,
            truncated: false,
        };
    }
    bytes.truncate(max);
    Truncated {
        bytes,
        truncated: true,
    }
}
