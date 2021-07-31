use std::error::Error;
use chrono::{Date, DateTime, Local, Duration};
use serde::{Deserialize, Serialize};
use crate::TrackieError;

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
            Err(Box::new(TrackieError::new("No time is currently tracked.".to_string())))
        }
    }

    pub fn for_day(&self, date: &Date<Local>) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|e| {
                e.start.date().eq(date)
            })
            .collect()
    }

    pub fn for_key(&self, key: &str) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|e| e.key.eq(key))
            .collect()
    }
}
