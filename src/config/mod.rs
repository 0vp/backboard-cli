use anyhow::{bail, Context, Result};
use std::{env, path::PathBuf, time::Duration};

#[derive(Clone, Debug)]
pub struct Config {
    pub backboard_api_key: String,
    pub jina_api_key: Option<String>,
    pub backboard_base_url: String,
    pub llm_provider: String,
    pub model_name: String,
    pub memory_mode: String,
    pub web_search_mode: String,
    pub workspace_root: PathBuf,
    pub prompts_dir: PathBuf,
    pub request_timeout: Duration,
    pub command_timeout: Duration,
    pub max_iterations: usize,
    pub execute_allowlist: Vec<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let _ = dotenvy::dotenv();

        let backboard_api_key = must_env("BACKBOARD_API_KEY")?;
        let jina_api_key = env::var("JINA_API_KEY")
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());

        let cwd = env::current_dir().context("failed to read current directory")?;
        let workspace_root = env_path("AGENT_WORKSPACE_ROOT").unwrap_or(cwd.clone());
        let prompts_dir = env_path("AGENT_PROMPTS_DIR").unwrap_or(cwd.join("prompts"));

        Ok(Self {
            backboard_api_key,
            jina_api_key,
            backboard_base_url: env_default("BACKBOARD_BASE_URL", "https://app.backboard.io/api"),
            llm_provider: env_default("BACKBOARD_LLM_PROVIDER", "openai"),
            model_name: env_default("BACKBOARD_MODEL_NAME", "gpt-4o"),
            memory_mode: env_default("BACKBOARD_MEMORY_MODE", "Auto"),
            web_search_mode: env_default("BACKBOARD_WEB_SEARCH_MODE", "off"),
            workspace_root,
            prompts_dir,
            request_timeout: Duration::from_secs(env_u64("AGENT_HTTP_TIMEOUT_SECS", 120)),
            command_timeout: Duration::from_secs(env_u64("AGENT_COMMAND_TIMEOUT_SECS", 60)),
            max_iterations: env_usize("AGENT_MAX_ITERATIONS", 24),
            execute_allowlist: env_allowlist("AGENT_EXECUTE_ALLOWLIST"),
        })
    }
}

fn must_env(key: &str) -> Result<String> {
    let v = env::var(key).unwrap_or_default();
    let trimmed = v.trim();
    if trimmed.is_empty() {
        bail!("missing required env var {key}")
    }
    Ok(trimmed.to_string())
}

fn env_default(key: &str, fallback: &str) -> String {
    env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

fn env_u64(key: &str, fallback: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(fallback)
}

fn env_usize(key: &str, fallback: usize) -> usize {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(fallback)
}

fn env_path(key: &str) -> Option<PathBuf> {
    env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

fn env_allowlist(key: &str) -> Vec<String> {
    if let Ok(raw) = env::var(key) {
        let list: Vec<String> = raw
            .split(',')
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect();
        if !list.is_empty() {
            return list;
        }
    }

    [
        "ls", "pwd", "rg", "cat", "head", "tail", "wc", "echo", "python", "python3", "node", "npm",
        "cargo", "rustc", "git", "go",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}
