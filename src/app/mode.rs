use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum Mode {
    CommandParameters,
    EnterCommand,
    Normal,
    Processing,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal
    }
}
