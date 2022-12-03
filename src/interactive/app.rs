use std::{collections::HashSet, fs, path::PathBuf, str::FromStr, sync::Arc};

use arboard::Clipboard;
use chrono::{Local, NaiveDate};
use tokio::sync::Mutex;

use crate::{
    errors::InteractiveError,
    persistence::{fs::FileWriter, EventStore, Format, Writer},
    processing,
    projections::{
        statistics::{EventProcessor, General},
        ListenTrackerRepository,
    },
    track_plays::ArtistName,
};

use super::{AppMessageSet, AppMode, AppState, CommandName, CommandParameters, OutputFolder};

pub struct App {
    store: Arc<dyn EventStore>,
    repository: Arc<Mutex<dyn ListenTrackerRepository>>,
    pub processor: EventProcessor,
    pub state: AppState,
}

impl App {
    pub fn new(
        store: Arc<dyn EventStore>,
        repository: Arc<Mutex<dyn ListenTrackerRepository>>,
    ) -> App {
        App {
            store,
            repository,
            processor: EventProcessor::default(),
            state: AppState::default(),
        }
    }

    pub async fn initialize(&mut self) -> Result<(), InteractiveError> {
        let event_stream = self
            .store
            .get_events("listens".to_string())
            .await
            .map_err(|e| InteractiveError::GetEventsError { error: e })?;
        for event in event_stream.events.iter() {
            self.processor.process_event(event);
        }
        Ok(())
    }

    pub async fn tick(&mut self) -> Result<(), InteractiveError> {
        match self.state.mode {
            AppMode::CommandParameters => {}
            AppMode::EnterCommand => {}
            AppMode::Normal => {}
            AppMode::Processing => {
                self.run_command().await;
                if self.state.command_parameters.is_none() {
                    self.reset_state(false);
                    self.state.mode = AppMode::Normal;
                }
            }
        }
        Ok(())
    }

    pub fn advance_command_input(&mut self) {
        let text: String = self.state.input.drain(..).collect();
        let spec = self.state.command_parameter_inputs.remove(0);
        match self.state.insert_command_parameter(&text, &spec) {
            Ok(_) => {
                if self.state.command_parameter_inputs.is_empty() {
                    self.state.mode = AppMode::Processing;
                }
            }
            Err(e) => {
                self.reset_state(true);
                self.state.error_message = Some(e.to_string());
                self.state.mode = AppMode::Normal;
            }
        }
    }

    pub fn command_name_entered(&mut self) {
        let text: String = self.state.input.drain(..).collect();
        match CommandName::from_str(&text) {
            Ok(it) => {
                self.state.command_parameter_inputs = it.parameters();
                self.state.command_parameters = Some(it.default_parameters());
                self.state.command_name = Some(it);
                self.state.mode = AppMode::CommandParameters;
            }
            Err(_) => {
                self.state.mode = AppMode::Normal;
                self.state.error_message = Some("Unknown command name".to_string());
            }
        }
    }

    pub async fn export_to_file(&mut self) {
        let folder = OutputFolder {
            root: "./output".to_string(),
        };
        let writer = FileWriter {
            folder: Box::new(folder),
        };

        let today = Local::now().format("%Y-%m-%d %H:%M:%S");
        writer
            .write(
                &self.state.message_sets,
                &format!("messages_{}", &today),
                Format::Json,
            )
            .await
            .unwrap();
        writer
            .write(
                &self.state.message_sets,
                &format!("messages_{}", &today),
                Format::Yaml,
            )
            .await
            .unwrap();
    }

    pub fn copy_to_clipboard(&mut self) {
        if self.state.message_sets.is_empty() {
            return;
        }

        let mut clipboard = match Clipboard::new().map_err(|e| InteractiveError::ClipboardError {
            message: e.to_string(),
        }) {
            Ok(it) => it,
            Err(e) => {
                self.state.error_message = Some(format!("{}", e));
                return;
            }
        };
        let text: Vec<String> = self
            .state
            .message_sets
            .iter()
            .flat_map(|message_set| {
                let mut message_set_text: Vec<String> = vec![message_set.title.clone()];
                message_set_text.append(&mut message_set.messages.to_vec());
                message_set_text
            })
            .collect();

        let to_copy = text.join("\n");

        match clipboard
            .set_text(to_copy)
            .map_err(|e| InteractiveError::ClipboardError {
                message: e.to_string(),
            }) {
            Ok(_) => {}
            Err(e) => {
                self.state.error_message = Some(format!("{}", e));
            }
        }
    }

    pub fn start_command_input(&mut self) {
        self.reset_state(true);
        self.state.mode = AppMode::EnterCommand;
    }

    pub fn cancel_command(&mut self) {
        self.reset_state(true);
        self.state.mode = AppMode::Normal;
    }

