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
            KeyCode::Char('r') => self.reload(),
            KeyCode::Char('q') => self.exit(),
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
                        self.modal_selected = if self.modal_selected + 1 > branches.len() as i32 - 1 { 0 } else { self.modal_selected + 1 };
                    }
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
                        self.modal_selected = if self.modal_selected - 1 < 0 { branches.len() as i32 - 1 } else { self.modal_selected - 1 };
                    }
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
                if self.focus != Focus::ModalCheckout {
                    self.graph_selected = 0;
                }
            }
            KeyCode::End => {
                if !self.lines_branches.is_empty() && self.focus != Focus::ModalCheckout {
                    self.graph_selected = self.lines_branches.len() - 1;
                }
            }
            KeyCode::PageUp => {
                if self.focus != Focus::ModalCheckout {
                    let page = 20;
                    if self.graph_selected >= page {
                        self.graph_selected -= page;
                    } else {
                        self.graph_selected = 0;
                    }
                }
            }
            KeyCode::PageDown => {
                if self.focus != Focus::ModalCheckout {
                    let page = 20;
                    if self.graph_selected + page < self.lines_branches.len() {
                        self.graph_selected += page;
                    } else {
                        self.graph_selected = self.lines_branches.len() - 1;
                    }
                }
            }
            KeyCode::Enter => {

                let branches = self
                    .tips
                    .entry(*self.oids.get(self.graph_selected).unwrap())
                    .or_default();

                if self.focus != Focus::ModalCheckout {
                    if self.graph_selected == 0 {
                        return;
                    }
                    if branches.is_empty() {
                        checkout_head(&self.repo, *self.oids.get(self.graph_selected).unwrap());
                        self.reload();
                    } else if branches.len() == 1 {
                        checkout_branch(&self.repo, branches.first().unwrap()).expect("Error");
                        self.reload();
                    } else {
                        self.focus = Focus::ModalCheckout;
                    }
                } else {
                    checkout_branch(&self.repo, branches.get(self.modal_selected as usize).unwrap()).expect("Error");
                    self.modal_selected = 0;
                    self.focus = Focus::Graph;
                    self.reload();
                }
            }
            KeyCode::Esc => {
                if self.focus == Focus::ModalCheckout {
                    self.modal_selected = 0;
                    self.focus = Focus::Graph;
                }
            }
            _ => {}
        }
    }
}
