use std::error::Error;

use chrono::Local;

use crate::cli::{
    Opts, Subcommand, TimingCommand, DEFAULT_EMPTY_STATUS_MSG, DEFAULT_STATUS_FORMAT,
};
use crate::persistence::{load_or_create_log, save_log};
use crate::pretty_string::PrettyString;
use crate::report_creator::ReportCreator;
use crate::time_log::TimeLog;
use colored::Colorize;
use std::fmt::Display;
use std::fmt::Formatter;

pub mod cli;
mod persistence;
mod pretty_string;
mod report_creator;
mod time_log;

pub fn run_app(o: Opts) -> Result<(), Box<dyn Error>> {
    let mut modified = false;
    let mut log = load_or_create_log()?;
    let report_creator = ReportCreator::new(&log);

    match o.sub_cmd {
        Subcommand::Start(p) => {
            modified = true;
            start_tracking(&mut log, p)?;
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
            let report = report_creator.report_days(Local::today(), o.days, o.include_empty_days);
            match o.json {
                true => println!("{}", serde_json::to_string_pretty(&report)?),
                false => println!("{}", report),
            };
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
        Subcommand::Resume(_) => match (&log.pending, log.get_latest_entry()) {
            (None, Some(s)) => {
                modified = true;
                let name = s.project_name.clone();
                start_tracking(&mut log, TimingCommand { project_name: name })?;
            }
            (Some(p), _) => {
                return Err(TrackieError::new(
                    format!("Already tracking time for project {}", p.project_name).as_str(),
                )
                .into())
            }
            (_, None) => {
                return Err(TrackieError::new(
                    "Unable to find latest time log. Maybe no time was ever tracked?",
                )
                .into());
            }
        },
    }

    if modified {
        save_log(&log)?;
    }

    Ok(())
}

fn start_tracking(log: &mut TimeLog, p: TimingCommand) -> Result<(), Box<dyn Error>> {
    if let Some(warn) = log.start_log(&p.project_name)? {
        println!("{} {}", "WARN:".yellow(), warn);
    }
    println!(
        "Tracking time for project {}",
        p.project_name.as_str().italic()
    );
    Ok(())
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
