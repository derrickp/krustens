use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::{errors::InteractiveError, persistence::Format};

use super::{
    chart::BarBreakdown, CommandName, CommandParameterSpec, CommandParameters, Input, MessageSet,
    Mode, Output,
};

#[derive(Default, Deserialize, Serialize)]
pub struct State {
    pub input: Input,
    pub mode: Mode,
    pub error_message: Option<String>,
    pub command_parameter_inputs: Vec<CommandParameterSpec>,
    pub command_parameters: Option<CommandParameters>,
    pub current_page: usize,
    output: Vec<Output>,
    is_dirty: bool,
}

impl State {
    pub fn reset_dirty(&mut self) {
        self.is_dirty = false;
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn insert_output(&mut self, index: usize, output: Output) {
        self.is_dirty = true;
        self.output.insert(index, output);
    }

    pub fn output(&self) -> &Vec<Output> {
        &self.output
    }

    pub fn output_mut(&mut self) -> Vec<&mut Output> {
        self.is_dirty = true;
        self.output.iter_mut().collect()
    }

    pub fn command_message_set(&self) -> MessageSet {
        let mut messages: Vec<String> = CommandName::iter()
            .map(|command| format!("{} - {}", command.to_string(), command.description()))
            .collect();
        messages.sort();
        MessageSet::with_messages("Commands", messages)
    }

    pub fn insert_command_parameter(
        &mut self,
        text: &str,
        spec: &CommandParameterSpec,
    ) -> Result<(), InteractiveError> {
        self.is_dirty = true;
        match spec {
            CommandParameterSpec::Year { description: _ } => {
                if let Ok(year) = text.parse::<i32>() {
                    self.add_year_parameter(year);
                    Ok(())
                } else {
                    Ok(())
                }
            }
            CommandParameterSpec::Month { description: _ } => {
                if let Some(month) = text.parse::<u32>().ok().filter(|m| (&1..=&12).contains(&m)) {
                    self.add_month_parameter(month);
                    Ok(())
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
            CommandParameterSpec::Count { description: _ } => {
                if let Ok(count) = text.parse::<usize>() {
                    self.add_artist_count_parameter(count);
                }

                Ok(())
            }
            CommandParameterSpec::Date { description: _ } => {
                match NaiveDate::parse_from_str(text, "%Y-%m-%d") {
                    Ok(date) => {
                        self.add_date_parameter(date);
                        Ok(())
                    }
                    Err(e) => Err(InteractiveError::ParsingIssue {
                        message: e.to_string(),
                    }),
                }
            }
            CommandParameterSpec::ArtistName { description: _ } => {
                if text.is_empty() {
                    Err(InteractiveError::RequiredParameterNotSet {
                        name: "artist name".to_string(),
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
            CommandParameterSpec::OutputFolder { description: _ } => {
                if !text.is_empty() {
                    self.add_output_folder_parameter(text);
                }
                Ok(())
            }
            CommandParameterSpec::FileFormat { description: _ } => {
                if let Ok(format) = Format::try_from(text.to_string()) {
                    self.add_format_parameter(format);
                }
                Ok(())
            }
            CommandParameterSpec::BarBreakdown { description: _ } => {
                if let Ok(breakdown) = BarBreakdown::try_from(text) {
                    self.add_bar_breakdown_parameter(breakdown);
                }

                Ok(())
            }
        }
    }

    fn add_bar_breakdown_parameter(&mut self, breakdown: BarBreakdown) {
        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_bar_breakdown_parameter(breakdown));
        }
    }

    fn add_input_folder_parameter(&mut self, input_folder: &str) {
        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_input_folder_parameter(input_folder));
        }
    }

    fn add_output_folder_parameter(&mut self, output_folder: &str) {
        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_output_folder_parameter(output_folder));
        }
    }

    fn add_format_parameter(&mut self, format: Format) {
        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_format_parameter(format));
        }
    }

    fn add_year_parameter(&mut self, year: i32) {
        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_year_parameter(year));
        }
    }

    fn add_month_parameter(&mut self, month: u32) {
        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_month_parameter(month));
        }
    }

    fn add_min_listens_parameter(&mut self, min_listens: u64) {
        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_min_listens_parameter(min_listens));
        }
    }

    fn add_artist_count_parameter(&mut self, count: usize) {
        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_count_parameter(count));
        }
    }

    fn add_date_parameter(&mut self, date: NaiveDate) {
        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_date_parameter(date));
        }
    }

    fn add_name_parameter(&mut self, name: &str) {
        if let Some(parameters) = &self.command_parameters {
            self.command_parameters = Some(parameters.with_name_parameter(name));
        }
    }

    pub fn next_page(&mut self) {
        let next_page = self.current_page + 1;
        if next_page >= self.output.len() {
            self.current_page = 0;
        } else {
            self.current_page = next_page;
        }
    }

    pub fn previous_page(&mut self) {
        if self.current_page == 0 {
            self.current_page = (self.output.len() - 1).max(0)
        } else {
            self.current_page -= 1;
        }
    }

    pub fn reset(&mut self, reset_error_message: bool) {
        if reset_error_message {
            self.error_message = None;
        }
        self.command_parameters = None;
        self.input.clear();
        self.command_parameter_inputs.clear();
    }

    pub fn setup_for_command(&mut self, command_name: &CommandName) {
        self.command_parameter_inputs = command_name.parameters();
        self.command_parameters = Some(command_name.default_parameters());
        self.mode = Mode::CommandParameters;
    }
}
