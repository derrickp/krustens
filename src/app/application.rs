use std::{collections::HashSet, fs, path::PathBuf, str::FromStr, sync::Arc};

use arboard::Clipboard;
use chrono::{Local, NaiveDate};
use strum::IntoEnumIterator;
use tokio::sync::Mutex;

use crate::{
    errors::InteractiveError,
    persistence::{fs::FileWriter, EventStore, Format, Writer},
    processing,
    projections::{
        statistics::{ArtistsCounts, EventProcessor, MonthCounts},
        ListenTrackerRepository,
    },
    track_plays::ArtistName,
};

use super::{
    BarDataPoint, CommandName, CommandParameters, MessageSet, Mode, Output, OutputFolder, State,
};

pub struct Application {
    store: Arc<dyn EventStore>,
    repository: Arc<Mutex<dyn ListenTrackerRepository>>,
    pub processor: EventProcessor,
    pub state: State,
}

impl Application {
    pub fn new(
        store: Arc<dyn EventStore>,
        repository: Arc<Mutex<dyn ListenTrackerRepository>>,
    ) -> Application {
        Application {
            store,
            repository,
            processor: EventProcessor::default(),
            state: State::default(),
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

    pub fn current_page_display(&self) -> usize {
        if self.state.output.is_empty() {
            0
        } else {
            self.state.current_page + 1
        }
    }

    pub fn num_pages(&self) -> usize {
        self.state.output.len()
    }

    pub fn go_to_next_page(&mut self) {
        let next_page = self.state.current_page + 1;
        if next_page >= self.state.output.len() {
            self.state.current_page = 0;
        } else {
            self.state.current_page = next_page;
        }
    }

    pub fn go_to_previous_page(&mut self) {
        if self.state.current_page == 0 {
            return;
        }

        self.state.current_page -= 1;
    }

    pub fn push_input_char(&mut self, c: char) {
        self.state.input.push(c);
    }

    pub fn pop_input_char(&mut self) {
        self.state.input.pop();
    }

    pub fn current_input(&self) -> &str {
        &self.state.input
    }

    pub fn mode(&self) -> &Mode {
        &self.state.mode
    }

    pub fn current_parameter_description(&self) -> Option<String> {
        self.state
            .command_parameter_inputs
            .get(0)
            .map(|spec| spec.description())
    }

    pub fn error_message(&self) -> &Option<String> {
        &self.state.error_message
    }

    pub fn current_output(&self) -> Option<Output> {
        match self.mode() {
            Mode::CommandParameters => None,
            Mode::EnterCommand => Some(Output::MessageSet(self.state.command_message_set())),
            Mode::Processing | Mode::Normal => {
                self.state.output.get(self.state.current_page).cloned()
            }
        }
    }

    pub fn autocomplete_command_name(&mut self) {
        let names: Vec<CommandName> = CommandName::iter()
            .filter(|name| name.to_string().starts_with(&self.state.input))
            .collect();
        if names.len() == 1 {
            let name = names.get(0).unwrap();
            self.state.input = name.to_string();
        }
    }

    pub async fn tick(&mut self) -> Result<(), InteractiveError> {
        match self.state.mode {
            Mode::CommandParameters => {}
            Mode::EnterCommand => {}
            Mode::Normal => {}
            Mode::Processing => {
                self.run_command().await;
                if self.state.command_parameters.is_none() {
                    self.reset_state(false);
                    self.state.mode = Mode::Normal;
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
                    self.state.mode = Mode::Processing;
                }
            }
            Err(e) => {
                self.reset_state(true);
                self.state.error_message = Some(e.to_string());
                self.state.mode = Mode::Normal;
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
                self.state.mode = Mode::CommandParameters;
            }
            Err(_) => {
                self.state.mode = Mode::Normal;
                self.state.error_message = Some("Unknown command name".to_string());
            }
        }
    }

    fn message_sets(&self) -> Vec<&MessageSet> {
        self.state
            .output
            .iter()
            .filter_map(|output| match output {
                super::Output::MessageSet(message_set) => Some(message_set),
                super::Output::BarChart(_) => None,
            })
            .collect()
    }

    async fn run_export_to_file(&mut self, output_folder: &str, format: Format) {
        let folder = OutputFolder {
            root: output_folder.to_string(),
        };
        let writer = FileWriter {
            folder: Box::new(folder),
        };

        let today = Local::now().format("%Y-%m-%d %H:%M:%S");
        let message_sets: Vec<MessageSet> = self
            .state
            .output
            .iter()
            .filter_map(|output| match output {
                super::Output::MessageSet(message_set) => Some(message_set.clone()),
                super::Output::BarChart(_) => None,
            })
            .collect();
        writer
            .write(&message_sets, &format!("messages_{}", &today), format)
            .await
            .unwrap();
        self.state.command_parameters = None;
    }

    pub fn copy_to_clipboard(&mut self) {
        let message_sets = self.message_sets();
        if message_sets.is_empty() {
            return;
        }

        let mut clipboard = match Clipboard::new().map_err(|e| InteractiveError::ClipboardError {
            message: e.to_string(),
        }) {
            Ok(it) => it,
            Err(e) => {
                self.state.error_message = Some(format!("{e}"));
                return;
            }
        };
        let text: Vec<String> = message_sets
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
                self.state.error_message = Some(format!("{e}"));
            }
        }
    }

