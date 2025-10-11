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

    for line in content.split('\n') {
        if line.is_empty() {
            // Preserve completely empty lines
            wrapped_lines.push(String::new());
            continue;
        }

        let mut start = 0;
        while start < line.len() {
            // Calculate end safely
            let end = (start + max_width).min(line.len());
            wrapped_lines.push(line[start..end].to_string());
            start = end;
        }
    }

    wrapped_lines
}
pub fn wrap_words(content: String, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![content.to_string()];
    }

    let mut wrapped_lines = Vec::new();

    for line in content.split('\n') {
        if line.is_empty() {
            // Preserve empty lines
            wrapped_lines.push(String::new());
            continue;
        }

        let mut current_line = String::new();
        let mut current_width = 0;

        // Split line into words **but keep spaces**
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if c.is_whitespace() {
                // Accumulate consecutive spaces
                let mut space = String::from(c);
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_whitespace() {
                        space.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                if current_width + space.len() > max_width {
                    wrapped_lines.push(current_line);
                    current_line = space.clone(); // preserve indentation/spaces
                    current_width = space.len();
                } else {
                    current_line.push_str(&space);
                    current_width += space.len();
                }
            } else {
                // Accumulate a word
                let mut word = String::from(c);
                while let Some(&next_c) = chars.peek() {
                    if !next_c.is_whitespace() {
                        word.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                if current_width + word.len() > max_width {
                    // Word doesn't fit on current line
                    if !current_line.is_empty() {
                        wrapped_lines.push(current_line);
                    }
                    current_line = word.clone();
                    current_width = word.len();
                } else {
                    current_line.push_str(&word);
                    current_width += word.len();
                }
            }
        }

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

pub fn decode_bytes(bytes: &[u8]) -> String {
    // Check BOM for UTF-16
    let decoded = if bytes.starts_with(&[0xFF, 0xFE]) {
        let utf16: Vec<u16> = bytes[2..]
            .chunks(2)
            .map(|c| u16::from_le_bytes([c[0], *c.get(1).unwrap_or(&0)]))
            .collect();
        String::from_utf16(&utf16).unwrap_or_default()
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        let utf16: Vec<u16> = bytes[2..]
            .chunks(2)
            .map(|c| u16::from_be_bytes([c[0], *c.get(1).unwrap_or(&0)]))
            .collect();
        String::from_utf16(&utf16).unwrap_or_default()
    } else if bytes.len() > 1 && bytes[1] == 0 {
        // Likely UTF-16 LE without BOM
        let utf16: Vec<u16> = bytes
            .chunks(2)
            .map(|c| u16::from_le_bytes([c[0], *c.get(1).unwrap_or(&0)]))
            .collect();
        String::from_utf16(&utf16).unwrap_or_default()
    } else {
        // Default UTF-8 fallback
        String::from_utf8_lossy(bytes).to_string()
    };

    // Step 2: normalize line endings to \n
    let decoded = decoded.replace("\r\n", "\n").replace("\r", "\n");

    // Step 3: sanitize characters and expand tabs to 4 spaces
    decoded
        .chars()
        .flat_map(|c| match c {
            '\t' => "    ".chars().collect::<Vec<_>>(), // expand tabs
            '\n' => vec!['\n'],
            c if c.is_control() => vec![], // remove other control chars
            _ => vec![c],
        })
        .collect()
}
