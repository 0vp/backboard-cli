use anyhow::{Context, Result};
use chrono::Utc;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct PromptStore {
    coder_prompt: String,
}

impl PromptStore {
    pub fn load(prompts_root: &Path) -> Result<Self> {
        let coder_path = prompts_root.join("system").join("coder.txt");
        let tools_path = prompts_root.join("tools").join("policy.txt");
        let coder_raw = std::fs::read_to_string(&coder_path)
            .with_context(|| format!("failed to read prompt file {}", coder_path.display()))?;
        let tools_raw = std::fs::read_to_string(&tools_path)
            .with_context(|| format!("failed to read prompt file {}", tools_path.display()))?;

        let raw = format!("{}\n\n{}", coder_raw.trim(), tools_raw.trim());
        Ok(Self {
            coder_prompt: with_runtime_vars(&raw),
        })
    }

    pub fn coder_prompt(&self) -> &str {
        &self.coder_prompt
    }
}

fn with_runtime_vars(raw: &str) -> String {
    let today = Utc::now().format("%Y-%m-%d").to_string();
    raw.replace("{{TODAY_DATE}}", &today)
}

#[cfg(test)]
mod tests {
    use super::with_runtime_vars;

    #[test]
    fn replaces_today_date() {
        let out = with_runtime_vars("Date={{TODAY_DATE}}");
        assert!(!out.contains("{{TODAY_DATE}}"));
        assert!(out.starts_with("Date="));
    }
}
