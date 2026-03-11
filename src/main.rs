mod agent;
mod backboard;
mod config;
mod runtime;
mod tools;
mod tui;

use crate::agent::prompts::PromptStore;
use crate::agent::runner::AgentRunner;
use crate::backboard::client::BackboardClient;
use crate::config::Config;
use crate::runtime::todos::TodoStore;
use crate::tools::registry::ToolRegistry;
use crate::tui::repl::{create_event_sink, run_repl};
use anyhow::Result;
use clap::Parser;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(name = "backboard-cli")]
#[command(about = "Backboard coding agent (single-agent tool-event streaming REPL)")]
struct Cli {
    #[arg()]
    prompt: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?;

    let prompts = PromptStore::load(&config.prompts_dir)?;
    let backboard = BackboardClient::new(
        config.backboard_base_url.clone(),
        config.backboard_api_key.clone(),
        config.request_timeout,
    )?;

    let tools = ToolRegistry::new();
    let todos = TodoStore::default();
    let (sink, rx) = create_event_sink();

    let runner = Arc::new(AgentRunner::new(
        backboard,
        config.clone(),
        prompts,
        tools,
        todos.clone(),
        sink,
    ));

    if let Some(prompt) = cli.prompt {
        std::mem::drop(tokio::spawn(async move {
            let mut rx = rx;
            while let Some(event) = rx.recv().await {
                println!("[{:?}] {}", event.kind, event.message);
            }
        }));

        let summary = runner.run_prompt("run-cli", &prompt).await?;
        println!("{summary}");
        return Ok(());
    }

    run_repl(&config, runner, todos, rx).await
}
