use chrono::Local;

use crate::persistence::{fs::Folder, OutputFolder};

pub fn setup_logging(folder: &OutputFolder) -> Result<(), std::io::Error> {
    folder.create_if_necessary();

    let today = Local::now().naive_local().format("%Y%m%d");
    let log_file = format!("{folder}/krustens_{today}.log");

    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        // Add blanket level filter -
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file(log_file)?)
        // Apply globally
        .apply()
        .unwrap();
    Ok(())
}
