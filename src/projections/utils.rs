use chrono::{NaiveDateTime, ParseResult};

pub fn parse_end_time(end_time: &str) -> ParseResult<NaiveDateTime> {
    NaiveDateTime::parse_from_str(end_time, "%Y-%m-%d %H:%M")
}
