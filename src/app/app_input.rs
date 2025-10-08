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
        actions::checkout,
    },
};
#[rustfmt::skip]
use crate::app::app::App;

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
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.selected > 0 && !self.is_modal {
                    self.selected -= 1;
                }
            }
            KeyCode::Char('f') => {
                self.is_minimal = !self.is_minimal;
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
                if !self.is_modal {
                    let branches = self
                        .tips
                        .entry(*self.oids.get(self.selected).unwrap())
                        .or_default();
                    if branches.len() > 1 {
                        self.is_modal = true;
                    } else {
                        checkout(&self.repo, *self.oids.get(self.selected).unwrap());
                        self.reload();
                    }
                }
            }
            KeyCode::Esc => {
                if self.is_modal {
                    self.is_modal = false;
                }
            }
            _ => {}
        }
    }
}
