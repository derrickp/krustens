use serde::Serialize;

use super::write_error::WriteError;

pub trait Writer<T: Serialize> {
    fn write(&self, value: &T) -> Result<bool, WriteError>;
}
