import re

def main():
    with open('crates/rusty-claude-cli/src/main_tests.rs', 'r') as f:
        main_content = f.read()
        
    helpers = []
    
    # Extract env_lock
    env_lock_match = re.search(r'fn env_lock\(\) -> MutexGuard<\'static, \(\)> \{.*?\n    \}', main_content, re.DOTALL)
    if env_lock_match:
        helpers.append(env_lock_match.group(0))
        
    # Extract temp_dir
    temp_dir_match = re.search(r'fn temp_dir\(\) -> std::path::PathBuf \{.*?\n    \}', main_content, re.DOTALL)
    if not temp_dir_match:
        temp_dir_match = re.search(r'fn temp_dir\(\) -> PathBuf \{.*?\n    \}', main_content, re.DOTALL)
    if temp_dir_match:
        helpers.append(temp_dir_match.group(0))
        
    # Extract write_plugin_fixture
    wpf_match = re.search(r'fn write_plugin_fixture\(.*?\n    \}', main_content, re.DOTALL)
    if wpf_match:
        helpers.append(wpf_match.group(0))
        
    # Extract write_mcp_server_fixture
    wmsf_match = re.search(r'fn write_mcp_server_fixture\(.*?\n    \}', main_content, re.DOTALL)
    if wmsf_match:
        helpers.append(wmsf_match.group(0))
        
    with open('crates/rusty-claude-cli/src/execution/client.rs', 'r') as f:
        client_content = f.read()
        
    if helpers:
        helper_code = '\n    ' + '\n    '.join(helpers).replace('\n', '\n    ') + '\n'
        # Insert helpers right after mod tests {
        client_content = client_content.replace('mod tests {\n    use super::*;\n', 'mod tests {\n    use super::*;\n' + helper_code)
        
        # Add necessary imports
        imports = '    use std::path::{Path, PathBuf};\n    use std::sync::{MutexGuard, Mutex, OnceLock};\n    use std::env;\n'
        client_content = client_content.replace('use super::*;\n', 'use super::*;\n' + imports)
        
        with open('crates/rusty-claude-cli/src/execution/client.rs', 'w') as f:
            f.write(client_content)
            
if __name__ == '__main__':
    main()
