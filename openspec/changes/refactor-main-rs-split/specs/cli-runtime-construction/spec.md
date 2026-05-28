## ADDED Requirements

### Requirement: Runtime construction shall be isolated in `runtime_builder.rs`

All runtime bootstrapping logic (building ConversationRuntime, plugins, MCP state)
SHALL be in `runtime_builder.rs`. This includes `BuiltRuntime`, `RuntimePluginState`,
`RuntimeMcpState`, `build_runtime`, `build_runtime_with_plugin_state`, all plugin
construction helpers, and `HookAbortMonitor`.

#### Scenario: All build_runtime variants are in runtime_builder.rs

- **WHEN** the refactoring is complete
- **THEN** `runtime_builder.rs` contains `build_runtime`,
  `build_runtime_with_plugin_state`, `build_runtime_plugin_state`,
  `build_runtime_plugin_state_with_loader`, `build_plugin_manager`,
  `build_system_prompt`, `build_runtime_mcp_state`

#### Scenario: runtime_builder.rs imports from api_client and tool_executor

- **WHEN** `runtime_builder.rs` constructs a `ConversationRuntime`
- **THEN** it uses `AnthropicRuntimeClient` from `api_client.rs` and
  `CliToolExecutor` from `tool_executor.rs`

### Requirement: BuiltRuntime shall be the single runtime lifecycle manager

`BuiltRuntime` SHALL own the `ConversationRuntime` option, plugin registry, and
MCP state. Its `Drop` implementation SHALL shut down plugins and MCP servers.

#### Scenario: BuiltRuntime::drop cleans up

- **WHEN** a `BuiltRuntime` is dropped
- **THEN** `shutdown_mcp()` and `shutdown_plugins()` are called
- **THEN** no plugin or MCP process is left running

### Requirement: AnthropicRuntimeClient shall be in `api_client.rs`

The API client adapter (`AnthropicRuntimeClient`) and its `ApiClient` trait impl
SHALL be in `api_client.rs`. Provider detection, auth resolution, and prompt
cache setup SHALL be contained in this module.

#### Scenario: api_client.rs contains AnthropicRuntimeClient

- **WHEN** the refactoring is complete
- **THEN** `api_client.rs` contains `AnthropicRuntimeClient` struct, its `new()`,
  its `ApiClient` trait implementation, and auth resolution helpers

### Requirement: CliToolExecutor shall be in `tool_executor.rs`

The tool execution adapter (`CliToolExecutor`) and its `ToolExecutor` trait impl
SHALL be in `tool_executor.rs`. This includes all MCP tool call helpers and the
`permission_policy` / `convert_messages` functions.

#### Scenario: tool_executor.rs contains CliToolExecutor

- **WHEN** the refactoring is complete
- **THEN** `tool_executor.rs` contains `CliToolExecutor`, `ToolSearchRequest`,
  `McpToolRequest`, `ListMcpResourcesRequest`, `ReadMcpResourceRequest`,
  and the `ToolExecutor` trait implementation

### Requirement: Permission prompting shall be in `permission.rs`

`CliPermissionPrompter` and its `PermissionPrompter` trait impl SHALL be in
`permission.rs`.

#### Scenario: permission.rs contains permission prompt logic

- **WHEN** the refactoring is complete
- **THEN** `permission.rs` contains `CliPermissionPrompter`, its `new()` and
  `decide()` methods, and `permission_policy` helper
