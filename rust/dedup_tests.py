import os
import re

def get_all_tests_in_dir(directory):
    tests = set()
    for root, _, files in os.walk(directory):
        for file in files:
            if file.endswith('.rs') and file != 'main_tests.rs':
                path = os.path.join(root, file)
                with open(path, 'r') as f:
                    content = f.read()
                # Find all #[test] functions
                matches = re.findall(r'#\[(?:tokio::)?test\].*?(?:async )?fn\s+([a-zA-Z0-9_]+)\s*\(', content, re.DOTALL)
                tests.update(matches)
    return tests

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

def main():
    src_dir = 'crates/rusty-claude-cli/src'
    tests = get_all_tests_in_dir(src_dir)
    print(f"Found {len(tests)} tests in other files")
    remove_tests(os.path.join(src_dir, 'main_tests.rs'), tests)

if __name__ == '__main__':
    main()