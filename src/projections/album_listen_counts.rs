use std::sync::Arc;

use chrono::Datelike;
use serde::Serialize;
use tokio::sync::Mutex;
use tune_sage::api::recordings::{RecordingApi, RecordingQuery, RecordingSearchBuilder};

use crate::events::{Event, EventData, TrackPlayAdded};

use super::{stats::SongPlayCount, utils::parse_end_time};

#[derive(Serialize, Clone)]
pub struct MonthlyListenCount {
    pub year: i32,
    pub month: u32,
    pub album: String,
    pub artist: String,
    pub total: u32,
    pub song_counts: Vec<SongPlayCount>,
}

impl MonthlyListenCount {
    fn add_track_listen(&mut self, track: &str) {
        self.total += 1;
        let existing = self
            .song_counts
            .iter_mut()
            .find(|song_count| song_count.0.eq(track));

        if let Some(song_count) = existing {
            song_count.1 += 1
        } else {
            self.song_counts.push(SongPlayCount(track.to_string(), 1))
        }
    }
}

pub struct AlbumListenCounts {
    pub recording_api: Arc<Mutex<RecordingApi>>,
}

#[derive(Default)]
struct ListenCountTracker {
    monthly_listens: Vec<MonthlyListenCount>,
}

impl ListenCountTracker {
    fn add_listen(&mut self, track: &str, artist: &str, album: &str, year: i32, month: u32) {
        let existing = self
            .monthly_listens
            .iter_mut()
            .find(|listen_count| listen_count.album.eq(album) && listen_count.artist.eq(artist));

        if let Some(monthly) = existing {
            monthly.add_track_listen(track);
        } else {
            self.monthly_listens.push(MonthlyListenCount {
                year,
                month,
                album: album.to_string(),
                artist: artist.to_string(),
                total: 1,
                song_counts: vec![SongPlayCount(track.to_string(), 1)],
            })
        }
    }
}

impl AlbumListenCounts {
    pub async fn monthly_listens(&mut self, events: Vec<&Event>) -> Vec<MonthlyListenCount> {
        let listen_tracker = Arc::new(Mutex::new(ListenCountTracker::default()));

        println!("total events - {}", &events.len());

        for (index, event) in events.into_iter().enumerate() {
            if index % 500 == 0 {
                println!("{}", &index);
            }

            let added = match &event.data {
                EventData::TrackPlayAdded(it) => it.to_owned(),
                EventData::TrackPlayIgnored(_) => continue,
            };

            let recording_api = self.recording_api.clone();
            let tracker = listen_tracker.clone();

            tokio::spawn(async move {
                AlbumListenCounts::process_listen(tracker, added, recording_api).await;
            })
            .await
            .unwrap();
        }

        let listens = listen_tracker.lock_owned().await.monthly_listens.clone();
        listens
    }

    async fn process_listen(
        listen_tracker: Arc<Mutex<ListenCountTracker>>,
        added: TrackPlayAdded,
        recording_api: Arc<Mutex<RecordingApi>>,
    ) {
        let (artist_name, track_name, date) = (
            &added.artist_name,
            &added.track_name,
            parse_end_time(&added.end_time),
        );

        let end_time = match &date {
            Ok(it) => it,
            Err(e) => {
                println!("{}", e);
                return;
            }
        };

        let search = RecordingSearchBuilder::new()
            .artist(artist_name)
            .recording(&track_name.replace("?", ""))
            .build()
            .unwrap();

        let recording_list = recording_api
            .lock()
            .await
            .query(&RecordingQuery::Search(Box::new(search)), None)
            .await
            .unwrap();

        let initial_count = recording_list
            .recordings
            .iter()
            .filter(|recording| recording.score.eq(&Some(100)))
            .count();

        let recording = if initial_count == 1 {
            recording_list
                .recordings
                .into_iter()
                .find(|recording| recording.score.eq(&Some(100)))
        } else {
            println!("Doing secondary search {} - {} - {}", &artist_name, &track_name, initial_count);
            let secondary_search = RecordingSearchBuilder::new()
                .artist(artist_name)
                .recording_accent(&track_name.replace("?", ""))
                .build()
                .unwrap();
            let secondary_list = recording_api
                .lock()
                .await
                .query(&RecordingQuery::Search(Box::new(secondary_search)), None)
                .await
                .unwrap();

            let secondary_count = secondary_list
                .recordings
                .iter()
                .filter(|recording| recording.score.eq(&Some(100)))
                .count();

            if secondary_count != 1 {
                println!("Too many from secondary query - {}", secondary_count);
                None
            } else {
                secondary_list
                    .recordings
                    .into_iter()
                    .find(|recording| recording.score.eq(&Some(100)))
            }
        };

        let release = match &recording {
            Some(rec) => rec.releases.as_ref().and_then(|releases| releases.get(0)),
            None => {
                println!("No recording found for {} - {}", &artist_name, &track_name);
                None
            }
        };

        if let Some(rel) = release {
            listen_tracker.lock().await.add_listen(
                track_name,
                artist_name,
                &rel.title,
                end_time.year(),
                end_time.month(),
            );
        } else {
            listen_tracker.lock().await.add_listen(
                track_name,
                artist_name,
                "Unknown",
                end_time.year(),
                end_time.month(),
            )
        }
    }
}