    pub async fn run_command(&mut self) {
        match self.state.command_parameters.clone() {
            Some(CommandParameters::RandomArtists {
                year,
                month,
                count: artist_count,
                min_listens,
            }) => self.run_random_artists(year, month, artist_count, min_listens),
            Some(CommandParameters::ArtistSongs { name }) => {
                self.run_artist_songs(&name.unwrap_or_default());
            }
            Some(CommandParameters::ArtistsOnDay { date }) => {
                self.run_artists_on_day(date.unwrap_or_default());
            }
            Some(CommandParameters::PrintStatistics { year }) => self.run_print_statistics(year),
            Some(CommandParameters::GetFileNames { input_folder }) => {
                self.get_listen_file_names(&input_folder).await;
            }
            Some(CommandParameters::ProcessListens { files }) => {
                self.run_process_listens(files).await;
            }
            Some(CommandParameters::TopArtists {
                count: artist_count,
                year,
            }) => {
                self.run_top_artists(artist_count, year);
            }
            Some(CommandParameters::TopSongs { count, year }) => self.run_top_songs(count, year),
            Some(CommandParameters::MostSkipped { count }) => self.run_most_skipped(count),
            None => {}
        }
    }

    async fn get_listen_file_names(&mut self, input_folder: &str) {
        let streaming_files = fs::read_dir(input_folder);

        match streaming_files {
            Ok(read_dir) => {
                let mut paths: Vec<PathBuf> = Vec::new();

                for entry in read_dir.flatten() {
                    paths.push(entry.path());
                }

                let messages = match paths.last() {
                    Some(it) => vec![
                        format!("Found {} possible files", paths.len()),
                        format!("Processing {}", it.display()),
                    ],
                    None => vec![format!("Found {} possible files", paths.len())],
                };

                let message_set = AppMessageSet {
                    title: "Process listens".to_string(),
                    messages,
                };
                let parameters = CommandParameters::ProcessListens { files: paths };
                self.state.command_parameters = Some(parameters);

                self.state.message_sets.insert(0, message_set);
            }
            Err(e) => {
                self.reset_state(true);
                self.state.error_message = Some(e.to_string());
            }
        }
    }

    async fn run_process_listens(&mut self, mut files: Vec<PathBuf>) {
        if let Some(path) = files.pop() {
            let result = processing::process_file(&path, &self.store, &self.repository).await;

            let message = match result {
                Ok(events) => {
                    let count = events.len();

                    events
                        .into_iter()
                        .for_each(|event| self.processor.process_event(&event));

                    format!("Added {} events from {}", count, path.display())
                }
                Err(e) => format!("Error: {}", e),
            };

            let mut messages = match files.last() {
                Some(it) => vec![message, format!("Processing {}", it.display())],
                None => vec![message],
            };

            match self
                .state
                .message_sets
                .iter_mut()
                .find(|set| set.title.eq("Process listens"))
            {
                Some(it) => it.messages.append(&mut messages),
                None => {
                    let message_set = AppMessageSet {
                        title: "Process listens".to_string(),
                        messages,
                    };

                    self.state.message_sets.insert(0, message_set);
                }
            }

            self.state.command_parameters = Some(CommandParameters::ProcessListens { files });
        } else {
            match self
                .state
                .message_sets
                .iter_mut()
                .find(|set| set.title.eq("Process listens"))
            {
                Some(it) => it.messages.push("Done processing".to_string()),
                None => {
                    let message_set = AppMessageSet {
                        title: "Process listens".to_string(),
                        messages: vec!["Done processing".to_string()],
                    };

                    self.state.message_sets.insert(0, message_set);
                }
            }
            self.state.command_parameters = None;
        }
    }

    fn run_top_artists(&mut self, artist_count: usize, year: Option<i32>) {
        if let Some(y) = year {
            let title = format!("Top artists (year: {}, count: {})", y, artist_count);
            if let Some(year_counts) = self.processor.year_count(y) {
                let artist_song_counters = year_counts.artists_counts.top(artist_count);
                let messages: Vec<String> = artist_song_counters
                    .into_iter()
                    .map(|counter| counter.total_plays_display())
                    .collect();
                self.state
                    .message_sets
                    .insert(0, AppMessageSet { title, messages })
            } else {
                self.state.message_sets.insert(
                    0,
                    AppMessageSet {
                        title,
                        messages: vec!["No artists found".to_string()],
                    },
                );
            }
        } else {
            let title = format!("Top artists (count: {})", artist_count);
            let artist_counters = self.processor.artists_counts.top(artist_count);
            let messages: Vec<String> = artist_counters
                .into_iter()
                .map(|counter| counter.total_plays_display())
                .collect();
            self.state
                .message_sets
                .insert(0, AppMessageSet { title, messages })
        }

        self.state.command_parameters = None;
    }

    fn run_top_songs(&mut self, count: usize, year: Option<i32>) {
        if let Some(y) = year {
            let title = format!("Top songs (year: {}, count: {})", y, count);
            if let Some(year_counts) = self.processor.year_count(y) {
                let artist_song_counters = year_counts.artists_counts.top_songs(count);
                let messages: Vec<String> = artist_song_counters
                    .into_iter()
                    .map(|count| format!("{}", count))
                    .collect();
                self.state
                    .message_sets
                    .insert(0, AppMessageSet { title, messages })
            } else {
                self.state.message_sets.insert(
                    0,
                    AppMessageSet {
                        title,
                        messages: vec!["No artists found".to_string()],
                    },
                );
            }
        } else {
            let title = format!("Top songs (count: {})", count);
            let artist_counters = self.processor.artists_counts.top_songs(count);
            let messages: Vec<String> = artist_counters
                .into_iter()
                .map(|count| format!("{}", count))
                .collect();
            self.state
                .message_sets
                .insert(0, AppMessageSet { title, messages })
        }

        self.state.command_parameters = None;
    }

