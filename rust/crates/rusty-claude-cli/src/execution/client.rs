use runtime::ApiClient;
use api::max_tokens_for_model;
use crate::resolve_cli_auth_source;

use crate::*;
use api::*;
use runtime::*;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::path::Path;

pub(crate) fn build_runtime_plugin_state() -> Result<RuntimePluginState, Box<dyn std::error::Error>> {
    let cwd = env::current_dir()?;
    let loader = ConfigLoader::default_for(&cwd);
    let runtime_config = loader.load()?;
    build_runtime_plugin_state_with_loader(&cwd, &loader, &runtime_config)
}

pub(crate) fn build_runtime_plugin_state_with_loader(
    cwd: &Path,
    loader: &ConfigLoader,
    runtime_config: &runtime::RuntimeConfig,
) -> Result<RuntimePluginState, Box<dyn std::error::Error>> {
    let plugin_manager = build_plugin_manager(cwd, loader, runtime_config);
    let plugin_registry = plugin_manager.plugin_registry()?;
    let plugin_hook_config =
        runtime_hook_config_from_plugin_hooks(plugin_registry.aggregated_hooks()?);
    let feature_config = runtime_config
        .feature_config()
        .clone()
        .with_hooks(runtime_config.hooks().merged(&plugin_hook_config));
    let (mcp_state, runtime_tools) = build_runtime_mcp_state(runtime_config)?;
    let tool_registry = GlobalToolRegistry::with_plugin_tools(plugin_registry.aggregated_tools()?)?
        .with_runtime_tools(runtime_tools)?;
    Ok(RuntimePluginState {
        feature_config,
        tool_registry,
        plugin_registry,
        mcp_state,
    })
}

pub(crate) struct RuntimePluginState {
    pub(crate) feature_config: runtime::RuntimeFeatureConfig,
    pub(crate) tool_registry: GlobalToolRegistry,
    pub(crate) plugin_registry: PluginRegistry,
    pub(crate) mcp_state: Option<Arc<Mutex<RuntimeMcpState>>>,
}

