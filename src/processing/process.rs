use std::{fs, sync::Arc};

use crate::{
    persistence::EventStore, processing::commands::AddTrackPlay,
    projections::ListenTrackerRepository, track_plays::read_track_plays,
};

pub const MIN_LISTEN_LENGTH: u64 = 1000 * 10;

pub async fn process_listens(
    input_folder: &str,
    store: Arc<impl EventStore>,
    repository: &mut impl ListenTrackerRepository,
) {
    let streaming_files =
        fs::read_dir(input_folder).expect(&format!("Could not read {}", &input_folder));

    for entry in streaming_files {
        let path = entry.unwrap().path().clone();

        let track_plays = match read_track_plays(&path) {
            Ok(it) => it,
            Err(e) => {
                println!("{:?}", e);
                continue;
            }
        };

        for track_play in track_plays.iter() {
            let command = AddTrackPlay {
                track_play: track_play.clone(),
                min_listen_length: MIN_LISTEN_LENGTH,
            };
            let tracker = repository.get().await;
            let handle_result = command.handle(tracker);

            if let Some(event) = handle_result {
                if let Err(err) = store
                    .add_event("listens".to_string(), &event, event.version)
                    .await
                {
                    println!("{:?}", err);
                }
            }
        }

        println!("processed {}", path.display());
    }

    let _ = repository.get().await;
    repository.flush().await;
}
