use std::error::Error;
use std::fs::{create_dir_all, read_to_string, File, OpenOptions};
use std::io::Write;

use chrono::Local;

use crate::cli::{Opts, Subcommand};
use crate::report_creator::ReportCreator;
use crate::time_log::TimeLog;
use clap::Clap;
use colored::Colorize;
use std::fmt::Display;
use std::fmt::Formatter;
use std::path::PathBuf;

mod cli;
mod report_creator;
mod time_log;

fn main() {
    include_str!("../Cargo.toml");
    if let Err(e) = run_app() {
        eprintln!("{} {}", "ERROR:".red(), e);
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
                println!("{} {}", "WARN:".yellow(), warn);
            }
            println!("Tracking time for project {}", p.key.as_str().italic());
        }
        Subcommand::Stop(_) => {
            modified = true;
            if let Some(warn) = log.stop_pending()? {
                println!("{} {}", "WARN:".yellow(), warn);
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
    let conf_file = config_file();
    create_dir_all(&conf_file.parent().unwrap())?;

    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(conf_file)?;

    serde_json::to_writer(&f, log)?;
    f.flush()?;
    Ok(f)
}

fn load_or_create_log() -> Result<TimeLog, Box<dyn Error>> {
   let conf_file = config_file();
    if conf_file.exists() {
        let content = read_to_string(conf_file)?;
        Ok(TimeLog::from_json(&content)?)
    } else {
        Ok(TimeLog::new())
    }
}

fn config_file() -> PathBuf{
    dirs::home_dir()
        .unwrap()
        .join(".config")
        .join("trackie.json")
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

impl Display for TrackieError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for TrackieError {}
