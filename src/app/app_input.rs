#[rustfmt::skip]
use std::{
    io,
};
#[rustfmt::skip]
use ratatui::crossterm::event::{
    self,
    Event,
    KeyCode,
    KeyEvent,
    KeyEventKind,
    KeyModifiers,
};
#[rustfmt::skip]
use edtui::{
    EditorMode,
};
#[rustfmt::skip]
use crate::{
    app::app::{
        App,
        Focus,
        Viewport
    },
    git::{
        actions::{
            commits::{
                checkout_head,
                checkout_branch,
                commit_staged,
                git_add_all,
                reset_to_commit,
                unstage_all,
                fetch_over_ssh,
                push_over_ssh
            }
        },
        queries::{
            diffs::{
                get_filenames_diff_at_oid,
            },
            commits::{
                get_current_branch
            }
        }
    },
    helpers::{
        text::{
            editor_state_to_string,
        }
    }
};

impl App {
    pub fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if matches!(key_event.kind, KeyEventKind::Press) => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
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
            KeyCode::Char('p') => {
                if self.focus == Focus::Viewport {
                    if self.viewport != Viewport::Graph { return; }
                    let handle = push_over_ssh(&self.path, "origin", get_current_branch(&self.repo).unwrap().as_str(), true);
                    match handle.join().expect("Thread panicked") {
                        Ok(_) => {
                            self.reload();
                        },
                        Err(e) => eprintln!("Fetch failed: {}", e),
                    }
                }
            }
            KeyCode::Char('f') => {
                if self.focus == Focus::Viewport {
                    if self.viewport != Viewport::Graph { return; }
                    let handle = fetch_over_ssh(&self.path, "origin");
                    match handle.join().expect("Thread panicked") {
                        Ok(_) => {
                            self.reload();
                        },
                        Err(e) => eprintln!("Fetch failed: {}", e),
                    }
                }
            }
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
                        if self.graph_selected + 1 < self.oids.len() {
                            self.graph_selected += 1;
                        }
                        if self.graph_selected != 0 && self.graph_selected < self.oids.len() {
                            let oid = self.oids.get(self.graph_selected).unwrap();
                            self.current_diff = get_filenames_diff_at_oid(&self.repo, *oid);
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
                            if self.graph_selected != 0 && self.graph_selected < self.oids.len() {
                                let oid = self.oids.get(self.graph_selected).unwrap();
                                self.current_diff = get_filenames_diff_at_oid(&self.repo, *oid);
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
            KeyCode::Char('.') => {
                self.is_minimal = !self.is_minimal;
            }
            KeyCode::Char('`') => {
                self.is_branches = !self.is_branches;
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
                        }
                    }
                    _ => {}
                }
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
                            if !self.oids.is_empty() {
                                self.graph_selected = self.oids.len() - 1;
                            }
                            if self.graph_selected != 0 && self.graph_selected < self.oids.len() {
                                let oid = self.oids.get(self.graph_selected).unwrap();
                                self.current_diff = get_filenames_diff_at_oid(&self.repo, *oid);
                            }
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
                        let page = self.layout.graph.height as usize - 1;
                        match self.viewport {
                            Viewport::Graph => {
                                if self.graph_selected >= page {
                                    self.graph_selected -= page;
                                } else {
                                    self.graph_selected = 0;
                                }
                                
                                if self.graph_selected != 0 && self.graph_selected < self.oids.len() {
                                    let oid = self.oids.get(self.graph_selected).unwrap();
                                    self.current_diff = get_filenames_diff_at_oid(&self.repo, *oid);
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

                if self.graph_selected != 0 && self.graph_selected < self.oids.len() {
                    let oid = self.oids.get(self.graph_selected).unwrap();
                    self.current_diff = get_filenames_diff_at_oid(&self.repo, *oid);
                }
            }
            KeyCode::PageDown => {
                match self.focus {
                    Focus::Viewport => {
                        let page = self.layout.graph.height as usize - 1;
                        match self.viewport {
                            Viewport::Graph => {
                                if self.graph_selected + page < self.oids.len() {
                                    self.graph_selected += page;
                                } else {
                                    self.graph_selected = self.oids.len() - 1;
                                }
                                if self.graph_selected != 0 && self.graph_selected < self.oids.len() {
                                    let oid = self.oids.get(self.graph_selected).unwrap();
                                    self.current_diff = get_filenames_diff_at_oid(&self.repo, *oid);
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

                if self.graph_selected != 0 && self.graph_selected < self.oids.len() {
                    let oid = self.oids.get(self.graph_selected).unwrap();
                    self.current_diff = get_filenames_diff_at_oid(&self.repo, *oid);
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
                        match self.viewport {
                            Viewport::Graph => {
                                self.viewport = Viewport::Settings;
                                self.focus = Focus::Viewport;
                                self.file_name = None;
                            }
                            _ => {
                                self.viewport = Viewport::Graph;
                                self.focus = Focus::Viewport;
                                self.file_name = None;
                            }
                        }
                    }
                };
            }
            _ => {}
        }
    }
}
