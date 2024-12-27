use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub owner: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    pub repository: Repository,
    pub files: Vec<File>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub interval: String,
    pub github_token: String,
    pub notifications: Vec<Notification>,
}

impl AppConfig {
    const CONFIG_FILENAME: &'static str = "config.yaml";
    const QUALIFIER: &'static str = "com";
    const ORGANIZATION: &'static str = "wildbit";
    const APPLICATION: &'static str = "github_notify";

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
        println!("Config path: {}", config_path.display().to_string());

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
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            interval: "6h".to_owned(),
            github_token: "GITHUB_TOKEN".to_owned(),
            notifications: vec![],
        }
    }
}
