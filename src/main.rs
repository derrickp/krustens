mod spotify;

use std::fs;

use spotify::listen::Listen;

fn main() {
    let file_name = "./data/streaming_history.json";
    let contents = fs::read_to_string(file_name).unwrap();
    let listens: Vec<Listen> = serde_json::from_str(&contents).unwrap();

    println!("{}", listens.len());

    println!("Hello, world!");
}
