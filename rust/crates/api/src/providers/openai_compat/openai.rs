use super::OpenAiCompatConfig;

pub const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
pub const OPENAI_ENV_VARS: &[&str] = &["OPENAI_API_KEY"];
const OPENAI_MAX_REQUEST_BODY_BYTES: usize = 104_857_600; // 100MB

impl OpenAiCompatConfig {
    #[must_use]
    pub const fn openai() -> Self {
        Self {
            provider_name: "OpenAI",
            api_key_env: "OPENAI_API_KEY",
            base_url_env: "OPENAI_BASE_URL",
            default_base_url: DEFAULT_OPENAI_BASE_URL,
            credential_env_vars: OPENAI_ENV_VARS,
            max_request_body_bytes: OPENAI_MAX_REQUEST_BODY_BYTES,
        }
    }
}
