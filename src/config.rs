use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub path: String,
    pub pattern: Option<Vec<String>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub owner: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    repository: Repository,
    files: Vec<File>,
    pattern: Option<Vec<String>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    interval: Option<String>,
    github_token: String,
    notifications: Vec<Notification>,
}

impl Clone for Repository {
    fn clone(&self) -> Self {
        Repository {
            name: self.name.clone(),
            owner: self.owner.clone(),
        }
    }
}

impl Notification {
    pub fn repository(&self) -> Repository {
        self.repository.clone()
    }
    pub fn files(&self) -> &Vec<File> {
        &self.files
    }
    pub fn patterns(&self) -> Option<Vec<String>> {
        self.pattern.clone()
    }
}

impl AppConfig {
    const CONFIG_FILENAME: &'static str = "config.yaml";
    const QUALIFIER: &'static str = "com";
    const ORGANIZATION: &'static str = "wildbit";
    const APPLICATION: &'static str = "vulngrep";

    fn parse_interval(&self, interval_str: &str) -> Result<u64, String> {
        let (value, unit) = interval_str.split_at(interval_str.len() - 1);
    
        let numeric_value: u64 = match value.parse() {
            Ok(num) => num,
            Err(_) => return Err(format!("Invalid numeric value in interval: {}", value)),
        };
    
        match unit {
            "h" => Ok(numeric_value * 3600), // hours to seconds
            "m" => Ok(numeric_value * 60),   // minutes to seconds
            _ => Err(format!(
                "Invalid time unit: {}. Use 'h' for hours or 'm' for minutes.",
                unit
            )),
        }
    }

    /// Get the configuration file path based on the operating system
    pub fn get_config_path() -> Result<PathBuf, Box<dyn Error>> {
        let proj_dirs = ProjectDirs::from(Self::QUALIFIER, Self::ORGANIZATION, Self::APPLICATION)
            .ok_or("Could not find config directory")?;

        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir)?;

        Ok(config_dir.join(Self::CONFIG_FILENAME))
    }

    /// Load configuration from the default location
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            // create default configuration
            let default = Self::default();
            // save it as a YAML file
            default.save()?;
            // return the configuration
            return Ok(default);
        }

        let contents = fs::read_to_string(config_path)?;
        let config: AppConfig = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to the default location
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let config_path = Self::get_config_path()?;
        let contents = serde_yaml::to_string(self)?;
        fs::write(config_path, contents)?;
        Ok(())
    }

    /// Returns None for one-time execution, or how many seconds to sleep between cycles
    pub fn interval(&self) -> Option<u64> {
        self.interval.as_ref().and_then(|interval_str| {
            match self.parse_interval(interval_str) {
                Ok(interval) => Some(interval),
                Err(err) => {
                    eprintln!("{}", err);
                    Some(60) // default to 60 secs
                }
            }
        })
    }

    /// The GitHub token
    pub fn token(&self) -> &String {
        return &self.github_token;
    }

    /// The user-defined notifications
    pub fn notifications(&self) -> &Vec<Notification> {
        return &self.notifications
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            interval: None,
            github_token: "GITHUB_TOKEN".to_owned(),
            notifications: vec![],
        }
    }
}
