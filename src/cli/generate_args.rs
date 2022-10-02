use clap::Args;

#[derive(Args, Debug)]
pub struct GenerateArgs {
    /// Folder to place output files into.
    #[arg(short, long, default_value = "./output")]
    pub output: String,

    /// How many artists/songs you want to include.
    #[arg(short, long, default_value_t = 25)]
    pub count: usize,

    /// Year to generate statistics for.
    #[arg(short, long)]
    pub year: Option<i32>,

    /// Split the statistics down by month.
    #[arg(short, long, default_value_t = false)]
    pub split_monthly: bool,
}
