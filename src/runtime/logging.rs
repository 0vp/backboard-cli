use crate::agent::events::{AgentEvent, EventSink};
use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::{json, Value};
use std::fs::{create_dir_all, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionLogger {
    file: Arc<Mutex<File>>,
    path: PathBuf,
}

impl SessionLogger {
    pub fn new(workspace_root: &Path) -> Result<Self> {
        let commit_id = resolve_commit_id(workspace_root);
        let timestamp = Utc::now().format("%Y%m%d-%H%M%S%.3f").to_string();
        let session_id = Uuid::new_v4().simple().to_string();
        let file_name = format!("{timestamp}-session-{}.log", &session_id[..8]);

        let dir = workspace_root.join("log").join(&commit_id);
        create_dir_all(&dir)
            .with_context(|| format!("failed to create log directory {}", dir.display()))?;

        let path = dir.join(file_name);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("failed to open log file {}", path.display()))?;

        let logger = Self {
            file: Arc::new(Mutex::new(file)),
            path,
        };

        logger.log(
            "session_start",
            json!({
                "workspace_root": workspace_root.display().to_string(),
                "commit_id": commit_id,
                "pid": std::process::id(),
                "log_file": logger.path.display().to_string(),
            }),
        );

        Ok(logger)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn log(&self, event: &str, payload: Value) {
        let line = json!({
            "ts": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            "event": event,
            "payload": payload,
        });
        self.write_line(line);
    }

    pub fn log_error(&self, event: &str, err: &anyhow::Error) {
        let chain: Vec<String> = err.chain().map(ToString::to_string).collect();
        self.log(
            event,
            json!({
                "error": err.to_string(),
                "chain": chain,
            }),
        );
    }

    pub fn log_agent_event(&self, event: &AgentEvent) {
        self.log(
            "agent_event",
            json!({
                "kind": format!("{:?}", event.kind),
                "message": event.message,
                "timestamp": event.timestamp.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                "metadata": event.metadata,
            }),
        );
    }

    fn write_line(&self, value: Value) {
        let encoded = match serde_json::to_string(&value) {
            Ok(v) => v,
            Err(_) => return,
        };

        if let Ok(mut guard) = self.file.lock() {
            let _ = writeln!(guard, "{encoded}");
            let _ = guard.flush();
        }
    }
}

pub struct LoggingEventSink {
    inner: Arc<dyn EventSink>,
    logger: Arc<SessionLogger>,
}

impl LoggingEventSink {
    pub fn new(inner: Arc<dyn EventSink>, logger: Arc<SessionLogger>) -> Self {
        Self { inner, logger }
    }
}

impl EventSink for LoggingEventSink {
    fn emit(&self, event: AgentEvent) {
        self.logger.log_agent_event(&event);
        self.inner.emit(event);
    }
}

fn resolve_commit_id(workspace_root: &Path) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace_root)
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !value.is_empty() {
                return sanitize_path_segment(&value);
            }
        }
    }

    "unknown-commit".to_string()
}

fn sanitize_path_segment(raw: &str) -> String {
    let filtered: String = raw
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
        .collect();
    if filtered.is_empty() {
        "unknown".to_string()
    } else {
        filtered
    }
}

#[cfg(test)]
mod tests {
    use super::SessionLogger;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn creates_log_file_under_commit_directory() {
        let tmp = tempdir().expect("tempdir");
        let logger = SessionLogger::new(tmp.path()).expect("logger");
        let path = logger.path().to_path_buf();
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("/log/"));

        logger.log("test_event", serde_json::json!({ "ok": true }));
        let data = fs::read_to_string(path).expect("read log");
        assert!(data.contains("session_start"));
        assert!(data.contains("test_event"));
    }
}
