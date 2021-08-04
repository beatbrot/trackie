use clap::{crate_authors, crate_version, Clap};

#[derive(Clap)]
#[clap(author=crate_authors!(), version=crate_version!())]
/// A simple, private, time tracking utility.
pub struct Opts {
    #[clap(subcommand)]
    pub sub_cmd: Subcommand,
}

#[derive(Clap)]
pub enum Subcommand {
    /// Starts the time tracking for a project
    Start(TimingCommand),
    /// Stops the time tracking for a project
    Stop(EmptyCommand),
    /// Creates a report for the logged times
    Report(ReportCommand),
}

#[derive(Clap)]
pub struct EmptyCommand {}

#[derive(Clap)]
pub struct TimingCommand {
    /// The name of the project
    pub project_name: String,
}

#[derive(Clap)]
pub struct ReportCommand {
    /// The amount of days to include in the report.
    #[clap(short, long, default_value = "5")]
    pub days: u32,

    /// Includes days without logged work in the report.
    #[clap(short, long)]
    pub include_empty_days: bool,
}
