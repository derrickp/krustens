use clap::Subcommand;

use super::{GenerateArgs, ProcessArgs};

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Process the streaming history files to generate listens database.
    Process(ProcessArgs),
    /// Generate statistics from the listens database.
    Generate(GenerateArgs),
}
