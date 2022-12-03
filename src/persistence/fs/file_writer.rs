use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
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

        let file_extension = match format {
            Format::Json => "json",
            Format::Yaml => "yaml",
        };

        let mut path_buf = PathBuf::new();
        path_buf.push(self.folder.path());
        path_buf.push(name);
        path_buf.set_extension(file_extension);

        let file = match File::create(&path_buf) {
            Ok(it) => it,
            Err(e) => {
                return Err(WriteError::CannotCreateFile {
                    path: format!("{}", &path_buf.display()),
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
                path: format!("{}", path_buf.display()),
                message: e.to_string(),
            }),
        }
    }
}
