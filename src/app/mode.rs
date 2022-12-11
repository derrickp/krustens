#[derive(Debug)]
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
