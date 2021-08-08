use std::error::Error;
use std::fs::{create_dir_all, read_to_string, File, OpenOptions};
use std::io::Write;

use chrono::Local;

use crate::cli::{Opts, Subcommand, DEFAULT_EMPTY_STATUS_MSG, DEFAULT_STATUS_FORMAT};
use crate::pretty_string::PrettyString;
use crate::report_creator::ReportCreator;
use crate::time_log::TimeLog;
use clap::Clap;
use colored::Colorize;
use std::fmt::Display;
use std::fmt::Formatter;
use std::path::PathBuf;

mod cli;
mod pretty_string;
mod report_creator;
mod time_log;

fn main() {
    include_str!("../Cargo.toml");
    if let Err(e) = run_app() {
        eprintln!("{} {}", "ERROR:".red(), e);
        std::process::exit(1);
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
            if let Some(warn) = log.start_log(&p.project_name)? {
                println!("{} {}", "WARN:".yellow(), warn);
            }
            println!(
                "Tracking time for project {}",
                p.project_name.as_str().italic()
            );
        }
        Subcommand::Stop(_) => {
            modified = true;
            let pending = log.stop_pending()?;
            let dur = pending.get_pending_duration();
            println!(
                "Tracked {} on project {}",
                dur.to_pretty_string().bold(),
                pending.project_name.italic()
            )
        }
        Subcommand::Report(o) => {
            println!(
                "{}",
                report_creator.report_days(Local::today(), o.days, o.include_empty_days)
            );
        }
        Subcommand::Status(s) => match &log.pending {
            None => {
                let msg = s
                    .fallback
                    .unwrap_or_else(|| DEFAULT_EMPTY_STATUS_MSG.to_string());
                println!("{}", msg);
                std::process::exit(1);
            }
            Some(p) => {
                let format = s
                    .format
                    .unwrap_or_else(|| DEFAULT_STATUS_FORMAT.to_string());

                let output = format
                    .replace("%p", p.project_name.as_str())
                    .replace("%d", p.start.format("%F").to_string().as_str())
                    .replace("%t", p.start.format("%R").to_string().as_str())
                    .replace("%D", p.get_pending_duration().to_pretty_string().as_str());

                println!("{}", output)
            }
        },
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
        Ok(TimeLog::default())
    }
}

fn config_file() -> PathBuf {
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
    fn new(msg: &str) -> TrackieError {
        TrackieError {
            msg: msg.to_string(),
        }
    }
}

impl Display for TrackieError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for TrackieError {}