    fn run_most_skipped(&mut self, count: usize) {
        let most_skipped = self.processor.top_skipped(count);
        let title = format!("Most skipped songs: (count: {})", count);
        let messages: Vec<String> = most_skipped
            .iter()
            .map(|song_count| format!("{}", song_count))
            .collect();
        self.state
            .message_sets
            .insert(0, AppMessageSet { title, messages });

        self.state.command_parameters = None;
    }

    fn run_random_artists(
        &mut self,
        year: Option<i32>,
        month: Option<u32>,
        artist_count: usize,
        min_listens: u64,
    ) {
        let mut artist_names: HashSet<String> = HashSet::new();

        let year_counts = if let Some(y) = year {
            self.processor
                .year_count(y)
                .map(|year_count| vec![year_count])
                .unwrap_or_default()
        } else {
            self.processor.year_counts()
        };

        for year_count in year_counts.iter() {
            let artist_counts = if let Some(m) = month {
                year_count
                    .month_count(m)
                    .map(|month_counts| month_counts.over_min_plays(min_listens))
            } else {
                Some(year_count.over_min_plays(min_listens))
            }
            .unwrap_or_default();

            for artist_count in artist_counts.iter() {
                if !artist_names.contains(&artist_count.artist_name.0) {
                    artist_names.insert(artist_count.artist_name.0.clone());
                }
            }
        }

        let year_text = year
            .map(|y| format!("{}", y))
            .unwrap_or_else(|| "None".to_string());
        let month_text = month
            .map(|m| format!("{}", m))
            .unwrap_or_else(|| "None".to_string());
        let title = format!(
            "Artist search (year: {}, month: {}, min listens: {}, count: {})",
            year_text, month_text, min_listens, artist_count
        );

        let messages = if artist_names.is_empty() {
            vec!["No artists found".to_string()]
        } else {
            artist_names.iter().take(artist_count).cloned().collect()
        };

        self.state
            .message_sets
            .insert(0, AppMessageSet { title, messages });

        self.state.command_parameters = None;
    }

    fn run_artist_songs(&mut self, name: &str) {
        let mut songs: Vec<String> = self
            .processor
            .artist_song_counter(&ArtistName(name.to_string()))
            .map(|artist_counter| {
                artist_counter
                    .play_details
                    .all_song_plays()
                    .iter()
                    .map(|song_play| song_play.0.clone())
                    .collect()
            })
            .unwrap_or_default();

        songs.sort();
        songs.dedup();

        self.state.message_sets.insert(
            0,
            AppMessageSet {
                title: format!("Songs for {}", name),
                messages: songs,
            },
        );

        self.state.command_parameters = None;
    }

    fn run_artists_on_day(&mut self, date: NaiveDate) {
        let names = self
            .processor
            .artists_on_day(date)
            .into_iter()
            .map(|artist_counter| artist_counter.total_plays_display())
            .collect::<Vec<String>>();

        let title = format!("Artists listened to on {}", date.format("%Y-%m-%d"));

        let messages = if names.is_empty() {
            vec!["No artists found".to_string()]
        } else {
            names
        };

        self.state
            .message_sets
            .insert(0, AppMessageSet { title, messages });

        self.state.command_parameters = None;
    }

    fn run_print_statistics(&mut self, year: Option<i32>) {
        if let Some(y) = year {
            let title = format!("Statistics for {}", y);
            if let Some(year_counts) = self.processor.year_count(y) {
                let general = year_counts.artists_counts.general_stats(5);
                self.add_general_stats_to_messages(&general, &title);
            } else {
                self.state.message_sets.insert(
                    0,
                    AppMessageSet {
                        title: format!("Statistics for {}", y),
                        messages: vec!["No statistics gathered".to_string()],
                    },
                );
            }
        } else {
            let general = self.processor.artists_counts.general_stats(5);
            self.add_general_stats_to_messages(&general, "Statistics");
        }

        self.state.command_parameters = None;
    }

    fn add_general_stats_to_messages(&mut self, general: &General, title: &str) {
        self.state.message_sets.insert(
            0,
            AppMessageSet {
                title: title.to_string(),
                messages: vec![format!(
                    "You've listened to {} artists",
                    general.count_artists_listened_to
                )],
            },
        );

        self.state.message_sets.insert(
            1,
            AppMessageSet {
                title: "Most listened to artists".to_string(),
                messages: general.artist_total_plays.to_vec(),
            },
        );
    }

    fn reset_state(&mut self, reset_error_message: bool) {
        if reset_error_message {
            self.state.error_message = None;
        }
        self.state.command_name = None;
        self.state.command_parameters = None;
        self.state.input.clear();
        self.state.command_parameter_inputs.clear();
    }
}
