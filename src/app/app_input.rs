#[rustfmt::skip]
use std::{
    io,
};
#[rustfmt::skip]
use crossterm::event::{
    self,
    Event,
    KeyCode,
    KeyEvent,
    KeyEventKind,
    KeyModifiers
};
#[rustfmt::skip]
use git2::Oid;
#[rustfmt::skip]
use edtui::{
    EditorMode,
};
#[rustfmt::skip]
use ratatui::{
    style::Style,
    text::{
        Line,
        Span
    },
    widgets::ListItem
};
#[rustfmt::skip]
use crate::{
    utils::colors::*,
    app::app::{
        App,
        Focus,
        Viewport
    },
    git::{
        actions::{
            checkout_head,
            checkout_branch,
            commit_staged,
            git_add_all,
            reset_to_commit,
            unstage_all
        },
        queries::{
            get_changed_filenames,
            get_file_diff,
            get_file_lines_at_commit,
            get_file_lines_in_workdir,
            get_uncommitted_file_diff
        }
    },
    utils::symbols::{
        editor_state_to_string,
        wrap_words
    }
};

impl App {
    pub fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    pub fn update_viewer(&mut self, oid: Oid) {
        let filename = self.file_name.clone().unwrap();

        // Decide whether to use committed or uncommitted version
        let (original_lines, hunks) = if oid == Oid::zero() {
            (
                get_file_lines_in_workdir(&self.repo, &filename),
                get_uncommitted_file_diff(&self.repo, &filename).unwrap_or_default(),
            )
        } else {
            (
                get_file_lines_at_commit(&self.repo, oid, &filename),
                get_file_diff(&self.repo, oid, &filename).unwrap_or_default(),
            )
        };

        self.viewer_lines.clear();
        let mut current_line: usize = 0;
        let mut current_line_old: usize = 0;

        for hunk in hunks.iter() {
            // Parse hunk header to extract start line and length for the old file.
            // Example header: "@@ -22,8 +22,14 @@"
            let header = &hunk.header;
            let (old_start, _old_len) = header
                .split_whitespace()
                .nth(1) // "-22,8"
                .and_then(|s| s.strip_prefix('-'))
                .and_then(|s| {
                    let mut parts = s.split(',');
                    Some((
                        parts.next()?.parse::<usize>().ok()?,
                        parts
                            .next()
                            .and_then(|n| n.parse::<usize>().ok())
                            .unwrap_or(0),
                    ))
                })
                .unwrap_or((1, 0));
            let old_start_idx = old_start.saturating_sub(1);

            // Add unchanged lines before this hunk
            while current_line < old_start_idx && current_line < original_lines.len() {
                let wrapped = wrap_words(
                    original_lines[current_line].clone(),
                    (self.layout.graph.width as usize).saturating_sub(8),
                );
                let mut i = 0;
                for line in wrapped {
                    self.viewer_lines.push(ListItem::new(
                        Line::from(vec![
                            Span::styled(
                                format!(
                                    "{}",
                                    if i == 0 {
                                        format!("{:3}  ", current_line + 1)
                                    } else {
                                        format!("     ")
                                    }
                                ),
                                Style::default().fg(COLOR_BORDER),
                            ),
                            Span::styled(format!("{}", line), Style::default().fg(COLOR_GREY_500)),
                        ])
                        .style(Style::default()),
                    ));
                    i += 1;
                }
                current_line += 1;
                current_line_old += 1;
            }
            
            // Process lines in the hunk
            for line in hunk.lines.iter().filter(|l| l.origin != 'H') {
                let text = line.content.trim_end_matches('\n');

                match line.origin {
                    '-' => {
                        let wrapped = wrap_words(
                            format!("- {}", text),
                            (self.layout.graph.width as usize).saturating_sub(9),
                        );
                        let mut i = 0;
                        for line in wrapped {
                            self.viewer_lines.push(
                                ListItem::new(Line::from(vec![
                                    Span::styled(
                                        format!(
                                            "{}",
                                            if i == 0 {
                                                format!("{:3}  ", current_line_old + 1)
                                            } else {
                                                format!("     ")
                                            }
                                        ),
                                        Style::default().fg(COLOR_RED),
                                    ),
                                    Span::styled(
                                        format!("{}", line),
                                        Style::default().fg(COLOR_RED),
                                    ),
                                ]))
                                .style(Style::default().bg(COLOR_DARK_RED).fg(COLOR_RED)),
                            );
                            i += 1;
                        }
                        current_line_old += 1;
                    }
                    '+' => {
                        let wrapped = wrap_words(
                            format!("+ {}", text),
                            (self.layout.graph.width as usize).saturating_sub(9),
                        );
                        let mut i = 0;
                        for line in wrapped {
                            self.viewer_lines.push(
                                ListItem::new(Line::from(vec![
                                    Span::styled(
                                        format!(
                                            "{}",
                                            if i == 0 {
                                                format!("{:3}  ", current_line + 1)
                                            } else {
                                                format!("     ")
                                            }
                                        ),
                                        Style::default().fg(COLOR_GREEN),
                                    ),
                                    Span::styled(
                                        format!("{}", line),
                                        Style::default().fg(COLOR_GREEN),
                                    ),
                                ]))
                                .style(Style::default().bg(COLOR_LIGHT_GREEN_900).fg(COLOR_GREEN)),
                            );
                            i += 1;
                        }
                        current_line += 1;
                    }
                    ' ' => {
                        let wrapped = wrap_words(
                            text.to_string(),
                            (self.layout.graph.width as usize).saturating_sub(9),
                        );
                        let mut i = 0;
                        for line in wrapped {
                            self.viewer_lines.push(
                                ListItem::new(Line::from(vec![
                                    Span::styled(
                                        format!(
                                            "{}",
                                            if i == 0 {
                                                format!("{:3}  ", current_line + 1)
                                            } else {
                                                format!("     ")
                                            }
                                        ),
                                        Style::default().fg(COLOR_BORDER),
                                    ),
                                    Span::styled(
                                        format!("{}", line),
                                        Style::default().fg(COLOR_GREY_500),
                                    ),
                                ]))
                                .style(Style::default()),
                            );
                            i += 1;
                        }
                        current_line += 1;
                        current_line_old += 1;
                    }
                    _ => {}
                }
            }
        }

        // Add remaining lines after the last hunk
        while current_line < original_lines.len() {
            let wrapped = wrap_words(
                original_lines[current_line].clone(),
                (self.layout.graph.width as usize).saturating_sub(8),
            );
            let mut i = 0;
            for line in wrapped {
                self.viewer_lines.push(
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!(
                                "{}",
                                if i == 0 {
                                    format!("{:3}  ", current_line + 1)
                                } else {
                                    format!("     ")
                                }
                            ),
                            Style::default().fg(COLOR_BORDER),
                        ),
                        Span::styled(format!("{}", line), Style::default().fg(COLOR_GREY_500)),
                    ]))
                    .style(Style::default()),
                );
                i += 1;
            }
            current_line += 1;
            current_line_old += 1;
        }
    }

    pub fn open_viewer(&mut self) {
        match self.focus {
            Focus::StatusTop => {
                if self.graph_selected != 0 && self.current_diff.len() > 0 {
                    self.file_name = Some(
                        self.current_diff
                            .get(self.status_top_selected)
                            .unwrap()
                            .filename
                            .to_string(),
                    );
                    self.update_viewer(self.oids.get(self.graph_selected).unwrap().clone());
                    self.viewport = Viewport::Viewer;
                } else if self.graph_selected == 0 && self.uncommitted.is_staged {
                    let modified_len = self.uncommitted.staged.modified.len();
                    let added_len = self.uncommitted.staged.added.len();
                    let index = self.status_top_selected;
                    self.file_name = if index < modified_len {
                        self.uncommitted.staged.modified.get(index).cloned()
                    } else if index < modified_len + added_len {
                        self.uncommitted
                            .staged
                            .added
                            .get(index - modified_len)
                            .cloned()
                    } else {
                        self.uncommitted
                            .staged
                            .deleted
                            .get(index - modified_len - added_len)
                            .cloned()
                    };
                    self.update_viewer(Oid::zero());
                    self.viewport = Viewport::Viewer;
                }
            }
            Focus::StatusBottom => {
                if self.graph_selected == 0 && self.uncommitted.is_unstaged {
                    let modified_len = self.uncommitted.unstaged.modified.len();
                    let added_len = self.uncommitted.unstaged.added.len();
                    let index = self.status_bottom_selected;
                    self.file_name = if index < modified_len {
                        self.uncommitted.unstaged.modified.get(index).cloned()
                    } else if index < modified_len + added_len {
                        self.uncommitted
                            .unstaged
                            .added
                            .get(index - modified_len)
                            .cloned()
                    } else {
                        self.uncommitted
                            .unstaged
                            .deleted
                            .get(index - modified_len - added_len)
                            .cloned()
                    };
                    self.update_viewer(Oid::zero());
                    self.viewport = Viewport::Viewer;
                }
            }
            _ => {}
        }
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        // Handle text editing
        match self.focus {
            Focus::ModalCommit => {
                if self.commit_editor.mode == EditorMode::Normal {
                    match key_event.code {
                        KeyCode::Esc => {
                            self.focus = Focus::Viewport;
                        }
                        KeyCode::Enter => {
                            commit_staged(
                                &self.repo,
                                &editor_state_to_string(&self.commit_editor),
                                &self.name,
                                &self.email,
                            )
                            .expect("Error");
                            self.reload();
                            self.focus = Focus::Viewport;
                        }
                        _ => {
                            self.commit_editor_event_handler
                                .on_key_event(key_event, &mut self.commit_editor);
                        }
                    }
                } else {
                    self.commit_editor_event_handler
                        .on_key_event(key_event, &mut self.commit_editor);
                }
                return;
            }
            Focus::Viewport => {
                if self.viewport == Viewport::Editor {
                    if self.file_editor.mode == EditorMode::Normal {
                        match key_event.code {
                            KeyCode::Char('c')
                                if key_event.modifiers.contains(KeyModifiers::CONTROL) => {}
                            KeyCode::Char('f') => {}
                            KeyCode::Char('s') => {}
                            KeyCode::Char('i') => {}
                            KeyCode::Esc => {
                                self.viewport = Viewport::Graph;
                            }
                            _ => {
                                self.file_editor_event_handler
                                    .on_key_event(key_event, &mut self.file_editor);
                            }
                        }
                    } else {
                        self.file_editor_event_handler
                            .on_key_event(key_event, &mut self.file_editor);
                        return;
                    }
                }
            }
            _ => {}
        }

        // Handle the application
        match key_event.code {
            KeyCode::Char('r') => {
                self.reload();
                match self.focus {
                    Focus::ModalCheckout | Focus::ModalActions | Focus::ModalCommit => {
                        self.focus = Focus::Viewport;
                    }
                    _ => {}
                }
            }
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.exit()
            }
            KeyCode::Char('j') | KeyCode::Down => match self.focus {
                Focus::Viewport => match self.viewport {
                    Viewport::Graph => {
                        if self.graph_selected + 1 < self.lines_branches.len() {
                            self.graph_selected += 1;
                        }
                        if self.graph_selected != 0 {
                            let oid = self.oids.get(self.graph_selected).unwrap();
                            self.current_diff = get_changed_filenames(&self.repo, *oid);
                        }
                    }
                    Viewport::Viewer => {
                        if self.viewer_selected + 1 < self.viewer_lines.len() {
                            self.viewer_selected += 1;
                        }
                    }
                    _ => {}
                },
                Focus::Inspector => {
                    self.inspector_selected += 1;
                }
                Focus::StatusTop => {
                    self.status_top_selected += 1;
                }
                Focus::StatusBottom => {
                    self.status_bottom_selected += 1;
                }
                Focus::ModalCheckout => {
                    let branches = self
                        .tips
                        .entry(*self.oids.get(self.graph_selected).unwrap())
                        .or_default();
                    self.modal_checkout_selected =
                        if self.modal_checkout_selected + 1 > branches.len() as i32 - 1 {
                            0
                        } else {
                            self.modal_checkout_selected + 1
                        };
                }
                _ => {}
            },
            KeyCode::Char('k') | KeyCode::Up => match self.focus {
                Focus::Viewport => {
                    match self.viewport {
                        Viewport::Graph => {
                            if self.graph_selected > 0 {
                                self.graph_selected -= 1;
                                if self.graph_selected == 0 && self.focus == Focus::Inspector {
                                    self.focus = Focus::Viewport;
                                }
                            }
                            if self.graph_selected != 0 {
                                let oid = self.oids.get(self.graph_selected).unwrap();
                                self.current_diff = get_changed_filenames(&self.repo, *oid);
                            }
                        }
                        Viewport::Viewer => {
                            if self.viewer_selected > 0 {
                                self.viewer_selected -= 1;
                            }
                        }
                        _ => {}
                    }
                    if self.viewport == Viewport::Graph {}
                }
                Focus::Inspector => {
                    if self.inspector_selected > 0 {
                        self.inspector_selected -= 1;
                    }
                }
                Focus::StatusTop => {
                    if self.status_top_selected > 0 {
                        self.status_top_selected -= 1;
                    }
                }
                Focus::StatusBottom => {
                    if self.status_bottom_selected > 0 {
                        self.status_bottom_selected -= 1;
                    }
                }
                Focus::ModalCheckout => {
                    let branches = self
                        .tips
                        .entry(*self.oids.get(self.graph_selected).unwrap())
                        .or_default();
                    self.modal_checkout_selected = if self.modal_checkout_selected - 1 < 0 {
                        branches.len() as i32 - 1
                    } else {
                        self.modal_checkout_selected - 1
                    };
                }
                _ => {}
            },
            KeyCode::Char('f') => {
                self.is_minimal = !self.is_minimal;
            }
            KeyCode::Char('s') => {
                self.is_status = !self.is_status;
                if !self.is_status
                    && (self.focus == Focus::StatusTop || self.focus == Focus::StatusBottom)
                {
                    self.focus = Focus::Viewport;
                }
            }
            KeyCode::Char('i') => {
                self.is_inspector = !self.is_inspector;
                if !self.is_inspector && self.focus == Focus::Inspector {
                    if self.is_status {
                        self.focus = Focus::StatusTop;
                    } else {
                        self.focus = Focus::Viewport;
                    }
                }
            }
            KeyCode::Char('x') => {
                match self.focus {
                    Focus::ModalActions | Focus::ModalCommit => {
                        self.focus = Focus::Viewport;
                    }
                    Focus::ModalCheckout => {
                        self.modal_checkout_selected = 0;
                        self.focus = Focus::Viewport;
                    }
                    _ => {}
                };
            }
            KeyCode::Char('c') => {
                match self.focus {
                    Focus::Viewport | Focus::ModalActions => {
                        if self.focus == Focus::Viewport && self.viewport != Viewport::Graph {
                            return;
                        }

                        if self.graph_selected == 0 && self.uncommitted.is_staged {
                            self.focus = Focus::ModalCommit;
                            self.commit_editor.mode = EditorMode::Insert;
                            return;
                        }
                        let branches = self
                            .tips
                            .entry(*self.oids.get(self.graph_selected).unwrap())
                            .or_default();
                        if self.graph_selected == 0 {
                            self.focus = Focus::Viewport;
                            return;
                        }
                        if branches.is_empty() {
                            checkout_head(&self.repo, *self.oids.get(self.graph_selected).unwrap());
                            self.focus = Focus::Viewport;
                            self.reload();
                        } else if branches.len() == 1 {
                            checkout_branch(&self.repo, branches.first().unwrap()).expect("Error");
                            self.focus = Focus::Viewport;
                            self.reload();
                        } else {
                            self.focus = Focus::ModalCheckout;
                            return;
                        }
                    }
                    _ => {}
                };
            }
            KeyCode::Char('h') => match self.focus {
                Focus::Viewport | Focus::ModalActions => {
                    if self.focus == Focus::Viewport && self.viewport != Viewport::Graph {
                        return;
                    }
                    let oid = self.oids.get(self.graph_selected).unwrap();
                    reset_to_commit(&self.repo, *oid, git2::ResetType::Hard).expect("Error");
                    self.reload();
                    self.focus = Focus::Viewport;
                }
                _ => {}
            },
            KeyCode::Char('m') => match self.focus {
                Focus::Viewport | Focus::ModalActions => {
                    if self.focus == Focus::Viewport && self.viewport != Viewport::Graph {
                        return;
                    }
                    let oid = self.oids.get(self.graph_selected).unwrap();
                    reset_to_commit(&self.repo, *oid, git2::ResetType::Mixed).expect("Error");
                    self.reload();
                    self.focus = Focus::Viewport;
                }
                _ => {}
            },
            KeyCode::Char('a') => match self.focus {
                Focus::Viewport | Focus::ModalActions => {
                    if self.focus == Focus::Viewport && self.viewport != Viewport::Graph {
                        return;
                    }
                    if self.uncommitted.is_unstaged {
                        git_add_all(&self.repo).expect("Error");
                        self.reload();
                    }
                }
                _ => {}
            },
            KeyCode::Char('u') => match self.focus {
                Focus::Viewport | Focus::ModalActions => {
                    if self.focus == Focus::Viewport && self.viewport != Viewport::Graph {
                        return;
                    }
                    if self.uncommitted.is_staged {
                        unstage_all(&self.repo).expect("Error");
                        self.reload();
                    }
                }
                _ => {}
            },
            KeyCode::Enter => {
                match self.focus {
                    Focus::Viewport => {
                        if self.focus == Focus::Viewport && self.viewport == Viewport::Editor {
                            return;
                        }
                        self.focus = Focus::ModalActions;
                    }
                    Focus::ModalCheckout => {
                        let branches = self
                            .tips
                            .entry(*self.oids.get(self.graph_selected).unwrap())
                            .or_default();
                        checkout_branch(
                            &self.repo,
                            branches.get(self.modal_checkout_selected as usize).unwrap(),
                        )
                        .expect("Error");
                        self.modal_checkout_selected = 0;
                        self.focus = Focus::Viewport;
                        self.reload();
                    }
                    Focus::StatusTop | Focus::StatusBottom => {
                        self.open_viewer();
                        self.focus = Focus::Viewport;
                    }
                    _ => {}
                };
            }
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::Viewport => {
                        if self.focus == Focus::Viewport && self.viewport == Viewport::Editor {
                            return;
                        }
                        if self.is_inspector && self.graph_selected != 0 {
                            Focus::Inspector
                        } else if self.is_status {
                            Focus::StatusTop
                        } else {
                            Focus::Viewport
                        }
                    }
                    Focus::Inspector => {
                        if self.is_status {
                            Focus::StatusTop
                        } else {
                            Focus::Viewport
                        }
                    }
                    Focus::StatusTop => {
                        if self.graph_selected == 0 {
                            Focus::StatusBottom
                        } else {
                            Focus::Viewport
                        }
                    }
                    Focus::StatusBottom => Focus::Viewport,
                    _ => Focus::Viewport,
                };
            }
            KeyCode::Home => {
                match self.focus {
                    Focus::Viewport => match self.viewport {
                        Viewport::Graph => {
                            self.graph_selected = 0;
                        }
                        Viewport::Viewer => {
                            self.viewer_selected = 0;
                        }
                        _ => {}
                    },
                    Focus::Inspector => {
                        self.inspector_selected = 0;
                    }
                    Focus::StatusTop => {
                        self.status_top_selected = 0;
                    }
                    Focus::StatusBottom => {
                        self.status_bottom_selected = 0;
                    }
                    _ => {}
                };
            }
            KeyCode::End => {
                match self.focus {
                    Focus::Viewport => match self.viewport {
                        Viewport::Graph => {
                            if !self.lines_branches.is_empty() {
                                self.graph_selected = self.lines_branches.len() - 1;
                            }
                            let oid = self.oids.get(self.graph_selected).unwrap();
                            self.current_diff = get_changed_filenames(&self.repo, *oid);
                        }
                        Viewport::Viewer => {
                            if !self.viewer_lines.is_empty() {
                                self.viewer_selected = self.viewer_lines.len() - 1;
                            }
                        }
                        _ => {}
                    },
                    Focus::Inspector => {
                        self.inspector_selected = usize::MAX;
                    }
                    Focus::StatusTop => {
                        self.status_top_selected = usize::MAX;
                    }
                    Focus::StatusBottom => {
                        self.status_bottom_selected = usize::MAX;
                    }
                    _ => {}
                };
            }
            KeyCode::PageUp => {
                match self.focus {
                    Focus::Viewport => {
                        let page = self.layout.graph.height as usize - 3;
                        match self.viewport {
                            Viewport::Graph => {
                                if self.graph_selected >= page {
                                    self.graph_selected -= page;
                                } else {
                                    self.graph_selected = 0;
                                }
                                if self.graph_selected != 0 {
                                    let oid = self.oids.get(self.graph_selected).unwrap();
                                    self.current_diff = get_changed_filenames(&self.repo, *oid);
                                }
                            }
                            Viewport::Viewer => {
                                if self.viewer_selected >= page {
                                    self.viewer_selected -= page;
                                } else {
                                    self.viewer_selected = 0;
                                }
                            }
                            _ => {}
                        }
                    }
                    Focus::Inspector => {
                        let page = self.layout.inspector.height as usize - 3;
                        self.inspector_selected = self.inspector_selected.saturating_sub(page);
                    }
                    Focus::StatusTop => {
                        let page = self.layout.status_top.height as usize - 3;
                        self.status_top_selected = self.status_top_selected.saturating_sub(page);
                    }
                    Focus::StatusBottom => {
                        let page = self.layout.status_bottom.height as usize - 3;
                        self.status_bottom_selected =
                            self.status_bottom_selected.saturating_sub(page);
                    }
                    _ => {}
                };

                if self.graph_selected != 0 {
                    let oid = self.oids.get(self.graph_selected).unwrap();
                    self.current_diff = get_changed_filenames(&self.repo, *oid);
                }
            }
            KeyCode::PageDown => {
                match self.focus {
                    Focus::Viewport => {
                        let page = self.layout.graph.height as usize - 3;
                        match self.viewport {
                            Viewport::Graph => {
                                if self.graph_selected + page < self.lines_branches.len() {
                                    self.graph_selected += page;
                                } else {
                                    self.graph_selected = self.lines_branches.len() - 1;
                                }
                                if self.graph_selected != 0 {
                                    let oid = self.oids.get(self.graph_selected).unwrap();
                                    self.current_diff = get_changed_filenames(&self.repo, *oid);
                                }
                            }
                            Viewport::Viewer => {
                                if self.viewer_selected + page < self.viewer_lines.len() {
                                    self.viewer_selected += page;
                                } else {
                                    self.viewer_selected = self.viewer_lines.len() - 1;
                                }
                            }
                            _ => {}
                        }
                    }
                    Focus::Inspector => {
                        let page = self.layout.inspector.height as usize - 3;
                        self.inspector_selected += page;
                    }
                    Focus::StatusTop => {
                        let page = self.layout.status_top.height as usize - 3;
                        self.status_top_selected += page;
                    }
                    Focus::StatusBottom => {
                        let page = self.layout.status_bottom.height as usize - 3;
                        self.status_bottom_selected += page;
                    }
                    _ => {}
                };

                if self.graph_selected != 0 {
                    let oid = self.oids.get(self.graph_selected).unwrap();
                    self.current_diff = get_changed_filenames(&self.repo, *oid);
                }
            }
            KeyCode::Esc => {
                match self.focus {
                    Focus::ModalActions | Focus::ModalCommit => {
                        self.focus = Focus::Viewport;
                    }
                    Focus::ModalCheckout => {
                        self.modal_checkout_selected = 0;
                        self.focus = Focus::Viewport;
                    }
                    _ => {
                        self.viewport = Viewport::Graph;
                        self.focus = Focus::Viewport;
                        self.file_name = None;
                    }
                };
            }
            _ => {}
        }
    }
}
