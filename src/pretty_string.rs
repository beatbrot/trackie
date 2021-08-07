use chrono::Duration;

const MINS_IN_HOUR: i64 = 60;

pub trait PrettyString {
    fn to_pretty_string(&self) -> String;
}

impl PrettyString for Duration {

    fn to_pretty_string(&self) -> String {
        let remaining_min = self.num_minutes() - (self.num_hours() * MINS_IN_HOUR);
        format!("{:02}h {:02}m", self.num_hours(), remaining_min)
    }
}
