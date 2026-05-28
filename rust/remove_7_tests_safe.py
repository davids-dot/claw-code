def remove_tests(file_path, tests_to_remove):
    with open(file_path, 'r') as f:
        content = f.read()
        
    for name in tests_to_remove:
        # Find the function definition
        func_def = f'fn {name}('
        idx = content.find(func_def)
        if idx == -1:
            print(f"Warning: Test {name} not found")
            continue
            
        # Backtrack to find the preceding #[test]
        test_attr_idx = content.rfind('#[test]', 0, idx)
        if test_attr_idx == -1:
            # maybe tokio::test?
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
        content = content[:start_idx] + content[end_idx:]
        print(f"Removed {name} from {file_path}")
        
    with open(file_path, 'w') as f:
        f.write(content)

with open('dup_tests.txt', 'r') as f:
    tests = [line.strip() for line in f if line.strip()]

remove_tests('crates/rusty-claude-cli/src/main_tests.rs', tests)
