use trackie::run_app;
use colored::Colorize;
use clap::Clap;
use trackie::cli::Opts;


pub fn main() {
    include_str!("../Cargo.toml");
    if let Err(e) = run_app(Opts::parse()) {
        eprintln!("{} {}", "ERROR:".red(), e);
        std::process::exit(1);
    }
}
