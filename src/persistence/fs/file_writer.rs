use std::{
    fs::File,
    io::{BufWriter, Write},
};

use async_trait::async_trait;
use serde::Serialize;

use crate::{
    errors::WriteError,
    persistence::{Format, Writer},
};

use super::folder::Folder;

pub struct FileWriter {
    pub folder: Box<dyn Folder + Sync>,
}

impl From<Box<dyn Folder + Sync>> for FileWriter {
    fn from(folder: Box<dyn Folder + Sync>) -> Self {
        Self { folder }
    }
}

#[async_trait]
impl Writer for FileWriter {
    async fn write<T: Serialize + Sync>(
        &self,
        value: &T,
        name: &str,
        format: Format,
    ) -> Result<bool, WriteError> {
        self.folder.create_if_necessary();

        let full_path = self.folder.full_path(name);
        let file = match File::create(&full_path) {
            Ok(it) => it,
            Err(e) => {
                return Err(WriteError::CannotCreateFile {
                    path: full_path,
                    message: e.to_string(),
                })
            }
        };
        let mut writer = BufWriter::new(file);

        match format {
            Format::Json => match serde_json::to_writer_pretty(&mut writer, value) {
                Ok(_) => {}
                Err(e) => {
                    return Err(WriteError::FailedToSerializeJson {
                        message: e.to_string(),
                    })
                }
            },
            Format::Yaml => match serde_yaml::to_writer(&mut writer, value) {
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
                path: full_path,
                message: e.to_string(),
            }),
        }
    }
}
