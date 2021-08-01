use crate::log_analyzer::Category::{DateRange, Project};
use crate::time_log::{LogEntry, TimeLog};
use chrono::{Date, Duration, Local};
use colored::Colorize;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::ops::{Add, Range, Sub};

type GroupBy<'a, K> = HashMap<K, Vec<&'a LogEntry>>;

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
            let arrow = "❯";
            match level {
                1 => write!(
                    f,
                    "{} {}{:<25}[{}]\n",
                    arrow.green(),
                    target.category,
                    ' ',
                    print_duration(&target.overall_duration)
                )?,
                2 => write!(
                    f,
                    "    {} {:<35} [{}]\n",
                    arrow,
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
    fn create_duration_sum(reports: &Vec<Report>) -> Duration {
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
            (Project(_), Category::Date(_)) => Ordering::Greater,
            (Project(_), DateRange(_)) => Ordering::Greater,
            (Category::Date(_), DateRange(_)) => Ordering::Greater,
            (Category::Date(d1), Category::Date(d2)) => d1.cmp(d2),
            (Category::Date(_), Project(_)) => Ordering::Less,
            (DateRange(r1), DateRange(r2)) => r1.start.cmp(&r2.start),
            (DateRange(_), Category::Date(_)) => Ordering::Less,
            (DateRange(_), Project(_)) => Ordering::Less,
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
        let start_date = date.sub(Duration::days((days as i64) - 1)).to_owned();
        let range = start_date..date;

        let mut reports: Vec<Report> = Vec::new();
        let mut curr_date: Date<Local> = start_date;
        loop {
            if !self.time_log.for_day(&curr_date).is_empty() || include_empty_days {
                reports.push(self.report_day(&curr_date));
            }
            if curr_date.eq(&date) {
                break;
            }
            curr_date = curr_date.succ();
        }

        Report {
            category: Category::DateRange(range),
            overall_duration: Report::create_duration_sum(&reports),
            child_reports: reports,
        }
    }

    pub fn report_day(&self, date: &Date<Local>) -> Report {
        let log = self.time_log.for_day(date);

        let groups = Self::group_by_key(&log, |i| String::from(&i.key));
        let mut child_reports: Vec<Report> = groups.iter().map(Self::report_ticket).collect();
        child_reports.sort_unstable_by(|a, b| a.category.cmp(&b.category));

        Report {
            category: Category::Date(date.to_owned()),
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

    fn group_by_key<'a, K: Eq + Hash>(
        vec: &[&'a LogEntry],
        key_extractor: fn(&LogEntry) -> K,
    ) -> GroupBy<'a, K> {
        let mut result: HashMap<K, Vec<&LogEntry>> = HashMap::new();

        for entry in vec.iter() {
            let key = key_extractor(entry);
            let v = result.entry(key).or_insert_with(Vec::<&LogEntry>::new);
            v.push(&entry)
        }

        result
    }
}
