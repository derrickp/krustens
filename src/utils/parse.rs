use chrono::{DateTime, NaiveDateTime, ParseResult};

pub fn parse_formatted_end_time(end_time: &str) -> ParseResult<NaiveDateTime> {
    NaiveDateTime::parse_from_str(end_time, "%Y-%m-%d %H:%M:%S")
}

pub fn parse_spotify_end_time(end_time: &str) -> ParseResult<NaiveDateTime> {
    NaiveDateTime::parse_from_str(end_time, "%Y-%m-%d %H:%M")
}

pub fn parse_end_time_rfc3339(end_time: &str) -> ParseResult<NaiveDateTime> {
    DateTime::parse_from_rfc3339(end_time).map(|date_time| date_time.naive_local())
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;

    use super::parse_end_time_rfc3339;

    #[test]
    fn end_time_parse_rfc3339() {
        let end_time = "2022-10-07T17:36:52.202Z";
        let parsed = parse_end_time_rfc3339(end_time).unwrap();
        assert_eq!(parsed.year(), 2022);
    }
}
