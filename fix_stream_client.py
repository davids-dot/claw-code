import os

with open('rust/crates/rusty-claude-cli/src/execution/stream.rs', 'r') as f:
    stream_content = f.read()

import re

# extract consume_stream from stream.rs
match = re.search(r'impl AnthropicRuntimeClient \{.*?\n\}', stream_content, re.MULTILINE | re.DOTALL)
if match:
    consume_stream_code = match.group(0)
    stream_content = stream_content.replace(consume_stream_code, "")
    
    with open('rust/crates/rusty-claude-cli/src/execution/stream.rs', 'w') as f:
        f.write(stream_content)
    
    with open('rust/crates/rusty-claude-cli/src/execution/client.rs', 'a') as f:
        f.write("\n" + consume_stream_code + "\n")

print("Fixed stream and client")
