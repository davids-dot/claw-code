use crate::DEFAULT_MODEL;
use runtime::ConfigLoader;
use std::env;
pub(crate) fn resolve_model_alias(model: &str) -> &str {
    match model {
        "opus" => "claude-opus-4-6",
        "sonnet" => "claude-sonnet-4-6",
        "haiku" => "claude-haiku-4-5-20251213",
        _ => model,
    }
}

/// Resolve a model name through user-defined config aliases first, then fall
/// back to the built-in alias table. This is the entry point used wherever a
/// user-supplied model string is about to be dispatched to a provider.
pub(crate) fn resolve_model_alias_with_config(model: &str) -> String {
    let trimmed = model.trim();
    if let Some(resolved) = config_alias_for_current_dir(trimmed) {
        return resolve_model_alias(&resolved).to_string();
    }
    resolve_model_alias(trimmed).to_string()
}

/// Validate model syntax at parse time.
/// Accepts: known aliases (opus, sonnet, haiku) or provider/model pattern.
/// Rejects: empty, whitespace-only, strings with spaces, or invalid chars.
pub(crate) fn validate_model_syntax(model: &str) -> Result<(), String> {
    let trimmed = model.trim();
    if trimmed.is_empty() {
        return Err("model string cannot be empty".to_string());
    }
    // Known aliases are always valid
    match trimmed {
        "opus" | "sonnet" | "haiku" | "grok" | "grok-2" | "grok-3" | "grok-mini"
        | "grok-3-mini" | "kimi" | "glm-5" => return Ok(()),
        _ => {}
    }

    // Dashscope and xAI models that don't use provider/model syntax
    if trimmed.starts_with("qwen-")
        || trimmed.starts_with("ali-")
        || trimmed.starts_with("glm-")
        || trimmed.starts_with("kimi-")
    {
        return Ok(());
    }
    // Check for spaces (malformed)
    if trimmed.contains(' ') {
        return Err(format!(
            "invalid model syntax: '{trimmed}' contains spaces. Use provider/model format or known alias"
        ));
    }
    // Check provider/model format: provider_id/model_id
    let parts: Vec<&str> = trimmed.split('/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        // #154: hint if the model looks like it belongs to a different provider
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

#[cfg(test)]
mod tests {
    use super::*;
    
    use std::sync::{Mutex, MutexGuard, OnceLock};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::path::{Path, PathBuf};
    use std::fs;

    fn env_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn cwd_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn temp_dir() -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("rusty-claude-cli-{nanos}-{unique}"))
    }

    fn with_current_dir<T>(cwd: &Path, f: impl FnOnce() -> T) -> T {
        let _guard = cwd_lock();
        let previous = std::env::current_dir().expect("cwd should load");
        std::env::set_current_dir(cwd).expect("cwd should change");
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        std::env::set_current_dir(previous).expect("cwd should restore");
        match result {
            Ok(value) => value,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }


    #[test]
        fn resolves_known_model_aliases() {
            assert_eq!(resolve_model_alias("opus"), "claude-opus-4-6");
            assert_eq!(resolve_model_alias("sonnet"), "claude-sonnet-4-6");
            assert_eq!(resolve_model_alias("haiku"), "claude-haiku-4-5-20251213");
            assert_eq!(resolve_model_alias("claude-opus"), "claude-opus");
        }

    #[test]
        fn user_defined_aliases_resolve_before_provider_dispatch() {
            // given
            let _guard = env_lock();
            let root = temp_dir();
            let cwd = root.join("project");
            let config_home = root.join("config-home");
            std::fs::create_dir_all(cwd.join(".claw")).expect("project config dir should exist");
            std::fs::create_dir_all(&config_home).expect("config home should exist");
            std::fs::write(
                cwd.join(".claw").join("settings.json"),
                r#"{"aliases":{"fast":"claude-haiku-4-5-20251213","smart":"opus","cheap":"grok-3-mini"}}"#,
            )
            .expect("project config should write");
    
            let original_config_home = std::env::var("CLAW_CONFIG_HOME").ok();
            std::env::set_var("CLAW_CONFIG_HOME", &config_home);
    
            // when
            let direct = with_current_dir(&cwd, || resolve_model_alias_with_config("fast"));
            let chained = with_current_dir(&cwd, || resolve_model_alias_with_config("smart"));
            let cross_provider = with_current_dir(&cwd, || resolve_model_alias_with_config("cheap"));
            let unknown = with_current_dir(&cwd, || resolve_model_alias_with_config("unknown-model"));
            let builtin = with_current_dir(&cwd, || resolve_model_alias_with_config("haiku"));
    
            match original_config_home {
                Some(value) => std::env::set_var("CLAW_CONFIG_HOME", value),
                None => std::env::remove_var("CLAW_CONFIG_HOME"),
            }
            std::fs::remove_dir_all(root).expect("temp config root should clean up");
    
            // then
            assert_eq!(direct, "claude-haiku-4-5-20251213");
            assert_eq!(chained, "claude-opus-4-6");
            assert_eq!(cross_provider, "grok-3-mini");
            assert_eq!(unknown, "unknown-model");
            assert_eq!(builtin, "claude-haiku-4-5-20251213");
        }

    #[test]
        fn resolve_repl_model_returns_user_supplied_model_unchanged_when_explicit() {
            let user_model = "claude-sonnet-4-6".to_string();
    
            let resolved = resolve_repl_model(user_model);
    
            assert_eq!(resolved, "claude-sonnet-4-6");
        }

    #[test]
        fn resolve_repl_model_falls_back_to_anthropic_model_env_when_default() {
            let _guard = env_lock();
            let root = temp_dir();
            fs::create_dir_all(&root).expect("root dir");
            let config_home = root.join("config");
            fs::create_dir_all(&config_home).expect("config home dir");
            std::env::set_var("CLAW_CONFIG_HOME", &config_home);
            std::env::remove_var("ANTHROPIC_MODEL");
            std::env::set_var("ANTHROPIC_MODEL", "sonnet");
    
            let resolved = with_current_dir(&root, || resolve_repl_model(DEFAULT_MODEL.to_string()));
    
            assert_eq!(resolved, "claude-sonnet-4-6");
    
            std::env::remove_var("ANTHROPIC_MODEL");
            std::env::remove_var("CLAW_CONFIG_HOME");
            fs::remove_dir_all(root).expect("cleanup temp dir");
        }

    #[test]
        fn resolve_repl_model_returns_default_when_env_unset_and_no_config() {
            let _guard = env_lock();
            let root = temp_dir();
            fs::create_dir_all(&root).expect("root dir");
            let config_home = root.join("config");
            fs::create_dir_all(&config_home).expect("config home dir");
            std::env::set_var("CLAW_CONFIG_HOME", &config_home);
            std::env::remove_var("ANTHROPIC_MODEL");
    
            let resolved = with_current_dir(&root, || resolve_repl_model(DEFAULT_MODEL.to_string()));
    
            assert_eq!(resolved, DEFAULT_MODEL);
    
            std::env::remove_var("CLAW_CONFIG_HOME");
            fs::remove_dir_all(root).expect("cleanup temp dir");
        }
}
