pub use simplelog::*;
use std::fs::File;
use chrono::FixedOffset;

pub fn initialize_logging(log_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let config = ConfigBuilder::new()
        .set_thread_level(LevelFilter::Debug)
        .set_thread_mode(ThreadLogMode::Names)
        .set_target_level(LevelFilter::Trace)
        .set_time_format("%D %T".into())
        .set_time_offset(FixedOffset::west(6 * 3600))
        .build();
    if let Some(log_path) = log_path {
        CombinedLogger::init(vec![
            WriteLogger::new(LevelFilter::Info, config.clone(), File::create(log_path)?),
            TermLogger::new(LevelFilter::Debug, config.clone(), TerminalMode::Mixed),
        ])?;
    } else {
        TermLogger::init(LevelFilter::Debug, config, TerminalMode::Mixed)?;
    }
    Ok(())
}