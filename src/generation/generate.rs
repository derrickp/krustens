use std::sync::Arc;

use crate::{
    persistence::{write_artists_counts, EventStore},
    projections::statistics::{EventProcessor, Folder},
};

pub async fn generate_stats(
    output_folder: &str,
    count: usize,
    store: Arc<impl EventStore>,
    year: Option<i32>,
    split_monthly: bool,
) {
    match year {
        Some(it) => {
            generate_stats_for_single_year(output_folder, count, store, it, split_monthly).await
        }
        _ => generate_all_stats(output_folder, count, store, split_monthly).await,
    }
}

async fn generate_all_stats(
    output_folder: &str,
    count: usize,
    store: Arc<impl EventStore>,
    split_monthly: bool,
) {
    let folder = Folder::builder().root_path(output_folder).build();
    let event_stream = store.get_events("listens".to_string()).await.unwrap();
    let mut processor = EventProcessor::default();

    for event in event_stream.events.iter() {
        processor.process_event(event);
    }

    write_artists_counts(&folder, &processor.artists_counts, count).await;

    if split_monthly {
        for year_count in processor.year_counts() {
            let year_folder = Folder {
                output_folder: output_folder.to_string(),
                year: Some(year_count.year),
                month: None,
            };
            year_folder.create_if_necessary();
            write_artists_counts(&year_folder, &year_count.artists_counts, count).await;

            if split_monthly {
                for month_count in year_count.month_counts() {
                    let folder = Folder {
                        output_folder: output_folder.to_string(),
                        year: Some(year_count.year),
                        month: Some(month_count.month),
                    };

                    folder.create_if_necessary();
                    write_artists_counts(&folder, &month_count.artists_counts, count).await;
                }
            }
        }
    }
}

async fn generate_stats_for_single_year(
    output_folder: &str,
    count: usize,
    store: Arc<impl EventStore>,
    year: i32,
    split_monthly: bool,
) {
    let event_stream = store.get_events("listens".to_string()).await.unwrap();

    let mut processor = EventProcessor::default();
    for event in event_stream.events.iter() {
        processor.process_event(event);
    }

    for year_count in processor
        .year_counts()
        .iter()
        .filter(|year_count| year_count.year == year)
    {
        let year_folder = Folder::builder()
            .root_path(output_folder)
            .year(year)
            .build();
        year_folder.create_if_necessary();
        write_artists_counts(&year_folder, &year_count.artists_counts, count).await;

        if split_monthly {
            for month_count in year_count.month_counts() {
                let folder = Folder::builder()
                    .root_path(output_folder)
                    .year(year_count.year)
                    .month(month_count.month)
                    .build();

                folder.create_if_necessary();
                write_artists_counts(&folder, &month_count.artists_counts, count).await;
            }
        }
    }
}
