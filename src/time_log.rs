use std::error::Error;

use chrono::{Date, DateTime, Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::TrackieError;
use std::collections::BTreeMap;

type OptError = Result<Option<String>, Box<dyn Error>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PendingLog {
    pub project_name: String,
    pub start: DateTime<Local>,
}

impl PendingLog {
    pub fn get_pending_duration(&self) -> Duration {
        let now = Local::now();
        now.signed_duration_since(self.start)
    }
}

#[derive(Serialize, Deserialize)]
pub struct LogEntry {
    pub project_name: String,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
}

impl LogEntry {
    fn from_time_log(log: &PendingLog, end: DateTime<Local>) -> LogEntry {
        LogEntry {
            project_name: (&log.project_name).to_string(),
            start: log.start,
            end,
        }
    }

    pub fn to_duration(&self) -> Duration {
        self.end.signed_duration_since(self.start)
    }
}

#[derive(Serialize, Deserialize)]
pub struct TimeLog {
    pub pending: Option<PendingLog>,
    entries: BTreeMap<NaiveDate, Vec<LogEntry>>,
}

impl Default for TimeLog{
    fn default() -> Self {
        TimeLog::new()
    }
}

impl TimeLog {
    pub fn new() -> TimeLog {
        TimeLog {
            pending: None,
            entries: BTreeMap::new(),
        }
    }

    pub fn from_json(content: &str) -> serde_json::Result<TimeLog> {
        serde_json::from_str(content)
    }

    #[cfg(test)]
    pub fn new_testing_only(entries: BTreeMap<NaiveDate, Vec<LogEntry>>) -> TimeLog {
        Self {
            pending: None,
            entries,
        }
    }

    pub fn start_log(&mut self, project_name: &str) -> OptError {
        let mut warn: Option<String> = None;
        if let Some(p) = &self.pending {
            warn = Some(format!("Stopping time-tracking for {}", p.project_name));
            self.stop_pending()?;
        }
        let lg = PendingLog {
            project_name: project_name.to_string(),
            start: Local::now(),
        };
        self.pending = Some(lg);
        Ok(warn)
    }

    pub fn stop_pending(&mut self) -> Result<PendingLog, Box<dyn Error>> {
        if let Some(p) = &self.pending {
            let now = Local::now();
            let entry = LogEntry::from_time_log(p, now);
            self.entries
                .entry(now.date().naive_local())
                .or_default()
                .push(entry);
            let result = p.clone();
            self.pending = None;
            Ok(result)
        } else {
            Err(TrackieError::new("No time is currently tracked.").into())
        }
    }

    pub fn for_day(&self, date: Date<Local>) -> &[LogEntry] {
        self.entries
            .get(&date.naive_local())
            .map_or(&[], Vec::as_slice)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, TimeZone};
    use spectral::prelude::*;

    use super::*;
    use std::iter::FromIterator;

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
        let lg = create_tl_with_two_dates();

        let result = lg.for_day(test_date());

        assert_eq!(result.len(), 1);
        assert_eq!(&result.get(0).unwrap().project_name, "Target");
    }

    #[test]
    fn filter_items_for_day_empty_log() {
        let lg = TimeLog::new();

        let result = lg.for_day(test_date());

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_serialization_empty() {
        let l = TimeLog::new();
        let result = serde_json::to_string(&l).unwrap();
        assert_that!(result).contains("{}");

        let deserialized: TimeLog = TimeLog::from_json(result.as_str()).unwrap();

        assert_that!(deserialized.pending).is_none();
        assert_that!(deserialized.entries.len()).is_equal_to(0);
    }

    fn create_tl_with_two_dates() -> TimeLog {
        TimeLog {
            pending: None,
            entries: BTreeMap::from_iter(vec![
                (
                    test_date().with_day(1).unwrap().naive_local(),
                    vec![create_log(1, 30, "Target")],
                ),
                (
                    test_date().with_day(2).unwrap().naive_local(),
                    vec![create_log(2, 40, "Target")],
                ),
            ]),
        }
    }

    fn create_log(day: u32, dur: u32, name: &str) -> LogEntry {
        LogEntry {
            start: test_date().with_day(day).unwrap().and_hms(4, 0, 20),
            end: test_date().with_day(day).unwrap().and_hms(4, dur, 20),
            project_name: name.to_string(),
        }
    }

    fn test_date() -> Date<Local> {
        Local.ymd(2000, 1, 1)
    }
}
