#[rustfmt::skip]
use ratatui::crossterm::event::{
    KeyCode,
    KeyModifiers
};
#[rustfmt::skip]
use edtui::EditorState;

// Truncate a string to a maximum width and appends "..." if it was cut off
pub fn truncate_with_ellipsis(text: &str, max_width: usize) -> String {
    if text.chars().count() <= max_width {
        return text.to_string();
    }

    // If max width is very small, just fill with dots
    if max_width <= 3 {
        return ".".repeat(max_width);
    }

    // Take first (max_width - 3) characters and append ellipsis
    let truncated: String = text.chars().take(max_width - 3).collect();
    format!("{truncated}...")
}

// Wrap text by character count without regard to word boundaries
pub fn wrap_chars(content: String, max_width: usize) -> Vec<String> {
    let mut wrapped_lines = Vec::new();

    for line in content.split('\n') {
        if line.is_empty() {
            // Preserve empty lines
            wrapped_lines.push(String::new());
            continue;
        }

        let mut start = 0;
        while start < line.len() {
            // Slice line in chunks up to max_width
            let end = (start + max_width).min(line.len());
            wrapped_lines.push(line[start..end].to_string());
            start = end;
        }
    }

    wrapped_lines
}

// Wrap text by words, preserving spaces and indentation
// Fall back to wrap_chars() when a word is longer than max_width
pub fn wrap_words(content: String, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![content.to_string()];
    }

    let mut wrapped_lines = Vec::new();

    for line in content.split('\n') {
        if line.is_empty() {
            wrapped_lines.push(String::new());
            continue;
        }

        let mut current_line = String::new();
        let mut current_width = 0;
        let mut chars = line.chars().peekable();

        while let Some(c) = chars.next() {
            if c.is_whitespace() {
                // Collect consecutive whitespace
                let mut space = String::from(c);
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_whitespace() {
                        space.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                // If spaces overflow line width, start new line
                if current_width + space.len() > max_width {
                    wrapped_lines.push(current_line);
                    current_line = space.clone();
                    current_width = space.len();
                } else {
                    current_line.push_str(&space);
                    current_width += space.len();
                }
            } else {
                // Collect a word
                let mut word = String::from(c);
                while let Some(&next_c) = chars.peek() {
                    if !next_c.is_whitespace() {
                        word.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                // Fallback: long word exceeds max_width → wrap by characters
                if word.len() > max_width {
                    if !current_line.is_empty() {
                        wrapped_lines.push(current_line);
                        current_line = String::new();
                        current_width = 0;
                    }

                    let char_wrapped = wrap_chars(word.clone(), max_width);
                    wrapped_lines.extend(char_wrapped);
                    continue;
                }

                // If word doesn't fit, push current line and start new one
                if current_width + word.len() > max_width {
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

        // Push the final line
        wrapped_lines.push(current_line);
    }

    wrapped_lines
}

// Center a single line of text within a given width by adding leading spaces
pub fn center_line(line: &str, width: usize) -> String {
    if line.len() >= width {
        line.to_string()
    } else {
        let padding = (width - line.len()) / 2;
        format!("{}{}", " ".repeat(padding), line)
    }
}

// Convert an edtui's line data into a single string
pub fn editor_state_to_string(state: &EditorState) -> String {
    state
        .lines
        .iter()
        .filter_map(|(c, _)| c.copied())
        .collect::<String>()
}

// Attempt to decode raw byte data into a string, handling UTF-8 and UTF-16 (LE/BE)
pub fn decode(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFF, 0xFE]) {
        // UTF-16 Little Endian with BOM
        let utf16: Vec<u16> = bytes[2..]
            .chunks(2)
            .map(|c| u16::from_le_bytes([c[0], *c.get(1).unwrap_or(&0)]))
            .collect();
        String::from_utf16(&utf16).unwrap_or_default()
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        // UTF-16 Big Endian with BOM
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
        // Default: UTF-8 or fallback to lossy decode
        String::from_utf8_lossy(bytes).to_string()
    }
}

// Clean and normalizes a string
pub fn sanitize(string: String) -> String {
    string
        .replace("\r\n", "\n")
        .replace("\r", "\n") // Convert Windows/Mac newlines to '\n'
        .chars()
        .flat_map(|character| match character {
            '\t' => "    ".chars().collect::<Vec<_>>(), // Expand tabs
            '\n' => vec!['\n'],                         // keep newlines
            character if character.is_control() => vec![], // remove other control chars
            _ => vec![character],                       // Keep the rest of the characters
        })
        .collect()
}

// A helper to convert KeyCode to string
pub fn keycode_to_string(code: &KeyCode) -> String {
    match code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".into(),
        KeyCode::Tab => "Tab".into(),
        KeyCode::Backspace => "Backspace".into(),
        KeyCode::Esc => "Esc".into(),
        KeyCode::Up => "Up".into(),
        KeyCode::Down => "Down".into(),
        KeyCode::Left => "Left".into(),
        KeyCode::Right => "Right".into(),
        KeyCode::Home => "Home".into(),
        KeyCode::End => "End".into(),
        KeyCode::PageUp => "Pageup".into(),
        KeyCode::PageDown => "Pagedown".into(),
        KeyCode::F(n) => format!("F{}", n),
        _ => "?".into(),
    }
}

// A helper to convert KeyModifiers to string
pub fn modifiers_to_string(mods: KeyModifiers) -> String {
    let mut parts = Vec::new();
    if mods.contains(KeyModifiers::CONTROL) { parts.push("Ctrl"); }
    if mods.contains(KeyModifiers::SHIFT) { parts.push("Shift"); }
    if mods.contains(KeyModifiers::ALT) { parts.push("Alt"); }
    parts.join(" + ")
}

pub fn pascal_to_spaced(s: &str) -> String {
    let mut result = String::new();

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i != 0 {
            result.push(' ');
        }
        result.push(c);
    }

    result
}
