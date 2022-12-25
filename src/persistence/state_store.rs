use crate::{
    app::State,
    errors::{ReadError, WriteError},
};

#[async_trait::async_trait]
pub trait StateStore {
    async fn push(&mut self, state: &State) -> Result<(), WriteError>;
    async fn get(&self) -> Result<State, ReadError>;
}
