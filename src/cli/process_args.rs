use clap::Args;

#[derive(Args, Debug)]
pub struct ProcessArgs {
    /// Folder that contains the streaming history.
    #[arg(short, long, default_value = "./data/play_history")]
    pub input: String,
}
