use std::error::Error;
use crate::time_log::TimeLog;
use std::fs::{read_to_string, rename, create_dir_all, OpenOptions, File};
use std::path::PathBuf;
use std::env;
use crate::cli::ENV_TRACKIE_CONFIG;
use std::io::Write;

pub fn load_or_create_log() -> Result<TimeLog, Box<dyn Error>> {
    move_legacy_config_file()?;
    let trackie_file = trackie_file();
    if trackie_file.exists() {
        let content = read_to_string(trackie_file)?;
        Ok(TimeLog::from_json(&content)?)
    } else {
        Ok(TimeLog::default())
    }
}

pub fn save_log(log: &TimeLog) -> Result<File, Box<dyn Error>> {
    let trackie_file = trackie_file();
    println!("{}", trackie_file.to_str().unwrap());
    create_dir_all(&trackie_file.parent().unwrap())?;

    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(trackie_file)?;

    serde_json::to_writer(&f, log)?;
    f.flush()?;
    Ok(f)
}

fn move_legacy_config_file() -> Result<(), Box<dyn Error>> {
    if env::var(ENV_TRACKIE_CONFIG).is_ok() {
        return Ok(());
    }
    let legacy_path = dirs::home_dir()
        .unwrap()
        .join(".config")
        .join("trackie.json");

    if legacy_path.is_file() {
        eprintln!("Legacy data detected. Running migration...");
        let new_path = trackie_file();
        create_dir_all(new_path.parent().unwrap())?;
        assert!(
            !new_path.exists(),
            "Failed migration detected. Please delete either {:?} or {:?}",
            legacy_path,
            new_path
        );
        rename(legacy_path, new_path)?;
    }
    Ok(())
}

fn trackie_file() -> PathBuf {
    env::var(ENV_TRACKIE_CONFIG)
        .ok()
        .map(Into::<PathBuf>::into)
        .or_else(default_trackie_file)
        .unwrap()
}

fn default_trackie_file() -> Option<PathBuf> {
    dirs::data_dir().map(|i| i.join("trackie").join("trackie.json"))
}
