use std::error::Error;
use std::fs::{create_dir_all, read_to_string, File, OpenOptions};
use std::io::Write;

use chrono::Local;

use clap::Clap;
use crate::cli::{Opts, Subcommand};
use crate::log_analyzer::ReportCreator;
use crate::time_log::TimeLog;
use std::fmt::Formatter;

mod cli;
mod log_analyzer;
mod time_log;

fn main() {
    include_str!("../Cargo.toml");
    if let Err(e) = run_app() {
        eprintln!("{}", e);
    }
}

fn run_app() -> Result<(), Box<dyn Error>> {
    let o: Opts = Opts::parse();
    let mut modified = false;
    let mut log = load_or_create_log()?;
    let report_creator = ReportCreator::new(&log);

    match o.sub_cmd {
        Subcommand::Start(p) => {
            modified = true;
            if let Some(warn) = log.start_log(&p.key)? {
                println!("WARN: {}", warn);
            }
        }
        Subcommand::Stop(_) => {
            modified = true;
            if let Some(warn) = log.stop_pending()? {
                println!("WARN: {}", warn);
            }
        }
        Subcommand::Report(o) => {
            println!(
                "{}",
                report_creator.report_days(Local::today(), o.days, o.include_empty_days)
            );
        }
    }

    if modified {
        save_log(&log)?;
    }

    Ok(())
}

fn save_log(log: &TimeLog) -> Result<File, Box<dyn Error>> {
    let config_dir = dirs::home_dir().unwrap().join(".config");
    let trackie_json = config_dir.join("trackie.json");
    create_dir_all(&config_dir)?;

    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(trackie_json)?;

    serde_json::to_writer(&f, log)?;
    &f.flush()?;
    Ok(f)
}

fn load_or_create_log() -> Result<TimeLog, Box<dyn Error>> {
    let home_dir = dirs::home_dir()
        .unwrap()
        .join(".config")
        .join("trackie.json");
    if home_dir.exists() {
        let content = read_to_string(home_dir)?;
        Ok(TimeLog::from_json(&content)?)
    } else {
        Ok(TimeLog::new())
    }
}

#[derive(Debug)]
pub struct TrackieError {
    msg: String,
}

impl TrackieError {
    fn new(msg: String) -> TrackieError {
        TrackieError { msg }
    }
}

impl std::fmt::Display for TrackieError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ERROR: {}", self.msg)
    }
}

impl Error for TrackieError {}
