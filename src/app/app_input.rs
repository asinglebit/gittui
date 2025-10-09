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
    Panes
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
                if self.selected + 1 < self.lines_branches.len() && !self.is_modal {
                    self.selected += 1;
                } else if self.is_modal {
                    let branches = self
                        .tips
                        .entry(*self.oids.get(self.selected).unwrap())
                        .or_default();
                    self.modal_selected = if self.modal_selected + 1 > branches.len() as i32 - 1 { 0 } else { self.modal_selected + 1 };
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.selected > 0 && !self.is_modal {
                    self.selected -= 1;
                    if self.selected == 0 && self.focus == Panes::Inspector {
                        self.focus = Panes::Graph;
                    }
                } else if self.is_modal {
                    let branches = self
                        .tips
                        .entry(*self.oids.get(self.selected).unwrap())
                        .or_default();
                    self.modal_selected = if self.modal_selected - 1 < 0 { branches.len() as i32 - 1 } else { self.modal_selected - 1 };
                }
            }
            KeyCode::Char('f') => {
                self.is_minimal = !self.is_minimal;
            }
            KeyCode::Char('s') => {
                self.is_status = !self.is_status;
                if !self.is_status && (self.focus == Panes::StatusTop || self.focus == Panes::StatusBottom) {
                    self.focus = Panes::Graph;
                }
            }
            KeyCode::Char('i') => {
                self.is_inspector = !self.is_inspector;
                if !self.is_inspector && self.focus == Panes::Inspector {
                    if self.is_status {
                        self.focus = Panes::StatusTop;
                    } else {
                        self.focus = Panes::Graph;
                    }
                }
            }
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Panes::Graph => {
                        if self.is_inspector && self.selected != 0 { Panes::Inspector }
                        else if self.is_status { Panes::StatusTop }
                        else { Panes::Graph }
                    }
                    Panes::Inspector => {
                        if self.is_status { Panes::StatusTop }
                        else { Panes::Graph }
                    }
                    Panes::StatusTop => {
                        if self.selected == 0 { Panes::StatusBottom }
                        else { Panes::Graph }
                    }
                    Panes::StatusBottom => { Panes::Graph }
                    _ => Panes::Graph,
                };
            }
            KeyCode::Home => {
                if !self.is_modal {
                    self.selected = 0;
                }
            }
            KeyCode::End => {
                if !self.lines_branches.is_empty() && !self.is_modal {
                    self.selected = self.lines_branches.len() - 1;
                }
            }
            KeyCode::PageUp => {
                if !self.is_modal {
                    let page = 20;
                    if self.selected >= page {
                        self.selected -= page;
                    } else {
                        self.selected = 0;
                    }
                }
            }
            KeyCode::PageDown => {
                if !self.is_modal {
                    let page = 20;
                    if self.selected + page < self.lines_branches.len() {
                        self.selected += page;
                    } else {
                        self.selected = self.lines_branches.len() - 1;
                    }
                }
            }
            KeyCode::Enter => {

                let branches = self
                    .tips
                    .entry(*self.oids.get(self.selected).unwrap())
                    .or_default();

                if !self.is_modal {
                    if self.selected == 0 {
                        return;
                    }
                    if branches.is_empty() {
                        checkout_head(&self.repo, *self.oids.get(self.selected).unwrap());
                        self.reload();
                    } else if branches.len() == 1 {
                        checkout_branch(&self.repo, branches.first().unwrap()).expect("Error");
                        self.reload();
                    } else {
                        self.is_modal = true;
                    }
                } else {
                    checkout_branch(&self.repo, branches.get(self.modal_selected as usize).unwrap()).expect("Error");
                    self.modal_selected = 0;
                    self.is_modal = false;
                    self.reload();
                }
            }
            KeyCode::Esc => {
                if self.is_modal {
                    self.modal_selected = 0;
                    self.is_modal = false;
                }
            }
            _ => {}
        }
    }
}
