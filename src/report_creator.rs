use crate::report_creator::Category::{DateRange, Project};
use crate::time_log::{LogEntry, TimeLog};
use chrono::{Date, Duration, Local};
use colored::Colorize;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::ops::{Add, Range, Sub};

type GroupBy<'a, K> = HashMap<K, Vec<&'a LogEntry>>;

const ARROW: &str = "❯";

pub struct ReportCreator<'a> {
    time_log: &'a TimeLog,
}

pub struct Report {
    pub category: Category,
    pub overall_duration: Duration,
    pub child_reports: Vec<Report>,
}

impl Display for Report {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn print_duration(dur: &Duration) -> String {
            let remaining_min = dur.num_minutes() - (dur.num_hours() * 60);
            format!("{:02}h {:02}m", dur.num_hours(), remaining_min)
        }
        fn format(f: &mut Formatter, target: &Report, level: u32) -> std::fmt::Result {
            match level {
                1 => writeln!(
                    f,
                    "{} {}{:<25}[{}]",
                    ARROW.green(),
                    target.category,
                    ' ',
                    print_duration(&target.overall_duration)
                )?,
                2 => writeln!(
                    f,
                    "    {} {:<35} [{}]",
                    ARROW,
                    target.category.to_string().as_str().bold(),
                    print_duration(&target.overall_duration)
                )?,
                _ => {}
            };

            for c in &target.child_reports {
                format(f, c, level + 1)?;
            }
            Ok(())
        }

        format(f, self, 0)?;
        Ok(())
    }
}

impl Report {
    fn create_duration_sum(reports: &[Report]) -> Duration {
        reports
            .iter()
            .fold(Duration::zero(), |d, r| d.add(r.overall_duration))
    }
}

pub enum Category {
    Project(String),
    Date(Date<Local>),
    DateRange(Range<Date<Local>>),
}

impl Ord for Category {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Project(p1), Project(p2)) => p1.cmp(p2),
            (Category::Date(d1), Category::Date(d2)) => d1.cmp(d2),
            (DateRange(r1), DateRange(r2)) => r1.start.cmp(&r2.start),
            (Project(_), Category::Date(_))
            | (Project(_), DateRange(_))
            | (Category::Date(_), DateRange(_)) => Ordering::Greater,
            (DateRange(_), Category::Date(_))
            | (DateRange(_), Project(_))
            | (Category::Date(_), Project(_)) => Ordering::Less,
        }
    }
}

impl Eq for Category {}

impl PartialEq for Category {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for Category {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Category {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Project(t) => write!(f, "{}", t),
            Category::Date(d) => write!(f, "{}", d.format("%a. %F")),
            Category::DateRange(r) => {
                write!(f, "{} to {}", r.start.format("%F"), r.end.format("%F"))
            }
        }
    }
}

