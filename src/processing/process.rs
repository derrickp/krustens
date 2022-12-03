use std::{path::PathBuf, sync::Arc};

use tokio::sync::Mutex;

use crate::{
    errors::ReadError, events::Event, persistence::EventStore, processing::commands::AddTrackPlay,
    projections::ListenTrackerRepository, track_plays::read_track_plays,
};

pub const MIN_LISTEN_LENGTH: u64 = 1000 * 10;

pub async fn process_file(
    path: &PathBuf,
    store: &Arc<dyn EventStore>,
    repository: &Arc<Mutex<dyn ListenTrackerRepository>>,
) -> Result<Vec<Event>, ReadError> {
    let track_plays = read_track_plays(path)?;

    let mut repo = repository.lock().await;

    let mut events: Vec<Event> = Vec::new();

    for track_play in track_plays.iter() {
        let command = AddTrackPlay {
            track_play: track_play.clone(),
            min_listen_length: MIN_LISTEN_LENGTH,
        };
        let tracker = repo.get().await;
        let handle_result = command.handle(tracker);

        let listen_event = match handle_result {
            Some(event) => {
                let version = event.version;
                match store.add_event("listens".to_string(), event, version).await {
                    Ok(it) => it,
                    Err(_) => continue,
                }
            }
            None => continue,
        };

        events.push(listen_event);
    }

    repo.flush().await;

    Ok(events)
}
