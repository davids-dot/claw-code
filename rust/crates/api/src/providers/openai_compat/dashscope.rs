use super::OpenAiCompatConfig;

pub const DEFAULT_DASHSCOPE_BASE_URL: &str = "https://dashscope.aliyuncs.com/compatible-mode/v1";
pub const DASHSCOPE_ENV_VARS: &[&str] = &["DASHSCOPE_API_KEY"];
const DASHSCOPE_MAX_REQUEST_BODY_BYTES: usize = 6_291_456; // 6MB (observed limit in dogfood)

impl OpenAiCompatConfig {
    /// Alibaba `DashScope` compatible-mode endpoint (Qwen family models).
    /// Uses the OpenAI-compatible REST shape at /compatible-mode/v1.
    /// Requested via Discord #clawcode-get-help: native Alibaba API for
    /// higher rate limits than going through `OpenRouter`.
    #[must_use]
    pub const fn dashscope() -> Self {
        Self {
            provider_name: "DashScope",
            api_key_env: "DASHSCOPE_API_KEY",
            base_url_env: "DASHSCOPE_BASE_URL",
            default_base_url: DEFAULT_DASHSCOPE_BASE_URL,
            credential_env_vars: DASHSCOPE_ENV_VARS,
            max_request_body_bytes: DASHSCOPE_MAX_REQUEST_BODY_BYTES,
        }
    }
}
