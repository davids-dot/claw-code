use std::env;

use api::detect_provider_kind;
use runtime::ConfigLoader;

use crate::constants::{DEFAULT_MODEL, SESSION_REFERENCE_ALIASES};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ModelSource {
    Flag,
    Env,
    Config,
    Default,
}

impl ModelSource {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            ModelSource::Flag => "flag",
            ModelSource::Env => "env",
            ModelSource::Config => "config",
            ModelSource::Default => "default",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelProvenance {
    pub(crate) resolved: String,
    pub(crate) raw: Option<String>,
    pub(crate) source: ModelSource,
}

impl ModelProvenance {
    pub(crate) fn default_fallback() -> Self {
        Self {
            resolved: DEFAULT_MODEL.to_string(),
            raw: None,
            source: ModelSource::Default,
        }
    }

    pub(crate) fn from_flag(raw: &str) -> Self {
        Self {
            resolved: resolve_model_alias_with_config(raw),
            raw: Some(raw.to_string()),
            source: ModelSource::Flag,
        }
    }

    pub(crate) fn from_env_or_config_or_default(cli_model: &str) -> Self {
        if cli_model != DEFAULT_MODEL {
            return Self {
                resolved: cli_model.to_string(),
                raw: Some(cli_model.to_string()),
                source: ModelSource::Flag,
            };
        }
        if let Some(env_model) = env::var("ANTHROPIC_MODEL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            return Self {
                resolved: resolve_model_alias_with_config(&env_model),
                raw: Some(env_model),
                source: ModelSource::Env,
            };
        }
        if let Some(config_model) = config_model_for_current_dir() {
            return Self {
                resolved: resolve_model_alias_with_config(&config_model),
                raw: Some(config_model),
                source: ModelSource::Config,
            };
        }
        Self::default_fallback()
    }
}

pub(crate) fn max_tokens_for_model(model: &str) -> u32 {
    if model.contains("opus") {
        32_000
    } else {
        64_000
    }
}

pub(crate) fn resolve_model_alias(model: &str) -> &str {
    match model {
        "opus" => "claude-opus-4-6",
        "sonnet" => "claude-sonnet-4-6",
        "haiku" => "claude-haiku-4-5-20251213",
        _ => model,
    }
}

pub(crate) fn resolve_model_alias_with_config(model: &str) -> String {
    let trimmed = model.trim();
    if let Some(resolved) = config_alias_for_current_dir(trimmed) {
        return resolve_model_alias(&resolved).to_string();
    }
    resolve_model_alias(trimmed).to_string()
}

pub(crate) fn validate_model_syntax(model: &str) -> Result<(), String> {
    let trimmed = model.trim();
    if trimmed.is_empty() {
        return Err("model string cannot be empty".to_string());
    }
    match trimmed {
        "opus" | "sonnet" | "haiku" | "grok" | "grok-2" | "grok-3" | "grok-mini"
        | "grok-3-mini" | "kimi" | "glm-5" => return Ok(()),
        _ => {}
    }
    if trimmed.starts_with("qwen-")
        || trimmed.starts_with("ali-")
        || trimmed.starts_with("glm-")
        || trimmed.starts_with("kimi-")
    {
        return Ok(());
    }
    if trimmed.contains(' ') {
        return Err(format!(
            "invalid model syntax: '{trimmed}' contains spaces. Use provider/model format or known alias"
        ));
    }
    let parts: Vec<&str> = trimmed.split('/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        let mut err_msg = format!(
            "invalid model syntax: '{trimmed}'. Expected provider/model (e.g., anthropic/claude-opus-4-6) or known alias (opus, sonnet, haiku)"
        );
        if trimmed.starts_with("gpt-") || trimmed.starts_with("gpt_") {
            err_msg.push_str("\nDid you mean `openai/");
            err_msg.push_str(trimmed);
            err_msg.push_str("`? (Requires OPENAI_API_KEY env var)");
        } else if trimmed.starts_with("qwen") {
            err_msg.push_str("\nDid you mean `qwen/");
            err_msg.push_str(trimmed);
            err_msg.push_str("`? (Requires DASHSCOPE_API_KEY env var)");
        } else if trimmed.starts_with("grok") {
            err_msg.push_str("\nDid you mean `xai/");
            err_msg.push_str(trimmed);
            err_msg.push_str("`? (Requires XAI_API_KEY env var)");
        }
        return Err(err_msg);
    }
    Ok(())
}

pub(crate) fn config_alias_for_current_dir(alias: &str) -> Option<String> {
    if alias.is_empty() {
        return None;
    }
    let cwd = env::current_dir().ok()?;
    let loader = ConfigLoader::default_for(&cwd);
    let config = loader.load().ok()?;
    config.aliases().get(alias).cloned()
}

pub(crate) fn config_model_for_current_dir() -> Option<String> {
    let cwd = env::current_dir().ok()?;
    let loader = ConfigLoader::default_for(&cwd);
    loader.load().ok()?.model().map(ToOwned::to_owned)
}

pub(crate) fn resolve_repl_model(cli_model: String) -> String {
    if cli_model != DEFAULT_MODEL {
        return cli_model;
    }
    if let Some(env_model) = env::var("ANTHROPIC_MODEL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return resolve_model_alias_with_config(&env_model);
    }
    if let Some(config_model) = config_model_for_current_dir() {
        return resolve_model_alias_with_config(&config_model);
    }
    cli_model
}

pub(crate) fn provider_label(kind: api::ProviderKind) -> &'static str {
    match kind {
        api::ProviderKind::Anthropic => "anthropic",
        api::ProviderKind::Xai => "xai",
        api::ProviderKind::OpenAi => "openai",
    }
}

pub(crate) fn format_connected_line(model: &str) -> String {
    let provider = provider_label(detect_provider_kind(model));
    format!("Connected: {model} via {provider}")
}