    pub fn start_command_input(&mut self) {
        self.reset_state(true);
        self.state.mode = Mode::EnterCommand;
    }

    pub fn cancel_command(&mut self) {
        self.reset_state(true);
        self.state.mode = Mode::Normal;
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
                month,
            }) => {
                self.run_top_artists(artist_count, year, month);
            }
            Some(CommandParameters::TopSongs { count, year }) => self.run_top_songs(count, year),
            Some(CommandParameters::MostSkipped { count }) => self.run_most_skipped(count),
            Some(CommandParameters::Export {
                output_folder,
                format,
            }) => self.run_export_to_file(&output_folder, format).await,
            Some(CommandParameters::Chart { year }) => {
                self.run_chart(year);
            }
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

                let message_set = MessageSet {
                    title: "Process listens".to_string(),
                    messages,
                };
                let parameters = CommandParameters::ProcessListens { files: paths };
                self.state.command_parameters = Some(parameters);

                self.state.output.insert(0, Output::MessageSet(message_set));
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
                Err(e) => format!("Error: {e}"),
            };

            let mut messages = match files.last() {
                Some(it) => vec![message, format!("Processing {}", it.display())],
                None => vec![message],
            };

            match self
                .state
                .output
                .iter_mut()
                .find_map(|output| match output {
                    Output::MessageSet(set) => {
                        if set.title.eq("Process listens") {
                            Some(set)
                        } else {
                            None
                        }
                    }
                    Output::BarChart(_) => None,
                }) {
                Some(it) => it.messages.append(&mut messages),
                None => {
                    let message_set = MessageSet {
                        title: "Process listens".to_string(),
                        messages,
                    };

                    self.state.output.insert(0, Output::MessageSet(message_set));
                }
            }

            self.state.command_parameters = Some(CommandParameters::ProcessListens { files });
        } else {
            match self
                .state
                .output
                .iter_mut()
                .find_map(|output| match output {
                    Output::MessageSet(set) => {
                        if set.title.eq("Process listens") {
                            Some(set)
                        } else {
                            None
                        }
                    }
                    Output::BarChart(_) => None,
                }) {
                Some(it) => it.messages.push("Done processing".to_string()),
                None => {
                    let message_set = MessageSet {
                        title: "Process listens".to_string(),
                        messages: vec!["Done processing".to_string()],
                    };

                    self.state.output.insert(0, Output::MessageSet(message_set));
                }
            }
            self.state.command_parameters = None;
        }
    }

    fn run_chart(&mut self, year: i32) {
        if let Some(year_count) = self.processor.year_count(year) {
            let mut month_counts = year_count.month_counts();
            month_counts.sort_by_key(|month_count| month_count.month);
            let data_points: Vec<BarDataPoint> = month_counts
                .iter()
                .map(|month_count| {
                    BarDataPoint::new(
                        format!("{:02}", month_count.month),
                        month_count.artists_counts.total_count(),
                    )
                })
                .collect();
            self.state.output.insert(
                0,
                Output::BarChart(super::BarChart {
                    title: format!("Bar Chart {year}"),
                    data_points,
                }),
            );
        } else {
            let message_set = MessageSet {
                title: format!("Chart (year: {year})"),
                messages: vec!["No data for year".to_string()],
            };
            self.state.output.insert(0, Output::MessageSet(message_set));
        }

        self.state.command_parameters = None;
    }

    fn run_top_artists(&mut self, artist_count: usize, year: Option<i32>, month: Option<u32>) {
        println!("{year:?}");
        match (year, month) {
            (None, None) => self.top_artists(artist_count),
            (None, Some(m)) => self.top_artists_for_month(artist_count, m),
            (Some(y), None) => self.top_artists_for_year(artist_count, y),
            (Some(y), Some(m)) => self.top_artists_for_year_month(artist_count, y, m),
        }

        self.state.command_parameters = None;
    }

    fn top_artists_for_month(&mut self, artist_count: usize, month: u32) {
        let title = format!("Top artists (month: {month}, count: {artist_count})");
        let month_counts = self.processor.month_counts(month);
        let artist_counts = MonthCounts::merge(month_counts);
        let artist_song_counters = artist_counts.top(artist_count);
        let messages: Vec<String> = artist_song_counters
            .into_iter()
            .map(|counter| counter.total_plays_display())
            .collect();
        self.state
            .output
            .insert(0, Output::MessageSet(MessageSet { title, messages }));
    }

    fn top_artists_for_year_month(&mut self, artist_count: usize, year: i32, month: u32) {
        let title = format!("Top artists (year: {year}, month: {month}, count: {artist_count})");
        if let Some(year_counts) = self.processor.year_count(year) {
            if let Some(month_counts) = year_counts.month_count(month) {
                let artist_song_counters = month_counts.artists_counts.top(artist_count);
                let messages: Vec<String> = artist_song_counters
                    .into_iter()
                    .map(|counter| counter.total_plays_display())
                    .collect();
                self.state
                    .output
                    .insert(0, Output::MessageSet(MessageSet { title, messages }))
            } else {
                self.state.output.insert(
                    0,
                    Output::MessageSet(MessageSet {
                        title,
                        messages: vec!["No artists found".to_string()],
                    }),
                );
            }
        } else {
            self.state.output.insert(
                0,
                Output::MessageSet(MessageSet {
                    title,
                    messages: vec!["No artists found".to_string()],
                }),
            );
        }
    }

    fn top_artists_for_year(&mut self, artist_count: usize, year: i32) {
        let title = format!("Top artists (year: {year}, count: {artist_count})");
        if let Some(year_counts) = self.processor.year_count(year) {
            let artist_song_counters = year_counts.artists_counts.top(artist_count);
            let messages: Vec<String> = artist_song_counters
                .into_iter()
                .map(|counter| counter.total_plays_display())
                .collect();
            self.state
                .output
                .insert(0, Output::MessageSet(MessageSet { title, messages }))
        } else {
            self.state.output.insert(
                0,
                Output::MessageSet(MessageSet {
                    title,
                    messages: vec!["No artists found".to_string()],
                }),
            );
        }
    }

    fn top_artists(&mut self, artist_count: usize) {
        let title = format!("Top artists (count: {artist_count})");
        let artist_counters = self.processor.artists_counts.top(artist_count);
        let messages: Vec<String> = artist_counters
            .into_iter()
            .map(|counter| counter.total_plays_display())
            .collect();
        self.state
            .output
            .insert(0, Output::MessageSet(MessageSet { title, messages }))
    }

    fn run_top_songs(&mut self, count: usize, year: Option<i32>) {
        if let Some(y) = year {
            let title = format!("Top songs (year: {y}, count: {count})");
            if let Some(year_counts) = self.processor.year_count(y) {
                let artist_song_counters = year_counts.artists_counts.top_songs(count);
                let messages: Vec<String> = artist_song_counters
                    .into_iter()
                    .map(|count| format!("{count}"))
                    .collect();
                self.state
                    .output
                    .insert(0, Output::MessageSet(MessageSet { title, messages }))
            } else {
                self.state.output.insert(
                    0,
                    Output::MessageSet(MessageSet {
                        title,
                        messages: vec!["No artists found".to_string()],
                    }),
                );
            }
        } else {
            let title = format!("Top songs (count: {count})");
            let artist_counters = self.processor.artists_counts.top_songs(count);
            let messages: Vec<String> = artist_counters
                .into_iter()
                .map(|count| format!("{count}"))
                .collect();
            self.state
                .output
                .insert(0, Output::MessageSet(MessageSet { title, messages }))
        }

        self.state.command_parameters = None;
    }

    fn run_most_skipped(&mut self, count: usize) {
        let most_skipped = self.processor.top_skipped(count);
        let title = format!("Most skipped songs: (count: {count})");
        let messages: Vec<String> = most_skipped
            .iter()
            .map(|song_count| format!("{song_count}"))
            .collect();
        self.state
            .output
            .insert(0, Output::MessageSet(MessageSet { title, messages }));

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
            .map(|y| format!("{y}"))
            .unwrap_or_else(|| "None".to_string());
        let month_text = month
            .map(|m| format!("{m}"))
            .unwrap_or_else(|| "None".to_string());
        let title = format!(
            "Random artists (year: {year_text}, month: {month_text}, min listens: {min_listens}, count: {artist_count})"
        );

        let messages = if artist_names.is_empty() {
            vec!["No artists found".to_string()]
        } else {
            artist_names.iter().take(artist_count).cloned().collect()
        };

        self.state
            .output
            .insert(0, Output::MessageSet(MessageSet { title, messages }));

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

        self.state.output.insert(
            0,
            Output::MessageSet(MessageSet {
                title: format!("Songs for {name}"),
                messages: songs,
            }),
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
            .output
            .insert(0, Output::MessageSet(MessageSet { title, messages }));

        self.state.command_parameters = None;
    }

    fn run_print_statistics(&mut self, year: Option<i32>) {
        let mut message_sets = if let Some(y) = year {
            if let Some(year_counts) = self.processor.year_count(y) {
                self.summarize_to_message_sets(&year, &year_counts.artists_counts)
            } else {
                vec![MessageSet {
                    title: format!("Statistics for {y}"),
                    messages: vec!["No statistics gathered".to_string()],
                }]
            }
        } else {
            self.summarize_to_message_sets(&year, &self.processor.artists_counts)
        };

        message_sets.reverse();

        for message_set in message_sets.into_iter() {
            self.state.output.insert(0, Output::MessageSet(message_set));
        }

        self.state.command_parameters = None;
    }

    fn summarize_to_message_sets(
        &self,
        year: &Option<i32>,
        artist_counts: &ArtistsCounts,
    ) -> Vec<MessageSet> {
        let general = artist_counts.general_stats(5);
        let total_played_message = if artist_counts.time_played.time_hr > 2.0 {
            format!(
                "Listened for {:.1} hours",
                artist_counts.time_played.time_hr
            )
        } else {
            format!(
                "Listened for {:.1} minutes",
                artist_counts.time_played.time_min
            )
        };

        let general_stats_title = year
            .map(|y| format!("General statistics (year: {y})"))
            .unwrap_or_else(|| "General statistics (year: None)".to_string());
        let most_listened_title = year
            .map(|y| format!("Most listened artists (year: {y})"))
            .unwrap_or_else(|| "Most listened artists (year: None)".to_string());

        let most_listened_songs_title = year
            .map(|y| format!("Most listened songs (year: {y})"))
            .unwrap_or_else(|| "Most listened songs (year: None)".to_string());

        let most_listened_songs_unique_artist_title = year
            .map(|y| format!("Most listened songs (unique artist, year: {y})"))
            .unwrap_or_else(|| "Most listened songs (unique artist, year: None)".to_string());

        vec![
            MessageSet {
                title: general_stats_title,
                messages: vec![
                    format!(
                        "You've listened to {} artists",
                        general.count_artists_listened_to
                    ),
                    total_played_message,
                ],
            },
            MessageSet {
                title: most_listened_title,
                messages: general.artist_total_plays.to_vec(),
            },
            MessageSet {
                title: most_listened_songs_title,
                messages: general.most_played_songs.to_vec(),
            },
            MessageSet {
                title: most_listened_songs_unique_artist_title,
                messages: general.artist_most_played_songs.to_vec(),
            },
        ]
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
