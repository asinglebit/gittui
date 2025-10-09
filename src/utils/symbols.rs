pub const SYM_COMMIT_BRANCH: &str = "●";
pub const SYM_COMMIT: &str = "○";
pub const SYM_VERTICAL: &str = "│";
pub const SYM_VERTICAL_DOTTED: &str = "┊";
pub const SYM_HORIZONTAL: &str = "─";
pub const SYM_EMPTY: &str = " ";
pub const SYM_MERGE_LEFT_FROM: &str = "⎨";
pub const SYM_MERGE_RIGHT_FROM: &str = "╭";
pub const SYM_BRANCH_UP: &str = "╯";
pub const SYM_BRANCH_DOWN: &str = "╮";
pub const SYM_MERGE: &str = "•";
pub const SYM_UNCOMMITED: &str = "◌";

pub fn truncate_with_ellipsis(text: &str, max_width: usize) -> String {
    if text.len() <= max_width {
        text.to_string()
    } else if max_width <= 3 {
        ".".repeat(max_width)
    } else {
        format!("{}...", &text[..max_width - 3])
    }
}

pub fn wrap_line(content: String, max_width: usize) -> Vec<String> {
    let mut wrapped_lines = Vec::new();
    let mut start = 0;
    while start < content.len() {
        let end = (start + max_width).min(content.len());
        wrapped_lines.push(content[start..end].to_string());
        start = end;
    }
    wrapped_lines
}