pub(crate) fn mcp_wrapper_tool_definitions() -> Vec<RuntimeToolDefinition> {
    vec![
        RuntimeToolDefinition {
            name: "MCPTool".to_string(),
            description: Some(
                "Call a configured MCP tool by its qualified name and JSON arguments.".to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "qualifiedName": { "type": "string" },
                    "arguments": {}
                },
                "required": ["qualifiedName"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::DangerFullAccess,
        },
        RuntimeToolDefinition {
            name: "ListMcpResourcesTool".to_string(),
            description: Some(
                "List MCP resources from one configured server or from every connected server."
                    .to_string(),
            ),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "server": { "type": "string" }
                },
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        RuntimeToolDefinition {
            name: "ReadMcpResourceTool".to_string(),
            description: Some("Read a specific MCP resource from a configured server.".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "server": { "type": "string" },
                    "uri": { "type": "string" }
                },
                "required": ["server", "uri"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
    ]
}

pub(crate) fn mcp_runtime_tool_definition(tool: &runtime::ManagedMcpTool) -> RuntimeToolDefinition {
    RuntimeToolDefinition {
        name: tool.qualified_name.clone(),
        description: Some(
            tool.tool
                .description
                .clone()
                .unwrap_or_else(|| format!("Invoke MCP tool `{}`.", tool.qualified_name)),
        ),
        input_schema: tool
            .tool
            .input_schema
            .clone()
            .unwrap_or_else(|| json!({ "type": "object", "additionalProperties": true })),
        required_permission: permission_mode_for_mcp_tool(&tool.tool),
    }
}

pub(crate) fn permission_mode_for_mcp_tool(tool: &McpTool) -> PermissionMode {
    let read_only = mcp_annotation_flag(tool, "readOnlyHint");
    let destructive = mcp_annotation_flag(tool, "destructiveHint");
    let open_world = mcp_annotation_flag(tool, "openWorldHint");

    if read_only && !destructive && !open_world {
        PermissionMode::ReadOnly
    } else if destructive || open_world {
        PermissionMode::DangerFullAccess
    } else {
        PermissionMode::WorkspaceWrite
    }
}

pub(crate) fn mcp_annotation_flag(tool: &McpTool, key: &str) -> bool {
    tool.annotations
        .as_ref()
        .and_then(|annotations| annotations.get(key))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

pub(crate) struct RuntimeMcpState {
    pub(crate) runtime: tokio::runtime::Runtime,
    pub(crate) manager: McpServerManager,
    pub(crate) pending_servers: Vec<String>,
    pub(crate) degraded_report: Option<runtime::McpDegradedReport>,
}

pub(crate) struct BuiltRuntime {
    pub(crate) runtime: Option<ConversationRuntime<AnthropicRuntimeClient, CliToolExecutor>>,
    pub(crate) plugin_registry: PluginRegistry,
    pub(crate) plugins_active: bool,
    pub(crate) mcp_state: Option<Arc<Mutex<RuntimeMcpState>>>,
    pub(crate) mcp_active: bool,
}

impl BuiltRuntime {
    pub(crate) fn new(
        runtime: ConversationRuntime<AnthropicRuntimeClient, CliToolExecutor>,
        plugin_registry: PluginRegistry,
        mcp_state: Option<Arc<Mutex<RuntimeMcpState>>>,
    ) -> Self {
        Self {
            runtime: Some(runtime),
            plugin_registry,
            plugins_active: true,
            mcp_state,
            mcp_active: true,
        }
    }

    pub(crate) fn with_hook_abort_signal(mut self, hook_abort_signal: runtime::HookAbortSignal) -> Self {
        let runtime = self
            .runtime
            .take()
            .expect("runtime should exist before installing hook abort signal");
        self.runtime = Some(runtime.with_hook_abort_signal(hook_abort_signal));
        self
    }

    pub(crate) fn shutdown_plugins(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.plugins_active {
            self.plugin_registry.shutdown()?;
            self.plugins_active = false;
        }
        Ok(())
    }

    pub(crate) fn shutdown_mcp(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.mcp_active {
            if let Some(mcp_state) = &self.mcp_state {
                mcp_state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .shutdown()?;
            }
            self.mcp_active = false;
        }
        Ok(())
    }
}

impl Deref for BuiltRuntime {
    type Target = ConversationRuntime<AnthropicRuntimeClient, CliToolExecutor>;

    fn deref(&self) -> &Self::Target {
        self.runtime
            .as_ref()
            .expect("runtime should exist while built runtime is alive")
    }
}

impl DerefMut for BuiltRuntime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.runtime
            .as_mut()
            .expect("runtime should exist while built runtime is alive")
    }
}

impl Drop for BuiltRuntime {
    fn drop(&mut self) {
        let _ = self.shutdown_mcp();
        let _ = self.shutdown_plugins();
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct ToolSearchRequest {
    pub(crate) query: String,
    pub(crate) max_results: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct McpToolRequest {
    #[serde(rename = "qualifiedName")]
    pub(crate) qualified_name: Option<String>,
    pub(crate) tool: Option<String>,
    pub(crate) arguments: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ListMcpResourcesRequest {
    pub(crate) server: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReadMcpResourceRequest {
    pub(crate) server: String,
    pub(crate) uri: String,
}

impl RuntimeMcpState {
    pub(crate) fn new(
        runtime_config: &runtime::RuntimeConfig,
    ) -> Result<Option<(Self, runtime::McpToolDiscoveryReport)>, Box<dyn std::error::Error>> {
        let mut manager = McpServerManager::from_runtime_config(runtime_config);
        if manager.server_names().is_empty() && manager.unsupported_servers().is_empty() {
            return Ok(None);
        }

        let runtime = tokio::runtime::Runtime::new()?;
        let discovery = runtime.block_on(manager.discover_tools_best_effort());
        let pending_servers = discovery
            .failed_servers
            .iter()
            .map(|failure| failure.server_name.clone())
            .chain(
                discovery
                    .unsupported_servers
                    .iter()
                    .map(|server| server.server_name.clone()),
            )
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let available_tools = discovery
            .tools
            .iter()
            .map(|tool| tool.qualified_name.clone())
            .collect::<Vec<_>>();
        let failed_server_names = pending_servers.iter().cloned().collect::<BTreeSet<_>>();
        let working_servers = manager
            .server_names()
            .into_iter()
            .filter(|server_name| !failed_server_names.contains(server_name))
            .collect::<Vec<_>>();
        let failed_servers =
            discovery
                .failed_servers
                .iter()
                .map(|failure| runtime::McpFailedServer {
                    server_name: failure.server_name.clone(),
                    phase: runtime::McpLifecyclePhase::ToolDiscovery,
                    error: runtime::McpErrorSurface::new(
                        runtime::McpLifecyclePhase::ToolDiscovery,
                        Some(failure.server_name.clone()),
                        failure.error.clone(),
                        std::collections::BTreeMap::new(),
                        true,
                    ),
                })
                .chain(discovery.unsupported_servers.iter().map(|server| {
                    runtime::McpFailedServer {
                        server_name: server.server_name.clone(),
                        phase: runtime::McpLifecyclePhase::ServerRegistration,
                        error: runtime::McpErrorSurface::new(
                            runtime::McpLifecyclePhase::ServerRegistration,
                            Some(server.server_name.clone()),
                            server.reason.clone(),
                            std::collections::BTreeMap::from([(
                                "transport".to_string(),
                                format!("{:?}", server.transport).to_ascii_lowercase(),
                            )]),
                            false,
                        ),
                    }
                }))
                .collect::<Vec<_>>();
        let degraded_report = (!failed_servers.is_empty()).then(|| {
            runtime::McpDegradedReport::new(
                working_servers,
                failed_servers,
                available_tools.clone(),
                available_tools,
            )
        });

        Ok(Some((
            Self {
                runtime,
                manager,
                pending_servers,
                degraded_report,
            },
            discovery,
        )))
    }

    pub(crate) fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.runtime.block_on(self.manager.shutdown())?;
        Ok(())
    }

    pub(crate) fn pending_servers(&self) -> Option<Vec<String>> {
        (!self.pending_servers.is_empty()).then(|| self.pending_servers.clone())
    }

    pub(crate) fn degraded_report(&self) -> Option<runtime::McpDegradedReport> {
        self.degraded_report.clone()
    }

    pub(crate) fn server_names(&self) -> Vec<String> {
        self.manager.server_names()
    }

    pub(crate) fn call_tool(
        &mut self,
        qualified_tool_name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<String, ToolError> {
        let response = self
            .runtime
            .block_on(self.manager.call_tool(qualified_tool_name, arguments))
            .map_err(|error| ToolError::new(error.to_string()))?;
        if let Some(error) = response.error {
            return Err(ToolError::new(format!(
                "MCP tool `{qualified_tool_name}` returned JSON-RPC error: {} ({})",
                error.message, error.code
            )));
        }

        let result = response.result.ok_or_else(|| {
            ToolError::new(format!(
                "MCP tool `{qualified_tool_name}` returned no result payload"
            ))
        })?;
        serde_json::to_string_pretty(&result).map_err(|error| ToolError::new(error.to_string()))
    }

    pub(crate) fn list_resources_for_server(&mut self, server_name: &str) -> Result<String, ToolError> {
        let result = self
            .runtime
            .block_on(self.manager.list_resources(server_name))
            .map_err(|error| ToolError::new(error.to_string()))?;
        serde_json::to_string_pretty(&json!({
            "server": server_name,
            "resources": result.resources,
        }))
        .map_err(|error| ToolError::new(error.to_string()))
    }

    pub(crate) fn list_resources_for_all_servers(&mut self) -> Result<String, ToolError> {
        let mut resources = Vec::new();
        let mut failures = Vec::new();

        for server_name in self.server_names() {
            match self
                .runtime
                .block_on(self.manager.list_resources(&server_name))
            {
                Ok(result) => resources.push(json!({
                    "server": server_name,
                    "resources": result.resources,
                })),
                Err(error) => failures.push(json!({
                    "server": server_name,
                    "error": error.to_string(),
                })),
            }
        }

        if resources.is_empty() && !failures.is_empty() {
            let message = failures
                .iter()
                .filter_map(|failure| failure.get("error").and_then(serde_json::Value::as_str))
                .collect::<Vec<_>>()
                .join("; ");
            return Err(ToolError::new(message));
        }

        serde_json::to_string_pretty(&json!({
            "resources": resources,
            "failures": failures,
        }))
        .map_err(|error| ToolError::new(error.to_string()))
    }

    pub(crate) fn read_resource(&mut self, server_name: &str, uri: &str) -> Result<String, ToolError> {
        let result = self
            .runtime
            .block_on(self.manager.read_resource(server_name, uri))
            .map_err(|error| ToolError::new(error.to_string()))?;
        serde_json::to_string_pretty(&json!({
            "server": server_name,
            "contents": result.contents,
        }))
        .map_err(|error| ToolError::new(error.to_string()))
    }
}


pub(crate) struct AnthropicRuntimeClient {
    runtime: tokio::runtime::Runtime,
    client: ApiProviderClient,
    session_id: String,
    model: String,
    enable_tools: bool,
    emit_output: bool,
    allowed_tools: Option<AllowedToolSet>,
    tool_registry: GlobalToolRegistry,
    progress_reporter: Option<InternalPromptProgressReporter>,
    reasoning_effort: Option<String>,
}
impl AnthropicRuntimeClient {
    pub(crate) fn new(
        session_id: &str,
        model: String,
        enable_tools: bool,
        emit_output: bool,
        allowed_tools: Option<AllowedToolSet>,
        tool_registry: GlobalToolRegistry,
        progress_reporter: Option<InternalPromptProgressReporter>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Dispatch to the correct provider at construction time.
        // `ApiProviderClient` (exposed by the api crate as
        // `ProviderClient`) is an enum over Anthropic / xAI / OpenAI
        // variants, where xAI and OpenAI both use the OpenAI-compat
        // wire format under the hood. We consult
        // `detect_provider_kind(&resolved_model)` so model-name prefix
        // routing (`openai/`, `gpt-`, `grok`, `qwen/`) wins over
        // env-var presence.
        //
        // For Anthropic we build the client directly instead of going
        // through `ApiProviderClient::from_model_with_anthropic_auth`
        // so we can explicitly apply `api::read_base_url()` — that
        // reads `ANTHROPIC_BASE_URL` and is required for the local
        // mock-server test harness
        // (`crates/rusty-claude-cli/tests/compact_output.rs`) to point
        // claw at its fake Anthropic endpoint. We also attach a
        // session-scoped prompt cache on the Anthropic path; the
        // prompt cache is Anthropic-only so non-Anthropic variants
        // skip it.
        let resolved_model = api::resolve_model_alias(&model);
        let client = match detect_provider_kind(&resolved_model) {
            ProviderKind::Anthropic => {
                let auth = resolve_cli_auth_source()?;
                let inner = AnthropicClient::from_auth(auth)
                    .with_base_url(api::read_base_url())
                    .with_prompt_cache(PromptCache::new(session_id));
                ApiProviderClient::Anthropic(inner)
            }
            ProviderKind::Xai | ProviderKind::OpenAi => {
                // The api crate's `ProviderClient::from_model_with_anthropic_auth`
                // with `None` for the anthropic auth routes via
                // `detect_provider_kind` and builds an
                // `OpenAiCompatClient::from_env` with the matching
                // `OpenAiCompatConfig` (openai / xai / dashscope).
                // That reads the correct API-key env var and BASE_URL
                // override internally, so this one call covers OpenAI,
                // OpenRouter, xAI, DashScope, Ollama, and any other
                // OpenAI-compat endpoint users configure via
                // `OPENAI_BASE_URL` / `XAI_BASE_URL` / `DASHSCOPE_BASE_URL`.
                ApiProviderClient::from_model_with_anthropic_auth(&resolved_model, None)?
            }
        };
        Ok(Self {
            runtime: tokio::runtime::Runtime::new()?,
            client,
            session_id: session_id.to_string(),
            model,
            enable_tools,
            emit_output,
            allowed_tools,
            tool_registry,
            progress_reporter,
            reasoning_effort: None,
        })
    }

    pub(crate) fn set_reasoning_effort(&mut self, effort: Option<String>) {
        self.reasoning_effort = effort;
    }
}
impl AnthropicRuntimeClient {
    /// Consume a single streaming response, optionally applying a stall
    /// timeout on the first event for post-tool continuations.
    #[allow(clippy::too_many_lines)]
    pub(crate) async fn consume_stream(
        &self,
        message_request: &MessageRequest,
        apply_stall_timeout: bool,
    ) -> Result<Vec<AssistantEvent>, RuntimeError> {
        let mut stream = self
            .client
            .stream_message(message_request)
            .await
            .map_err(|error| {
                RuntimeError::new(format_user_visible_api_error(&self.session_id, &error))
            })?;
        let mut stdout = io::stdout();
        let mut sink = io::sink();
        let out: &mut dyn Write = if self.emit_output {
            &mut stdout
        } else {
            &mut sink
        };
        let renderer = TerminalRenderer::new();
        let mut markdown_stream = MarkdownStreamState::default();
        let mut events = Vec::new();
        let mut pending_tool: Option<(String, String, String)> = None;
        let mut block_has_thinking_summary = false;
        let mut saw_stop = false;
        let mut received_any_event = false;

        loop {
            let next = if apply_stall_timeout && !received_any_event {
                match tokio::time::timeout(POST_TOOL_STALL_TIMEOUT, stream.next_event()).await {
                    Ok(inner) => inner.map_err(|error| {
                        RuntimeError::new(format_user_visible_api_error(&self.session_id, &error))
                    })?,
                    Err(_elapsed) => {
                        return Err(RuntimeError::new(
                            "post-tool stall: model did not respond within timeout",
                        ));
                    }
                }
            } else {
                stream.next_event().await.map_err(|error| {
                    RuntimeError::new(format_user_visible_api_error(&self.session_id, &error))
                })?
            };

            let Some(event) = next else {
                break;
            };
            received_any_event = true;

            match event {
                ApiStreamEvent::MessageStart(start) => {
                    for block in start.message.content {
                        push_output_block(
                            block,
                            out,
                            &mut events,
                            &mut pending_tool,
                            true,
                            &mut block_has_thinking_summary,
                        )?;
                    }
                }
                ApiStreamEvent::ContentBlockStart(start) => {
                    push_output_block(
                        start.content_block,
                        out,
                        &mut events,
                        &mut pending_tool,
                        true,
                        &mut block_has_thinking_summary,
                    )?;
                }
                ApiStreamEvent::ContentBlockDelta(delta) => match delta.delta {
                    ContentBlockDelta::TextDelta { text } => {
                        if !text.is_empty() {
                            if let Some(progress_reporter) = &self.progress_reporter {
                                progress_reporter.mark_text_phase(&text);
                            }
                            if let Some(rendered) = markdown_stream.push(&renderer, &text) {
                                write!(out, "{rendered}")
                                    .and_then(|()| out.flush())
                                    .map_err(|error| RuntimeError::new(error.to_string()))?;
                            }
                            events.push(AssistantEvent::TextDelta(text));
                        }
                    }
                    ContentBlockDelta::InputJsonDelta { partial_json } => {
                        if let Some((_, _, input)) = &mut pending_tool {
                            input.push_str(&partial_json);
                        }
                    }
                    ContentBlockDelta::ThinkingDelta { thinking } => {
                        if !block_has_thinking_summary {
                            render_thinking_block_summary(out, None, false)?;
                            block_has_thinking_summary = true;
                        }
                        events.push(AssistantEvent::ThinkingDelta(thinking));
                    }
                    ContentBlockDelta::SignatureDelta { signature } => {
                        events.push(AssistantEvent::SignatureDelta(signature));
                    }
                },
                ApiStreamEvent::ContentBlockStop(_) => {
                    block_has_thinking_summary = false;
                    if let Some(rendered) = markdown_stream.flush(&renderer) {
                        write!(out, "{rendered}")
                            .and_then(|()| out.flush())
                            .map_err(|error| RuntimeError::new(error.to_string()))?;
                    }
                    if let Some((id, name, input)) = pending_tool.take() {
                        if let Some(progress_reporter) = &self.progress_reporter {
                            progress_reporter.mark_tool_phase(&name, &input);
                        }
                        // Display tool call now that input is fully accumulated
                        writeln!(out, "\n{}", format_tool_call_start(&name, &input))
                            .and_then(|()| out.flush())
                            .map_err(|error| RuntimeError::new(error.to_string()))?;
                        events.push(AssistantEvent::ToolUse { id, name, input });
                    }
                }
                ApiStreamEvent::MessageDelta(delta) => {
                    events.push(AssistantEvent::Usage(delta.usage.token_usage()));
                }
                ApiStreamEvent::MessageStop(_) => {
                    saw_stop = true;
                    if let Some(rendered) = markdown_stream.flush(&renderer) {
                        write!(out, "{rendered}")
                            .and_then(|()| out.flush())
                            .map_err(|error| RuntimeError::new(error.to_string()))?;
                    }
                    events.push(AssistantEvent::MessageStop);
                }
            }
        }

        push_prompt_cache_record(&self.client, &mut events);

        if !saw_stop
            && events.iter().any(|event| {
                matches!(event, AssistantEvent::TextDelta(text) if !text.is_empty())
                    || matches!(event, AssistantEvent::ToolUse { .. })
            })
        {
            events.push(AssistantEvent::MessageStop);
        }

        if events
            .iter()
            .any(|event| matches!(event, AssistantEvent::MessageStop))
        {
            return Ok(events);
        }

        let response = self
            .client
            .send_message(&MessageRequest {
                stream: false,
                ..message_request.clone()
            })
            .await
            .map_err(|error| {
                RuntimeError::new(format_user_visible_api_error(&self.session_id, &error))
            })?;
        let mut events = response_to_events(response, out)?;
        push_prompt_cache_record(&self.client, &mut events);
        Ok(events)
    }
}

impl ApiClient for AnthropicRuntimeClient {
    #[allow(clippy::too_many_lines)]
    fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
        if let Some(progress_reporter) = &self.progress_reporter {
            progress_reporter.mark_model_phase();
        }
        let is_post_tool = request_ends_with_tool_result(&request);
        let message_request = MessageRequest {
            model: self.model.clone(),
            max_tokens: max_tokens_for_model(&self.model),
            messages: convert_messages(&request.messages),
            system: (!request.system_prompt.is_empty()).then(|| request.system_prompt.join("\n\n")),
            tools: self
                .enable_tools
                .then(|| filter_tool_specs(&self.tool_registry, self.allowed_tools.as_ref())),
            tool_choice: self.enable_tools.then_some(ToolChoice::Auto),
            stream: true,
            reasoning_effort: self.reasoning_effort.clone(),
            ..Default::default()
        };

        self.runtime.block_on(async {
            // When resuming after tool execution, apply a stall timeout on the
            // first stream event.  If the model does not respond within the
            // deadline we drop the stalled connection and re-send the request as
            // a continuation nudge (one retry only).
            let max_attempts: usize = if is_post_tool { 2 } else { 1 };

            for attempt in 1..=max_attempts {
                let result = self
                    .consume_stream(&message_request, is_post_tool && attempt == 1)
                    .await;
                match result {
                    Ok(events) => return Ok(events),
                    Err(error)
                        if error.to_string().contains("post-tool stall")
                            && attempt < max_attempts =>
                    {
                        // Stalled after tool completion — nudge the model by
                        // re-sending the same request.
                    }
                    Err(error) => return Err(error),
                }
            }

            Err(RuntimeError::new("post-tool continuation nudge exhausted"))
        })
    }
}






#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::sync::{MutexGuard, Mutex, OnceLock};
    use std::env;
    use std::time::SystemTime;

    fn env_lock() -> MutexGuard<'static, ()> {
            static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
            LOCK.get_or_init(|| Mutex::new(()))
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        }
        fn temp_dir() -> PathBuf {
            use std::sync::atomic::{AtomicU64, Ordering};
    
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be after epoch")
                .as_nanos();
            let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
            std::env::temp_dir().join(format!("rusty-claude-cli-{nanos}-{unique}"))
        }
        fn write_plugin_fixture(root: &Path, name: &str, include_hooks: bool, include_lifecycle: bool) {
            fs::create_dir_all(root.join(".claude-plugin")).expect("manifest dir");
            if include_hooks {
                fs::create_dir_all(root.join("hooks")).expect("hooks dir");
                fs::write(
                    root.join("hooks").join("pre.sh"),
                    "#!/bin/sh\nprintf 'plugin pre hook'\n",
                )
                .expect("write hook");
            }
            if include_lifecycle {
                fs::create_dir_all(root.join("lifecycle")).expect("lifecycle dir");
                fs::write(
                    root.join("lifecycle").join("init.sh"),
                    "#!/bin/sh\nprintf 'init\\n' >> lifecycle.log\n",
                )
                .expect("write init lifecycle");
                fs::write(
                    root.join("lifecycle").join("shutdown.sh"),
                    "#!/bin/sh\nprintf 'shutdown\\n' >> lifecycle.log\n",
                )
                .expect("write shutdown lifecycle");
            }
    
            let hooks = if include_hooks {
                ",\n  \"hooks\": {\n    \"PreToolUse\": [\"./hooks/pre.sh\"]\n  }"
            } else {
                ""
            };
            let lifecycle = if include_lifecycle {
                ",\n  \"lifecycle\": {\n    \"Init\": [\"./lifecycle/init.sh\"],\n    \"Shutdown\": [\"./lifecycle/shutdown.sh\"]\n  }"
            } else {
                ""
            };
            fs::write(
                root.join(".claude-plugin").join("plugin.json"),
                format!(
                    "{{\n  \"name\": \"{name}\",\n  \"version\": \"1.0.0\",\n  \"description\": \"runtime plugin fixture\"{hooks}{lifecycle}\n}}"
                ),
            )
            .expect("write plugin manifest");
        }

    #[test]
        fn build_runtime_plugin_state_merges_plugin_hooks_into_runtime_features() {
            let config_home = temp_dir();
            let workspace = temp_dir();
            let source_root = temp_dir();
            fs::create_dir_all(&config_home).expect("config home");
            fs::create_dir_all(&workspace).expect("workspace");
            fs::create_dir_all(&source_root).expect("source root");
            write_plugin_fixture(&source_root, "hook-runtime-demo", true, false);
    
            let mut manager = PluginManager::new(PluginManagerConfig::new(&config_home));
            manager
                .install(source_root.to_str().expect("utf8 source path"))
                .expect("plugin install should succeed");
            let loader = ConfigLoader::new(&workspace, &config_home);
            let runtime_config = loader.load().expect("runtime config should load");
            let state = build_runtime_plugin_state_with_loader(&workspace, &loader, &runtime_config)
                .expect("plugin state should load");
            let pre_hooks = state.feature_config.hooks().pre_tool_use();
            assert_eq!(pre_hooks.len(), 1);
            assert!(
                pre_hooks[0].ends_with("hooks/pre.sh"),
                "expected installed plugin hook path, got {pre_hooks:?}"
            );
    
            let _ = fs::remove_dir_all(config_home);
            let _ = fs::remove_dir_all(workspace);
            let _ = fs::remove_dir_all(source_root);
        }

    #[test]
        #[allow(clippy::too_many_lines)]
        fn build_runtime_plugin_state_discovers_mcp_tools_and_surfaces_pending_servers() {
            let config_home = temp_dir();
            let workspace = temp_dir();
            fs::create_dir_all(&config_home).expect("config home");
            fs::create_dir_all(&workspace).expect("workspace");
            let script_path = workspace.join("fixture-mcp.py");
            write_mcp_server_fixture(&script_path);
            fs::write(
                config_home.join("settings.json"),
                format!(
                    r#"{{
                      "mcpServers": {{
                        "alpha": {{
                          "command": "python3",
                          "args": ["{}"]
                        }},
                        "broken": {{
                          "command": "python3",
                          "args": ["-c", "import sys; sys.exit(0)"]
                        }}
                      }}
                    }}"#,
                    script_path.to_string_lossy()
                ),
            )
            .expect("write mcp settings");
    
            let loader = ConfigLoader::new(&workspace, &config_home);
            let runtime_config = loader.load().expect("runtime config should load");
            let state = build_runtime_plugin_state_with_loader(&workspace, &loader, &runtime_config)
                .expect("runtime plugin state should load");
    
            let allowed = state
                .tool_registry
                .normalize_allowed_tools(&["mcp__alpha__echo".to_string(), "MCPTool".to_string()])
                .expect("mcp tools should be allow-listable")
                .expect("allow-list should exist");
            assert!(allowed.contains("mcp__alpha__echo"));
            assert!(allowed.contains("MCPTool"));
    
            let mut executor = CliToolExecutor::new(
                None,
                false,
                state.tool_registry.clone(),
                state.mcp_state.clone(),
            );
    
            let tool_output = executor
                .execute("mcp__alpha__echo", r#"{"text":"hello"}"#)
                .expect("discovered mcp tool should execute");
            let tool_json: serde_json::Value =
                serde_json::from_str(&tool_output).expect("tool output should be json");
            assert_eq!(tool_json["structuredContent"]["echoed"], "hello");
    
            let wrapped_output = executor
                .execute(
                    "MCPTool",
                    r#"{"qualifiedName":"mcp__alpha__echo","arguments":{"text":"wrapped"}}"#,
                )
                .expect("generic mcp wrapper should execute");
            let wrapped_json: serde_json::Value =
                serde_json::from_str(&wrapped_output).expect("wrapped output should be json");
            assert_eq!(wrapped_json["structuredContent"]["echoed"], "wrapped");
    
            let search_output = executor
                .execute("ToolSearch", r#"{"query":"alpha echo","max_results":5}"#)
                .expect("tool search should execute");
            let search_json: serde_json::Value =
                serde_json::from_str(&search_output).expect("search output should be json");
            assert_eq!(search_json["matches"][0], "mcp__alpha__echo");
            assert_eq!(search_json["pending_mcp_servers"][0], "broken");
            assert_eq!(
                search_json["mcp_degraded"]["failed_servers"][0]["server_name"],
                "broken"
            );
            assert_eq!(
                search_json["mcp_degraded"]["failed_servers"][0]["phase"],
                "tool_discovery"
            );
            assert_eq!(
                search_json["mcp_degraded"]["available_tools"][0],
                "mcp__alpha__echo"
            );
    
            let listed = executor
                .execute("ListMcpResourcesTool", r#"{"server":"alpha"}"#)
                .expect("resources should list");
            let listed_json: serde_json::Value =
                serde_json::from_str(&listed).expect("resource output should be json");
            assert_eq!(listed_json["resources"][0]["uri"], "file://guide.txt");
    
            let read = executor
                .execute(
                    "ReadMcpResourceTool",
                    r#"{"server":"alpha","uri":"file://guide.txt"}"#,
                )
                .expect("resource should read");
            let read_json: serde_json::Value =
                serde_json::from_str(&read).expect("resource read output should be json");
            assert_eq!(
                read_json["contents"][0]["text"],
                "contents for file://guide.txt"
            );
    
            if let Some(mcp_state) = state.mcp_state {
                mcp_state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .shutdown()
                    .expect("mcp shutdown should succeed");
            }
    
            let _ = fs::remove_dir_all(config_home);
            let _ = fs::remove_dir_all(workspace);
        }

    #[test]
        fn build_runtime_plugin_state_surfaces_unsupported_mcp_servers_structurally() {
            let config_home = temp_dir();
            let workspace = temp_dir();
            fs::create_dir_all(&config_home).expect("config home");
            fs::create_dir_all(&workspace).expect("workspace");
            fs::write(
                config_home.join("settings.json"),
                r#"{
                  "mcpServers": {
                    "remote": {
                      "url": "https://example.test/mcp"
                    }
                  }
                }"#,
            )
            .expect("write mcp settings");
    
            let loader = ConfigLoader::new(&workspace, &config_home);
            let runtime_config = loader.load().expect("runtime config should load");
            let state = build_runtime_plugin_state_with_loader(&workspace, &loader, &runtime_config)
                .expect("runtime plugin state should load");
            let mut executor = CliToolExecutor::new(
                None,
                false,
                state.tool_registry.clone(),
                state.mcp_state.clone(),
            );
    
            let search_output = executor
                .execute("ToolSearch", r#"{"query":"remote","max_results":5}"#)
                .expect("tool search should execute");
            let search_json: serde_json::Value =
                serde_json::from_str(&search_output).expect("search output should be json");
            assert_eq!(search_json["pending_mcp_servers"][0], "remote");
            assert_eq!(
                search_json["mcp_degraded"]["failed_servers"][0]["server_name"],
                "remote"
            );
            assert_eq!(
                search_json["mcp_degraded"]["failed_servers"][0]["phase"],
                "server_registration"
            );
            assert_eq!(
                search_json["mcp_degraded"]["failed_servers"][0]["error"]["context"]["transport"],
                "http"
            );
    
            let _ = fs::remove_dir_all(config_home);
            let _ = fs::remove_dir_all(workspace);
        }

    #[test]
        fn build_runtime_runs_plugin_lifecycle_init_and_shutdown() {
            // Serialize access to process-wide env vars so parallel tests that
            // set/remove ANTHROPIC_API_KEY do not race with this test.
            let _guard = env_lock();
            let config_home = temp_dir();
            // Inject a dummy API key so runtime construction succeeds without real credentials.
            // This test only exercises plugin lifecycle (init/shutdown), never calls the API.
            std::env::set_var("ANTHROPIC_API_KEY", "test-dummy-key-for-plugin-lifecycle");
            let workspace = temp_dir();
            let source_root = temp_dir();
            fs::create_dir_all(&config_home).expect("config home");
            fs::create_dir_all(&workspace).expect("workspace");
            fs::create_dir_all(&source_root).expect("source root");
            write_plugin_fixture(&source_root, "lifecycle-runtime-demo", false, true);
    
            let mut manager = PluginManager::new(PluginManagerConfig::new(&config_home));
            let install = manager
                .install(source_root.to_str().expect("utf8 source path"))
                .expect("plugin install should succeed");
            let log_path = install.install_path.join("lifecycle.log");
            let loader = ConfigLoader::new(&workspace, &config_home);
            let runtime_config = loader.load().expect("runtime config should load");
            let runtime_plugin_state =
                build_runtime_plugin_state_with_loader(&workspace, &loader, &runtime_config)
                    .expect("plugin state should load");
            let mut runtime = build_runtime_with_plugin_state(
                Session::new(),
                "runtime-plugin-lifecycle",
                DEFAULT_MODEL.to_string(),
                vec!["test system prompt".to_string()],
                true,
                false,
                None,
                PermissionMode::DangerFullAccess,
                None,
                runtime_plugin_state,
            )
            .expect("runtime should build");
    
            assert_eq!(
                fs::read_to_string(&log_path).expect("init log should exist"),
                "init\n"
            );
    
            runtime
                .shutdown_plugins()
                .expect("plugin shutdown should succeed");
    
            assert_eq!(
                fs::read_to_string(&log_path).expect("shutdown log should exist"),
                "init\nshutdown\n"
            );
    
            let _ = fs::remove_dir_all(config_home);
            let _ = fs::remove_dir_all(workspace);
            let _ = fs::remove_dir_all(source_root);
            std::env::remove_var("ANTHROPIC_API_KEY");
        }
}
