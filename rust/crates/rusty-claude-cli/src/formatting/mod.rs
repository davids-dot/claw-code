pub(crate) mod helpers;
pub(crate) mod reports;
pub(crate) mod tools;

pub(crate) use helpers::*;
pub(crate) use reports::*;
pub(crate) use tools::*;

pub(crate) fn full_help_text() -> String {
    reports::full_help_text()
}

pub(crate) const DISPLAY_TRUNCATION_NOTICE: &str =
    "\x1b[2m… output truncated for display; full result preserved in session.\x1b[0m";
pub(crate) const READ_DISPLAY_MAX_LINES: usize = 80;
pub(crate) const READ_DISPLAY_MAX_CHARS: usize = 6_000;
pub(crate) const TOOL_OUTPUT_DISPLAY_MAX_LINES: usize = 60;
pub(crate) const TOOL_OUTPUT_DISPLAY_MAX_CHARS: usize = 4_000;
pub(crate) const SESSION_MARKDOWN_TOOL_SUMMARY_LIMIT: usize = 280;
