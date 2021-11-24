use super::get_events_error::GetEventsError;

#[derive(Debug, Clone)]
pub enum Error {
    GetEventsError(GetEventsError),
}
