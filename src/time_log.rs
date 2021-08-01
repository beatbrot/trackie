use crate::TrackieError;
use chrono::{Date, DateTime, Duration, Local};
use serde::{Deserialize, Serialize};
use std::error::Error;

type OptError = Result<Option<String>, Box<dyn Error>>;

#[derive(Debug, Serialize, Deserialize)]
struct PendingLog {
    key: String,
    start: DateTime<Local>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub key: String,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
}

impl LogEntry {
    fn from_time_log(log: &PendingLog, end: DateTime<Local>) -> LogEntry {
        LogEntry {
            key: (&log.key).to_string(),
            start: log.start,
            end,
        }
    }

    pub fn to_duration(&self) -> Duration {
        self.end.signed_duration_since(self.start)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeLog {
    pending: Option<PendingLog>,
    entries: Vec<LogEntry>,
}

impl TimeLog {
    pub fn new() -> TimeLog {
        TimeLog {
            pending: None,
            entries: Vec::new(),
        }
    }

    pub fn from_json(content: &str) -> serde_json::Result<TimeLog> {
        serde_json::from_str(content)
    }

    pub fn start_log(&mut self, key: &str) -> OptError {
        let mut warn: Option<String> = None;
        if let Some(p) = &self.pending {
            warn = Some(format!("Stopping time-tracking for {}", p.key));
            self.stop_pending()?;
        }
        let lg = PendingLog {
            key: key.to_string(),
            start: Local::now(),
        };
        self.pending = Some(lg);
        Ok(warn)
    }

    pub fn stop_pending(&mut self) -> OptError {
        if let Some(p) = &self.pending {
            let now = Local::now();
            let entry = LogEntry::from_time_log(p, now);
            self.entries.push(entry);
            self.pending = None;
            Ok(None)
        } else {
            Err(Box::new(TrackieError::new(
                "No time is currently tracked.".to_string(),
            )))
        }
    }

    pub fn for_day(&self, date: &Date<Local>) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|e| e.start.date().eq(date))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use spectral::prelude::*;

    #[test]
    fn start_worklog_fresh() {
        let mut l = TimeLog::new();
        let result = l.start_log("ABC").unwrap();

        assert_that(&result).is_none();
    }

    #[test]
    fn start_worklog_overwrite() {
        let mut l = TimeLog::new();
        l.start_log("ABC").unwrap();

        let result = l.start_log("DEF").unwrap();

        assert_that(&result).is_some();
    }

    #[test]
    fn stop_nonexiting_workload() {
        let mut l = TimeLog::new();
        let result = l.stop_pending();

        assert_that(&result).is_err();
    }

    #[test]
    fn filter_items_for_day() {
        let lg = TimeLog {
            pending: None,
            entries: vec![create_log(01, 30, "Target"), create_log(02, 40, "Fail")],
        };

        let result = lg.for_day(&Local.ymd(2000, 01, 01));

        assert_that(&result).has_length(1);
        assert_eq!(&result.get(0).unwrap().key, "Target")
    }

    fn create_log(day: u32, dur: u32, name: &str) -> LogEntry {
        LogEntry {
            start: Local.ymd(2000, 01, day).and_hms(4, 0, 20),
            end: Local.ymd(2000, 01, day).and_hms(4, dur, 20),
            key: name.to_string(),
        }
    }
}
