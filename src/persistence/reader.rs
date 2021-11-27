use serde::Deserialize;

use super::read_error::ReadError;

pub trait Reader<'a, T: Deserialize<'a>> {
    fn read(&self) -> Result<T, ReadError>;
}
