use super::OpenAiCompatConfig;

pub const DEFAULT_DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com";
pub const DEEPSEEK_ENV_VARS: &[&str] = &["DEEPSEEK_API_KEY"];
const DEEPSEEK_MAX_REQUEST_BODY_BYTES: usize = 104_857_600; // 100MB (Using OpenAI default for now)

impl OpenAiCompatConfig {
    #[must_use]
    pub const fn deepseek() -> Self {
        Self {
            provider_name: "DeepSeek",
            api_key_env: "DEEPSEEK_API_KEY",
            base_url_env: "DEEPSEEK_BASE_URL",
            default_base_url: DEFAULT_DEEPSEEK_BASE_URL,
            credential_env_vars: DEEPSEEK_ENV_VARS,
            max_request_body_bytes: DEEPSEEK_MAX_REQUEST_BODY_BYTES,
        }
    }
}
