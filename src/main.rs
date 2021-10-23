use clap::Parser;
use colored::Colorize;
use trackie::cli::Opts;
use trackie::persistence::FsFileHandler;
use trackie::run_app;

pub fn main() {
    include_str!("../Cargo.toml");
    let mut fs = FsFileHandler::new();
    if let Err(e) = run_app(Opts::parse(), &mut fs) {
        if e.print_as_error {
            eprintln!("{} {}", "ERROR:".red(), e);
        } else {
            println!("{}", e);
        }
        std::process::exit(1);
    }
}
