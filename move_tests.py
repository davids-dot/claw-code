import re
import os

MAIN_TESTS_FILE = "rust/crates/rusty-claude-cli/src/main_tests.rs"
CONFIG_MODELS_FILE = "rust/crates/rusty-claude-cli/src/config/models.rs"
PERMISSIONS_PROMPTER_FILE = "rust/crates/rusty-claude-cli/src/permissions/prompter.rs"
UI_PROGRESS_FILE = "rust/crates/rusty-claude-cli/src/ui/progress.rs"

with open(MAIN_TESTS_FILE, 'r') as f:
    content = f.read()

def remove_test(name, content):
    pattern = r'#\[test\]\s+fn ' + name + r'\b.*?^    \}'
    new_content, count = re.subn(pattern, '', content, flags=re.MULTILINE | re.DOTALL)
    if count == 0:
        print(f"Warning: Test {name} not found to remove.")
    return new_content

tests_for_models = [
    "resolves_known_model_aliases",
    "user_defined_aliases_resolve_before_provider_dispatch",
    "resolve_repl_model_returns_user_supplied_model_unchanged_when_explicit",
    "resolve_repl_model_falls_back_to_anthropic_model_env_when_default",
    "resolve_repl_model_returns_default_when_env_unset_and_no_config",
]

for test in tests_for_models:
    content = remove_test(test, content)

content = remove_test("permission_policy_uses_plugin_tool_permissions", content)
content = remove_test("describe_tool_progress_summarizes_known_tools", content)

# Remove imports from main_tests.rs
imports_to_remove = [
    "resolve_model_alias,", "resolve_model_alias_with_config,", "resolve_repl_model,",
    "permission_policy,", "describe_tool_progress,"
]
for imp in imports_to_remove:
    content = content.replace(imp, "")

with open(MAIN_TESTS_FILE, 'w') as f:
    f.write(content)

# --- Append to src/config/models.rs ---
models_tests = []
for test in tests_for_models:
    with open(f"{test}.rs_chunk", "r") as f:
        models_tests.append("    " + f.read().replace("\n", "\n    ").strip())

models_test_mod = """
#[cfg(test)]
mod tests {
    use super::*;
    use runtime::test_utils::{env_lock, temp_dir, with_current_dir};

""" + "\n\n".join(models_tests) + "\n}\n"

with open(CONFIG_MODELS_FILE, 'a') as f:
    f.write(models_test_mod)

# --- Append to src/permissions/prompter.rs ---
with open("permission_policy_uses_plugin_tool_permissions.rs_chunk", "r") as f:
    prompter_test = "    " + f.read().replace("\n", "\n    ").strip()

prompter_test_mod = """
#[cfg(test)]
mod tests {
    use super::*;
    use plugins::{PluginTool, PluginToolDefinition, PluginToolPermission};
    use serde_json::json;
    use tools::GlobalToolRegistry;
    use runtime::{PermissionMode, RuntimeFeatureConfig};

    fn registry_with_plugin_tool() -> GlobalToolRegistry {
        GlobalToolRegistry::with_plugin_tools(vec![PluginTool::new(
            "plugin-demo@external",
            "plugin-demo",
            PluginToolDefinition {
                name: "plugin_echo".to_string(),
                description: Some("Echo plugin payload".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "message": { "type": "string" }
                    },
                    "required": ["message"],
                    "additionalProperties": false
                }),
            },
            "echo".to_string(),
            PluginToolPermission::WorkspaceWrite,
        )])
    }

""" + prompter_test + "\n}\n"

with open(PERMISSIONS_PROMPTER_FILE, 'a') as f:
    f.write(prompter_test_mod)

# --- Append to src/ui/progress.rs ---
with open("describe_tool_progress_summarizes_known_tools.rs_chunk", "r") as f:
    progress_test = "    " + f.read().replace("\n", "\n    ").strip()

progress_test_mod = """
#[cfg(test)]
mod tests {
    use super::*;

""" + progress_test + "\n}\n"

with open(UI_PROGRESS_FILE, 'a') as f:
    f.write(progress_test_mod)

print("Done moving tests.")
