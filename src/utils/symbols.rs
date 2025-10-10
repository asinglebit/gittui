#[rustfmt::skip]
use edtui::EditorState;

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
    if text.chars().count() <= max_width {
        return text.to_string();
    }

    if max_width <= 3 {
        return ".".repeat(max_width);
    }

    let truncated: String = text.chars().take(max_width - 3).collect();
    format!("{truncated}...")
}

pub fn wrap_chars(content: String, max_width: usize) -> Vec<String> {
    let mut wrapped_lines = Vec::new();
    let mut start = 0;
    while start < content.len() {
        let end = (start + max_width).min(content.len());
        wrapped_lines.push(content[start..end].to_string());
        start = end;
    }
    wrapped_lines
}

pub fn wrap_words(content: String, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![content];
    }

    let mut wrapped_lines = Vec::new();
    let mut current_line = String::new();

    for word in content.split_whitespace() {
        // If adding this word exceeds max_width
        if !current_line.is_empty() && current_line.len() + 1 + word.len() > max_width {
            wrapped_lines.push(current_line.clone());
            current_line.clear();
        }

        // If word itself is too long, split it directly
        if word.len() > max_width {
            if !current_line.is_empty() {
                wrapped_lines.push(current_line.clone());
                current_line.clear();
            }

            let mut start = 0;
            while start < word.len() {
                let end = (start + max_width).min(word.len());
                wrapped_lines.push(word[start..end].to_string());
                start = end;
            }
        } else {
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
    }

    if !current_line.is_empty() {
        wrapped_lines.push(current_line);
    }

    wrapped_lines
}

pub fn clean_commit_text(raw: &str, max_width: usize) -> Vec<String> {
    let mut cleaned_lines = Vec::new();

    // Normalize and iterate over lines
    for line in raw.replace("\r\n", "\n").lines() {
        let trimmed = line.trim_end().to_string();

        // If the line is empty, preserve it as an empty string
        if trimmed.is_empty() {
            cleaned_lines.push(String::new());
            continue;
        }

        // Wrap and extend
        let wrapped = wrap_words(trimmed, max_width);
        cleaned_lines.extend(wrapped);
    }

    cleaned_lines
}

pub fn center_line(line: &str, width: usize) -> String {
    if line.len() >= width {
        line.to_string()
    } else {
        let padding = (width - line.len()) / 2;
        format!("{}{}", " ".repeat(padding), line)
    }
}
pub fn editor_state_to_string(state: &EditorState) -> String {
    state
        .lines
        .iter()
        .filter_map(|(c, _)| c.copied())
        .collect::<String>()
}
