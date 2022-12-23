use serde::{Deserialize, Serialize};

use super::CommandName;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Input {
    pub text: String,
    pub command_history: Vec<CommandName>,
    pub history_index: Option<usize>,
}

impl Input {
    pub fn push(&mut self, c: char) {
        self.history_index = None;
        self.text.push(c);
    }

    pub fn pop(&mut self) {
        self.history_index = None;
        self.text.pop();
    }

    pub fn current(&self) -> &str {
        &self.text
    }

    pub fn set(&mut self, command_name: &CommandName) {
        self.history_index = None;
        self.text = command_name.to_string();
    }

    pub fn drain(&mut self) -> String {
        self.history_index = None;
        self.text.drain(..).collect()
    }

    pub fn clear(&mut self) {
        self.history_index = None;
        self.text.clear();
    }

    pub fn push_to_history(&mut self, command_name: CommandName) {
        if !self.command_history.last().eq(&Some(&command_name)) {
            self.command_history.push(command_name);
        }
    }

    pub fn set_from_previous_history(&mut self) {
        if self.command_history.is_empty() {
            return;
        }

        let new_index = match self.history_index {
            Some(index) => Some((index - 1).max(0)),
            None => Some(self.command_history.len() - 1),
        };

        match new_index {
            Some(index) => {
                self.history_index = Some(index);
                if let Some(command_name) = self.command_history.get(index) {
                    self.text = command_name.to_string();
                }
            }
            None => {
                self.text.clear();
                self.history_index = None;
            }
        }
    }

    pub fn set_from_next_history(&mut self) {
        if self.command_history.is_empty() {
            return;
        }

        let new_index = match self.history_index {
            Some(index) => {
                if index == self.command_history.len() - 1 {
                    None
                } else {
                    Some(index + 1)
                }
            }
            None => None,
        };

        match new_index {
            Some(index) => {
                self.history_index = Some(index);
                if let Some(command_name) = self.command_history.get(index) {
                    self.text = command_name.to_string();
                }
            }
            None => {
                self.text.clear();
                self.history_index = None;
            }
        }
    }
}
