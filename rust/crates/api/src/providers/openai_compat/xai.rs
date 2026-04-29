use super::OpenAiCompatConfig;

pub const DEFAULT_XAI_BASE_URL: &str = "https://api.x.ai/v1";
pub const XAI_ENV_VARS: &[&str] = &["XAI_API_KEY"];
const XAI_MAX_REQUEST_BODY_BYTES: usize = 52_428_800; // 50MB

impl OpenAiCompatConfig {
    #[must_use]
    pub const fn xai() -> Self {
        Self {
            provider_name: "xAI",
            api_key_env: "XAI_API_KEY",
            base_url_env: "XAI_BASE_URL",
            default_base_url: DEFAULT_XAI_BASE_URL,
            credential_env_vars: XAI_ENV_VARS,
            max_request_body_bytes: XAI_MAX_REQUEST_BODY_BYTES,
        }
    }
}
