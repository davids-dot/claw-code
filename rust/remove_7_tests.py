import re

def remove_tests(file_path, tests_to_remove):
    with open(file_path, 'r') as f:
        content = f.read()
        
    for name in tests_to_remove:
        # Find the function definition
        pattern = re.compile(r'(#\[(?:tokio::)?test\].*?(?:async )?fn ' + name + r'\s*\([^)]*\)\s*\{)', re.DOTALL)
        match = pattern.search(content)
        if not match:
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
        content = content[:start_idx] + content[end_idx:]
        print(f"Removed {name} from {file_path}")
        
    with open(file_path, 'w') as f:
        f.write(content)

with open('dup_tests.txt', 'r') as f:
    tests = [line.strip() for line in f if line.strip()]

remove_tests('crates/rusty-claude-cli/src/main_tests.rs', tests)
