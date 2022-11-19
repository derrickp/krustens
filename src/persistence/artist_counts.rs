use crate::projections::statistics::{ArtistsCounts, FileName};

use super::{Format, Writer};

pub async fn write_artists_counts(writer: impl Writer, stats: &ArtistsCounts, count: usize) {
    writer
        .write(
            &stats.general_stats(count),
            &FileName::General.to_string(),
            Format::Yaml,
        )
        .await
        .unwrap();
    writer
        .write(&stats.all(), &FileName::Complete.to_string(), Format::Json)
        .await
        .unwrap();
    writer
        .write(&stats.top(50), &FileName::Top50.to_string(), Format::Json)
        .await
        .unwrap();
    writer
        .write(&stats.top(100), &FileName::Top100.to_string(), Format::Json)
        .await
        .unwrap();
}
