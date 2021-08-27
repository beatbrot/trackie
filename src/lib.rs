use std::error::Error;

use chrono::Local;

use crate::cli::{
    Opts, Subcommand, TimingCommand, DEFAULT_EMPTY_STATUS_MSG, DEFAULT_STATUS_FORMAT,
};
use crate::persistence::{load_or_create_log, save_log, FileHandler};
use crate::pretty_string::PrettyString;
use crate::report_creator::ReportCreator;
use crate::time_log::TimeLog;
use colored::Colorize;
use std::fmt::Display;
use std::fmt::Formatter;

pub mod cli;
pub mod persistence;
mod pretty_string;
mod report_creator;
mod time_log;

pub fn run_app(o: Opts, fh: &mut dyn FileHandler) -> Result<(), TrackieError> {
    let mut modified = false;
    let mut log = load_or_create_log(fh)?;
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
            );
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

                println!("{}", output);
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
                ))
            }
            (_, None) => {
                return Err(TrackieError::new(
                    "Unable to find latest time log. Maybe no time was ever tracked?",
                ));
            }
        },
    }

    if modified {
        save_log(fh, &log)?;
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
        EmptyCommand, Opts, StatusCommand, Subcommand, TimingCommand, DEFAULT_EMPTY_STATUS_MSG,
    };
    use crate::persistence::FileHandler;
    use crate::run_app;
    use std::error::Error;

    #[test]
    fn status_on_empty_fallback() {
        let mut handler = TestFileHandler::default();
        let e = run_app(
            Opts {
                sub_cmd: Subcommand::Status(StatusCommand {
                    format: None,
                    fallback: Some("Foo".to_string()),
                }),
            },
            &mut handler,
        );
        assert!(e.is_err());
        assert_eq!(e.unwrap_err().msg, "Foo");
    }

    #[test]
    fn status_on_empty_no_fallback() {
        let mut handler = TestFileHandler::default();
        let e = run_app(
            Opts {
                sub_cmd: Subcommand::Status(StatusCommand {
                    format: None,
                    fallback: None,
                }),
            },
            &mut handler,
        );
        assert!(e.is_err());
        assert_eq!(e.unwrap_err().msg, DEFAULT_EMPTY_STATUS_MSG);
    }

    #[test]
    fn start_tracking() -> Result<(), Box<dyn Error>> {
        let mut handler = TestFileHandler::default();
        run_app(
            Opts {
                sub_cmd: Subcommand::Start(TimingCommand {
                    project_name: "Foo".to_string(),
                }),
            },
            &mut handler,
        )?;

        let content = handler.content.unwrap();
        assert!(content.contains("Foo"));
        Ok(())
    }

    #[test]
    fn resume_after_stop() -> Result<(), Box<dyn Error>> {
        let mut handler = TestFileHandler::default();

        let x = run_app(
            Opts {
                sub_cmd: Subcommand::Resume(EmptyCommand {}),
            },
            &mut handler,
        );
        assert!(x.is_err());

        run_app(
            Opts {
                sub_cmd: Subcommand::Start(TimingCommand {
                    project_name: "Foo".to_string(),
                }),
            },
            &mut handler,
        )?;
        run_app(
            Opts {
                sub_cmd: Subcommand::Stop(EmptyCommand {}),
            },
            &mut handler,
        )?;

        let status = run_app(
            Opts {
                sub_cmd: Subcommand::Status(StatusCommand {
                    fallback: None,
                    format: None,
                }),
            },
            &mut handler,
        );
        assert!(status.is_err());

        run_app(
            Opts {
                sub_cmd: Subcommand::Resume(EmptyCommand {}),
            },
            &mut handler,
        )?;

        let status = run_app(
            Opts {
                sub_cmd: Subcommand::Status(StatusCommand {
                    fallback: None,
                    format: None,
                }),
            },
            &mut handler,
        );
        assert!(status.is_ok());

        Ok(())
    }

    #[test]
    fn status_after_start_tracking() -> Result<(), Box<dyn Error>> {
        let mut handler = TestFileHandler::default();
        run_app(
            Opts {
                sub_cmd: Subcommand::Start(TimingCommand {
                    project_name: "Foo".to_string(),
                }),
            },
            &mut handler,
        )?;

        run_app(
            Opts {
                sub_cmd: Subcommand::Status(StatusCommand {
                    format: None,
                    fallback: None,
                }),
            },
            &mut handler,
        )?;
        Ok(())
    }

    #[test]
    fn stop_tracking() -> Result<(), Box<dyn Error>> {
        let mut handler = TestFileHandler::default();
        run_app(
            Opts {
                sub_cmd: Subcommand::Start(TimingCommand {
                    project_name: "Foo".to_string(),
                }),
            },
            &mut handler,
        )?;

        run_app(
            Opts {
                sub_cmd: Subcommand::Stop(EmptyCommand {}),
            },
            &mut handler,
        )?;

        let status = run_app(
            Opts {
                sub_cmd: Subcommand::Status(StatusCommand {
                    format: None,
                    fallback: None,
                }),
            },
            &mut handler,
        );

        assert!(status.is_err());

        let second_stop = run_app(
            Opts {
                sub_cmd: Subcommand::Stop(EmptyCommand {}),
            },
            &mut handler,
        );

        assert!(second_stop.is_err());
        Ok(())
    }

    struct TestFileHandler {
        content: Option<String>,
    }

    impl Default for TestFileHandler {
        fn default() -> Self {
            Self { content: None }
        }
    }

    impl FileHandler for TestFileHandler {
        fn read_file(&self) -> Result<Option<String>, Box<dyn Error>> {
            Ok(self.content.clone())
        }

        fn write_file(&mut self, content: &str) -> Result<(), Box<dyn Error>> {
            self.content = Some(content.to_string());
            Ok(())
        }
    }
}
