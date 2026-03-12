use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::{env, path::Path, path::PathBuf, time::Duration};

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
    pub model_catalog_path: PathBuf,
    pub request_timeout: Duration,
    pub command_timeout: Duration,
    pub max_iterations: usize,
    pub execute_allowlist: Vec<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let _ = dotenvy::dotenv();

        let cwd = env::current_dir().context("failed to read current directory")?;
        let prompts_dir = env_path("AGENT_PROMPTS_DIR").unwrap_or(cwd.join("prompts"));
        let file_config_path = resolve_file_config_path(&cwd, &prompts_dir);
        let file_config = load_file_config(file_config_path.as_deref())?;

        let backboard_api_key = must_env_or_file(
            "BACKBOARD_API_KEY",
            file_config.backboard_api_key,
            file_config_path.as_deref(),
        )?;
        let jina_api_key = env_or_file_optional("JINA_API_KEY", file_config.jina_api_key);

        let workspace_root = env_path("AGENT_WORKSPACE_ROOT").unwrap_or(cwd.clone());
        let model_catalog_path =
            env_path("AGENT_MODEL_CATALOG_PATH").unwrap_or(cwd.join("config").join("models.json"));

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
            model_catalog_path,
            request_timeout: Duration::from_secs(env_u64("AGENT_HTTP_TIMEOUT_SECS", 120)),
            command_timeout: Duration::from_secs(env_u64("AGENT_COMMAND_TIMEOUT_SECS", 60)),
            max_iterations: env_usize("AGENT_MAX_ITERATIONS", 24),
            execute_allowlist: env_allowlist("AGENT_EXECUTE_ALLOWLIST"),
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
struct FileConfig {
    backboard_api_key: Option<String>,
    jina_api_key: Option<String>,
}

fn must_env_or_file(
    key: &str,
    file_value: Option<String>,
    config_path: Option<&Path>,
) -> Result<String> {
    let v = env::var(key).unwrap_or_default();
    let trimmed = v.trim();
    if trimmed.is_empty() {
        if let Some(value) = file_value
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
        {
            return Ok(value);
        }

        let location_hint = config_path
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "config/local.json or ~/.config/wuvo/config.json".to_string());

        bail!("missing required env var {key} and no {key} in {location_hint}")
    }
    Ok(trimmed.to_string())
}

fn env_or_file_optional(key: &str, file_value: Option<String>) -> Option<String> {
    env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .or_else(|| {
            file_value
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        })
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

fn resolve_file_config_path(cwd: &Path, prompts_dir: &Path) -> Option<PathBuf> {
    if let Some(explicit) = env_path("AGENT_CONFIG_PATH") {
        return Some(explicit);
    }

    let agent_root = prompts_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| cwd.to_path_buf());

    [
        agent_root.join("config").join("local.json"),
        cwd.join("config").join("local.json"),
        home_dir().join(".config").join("wuvo").join("config.json"),
    ]
    .into_iter()
    .find(|path| path.exists())
}

fn load_file_config(path: Option<&Path>) -> Result<FileConfig> {
    let Some(path) = path else {
        return Ok(FileConfig::default());
    };

    if !path.exists() {
        return Ok(FileConfig::default());
    }

    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config file {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("invalid JSON in config file {}", path.display()))
}

fn home_dir() -> PathBuf {
    env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
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
        "ls", "mkdir", "touch", "pwd", "which", "echo", "cat", "head", "tail", "wc", "rg", "tree",
        "find", "date", "uname", "whoami", "python", "python3", "pip", "pip3", "node", "npm",
        "npx", "pnpm", "yarn", "bun", "cargo", "rustc", "git", "go",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}
