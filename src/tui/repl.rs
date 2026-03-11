use crate::agent::events::{AgentEvent, EventKind, EventSink};
use crate::agent::runner::AgentRunner;
use crate::config::Config;
use crate::runtime::todos::TodoStore;
use anyhow::Result;
use chrono::Utc;
use rustyline::DefaultEditor;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

const LAVENDER: &str = "\x1b[38;2;196;167;231m";
const RESET: &str = "\x1b[0m";
const DIM: &str = "\x1b[2m";
const STRIKE: &str = "\x1b[9m";

pub struct ChannelEventSink {
    tx: UnboundedSender<AgentEvent>,
}

impl EventSink for ChannelEventSink {
    fn emit(&self, event: AgentEvent) {
        let _ = self.tx.send(event);
    }
}

pub fn create_event_sink() -> (Arc<dyn EventSink>, UnboundedReceiver<AgentEvent>) {
    let (tx, rx) = unbounded_channel();
    (Arc::new(ChannelEventSink { tx }), rx)
}

pub async fn run_repl(
    config: &Config,
    runner: Arc<AgentRunner>,
    todos: TodoStore,
    mut rx: UnboundedReceiver<AgentEvent>,
) -> Result<()> {
    print_header(config);

    let todo_store = todos.clone();
    let printer = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            print_event(&event);
            if should_render_todos(&event) {
                print_todos(&todo_store);
            }
        }
    });

    let mut editor = DefaultEditor::new()?;
    let mut turn: u64 = 0;

    loop {
        let line = editor.readline(&format!("{LAVENDER}>{RESET} "));
        let input = match line {
            Ok(v) => v.trim().to_string(),
            Err(_) => break,
        };

        if input.is_empty() {
            continue;
        }
        if input == "/exit" || input == "/quit" {
            break;
        }

        editor.add_history_entry(input.clone())?;
        turn += 1;
        let run_id = format!("run-{}-{turn}", Utc::now().timestamp_millis());

        println!("{DIM}running {run_id}...{RESET}");
        match runner.run_prompt(&run_id, &input).await {
            Ok(summary) => {
                println!("{LAVENDER}summary:{RESET} {summary}");
            }
            Err(err) => {
                eprintln!("\x1b[31merror:{RESET} {err}");
            }
        }
        print_todos(&todos);
    }

    printer.abort();
    println!("{DIM}bye{RESET}");
    Ok(())
}

fn print_header(config: &Config) {
    println!("{LAVENDER}BACKBOARD CODING AGENT{RESET}");
    println!("{DIM}workspace: {}{RESET}", config.workspace_root.display());
    println!(
        "{DIM}model: {}/{}{RESET}",
        config.llm_provider, config.model_name
    );
    println!("{DIM}type /exit to quit{RESET}\n");
}

fn print_event(event: &AgentEvent) {
    let stamp = event.timestamp.format("%H:%M:%S");
    let kind = match event.kind {
        EventKind::Status => "status",
        EventKind::ToolQueued => "tool:queued",
        EventKind::ToolRunning => "tool:running",
        EventKind::ToolResult => "tool:result",
        EventKind::Finished => "finished",
    };
    println!("{DIM}[{stamp}] {kind}{RESET} {}", event.message);
}

fn should_render_todos(event: &AgentEvent) -> bool {
    event
        .metadata
        .as_ref()
        .and_then(|meta| meta.get("tool").and_then(Value::as_str))
        .is_some_and(|tool| tool.starts_with("todo_"))
}

fn print_todos(todos: &TodoStore) {
    let items = todos.list();
    if items.is_empty() {
        return;
    }

    println!("{LAVENDER}todos:{RESET}");
    for item in items {
        if item.completed {
            println!("  - {STRIKE}{} ({}){RESET}", item.title, item.id);
        } else {
            println!("  - {} ({})", item.title, item.id);
        }
    }
}
