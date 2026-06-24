use std::io::{self, IsTerminal, Write};

use runtime::{save_user_provider_settings, ConfigLoader, RuntimeProviderConfig};

use serde_json;

const PROVIDERS: &[(&str, &str, &str)] = &[
    ("1", "Anthropic", "anthropic"),
    ("2", "xAI (Grok)", "xai"),
    ("3", "OpenAI", "openai"),
    ("4", "DashScope (Qwen/Kimi)", "dashscope"),
    ("5", "Custom (OpenAI-compat)", "openai"),
];

const PROVIDER_MODELS: &[(&str, &[&str])] = &[
    ("anthropic", &["opus", "sonnet", "haiku"]),
    ("xai", &["grok", "grok-mini", "grok-2"]),
    ("openai", &["gpt-4.1", "gpt-4.1-mini", "gpt-4.1-nano"]),
    ("dashscope", &["qwen-plus", "qwen-max", "kimi"]),
];

const DEFAULT_BASE_URLS: &[(&str, &str)] = &[
    ("anthropic", "https://api.anthropic.com"),
    ("xai", "https://api.x.ai/v1"),
    ("openai", "https://api.openai.com/v1"),
    (
        "dashscope",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
    ),
];

const API_KEY_ENV_VARS: &[(&str, &str)] = &[
    ("anthropic", "ANTHROPIC_API_KEY"),
    ("xai", "XAI_API_KEY"),
    ("openai", "OPENAI_API_KEY"),
    ("dashscope", "DASHSCOPE_API_KEY"),
];

pub fn run_setup_wizard() -> Result<(), Box<dyn std::error::Error>> {
    if !io::stdin().is_terminal() {
        return Err("setup wizard requires an interactive terminal".into());
    }

    let current = load_current_provider_config();

    println!();
    println!("  \x1b[1mClaw Code Setup Wizard\x1b[0m");
    println!("  Configure your provider, API key, and model.");
    println!("  Press Enter to keep current value.\n");

    let provider = select_provider(&current)?;
    let api_key = input_api_key(&provider, &current)?;
    let base_url = input_base_url(&provider, &current)?;
    let model = input_model(&provider, &current)?;

    let config = RuntimeProviderConfig::new(
        Some(provider.to_string()),
        Some(api_key),
        Some(base_url),
        Some(model),
    );

    runtime::save_user_provider_settings(&config)?;
    println!("\n✅ 设置已保存到 ~/.claw/settings.json\n");
    Ok(())
}

fn load_current_provider_config() -> Option<RuntimeProviderConfig> {
    // For now, return None. This should be implemented based on how
    // config loading is actually done in the runtime.
    None
}

fn select_provider(
    _current: &Option<RuntimeProviderConfig>,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("Available providers:");
    for (id, name, _) in PROVIDERS {
        println!("  {}. {}", id, name);
    }
    println!();

    loop {
        print!("Select provider (1-5): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        if let Some((_, _, kind)) = PROVIDERS.iter().find(|(id, _, _)| *id == choice) {
            return Ok(kind.to_string());
        }

        println!("Invalid choice. Please enter 1-5.");
    }
}

fn input_api_key(
    provider: &str,
    current: &Option<RuntimeProviderConfig>,
) -> Result<String, Box<dyn std::error::Error>> {
    let env_var = API_KEY_ENV_VARS
        .iter()
        .find(|(p, _)| *p == provider)
        .map(|(_, env)| *env)
        .unwrap_or("API_KEY");

    let current_value = current.as_ref().and_then(|c| c.api_key()).unwrap_or("");

    println!("\nAPI Key (from {} environment variable):", env_var);
    if !current_value.is_empty() {
        println!(
            "  Current: {}...",
            &current_value[..std::cmp::min(8, current_value.len())]
        );
    }

    loop {
        print!("Enter API key (or press Enter to keep current): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let key = input.trim();

        if key.is_empty() {
            if current_value.is_empty() {
                println!("API key is required.");
                continue;
            }
            return Ok(current_value.to_string());
        }

        if key.len() >= 16 {
            return Ok(key.to_string());
        }

        println!("API key seems too short. Please check your input.");
    }
}

fn input_base_url(
    provider: &str,
    current: &Option<RuntimeProviderConfig>,
) -> Result<String, Box<dyn std::error::Error>> {
    let default_url = DEFAULT_BASE_URLS
        .iter()
        .find(|(p, _)| *p == provider)
        .map(|(_, url)| *url)
        .unwrap_or("https://api.openai.com/v1");

    let current_value = current.as_ref().and_then(|c| c.base_url()).unwrap_or("");

    println!("\nBase URL:");
    if current_value.is_empty() {
        println!("  Default: {}", default_url);
    } else {
        println!("  Current: {}", current_value);
    }

    print!("Enter base URL (or press Enter for default): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let url = input.trim();

    if url.is_empty() {
        Ok(if current_value.is_empty() {
            default_url.to_string()
        } else {
            current_value.to_string()
        })
    } else {
        Ok(url.to_string())
    }
}

fn input_model(
    provider: &str,
    current: &Option<RuntimeProviderConfig>,
) -> Result<String, Box<dyn std::error::Error>> {
    let available_models = PROVIDER_MODELS
        .iter()
        .find(|(p, _)| *p == provider)
        .map(|(_, models)| *models)
        .unwrap_or(&["custom-model"]);

    let current_value = current.as_ref().and_then(|c| c.model()).unwrap_or("");

    println!("\nAvailable models for {}:", provider);
    for (i, model) in available_models.iter().enumerate() {
        println!("  {}. {}", i + 1, model);
    }

    if current_value.is_empty() {
        println!("  Current: (none)");
    } else {
        println!("  Current: {}", current_value);
    }

    loop {
        print!("Enter model name (or press Enter for first option): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let model = input.trim();

        if model.is_empty() {
            let default = if current_value.is_empty() {
                available_models[0]
            } else {
                current_value
            };
            return Ok(default.to_string());
        }

        return Ok(model.to_string());
    }
}
