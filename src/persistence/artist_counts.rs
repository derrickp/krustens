use crate::projections::statistics::{ArtistsCounts, FileName, Folder};

use super::{fs::FileWriter, Writer};

pub async fn write_artists_counts(stats_folder: &Folder, stats: &ArtistsCounts, count: usize) {
    stats_folder.create_if_necessary();

    FileWriter::yaml_writer(stats_folder.file_name(&FileName::General))
        .write(&stats.general_stats(count))
        .await
        .unwrap();
    FileWriter::from(stats_folder.file_name(&FileName::Complete))
        .write(&stats.all())
        .await
        .unwrap();
    FileWriter::from(stats_folder.file_name(&FileName::Top50))
        .write(&stats.top(50))
        .await
        .unwrap();
    FileWriter::from(stats_folder.file_name(&FileName::Top100))
        .write(&stats.top(100))
        .await
        .unwrap();
}
