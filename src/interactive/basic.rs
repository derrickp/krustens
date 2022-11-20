use std::sync::Arc;

use super::{prompt, prompt_artist_songs, prompt_artists_on_day, prompt_random_artist};

use crate::{persistence::EventStore, projections::statistics::EventProcessor};

pub async fn basic_interactive(store: Arc<impl EventStore>) {
    println!("Loading...");
    let mut processor = EventProcessor::default();
    let event_stream = store.get_events("listens".to_string()).await.unwrap();
    for event in event_stream.events.iter() {
        processor.process_event(event);
    }

    loop {
        let input = prompt("(q to exit, c for command list) > ");

        if input == "q" {
            break;
        } else if input == "random artists" {
            let artist_names = prompt_random_artist(&processor);
            for name in artist_names {
                println!(">> {}", name);
            }
        } else if input == "artist songs" {
            let song_names = prompt_artist_songs(&processor);
            for name in song_names {
                println!(">> {}", name);
            }
        } else if input == "artists on day" {
            let names = prompt_artists_on_day(&processor);
            for name in names {
                println!(">> {}", name);
            }
        } else if input == "c" {
            println!(">> random artists -> search for a random artist from your listening history");
            println!(">> artist songs -> list out all of the songs for a single artist from your listening history");
            println!(">> artists on day -> list all artists listened to on a specific day")
        }
    }
}
