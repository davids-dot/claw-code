def extract_tests(file_path, test_names, output_file):
    with open(file_path, 'r') as f:
        content = f.read()
        
    extracted_tests = []
    
    for name in test_names:
        # Find the function definition
        func_def = f'fn {name}('
        idx = content.find(func_def)
        if idx == -1:
            print(f"Warning: Test {name} not found")
            continue
            
        # Backtrack to find the preceding #[test]
        test_attr_idx = content.rfind('#[test]', 0, idx)
        if test_attr_idx == -1:
            test_attr_idx = content.rfind('#[tokio::test]', 0, idx)
        
        if test_attr_idx == -1:
            print(f"Warning: #[test] not found for {name}")
            continue
            
        start_idx = test_attr_idx
        
        # Find the opening brace of the function
        brace_idx = content.find('{', idx)
        if brace_idx == -1:
            print(f"Warning: opening brace not found for {name}")
            continue
            
        brace_count = 1
        curr_idx = brace_idx + 1
        
        while brace_count > 0 and curr_idx < len(content):
            if content[curr_idx] == '{':
                brace_count += 1
            elif content[curr_idx] == '}':
                brace_count -= 1
            curr_idx += 1
            
        end_idx = curr_idx
        extracted_tests.append(content[start_idx:end_idx])
        content = content[:start_idx] + content[end_idx:]
        print(f"Extracted {name}")
        
    if extracted_tests:
        with open(output_file, 'a') as f:
            f.write('\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n')
            for t in extracted_tests:
                f.write('\n    ' + t.replace('\n', '\n    ') + '\n')
            f.write('}\n')

    with open(file_path, 'w') as f:
        f.write(content)

client_test_names = [
    'build_runtime_plugin_state_merges_plugin_hooks_into_runtime_features',
    'build_runtime_plugin_state_discovers_mcp_tools_and_surfaces_pending_servers',
    'build_runtime_plugin_state_surfaces_unsupported_mcp_servers_structurally',
    'build_runtime_runs_plugin_lifecycle_init_and_shutdown'
]

stream_test_names = [
    'response_to_events_preserves_empty_object_json_input_outside_streaming',
    'response_to_events_preserves_non_empty_json_input_outside_streaming',
    'response_to_events_renders_collapsed_thinking_summary'
]

executor_test_names = [
    'tool_rendering_helpers_compact_output',
    'tool_rendering_truncates_large_read_output_for_display_only',
    'tool_rendering_truncates_large_bash_output_for_display_only',
    'tool_rendering_truncates_generic_long_output_for_display_only',
    'tool_rendering_truncates_raw_generic_output_for_display_only',
    'short_tool_id_truncates_long_identifiers_with_ellipsis'
]

extract_tests('crates/rusty-claude-cli/src/main_tests.rs', client_test_names, 'crates/rusty-claude-cli/src/execution/client.rs')
extract_tests('crates/rusty-claude-cli/src/main_tests.rs', stream_test_names, 'crates/rusty-claude-cli/src/execution/stream.rs')
extract_tests('crates/rusty-claude-cli/src/main_tests.rs', executor_test_names, 'crates/rusty-claude-cli/src/execution/executor.rs')
