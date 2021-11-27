use std::{
    fs::File,
    io::{BufWriter, Write},
};

use serde::Serialize;

use super::{write_error::WriteError, writer::Writer};

pub struct FileWriter {
    path: String,
}

impl From<String> for FileWriter {
    fn from(path: String) -> Self {
        Self { path }
    }
}

impl<T: Serialize> Writer<T> for FileWriter {
    fn write(&self, value: &T) -> Result<bool, WriteError> {
        let file = match File::create(self.path.clone()) {
            Ok(it) => it,
            Err(e) => {
                return Err(WriteError {
                    message: e.to_string(),
                })
            }
        };
        let mut writer = BufWriter::new(file);
        match serde_json::to_writer(&mut writer, &value) {
            Ok(_) => {}
            Err(e) => {
                return Err(WriteError {
                    message: e.to_string(),
                })
            }
        }

        match writer.flush() {
            Ok(_) => Ok(true),
            Err(e) => Err(WriteError {
                message: e.to_string(),
            }),
        }
    }
}
