use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Clone, Debug, Deserialize)]
pub struct ModelCatalog {
    pub providers: Vec<ProviderModels>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProviderModels {
    pub provider: String,
    pub models: Vec<String>,
}

impl ModelCatalog {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read model catalog: {}", path.display()))?;
        let parsed: Self = serde_json::from_str(&raw)
            .with_context(|| format!("invalid JSON in {}", path.display()))?;
        Ok(parsed)
    }

    pub fn contains(&self, provider: &str, model: &str) -> bool {
        self.providers.iter().any(|entry| {
            entry.provider.eq_ignore_ascii_case(provider)
                && entry
                    .models
                    .iter()
                    .any(|item| item.eq_ignore_ascii_case(model))
        })
    }

    pub fn find_exact_provider(&self, provider: &str) -> Option<&str> {
        self.providers
            .iter()
            .find(|entry| entry.provider.eq_ignore_ascii_case(provider))
            .map(|entry| entry.provider.as_str())
    }

    pub fn find_exact_model(&self, provider: &str, model: &str) -> Option<&str> {
        self.providers
            .iter()
            .find(|entry| entry.provider.eq_ignore_ascii_case(provider))
            .and_then(|entry| {
                entry
                    .models
                    .iter()
                    .find(|candidate| candidate.eq_ignore_ascii_case(model))
                    .map(|value| value.as_str())
            })
    }

    pub fn has_entries(&self) -> bool {
        !self.providers.is_empty()
    }

    pub fn flattened_entries(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for provider in &self.providers {
            for model in &provider.models {
                out.push((provider.provider.clone(), model.clone()));
            }
        }
        out
    }
}
