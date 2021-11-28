use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub history_folder: String,
    pub output_folder: String,
    pub count_general_stats_to_compile: usize,
}

impl Config {
    pub fn stats_file_name(&self, name: &str) -> String {
        format!("{}/stats/{}", self.output_folder, name)
    }

    pub fn stats_folder(&self) -> String {
        format!("{}/stats", self.output_folder)
    }

    pub fn general_stats_file(&self) -> String {
        self.stats_file_name("general.yaml")
    }

    pub fn complete_stats_file(&self) -> String {
        self.stats_file_name("complete.json")
    }

    pub fn top_50_stats_file(&self) -> String {
        self.stats_file_name("top_50.json")
    }

    pub fn top_100_stats_file(&self) -> String {
        self.stats_file_name("top_100.json")
    }
}
