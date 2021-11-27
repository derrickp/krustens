use serde::Deserialize;

use super::{read_error::ReadError, reader::Reader};

pub struct JsonReader<'src> {
    pub contents: &'src String,
}

impl<'a, T: Deserialize<'a>> Reader<'a, T> for JsonReader<'a> {
    fn read(&self) -> Result<T, ReadError> {
        match serde_json::from_str(self.contents) {
            Ok(it) => Ok(it),
            Err(e) => Err(ReadError {
                message: e.to_string(),
            }),
        }
    }
}
