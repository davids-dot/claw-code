import re

with open('rust/crates/rusty-claude-cli/src/main_tests.rs', 'r') as f:
    content = f.read()

def extract_test(name):
    pattern = r'(#\[test\]\s+fn ' + name + r'\b.*?^    \})'
    match = re.search(pattern, content, re.MULTILINE | re.DOTALL)
    if match:
        return match.group(1)
    return None

tests = [
    "resolves_known_model_aliases",
    "user_defined_aliases_resolve_before_provider_dispatch",
    "resolve_repl_model_returns_user_supplied_model_unchanged_when_explicit",
    "resolve_repl_model_falls_back_to_anthropic_model_env_when_default",
    "resolve_repl_model_returns_default_when_env_unset_and_no_config",
    "permission_policy_uses_plugin_tool_permissions",
    "describe_tool_progress_summarizes_known_tools"
]

for test in tests:
    extracted = extract_test(test)
    if extracted:
        with open(f"{test}.rs_chunk", "w") as out:
            out.write(extracted)
    else:
        print(f"Test not found: {test}")

