use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub history_folder: String,
    pub output_folder: String,
    pub count_general_stats_to_compile: usize,
}

impl Config {
    pub fn stats_file_name(output_folder: &str, name: &str) -> String {
        format!("{}/stats/{}", output_folder, name)
    }

    pub fn stats_folder(output_folder: &str) -> String {
        format!("{}/stats", output_folder)
    }

    pub fn general_stats_file(output_folder: &str) -> String {
        Self::stats_file_name(output_folder, "general.yaml")
    }

    pub fn complete_stats_file(output_folder: &str) -> String {
        Self::stats_file_name(output_folder, "complete.json")
    }

    pub fn top_50_stats_file(output_folder: &str) -> String {
        Self::stats_file_name(output_folder, "top_50.json")
    }

    pub fn top_100_stats_file(output_folder: &str) -> String {
        Self::stats_file_name(output_folder, "top_100.json")
    }
}
