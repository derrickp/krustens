use std::io::Write;

use rand::Rng;

use crate::{projections::statistics::EventProcessor, track_plays::ArtistName};

pub fn prompt(name: &str) -> String {
    let mut line = String::new();
    print!("{}", name);
    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut line)
        .expect("Error: Could not read a line");

    return line.trim().to_string();
}

pub fn prompt_random_artist(processor: &EventProcessor) -> Vec<String> {
    let year = prompt("What year to look in? > ")
        .parse::<i32>()
        .expect("Error: Not a valid number");
    let num_artists = prompt("How many artists do you want names of (Default: 1) > ")
        .parse::<u32>()
        .ok()
        .filter(|num| num > &0)
        .unwrap_or(1);
    let min_listens = prompt("What minimum number of listens? > ")
        .parse::<u64>()
        .unwrap_or_default();
    let month = prompt("What month do you want to look in (1-12)? > ")
        .parse::<u32>()
        .ok()
        .filter(|m| (&1..=&12).contains(&m))
        .unwrap_or_default();

    processor
        .year_count(year)
        .map(|year_count| {
            let mut rng = rand::thread_rng();
            let mut artist_counters = if month == 0 {
                year_count.over_min_plays(min_listens)
            } else {
                year_count
                    .month_count(month)
                    .map(|month_count| month_count.over_min_plays(min_listens))
                    .unwrap_or_default()
            };

            if artist_counters.is_empty() {
                vec!["No artists found".to_string()]
            } else {
                let mut names: Vec<String> = Vec::new();

                for _ in 0..num_artists {
                    if artist_counters.is_empty() {
                        break;
                    }
                    let index = rng.gen_range(0..artist_counters.len());
                    let artist_counter = artist_counters.remove(index);
                    names.push(artist_counter.artist_name.to_string());
                }

                names
            }
        })
        .unwrap_or_else(|| vec!["No listens for that year".to_string()])
}

pub fn prompt_artists_on_day(processor: &EventProcessor) -> Vec<String> {
    let year = prompt("What year to look in? > ")
        .parse::<i32>()
        .expect("Error: Not a valid number");
    let month = prompt("What month do you want to look in (1-12)? > ")
        .parse::<u32>()
        .ok()
        .filter(|m| (1..=12).contains(m))
        .expect("Error: Not a valid month");
    let day = prompt("What day of the month? > ")
        .parse::<u32>()
        .ok()
        .filter(|d| (1..=31).contains(d))
        .expect("Error: Not a valid day");

    chrono::NaiveDate::from_ymd_opt(year, month, day)
        .map(|date| {
            let names = processor
                .artists_on_day(date)
                .into_iter()
                .map(|artist_counter| artist_counter.total_plays_display())
                .collect::<Vec<String>>();

            if names.is_empty() {
                vec!["No artists listened to on that day".to_string()]
            } else {
                names
            }
        })
        .unwrap_or_else(|| vec!["Error: Invalid date".to_string()])
}

pub fn prompt_artist_songs(processor: &EventProcessor) -> Vec<String> {
    let artist_name = ArtistName(prompt("What artist do you want to look for? > "));

    processor
        .artist_song_counter(&artist_name)
        .map(|artist_counter| {
            artist_counter
                .play_details
                .all_song_plays()
                .iter()
                .map(|song_play| song_play.0.clone())
                .collect()
        })
        .unwrap_or_default()
}
