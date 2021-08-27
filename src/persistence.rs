use crate::cli::ENV_TRACKIE_CONFIG;
use crate::time_log::TimeLog;
use std::env;
use std::error::Error;
use std::fs::{create_dir_all, read_to_string, rename,  OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub trait FileHandler {
    fn read_file(&self) -> Result<Option<String>, Box<dyn Error>>;

    fn write_file(&mut self, content: &str) -> Result<(), Box<dyn Error>>;
}

pub struct FsFileHandler {}

impl FsFileHandler {
    pub fn new() -> Self {
        Self {}
    }

    fn default_trackie_file() -> Option<PathBuf> {
        dirs::data_dir().map(|i| i.join("trackie").join("trackie.json"))
    }

    fn trackie_file() -> PathBuf {
        env::var(ENV_TRACKIE_CONFIG)
            .ok()
            .map(Into::<PathBuf>::into)
            .or_else(Self::default_trackie_file)
            .unwrap()
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
            let new_path = Self::trackie_file();
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
}

impl FileHandler for FsFileHandler {
    fn read_file(&self) -> Result<Option<String>, Box<dyn Error>> {
        Self::move_legacy_config_file()?;
        let trackie_file = Self::trackie_file();
        if trackie_file.exists() {
            Ok(Some(read_to_string(trackie_file)?))
        } else {
            Ok(None)
        }
    }

    fn write_file(&mut self, content: &str) -> Result<(), Box<dyn Error>> {
        let trackie_file = Self::trackie_file();
        create_dir_all(&trackie_file.parent().unwrap())?;

        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(trackie_file)?;

        f.write(content.as_bytes())?;
        f.flush()?;
        Ok(())
    }
}

pub fn load_or_create_log(handler: &dyn FileHandler) -> Result<TimeLog, Box<dyn Error>> {
    match handler.read_file()? {
        Some(content) => Ok(TimeLog::from_json(&content)?),
        None => Ok(TimeLog::default()),
    }
}

pub fn save_log(handler: &mut dyn FileHandler, log: &TimeLog) -> Result<(), Box<dyn Error>> {
    let content = serde_json::to_string(log)?;
    handler.write_file(&content)
}
