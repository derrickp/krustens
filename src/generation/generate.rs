use std::sync::Arc;

use crate::{
    persistence::{fs::FileWriter, write_artists_counts, EventStore},
    projections::statistics::{EventProcessor, StatisticsFolder},
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
    let folder = StatisticsFolder::builder().root_path(output_folder).build();
    let event_stream = store.get_events("listens".to_string()).await.unwrap();
    let mut processor = EventProcessor::default();

    for event in event_stream.events.iter() {
        processor.process_event(event);
    }

    let full_writer = FileWriter {
        folder: Box::new(folder),
    };
    write_artists_counts(full_writer, &processor.artists_counts, count).await;

    if split_monthly {
        for year_count in processor.year_counts() {
            let year_folder = StatisticsFolder::builder()
                .root_path(output_folder)
                .year(year_count.year)
                .build();
            let year_writer = FileWriter {
                folder: Box::new(year_folder),
            };
            write_artists_counts(year_writer, &year_count.artists_counts, count).await;

            if split_monthly {
                for month_count in year_count.month_counts() {
                    let month_folder = StatisticsFolder::builder()
                        .root_path(output_folder)
                        .year(year_count.year)
                        .month(month_count.month)
                        .build();
                    let month_writer = FileWriter {
                        folder: Box::new(month_folder),
                    };

                    write_artists_counts(month_writer, &month_count.artists_counts, count).await;
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
        let year_folder = StatisticsFolder::builder()
            .root_path(output_folder)
            .year(year)
            .build();

        write_artists_counts(
            FileWriter {
                folder: Box::new(year_folder),
            },
            &year_count.artists_counts,
            count,
        )
        .await;

        if split_monthly {
            for month_count in year_count.month_counts() {
                let folder = StatisticsFolder::builder()
                    .root_path(output_folder)
                    .year(year_count.year)
                    .month(month_count.month)
                    .build();

                write_artists_counts(
                    FileWriter {
                        folder: Box::new(folder),
                    },
                    &month_count.artists_counts,
                    count,
                )
                .await;
            }
        }
    }
}
