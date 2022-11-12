use std::{
    fs::File,
    io::{BufWriter, Write},
};

use async_trait::async_trait;
use serde::Serialize;

use crate::errors::WriteError;

use super::writer::Writer;

pub enum FileType {
    Yaml,
    Json,
}

pub struct FileWriter {
    path: String,
    file_type: FileType,
}

impl From<String> for FileWriter {
    fn from(path: String) -> Self {
        Self {
            path,
            file_type: FileType::Json,
        }
    }
}

impl FileWriter {
    pub fn yaml_writer(path: String) -> Self {
        Self {
            path,
            file_type: FileType::Yaml,
        }
    }
}

#[async_trait]
impl<T: Serialize + std::marker::Sync> Writer<T> for FileWriter {
    async fn write(&self, value: &T) -> Result<bool, WriteError> {
        let file = match File::create(self.path.clone()) {
            Ok(it) => it,
            Err(e) => {
                return Err(WriteError::CannotCreateFile {
                    path: self.path.to_string(),
                    message: e.to_string(),
                })
            }
        };
        let mut writer = BufWriter::new(file);

        match self.file_type {
            FileType::Json => match serde_json::to_writer_pretty(&mut writer, value) {
                Ok(_) => {}
                Err(e) => {
                    return Err(WriteError::FailedToSerializeJson {
                        message: e.to_string(),
                    })
                }
            },
            FileType::Yaml => match serde_yaml::to_writer(&mut writer, value) {
                Ok(_) => {}
                Err(e) => {
                    return Err(WriteError::FailedToSerializeYaml {
                        message: e.to_string(),
                    })
                }
            },
        }

        match writer.flush() {
            Ok(_) => Ok(true),
            Err(e) => Err(WriteError::CannotWriteToFile {
                path: self.path.to_string(),
                message: e.to_string(),
            }),
        }
    }
}
