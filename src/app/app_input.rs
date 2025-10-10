use std::usize;
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
use crate::git::{actions::reset_hard, queries::get_current_branch};
#[rustfmt::skip]
use crate::{
    git::{
        actions::{
            checkout_head,
            checkout_branch
        }
    },
};
#[rustfmt::skip]
use crate::app::app::{
    App,
    Focus
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

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('r') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.reload()
            },
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.exit()
            }
            KeyCode::Char('j') | KeyCode::Down => {
                match self.focus {
                    Focus::Graph => {
                        if self.graph_selected + 1 < self.lines_branches.len() {
                            self.graph_selected += 1;
                        }
                    }
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
                        self.modal_checkout_selected = if self.modal_checkout_selected + 1 > branches.len() as i32 - 1 { 0 } else { self.modal_checkout_selected + 1 };
                    }
                    _ => {}
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                match self.focus {
                    Focus::Graph => {
                        if self.graph_selected > 0 {
                            self.graph_selected -= 1;
                            if self.graph_selected == 0 && self.focus == Focus::Inspector {
                                self.focus = Focus::Graph;
                            }
                        }
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
                        self.modal_checkout_selected = if self.modal_checkout_selected - 1 < 0 { branches.len() as i32 - 1 } else { self.modal_checkout_selected - 1 };
                    }
                    _ => {}
                }
            }
            KeyCode::Char('f') => {
                self.is_minimal = !self.is_minimal;
            }
            KeyCode::Char('s') => {
                self.is_status = !self.is_status;
                if !self.is_status && (self.focus == Focus::StatusTop || self.focus == Focus::StatusBottom) {
                    self.focus = Focus::Graph;
                }
            }
            KeyCode::Char('i') => {
                self.is_inspector = !self.is_inspector;
                if !self.is_inspector && self.focus == Focus::Inspector {
                    if self.is_status {
                        self.focus = Focus::StatusTop;
                    } else {
                        self.focus = Focus::Graph;
                    }
                }
            }
            KeyCode::Char('x') => {
                match self.focus {
                    Focus::ModalActions => {
                        self.focus = Focus::Graph;
                    }
                    Focus::ModalCheckout => {
                        self.modal_checkout_selected = 0;
                        self.focus = Focus::Graph;
                    }
                    _ => {},
                };
            }
            KeyCode::Char('c') => {
                match self.focus {
                    Focus::Graph | Focus::ModalActions => {
                        let branches = self
                            .tips
                            .entry(*self.oids.get(self.graph_selected).unwrap())
                            .or_default();
                        if self.graph_selected == 0 {
                            self.focus = Focus::Graph;
                            return;
                        }
                        if branches.is_empty() {
                            checkout_head(&self.repo, *self.oids.get(self.graph_selected).unwrap());
                            self.focus = Focus::Graph;
                            self.reload();
                        } else if branches.len() == 1 {
                            checkout_branch(&self.repo, branches.first().unwrap()).expect("Error");
                            self.focus = Focus::Graph;
                            self.reload();
                        } else {
                            self.focus = Focus::ModalCheckout;
                        }
                    }
                    _ => {}
                };
            }
            KeyCode::Char('r') => {
                match self.focus {
                    Focus::Graph | Focus::ModalActions => {
                        let target = match get_current_branch(&self.repo) {
                            Some(branch) => branch,
                            None => "HEAD".to_string()
                        };
                        reset_hard(&self.repo, &target).expect("Error");
                        self.reload();
                        self.focus = Focus::Graph;
                    }
                    _ => {}
                }
            }
            KeyCode::Enter => {
                match self.focus {
                    Focus::Graph => {
                        self.focus = Focus::ModalActions;
                    }
                    Focus::ModalCheckout => {
                        let branches = self
                            .tips
                            .entry(*self.oids.get(self.graph_selected).unwrap())
                            .or_default();
                        checkout_branch(&self.repo, branches.get(self.modal_checkout_selected as usize).unwrap()).expect("Error");
                        self.modal_checkout_selected = 0;
                        self.focus = Focus::Graph;
                        self.reload();
                    }
                    _ => {}
                };
            }
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::Graph => {
                        if self.is_inspector && self.graph_selected != 0 { Focus::Inspector }
                        else if self.is_status { Focus::StatusTop }
                        else { Focus::Graph }
                    }
                    Focus::Inspector => {
                        if self.is_status { Focus::StatusTop }
                        else { Focus::Graph }
                    }
                    Focus::StatusTop => {
                        if self.graph_selected == 0 { Focus::StatusBottom }
                        else { Focus::Graph }
                    }
                    Focus::StatusBottom => { Focus::Graph }
                    _ => Focus::Graph,
                };
            }
            KeyCode::Home => {
                match self.focus {
                    Focus::Graph => {
                        self.graph_selected = 0;
                    }
                    Focus::Inspector => {
                        self.inspector_selected = 0;
                    }
                    Focus::StatusTop => {
                        self.status_top_selected = 0;
                    }
                    Focus::StatusBottom => {
                        self.status_bottom_selected = 0;
                    }
                    _ => {},
                };
            }
            KeyCode::End => {
                match self.focus {
                    Focus::Graph => {
                        if !self.lines_branches.is_empty() {
                            self.graph_selected = self.lines_branches.len() - 1;
                        }
                    }
                    Focus::Inspector => {
                        self.inspector_selected = usize::MAX;
                    }
                    Focus::StatusTop => {
                        self.status_top_selected = usize::MAX;
                    }
                    Focus::StatusBottom => {
                        self.status_bottom_selected = usize::MAX;
                    }
                    _ => {},
                };
            }
            KeyCode::PageUp => {
                let page = 20;
                match self.focus {
                    Focus::Graph => {
                        if self.graph_selected >= page {
                            self.graph_selected -= page;
                        } else {
                            self.graph_selected = 0;
                        }
                    }
                    Focus::Inspector => {
                        self.inspector_selected = self.inspector_selected.saturating_sub(page);
                    }
                    Focus::StatusTop => {
                        self.status_top_selected = self.status_top_selected.saturating_sub(page);
                    }
                    Focus::StatusBottom => {
                        self.status_bottom_selected = self.status_bottom_selected.saturating_sub(page);
                    }
                    _ => {},
                };
            }
            KeyCode::PageDown => {
                let page = 20;
                match self.focus {
                    Focus::Graph => {
                        if self.graph_selected + page < self.lines_branches.len() {
                            self.graph_selected += page;
                        } else {
                            self.graph_selected = self.lines_branches.len() - 1;
                        }
                    }
                    Focus::Inspector => {
                        self.inspector_selected += page;
                    }
                    Focus::StatusTop => {
                        self.status_top_selected += page;
                    }
                    Focus::StatusBottom => {
                        self.status_bottom_selected += page;
                    }
                    _ => {},
                };
            }
            KeyCode::Esc => {
                match self.focus {
                    Focus::ModalActions => {
                        self.focus = Focus::Graph;
                    }
                    Focus::ModalCheckout => {
                        self.modal_checkout_selected = 0;
                        self.focus = Focus::Graph;
                    }
                    _ => {},
                };
            }
            _ => {}
        }
    }
}
