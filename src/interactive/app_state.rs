use chrono::NaiveDate;
use strum::IntoEnumIterator;

use crate::errors::InteractiveError;

use super::{AppMessageSet, AppMode, CommandName, CommandParameterSpec, CommandParameters};

#[derive(Default)]
pub struct AppState {
    pub input: String,
    pub mode: AppMode,
    pub command_name: Option<CommandName>,
    pub error_message: Option<String>,
    pub message_sets: Vec<AppMessageSet>,
    pub command_parameter_inputs: Vec<CommandParameterSpec>,
    pub command_parameters: Option<CommandParameters>,
    pub processing_messages: Vec<AppMessageSet>,
}

impl AppState {
    pub fn display_sets(&self) -> Vec<AppMessageSet> {
        match self.mode {
            AppMode::CommandParameters => Vec::new(),
            AppMode::EnterCommand => {
                vec![AppMessageSet {
                    title: "Commands".to_string(),
                    messages: CommandName::iter()
                        .map(|command| {
                            format!("{} - {}", command.to_string(), command.description())
                        })
                        .collect(),
                }]
            }
            AppMode::Normal | AppMode::Processing => self.message_sets.to_vec(),
        }
    }

    pub fn insert_command_parameter(
        &mut self,
        text: &str,
        spec: &CommandParameterSpec,
    ) -> Result<(), InteractiveError> {
        match spec {
            CommandParameterSpec::Year {
                optional,
                description: _,
            } => {
                if let Ok(year) = text.parse::<i32>() {
                    self.add_year_parameter(year);
                    Ok(())
                } else if !optional {
                    Err(InteractiveError::RequiredParameterNotSet {
                        name: "year".to_string(),
                    })
                } else {
                    Ok(())
                }
            }
            CommandParameterSpec::Month {
                optional,
                description: _,
            } => {
                if let Some(month) = text.parse::<u32>().ok().filter(|m| (&1..=&12).contains(&m)) {
                    self.add_month_parameter(month);
                    Ok(())
                } else if !optional {
                    Err(InteractiveError::RequiredParameterNotSet {
                        name: "month".to_string(),
                    })
                } else {
                    Ok(())
                }
            }
            CommandParameterSpec::MinListens { description: _ } => {
                if let Ok(min) = text.parse::<u64>() {
                    self.add_min_listens_parameter(min);
                }

                Ok(())
            }
            CommandParameterSpec::ArtistCount { description: _ } => {
                if let Ok(count) = text.parse::<usize>() {
                    self.add_artist_count_parameter(count);
                }

                Ok(())
            }
            CommandParameterSpec::Date {
                optional: _,
                description: _,
            } => match NaiveDate::parse_from_str(text, "%Y-%m-%d") {
                Ok(date) => {
                    self.add_date_parameter(date);
                    Ok(())
                }
                Err(e) => Err(InteractiveError::ParsingIssue {
                    message: e.to_string(),
                }),
            },
            CommandParameterSpec::ArtistName {
                optional,
                description: _,
            } => {
                if text.is_empty() && !optional {
                    Err(InteractiveError::RequiredParameterNotSet {
                        name: "name".to_string(),
                    })
                } else {
                    self.add_name_parameter(text);
                    Ok(())
                }
            }
            CommandParameterSpec::InputFolder { description: _ } => {
                if !text.is_empty() {
                    self.add_input_folder_parameter(text);
                }
                Ok(())
            }
        }
    }

    fn add_input_folder_parameter(&mut self, input_folder: &str) {
        if self.command_parameters.is_none() {
            self.set_default_command_parameters();
        }

        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_input_folder_parameter(input_folder));
        }
    }

    fn add_year_parameter(&mut self, year: i32) {
        if self.command_parameters.is_none() {
            self.set_default_command_parameters();
        }

        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_year_parameter(year));
        }
    }

    fn add_month_parameter(&mut self, month: u32) {
        if self.command_parameters.is_none() {
            self.set_default_command_parameters();
        }

        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_month_parameter(month));
        }
    }

    fn add_min_listens_parameter(&mut self, min_listens: u64) {
        if self.command_parameters.is_none() {
            self.set_default_command_parameters();
        }

        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_min_listens_parameter(min_listens));
        }
    }

    fn add_artist_count_parameter(&mut self, count: usize) {
        if self.command_parameters.is_none() {
            self.set_default_command_parameters();
        }

        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_artist_count_parameter(count));
        }
    }

    fn add_date_parameter(&mut self, date: NaiveDate) {
        if self.command_parameters.is_none() {
            self.set_default_command_parameters();
        }

        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_date_parameter(date));
        }
    }

    fn add_name_parameter(&mut self, name: &str) {
        if self.command_parameters.is_none() {
            self.set_default_command_parameters();
        }

        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_name_parameter(name));
        }
    }

    fn set_default_command_parameters(&mut self) {
        if self.command_parameters.is_some() {
            return;
        }

        match &self.command_name {
            Some(name) => {
                self.command_parameters = Some(name.default_parameters());
            }
            None => {}
        }
    }
}
