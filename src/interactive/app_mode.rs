#[derive(Debug)]
pub enum AppMode {
    CommandParameters,
    EnterCommand,
    Normal,
    Processing,
}

impl Default for AppMode {
    fn default() -> Self {
        Self::Normal
    }
}
