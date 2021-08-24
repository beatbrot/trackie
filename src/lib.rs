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

pub fn run_app(o: Opts) -> Result<(), TrackieError> {
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

                return Err(TrackieError {
                    msg,
                    print_as_error: false,
                });
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
    pub print_as_error: bool,
}

impl TrackieError {
    fn new(msg: &str) -> TrackieError {
        TrackieError {
            msg: msg.to_string(),
            print_as_error: true,
        }
    }
}

impl Display for TrackieError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl From<Box<dyn Error>> for TrackieError {
    fn from(i: Box<dyn Error>) -> Self {
        TrackieError::new(i.to_string().as_str())
    }
}

impl From<serde_json::Error> for TrackieError {
    fn from(e: serde_json::Error) -> Self {
        TrackieError::new(e.to_string().as_str())
    }
}

impl Error for TrackieError {}

#[cfg(test)]
mod tests {
    use crate::cli::{
        Opts, StatusCommand, Subcommand, TimingCommand, DEFAULT_EMPTY_STATUS_MSG,
        ENV_TRACKIE_CONFIG,
    };
    use crate::run_app;
    use rand::Rng;
    use std::env;
    use std::error::Error;
    use std::path::PathBuf;
    use std::str::FromStr;

    #[test]
    fn status_on_empty_fallback() {
        let _ = TestDirectory::create();
        let e = run_app(Opts {
            sub_cmd: Subcommand::Status(StatusCommand {
                format: None,
                fallback: Some("Foo".to_string()),
            }),
        });
        assert!(e.is_err());
        assert_eq!(e.unwrap_err().msg, "Foo");
    }

    #[test]
    fn status_on_empty_no_fallback() {
        let _ = TestDirectory::create();
        let e = run_app(Opts {
            sub_cmd: Subcommand::Status(StatusCommand {
                format: None,
                fallback: None,
            }),
        });
        assert!(e.is_err());
        assert_eq!(e.unwrap_err().msg, DEFAULT_EMPTY_STATUS_MSG);
    }

    #[test]
    fn start_tracking() -> Result<(), Box<dyn Error>> {
        let t = TestDirectory::create();
        run_app(Opts {
            sub_cmd: Subcommand::Start(TimingCommand {
                project_name: "Foo".to_string(),
            }),
        })?;

        let json_path = t.path.join("trackie.json");
        assert!(json_path.is_file());
        assert!(std::fs::read_to_string(json_path).unwrap().contains("Foo"));
        Ok(())
    }

    #[test]
    fn status_after_start_tracking() -> Result<(), Box<dyn Error>> {
        let _ = TestDirectory::create();
        run_app(Opts {
            sub_cmd: Subcommand::Start(TimingCommand {
                project_name: "Foo".to_string(),
            }),
        })?;

        run_app(Opts {
            sub_cmd: Subcommand::Status(StatusCommand {
                format: None,
                fallback: None,
            }),
        })?;
        Ok(())
    }

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn create() -> Self {
            let mut r = rand::thread_rng();
            let n: u64 = r.gen();

            let path = PathBuf::from_str(".")
                .unwrap()
                .join("target")
                .join("test-data")
                .join(n.to_string());

            std::fs::create_dir_all(path.clone()).unwrap();
            env::set_var(ENV_TRACKIE_CONFIG, &path.join("trackie.json"));
            Self { path }
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            std::fs::remove_dir_all(self.path.clone()).unwrap();
            assert!(!self.path.exists(), "Could not delete directory")
        }
    }
}
