use crate::pretty_string::PrettyString;
use crate::time_log::{LogEntry, TimeLog};
use chrono::{Date, Duration, Local, NaiveDate};
use colored::Colorize;
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::ops::{Add, Range};

type GroupBy<'a, K> = HashMap<K, Vec<&'a LogEntry>>;

const ARROW: &str = "❯";

pub struct ReportCreator<'a> {
    time_log: &'a TimeLog,
}

#[derive(Serialize)]
pub struct DateRangeReport {
    pub range: Range<NaiveDate>,
    #[serde(serialize_with = "serialize_duration", rename = "total")]
    pub total_duration: Duration,
    pub days: Vec<DayReport>,
}

impl DateRangeReport {
    fn new(range: Range<Date<Local>>, days: Vec<DayReport>) -> Self {
        Self {
            range: range.start.naive_local()..range.end.naive_local(),
            total_duration: days
                .iter()
                .map(|r| r.total_duration)
                .fold(Duration::zero(), |a, b| a.add(b)),
            days,
        }
    }
}

impl Display for DateRangeReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.days.iter().try_for_each(|i| i.fmt(f))?;
        Ok(())
    }
}

#[derive(Serialize)]
pub struct DayReport {
    pub date: NaiveDate,
    #[serde(serialize_with = "serialize_duration", rename = "total")]
    pub total_duration: Duration,
    pub projects: Vec<ProjectReport>,
}

impl DayReport {
    fn new(date: NaiveDate, projects: Vec<ProjectReport>) -> Self {
        Self {
            date,
            total_duration: projects
                .iter()
                .map(|p| p.duration)
                .fold(Duration::zero(), |a, b| a.add(b)),
            projects,
        }
    }
}

impl Display for DayReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} {}{:<25}[{}]",
            ARROW.green(),
            self.date.format("%a. %F"),
            ' ',
            self.total_duration.to_pretty_string()
        )?;
        self.projects.iter().try_for_each(|p| p.fmt(f))?;
        Ok(())
    }
}

#[derive(Serialize)]
pub struct ProjectReport {
    pub project: String,
    #[serde(serialize_with = "serialize_duration")]
    pub duration: Duration,
}

impl Display for ProjectReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "    {} {:<35} [{}]",
            ARROW,
            self.project.as_str().bold(),
            self.duration.to_pretty_string(),
        )
    }
}

fn serialize_duration<S>(d: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i64(d.num_minutes())
}