impl ReportCreator<'_> {
    pub fn new(time_log: &TimeLog) -> ReportCreator {
        ReportCreator { time_log }
    }

    pub fn report_days(&self, date: Date<Local>, days: u32, include_empty_days: bool) -> Report {
        let start_date = date.sub(Duration::days(i64::from(days) - 1));

        let mut child_reports: Vec<Report> = Vec::new();
        let mut curr_date: Date<Local> = start_date;
        loop {
            if !self.time_log.for_day(curr_date).is_empty() || include_empty_days {
                child_reports.push(self.report_day(curr_date));
            }
            if curr_date == date {
                break;
            }
            curr_date = curr_date.succ();
        }

        Report {
            category: Category::DateRange(start_date..date),
            overall_duration: Report::create_duration_sum(&child_reports),
            child_reports,
        }
    }

    pub fn report_day(&self, date: Date<Local>) -> Report {
        let log = self.time_log.for_day(date);

        let groups = Self::group_by_key(log, |i| String::from(&i.key));
        let mut child_reports: Vec<Report> = groups.iter().map(Self::report_ticket).collect();
        child_reports.sort_unstable_by(|a, b| a.category.cmp(&b.category));

        Report {
            category: Category::Date(date),
            overall_duration: Report::create_duration_sum(&child_reports),
            child_reports,
        }
    }

    fn report_ticket(tuple: (&String, &Vec<&LogEntry>)) -> Report {
        let (key, entries) = tuple;
        Report {
            category: Category::Project(key.to_string()),
            overall_duration: Self::sum_time(entries),
            child_reports: Vec::new(),
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
    use spectral::assert_that;
    use spectral::prelude::{VecAssertions, StrAssertions};
    use std::collections::BTreeMap;
    use std::iter::FromIterator;

    #[test]
    fn test_empty_log() {
        let lg = TimeLog::new();
        let rc = ReportCreator::new(&lg);
        let today = Local::today();
        let rep = rc.report_day(today);

        assert!(matches!(rep.category, Category::Date(_)));
        assert_eq!(rep.overall_duration, Duration::zero());
        assert_that!(rep.child_reports).is_empty();
    }

    #[test]
    fn test_sum_over_multiple_logs_for_same_ticket() {
        let today = test_date().with_day(1).unwrap();
        let tl = TimeLog::new_testing_only(BTreeMap::from_iter(vec![(
            today.naive_local(),
            vec![create_log(1, 30, "Foo"), create_log(1, 10, "Foo")],
        )]));

        let rc = ReportCreator::new(&tl);
        let report = rc.report_day(today);

        assert!(matches!(report.category, Category::Date(_)));
        assert_that!(report.overall_duration).is_equal_to(Duration::minutes(40));
        assert_that!(report.child_reports).has_length(1);
    }

    #[test]
    fn test_sum_over_multiple_logs_for_different_tickets() {
        let today = test_date().with_day(1).unwrap();
        let tl = TimeLog::new_testing_only(BTreeMap::from_iter(vec![(
            today.naive_local(),
            vec![create_log(1, 30, "Foo"), create_log(1, 10, "Bar")],
        )]));

        let rc = ReportCreator::new(&tl);
        let report = rc.report_day(today);

        assert!(matches!(report.category, Category::Date(_)));
        assert_that!(report.overall_duration).is_equal_to(Duration::minutes(40));
        assert_that!(report.child_reports).has_length(2);
    }

    #[test]
    fn test_sum_over_multiple_logs_for_different_days() {
        let today = test_date().with_day(1).unwrap();
        let tomorrow = test_date().with_day(2).unwrap();
        let tl = tl_multiple_days(today, tomorrow);

        let rc = ReportCreator::new(&tl);
        let report = rc.report_days(tomorrow, 2,true);

        assert!(matches!(report.category, Category::DateRange(_)));
        assert_that!(report.overall_duration).is_equal_to(Duration::minutes(50));
        assert_that!(report.child_reports).has_length(2);
    }

    #[test]
    fn test_display() {
        let today = test_date().with_day(1).unwrap();
        let tomorrow = test_date().with_day(2).unwrap();
        let tl = tl_multiple_days(today, tomorrow);
        let rc = ReportCreator::new(&tl);

        let report = rc.report_days(tomorrow,2,true);
        let r_string = report.to_string();

        assert_that!(r_string).contains("Foo");
        assert_that!(r_string).contains("Bar");
        assert_that!(r_string).contains("[00h 30m]");
        assert_that!(r_string).contains("[00h 10m]");
    }

    fn tl_multiple_days(today: Date<Local>, tomorrow: Date<Local>) -> TimeLog {
        TimeLog::new_testing_only(BTreeMap::from_iter(vec![
            (
                today.naive_local(),
                vec![create_log(1, 30, "Foo"), create_log(1, 10, "Bar")],
            ),
            (
                tomorrow.naive_local(),
                vec![create_log(2, 10, "Bar")],
            ),
        ]))
    }

    fn create_log(day: u32, dur: u32, name: &str) -> LogEntry {
        LogEntry {
            start: test_date().with_day(day).unwrap().and_hms(4, 0, 20),
            end: test_date().with_day(day).unwrap().and_hms(4, dur, 20),
            key: name.to_string(),
        }
    }

    fn test_date() -> Date<Local> {
        Local.ymd(2000, 1, 1)
    }
}
