use clap::{crate_authors, crate_version, Parser};
use clap_complete::Shell;

pub const DEFAULT_STATUS_FORMAT: &str = "Tracking %p since %d (%t) [%D]";
pub const DEFAULT_EMPTY_STATUS_MSG: &str = "Currently tracking no project.";
pub const ENV_TRACKIE_CONFIG: &str = "TRACKIE_CONFIG";

#[derive(Parser)]
#[clap(author=crate_authors!(), version=crate_version!())]
/// A simple, private, time tracking utility.
pub struct Opts {
    #[clap(subcommand)]
    pub sub_cmd: Subcommand,
}

#[derive(Parser)]
pub enum Subcommand {
    /// Starts the time tracking for a project
    Start(TimingCommand),
    /// Stops the time tracking for a project
    Stop(EmptyCommand),
    /// Creates a report for the logged times
    Report(ReportCommand),
    /// Shows information about the currently tracked work log, if present
    ///
    /// The commands supports the following variables in its format strings (provided via -f):
    ///     - %p: The name of the project
    ///     - %d: The date on which the tracking started
    ///     - %t: The time at which the tracking started
    ///     - %D: The duration of the current tracking
    ///
    #[clap(verbatim_doc_comment)]
    Status(StatusCommand),
    /// Resumes time tracking for the last tracked project.
    #[clap(visible_alias = "rs")]
    Resume(EmptyCommand),
    /// Generate tab-completion scripts for your shell
    Completion(CompletionCommand),
}

#[derive(Parser)]
pub struct CompletionCommand {
    #[clap(value_parser)]
    pub shell: Shell,
}

#[derive(Parser)]
pub struct StatusCommand {
    /// A format string describing the output of the command.
    #[clap(short, long)]
    pub format: Option<String>,

    /// The message that gets printed to the console if no time is currently tracked.
    #[clap(long)]
    pub fallback: Option<String>,
}

#[derive(Parser)]
pub struct EmptyCommand {}

#[derive(Parser)]
pub struct TimingCommand {
    /// The name of the project
    pub project_name: String,
}

#[derive(Parser)]
pub struct ReportCommand {
    /// The amount of days to include in the report.
    #[clap(short, long, default_value = "5")]
    pub days: u32,

    /// Includes days without logged work in the report.
    #[clap(short, long)]
    pub include_empty_days: bool,

    /// Dump report as JSON
    #[clap(long)]
    pub json: bool,
}
