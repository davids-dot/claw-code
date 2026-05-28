import re

MAIN_FILE = "rust/crates/rusty-claude-cli/src/main.rs"

with open(MAIN_FILE, 'r') as f:
    content = f.read()

def extract_and_remove(pattern, content):
    match = re.search(pattern, content, re.MULTILINE | re.DOTALL)
    if match:
        extracted = match.group(0)
        content = content.replace(extracted, "")
        return extracted, content
    return None, content

anthropic_client_pattern = r'struct AnthropicRuntimeClient \{.*?\n\}\n\nimpl AnthropicRuntimeClient \{.*?\n\}\n\nimpl runtime::Client for AnthropicRuntimeClient \{.*?\n\}'
anthropic_client_code, content = extract_and_remove(anthropic_client_pattern, content)

build_runtime_pattern = r'fn build_runtime\(\n    session: Session,\n    session_id: &str,\n    model: String,\n    system_prompts: Vec<String>,\n    enable_tools: bool,\n    emit_output: bool,\n    allowed_tools: Option<AllowedToolSet>,\n    permission_mode: PermissionMode,\n    reasoning_effort: Option<String>,\n\) -> Result<Runtime, Box<dyn std::error::Error>> \{.*?\n\}'
build_runtime_code, content = extract_and_remove(build_runtime_pattern, content)

build_runtime_with_plugin_state_pattern = r'fn build_runtime_with_plugin_state\(\n    session: Session,\n    session_id: &str,\n    model: String,\n    system_prompts: Vec<String>,\n    enable_tools: bool,\n    emit_output: bool,\n    allowed_tools: Option<AllowedToolSet>,\n    permission_mode: PermissionMode,\n    reasoning_effort: Option<String>,\n    plugin_state: RuntimePluginState,\n\) -> Result<Runtime, Box<dyn std::error::Error>> \{.*?\n\}'
build_runtime_with_plugin_state_code, content = extract_and_remove(build_runtime_with_plugin_state_pattern, content)

build_runtime_plugin_state_pattern = r'fn build_runtime_plugin_state\(\) -> Result<RuntimePluginState, Box<dyn std::error::Error>> \{.*?\n\}'
build_runtime_plugin_state_code, content = extract_and_remove(build_runtime_plugin_state_pattern, content)

build_runtime_plugin_state_with_loader_pattern = r'fn build_runtime_plugin_state_with_loader\(\n    cwd: &Path,\n    loader: &ConfigLoader,\n    runtime_config: &runtime::RuntimeConfig,\n\) -> Result<RuntimePluginState, Box<dyn std::error::Error>> \{.*?\n\}'
build_runtime_plugin_state_with_loader_code, content = extract_and_remove(build_runtime_plugin_state_with_loader_pattern, content)

runtime_plugin_state_pattern = r'struct RuntimePluginState \{.*?\n\}'
runtime_plugin_state_code, content = extract_and_remove(runtime_plugin_state_pattern, content)

build_runtime_mcp_state_pattern = r'fn build_runtime_mcp_state\(\n    mcp_config: Option<&runtime::McpConfig>,\n    cwd: &Path,\n\) -> Option<Arc<Mutex<RuntimeMcpState>>> \{.*?\n\}'
build_runtime_mcp_state_code, content = extract_and_remove(build_runtime_mcp_state_pattern, content)

mcp_wrapper_tools_pattern = r'fn mcp_wrapper_tool_definitions\(\) -> Vec<RuntimeToolDefinition> \{.*?\n\}'
mcp_wrapper_tools_code, content = extract_and_remove(mcp_wrapper_tools_pattern, content)

mcp_runtime_tool_definition_pattern = r'fn mcp_runtime_tool_definition\(tool: &runtime::ManagedMcpTool\) -> RuntimeToolDefinition \{.*?\n\}'
mcp_runtime_tool_definition_code, content = extract_and_remove(mcp_runtime_tool_definition_pattern, content)

mcp_permission_mode_pattern = r'fn permission_mode_for_mcp_tool\(tool: &McpTool\) -> PermissionMode \{.*?\n\}'
mcp_permission_mode_code, content = extract_and_remove(mcp_permission_mode_pattern, content)

mcp_annotation_flag_pattern = r'fn mcp_annotation_flag\(tool: &McpTool, key: &str\) -> bool \{.*?\n\}'
mcp_annotation_flag_code, content = extract_and_remove(mcp_annotation_flag_pattern, content)

mcp_state_pattern = r'struct RuntimeMcpState \{.*?\n\}\n\nimpl RuntimeMcpState \{.*?\n\}'
mcp_state_code, content = extract_and_remove(mcp_state_pattern, content)

with open(MAIN_FILE, 'w') as f:
    f.write(content)

with open('client_extracted.rs', 'w') as f:
    blocks = [
        anthropic_client_code, build_runtime_code, build_runtime_with_plugin_state_code,
        build_runtime_plugin_state_code, build_runtime_plugin_state_with_loader_code,
        runtime_plugin_state_code, build_runtime_mcp_state_code, mcp_wrapper_tools_code,
        mcp_runtime_tool_definition_code, mcp_permission_mode_code, mcp_annotation_flag_code,
        mcp_state_code
    ]
    for block in blocks:
        if block:
            f.write(block + "\n\n")

print("Client extraction complete")