impl ReportCreator<'_> {
    pub fn new(time_log: &TimeLog) -> ReportCreator {
        ReportCreator { time_log }
    }

    pub fn report_days(
        &self,
        date: Date<Local>,
        days: u32,
        include_empty_days: bool,
    ) -> DateRangeReport {
        let start_date: Date<Local> = date - Duration::days(days as i64 - 1);

        let mut child_reports: Vec<DayReport> = Vec::new();
        let mut curr_date: Date<Local> = start_date;
        while curr_date <= date {
            if !self.time_log.for_day(curr_date).is_empty() || include_empty_days {
                child_reports.push(self.report_day(curr_date));
            }
            curr_date = curr_date.succ();
        }

        DateRangeReport::new(start_date..date, child_reports)
    }

    pub fn report_day(&self, date: Date<Local>) -> DayReport {
        let log = self.time_log.for_day(date);

        let groups = Self::group_by_key(log, |i| String::from(&i.project_name));
        let mut projects: Vec<ProjectReport> = groups.iter().map(Self::report_project).collect();
        projects.sort_unstable_by(|a, b| a.project.cmp(&b.project));

        DayReport::new(date.naive_local(), projects)
    }

    fn report_project(tuple: (&String, &Vec<&LogEntry>)) -> ProjectReport {
        let (name, entries) = tuple;
        ProjectReport {
            duration: Self::sum_time(entries),
            project: name.to_string(),
        }
    }

    fn sum_time(vec: &[&LogEntry]) -> Duration {
        vec.iter()
            .fold(Duration::zero(), |d, e| d.add(e.to_duration()))
    }

    fn group_by_key<K: Eq + Hash>(
        vec: &[LogEntry],
        key_extractor: fn(&LogEntry) -> K,
    ) -> GroupBy<'_, K> {
        let mut result: HashMap<K, Vec<&LogEntry>> = HashMap::new();

        for entry in vec.iter() {
            let key = key_extractor(entry);
            let v = result.entry(key).or_insert_with(Vec::<&LogEntry>::new);
            v.push(entry);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, TimeZone};
    use std::collections::BTreeMap;
    use std::iter::FromIterator;

    #[test]
    fn test_empty_log() {
        let lg = TimeLog::new();
        let rc = ReportCreator::new(&lg);
        let today = Local::today();
        let rep = rc.report_day(today);

        assert_eq!(rep.total_duration, Duration::zero());
        assert!(rep.projects.is_empty());
    }

    #[test]
    fn test_sum_over_multiple_logs_for_same_project() {
        let today = test_date().with_day(1).unwrap();
        let tl = TimeLog::new_testing_only(BTreeMap::from_iter(vec![(
            today.naive_local(),
            vec![create_log(1, 30, "Foo"), create_log(1, 10, "Foo")],
        )]));

        let rc = ReportCreator::new(&tl);
        let report = rc.report_day(today);

        assert_eq!(report.total_duration, Duration::minutes(40));
        assert_eq!(report.projects.len(), 1);
    }

    #[test]
    fn test_report_days_number() {
        let today = test_date().with_day(1).unwrap();
        let tl = TimeLog::new_testing_only(BTreeMap::from_iter(vec![(
            today.naive_local(),
            vec![create_log(1, 30, "Foo")],
        )]));

        let rc = ReportCreator::new(&tl);

        let empty_rep = rc.report_days(today, 0, true);
        assert_eq!(empty_rep.days.len(), 0);

        let single_day_rep = rc.report_days(today, 1, true);
        assert_eq!(single_day_rep.days.len(), 1);
    }

    #[test]
    fn test_sum_over_multiple_logs_for_different_project() {
        let today = test_date().with_day(1).unwrap();
        let tl = TimeLog::new_testing_only(BTreeMap::from_iter(vec![(
            today.naive_local(),
            vec![create_log(1, 30, "Foo"), create_log(1, 10, "Bar")],
        )]));

        let rc = ReportCreator::new(&tl);
        let report = rc.report_day(today);

        assert_eq!(report.total_duration, Duration::minutes(40));
        assert_eq!(report.projects.len(), 2);
    }

    #[test]
    fn test_sum_over_multiple_logs_for_different_days() {
        let today = test_date().with_day(1).unwrap();
        let tomorrow = test_date().with_day(2).unwrap();
        let tl = tl_multiple_days(today, tomorrow);

        let rc = ReportCreator::new(&tl);
        let report = rc.report_days(tomorrow, 2, true);

        assert_eq!(report.total_duration, Duration::minutes(50));
        assert_eq!(report.days.len(), 2);
    }

    #[test]
    fn test_display() {
        let today = test_date().with_day(1).unwrap();
        let tomorrow = test_date().with_day(2).unwrap();
        let tl = tl_multiple_days(today, tomorrow);
        let rc = ReportCreator::new(&tl);

        let report = rc.report_days(tomorrow, 2, true);
        let r_string = report.to_string();

        assert!(r_string.contains("Foo"));
        assert!(r_string.contains("Bar"));
        assert!(r_string.contains("[00h 30m]"));
        assert!(r_string.contains("[00h 10m]"));
    }

    fn tl_multiple_days(today: Date<Local>, tomorrow: Date<Local>) -> TimeLog {
        TimeLog::new_testing_only(BTreeMap::from_iter(vec![
            (
                today.naive_local(),
                vec![create_log(1, 30, "Foo"), create_log(1, 10, "Bar")],
            ),
            (tomorrow.naive_local(), vec![create_log(2, 10, "Bar")]),
        ]))
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
