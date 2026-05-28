import re

with open("rust/crates/rusty-claude-cli/src/ui/progress.rs", "r") as f:
    content = f.read()

content = re.sub(r"^struct (InternalPromptProgressState|InternalPromptProgressShared|InternalPromptProgressReporter|InternalPromptProgressRun|CliHookProgressReporter)", r"pub(crate) struct \1", content, flags=re.M)
content = re.sub(r"^enum (InternalPromptProgressEvent)", r"pub(crate) enum \1", content, flags=re.M)
content = re.sub(r"fn (ultraplan|emit|mark_model_phase|mark_tool_phase|mark_text_phase|emit_heartbeat|start_ultraplan|reporter|finish_success|finish_failure|stop_heartbeat)\(", r"pub(crate) fn \1(", content)
content = re.sub(r"^fn (describe_tool_progress|extract_tool_path|first_visible_line|summarize_tool_payload|truncate_for_summary)\(", r"pub(crate) fn \1(", content, flags=re.M)

prefix = """use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};
use std::thread;
use std::io::{self, Write};
use crate::INTERNAL_PROGRESS_HEARTBEAT_INTERVAL;
use crate::render::format_internal_prompt_progress_line;

"""

with open("rust/crates/rusty-claude-cli/src/ui/progress.rs", "w") as f:
    f.write(prefix + content)

print("done")