import re
import sys

def main():
    with open('crates/rusty-claude-cli/src/main_tests.rs', 'r') as f:
        content = f.read()

    # Tests for client.rs
    client_test_names = [
        'build_runtime_plugin_state_merges_plugin_hooks_into_runtime_features',
        'build_runtime_plugin_state_discovers_mcp_tools_and_surfaces_pending_servers',
        'build_runtime_plugin_state_surfaces_unsupported_mcp_servers_structurally',
        'build_runtime_runs_plugin_lifecycle_init_and_shutdown'
    ]

    # Tests for stream.rs
    stream_test_names = [
        'response_to_events_preserves_empty_object_json_input_outside_streaming',
        'response_to_events_preserves_non_empty_json_input_outside_streaming',
        'response_to_events_renders_collapsed_thinking_summary'
    ]

    # Tests for executor.rs (tool rendering)
    executor_test_names = [
        'tool_rendering_helpers_compact_output',
        'tool_rendering_truncates_large_read_output_for_display_only',
        'tool_rendering_truncates_large_bash_output_for_display_only',
        'tool_rendering_truncates_generic_long_output_for_display_only',
        'tool_rendering_truncates_raw_generic_output_for_display_only',
        'short_tool_id_truncates_long_identifiers_with_ellipsis'
    ]

    def extract_and_remove(test_names, output_file):
        nonlocal content
        extracted_tests = []
        for name in test_names:
            # Find the function definition
            pattern = re.compile(r'(#\[(?:tokio::)?test\].*?(?:async )?fn ' + name + r'\s*\([^)]*\)\s*\{)', re.DOTALL)
            match = pattern.search(content)
            if not match:
                print(f"Warning: Test {name} not found")
                continue
                
            start_idx = match.start()
            brace_count = 1
            curr_idx = match.end()
            
            while brace_count > 0 and curr_idx < len(content):
                if content[curr_idx] == '{':
                    brace_count += 1
                elif content[curr_idx] == '}':
                    brace_count -= 1
                curr_idx += 1
                
            end_idx = curr_idx
            extracted_tests.append(content[start_idx:end_idx])
            content = content[:start_idx] + content[end_idx:]
            
        if extracted_tests:
            with open(output_file, 'a') as f:
                f.write('\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n')
                for t in extracted_tests:
                    f.write('\n    ' + t.replace('\n', '\n    ') + '\n')
                f.write('}\n')

    extract_and_remove(client_test_names, 'crates/rusty-claude-cli/src/execution/client.rs')
    extract_and_remove(stream_test_names, 'crates/rusty-claude-cli/src/execution/stream.rs')
    extract_and_remove(executor_test_names, 'crates/rusty-claude-cli/src/execution/executor.rs')

    with open('crates/rusty-claude-cli/src/main_tests.rs', 'w') as f:
        f.write(content)

if __name__ == '__main__':
    main()