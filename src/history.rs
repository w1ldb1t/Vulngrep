use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use crate::repository::GithubRepository;

#[derive(Debug, Serialize, Deserialize)]
pub struct History {
    repositories: HashMap<String, String>,
}

impl History {
    const CONFIG_FILENAME: &'static str = "history.yaml";
    const QUALIFIER: &'static str = "com";
    const ORGANIZATION: &'static str = "wildbit";
    const APPLICATION: &'static str = "vulngrep";

    /// Get the configuration file path based on the operating system
    pub fn get_config_path() -> Result<PathBuf, Box<dyn Error>> {
        let proj_dirs = ProjectDirs::from(Self::QUALIFIER, Self::ORGANIZATION, Self::APPLICATION)
            .ok_or("Could not find config directory")?;

        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir)?;

        Ok(data_dir.join(Self::CONFIG_FILENAME))
    }

    /// Load configuration from the default location
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            // return (the) default configuration
            let default = Self::default();
            return Ok(default);
        }

        let contents = fs::read_to_string(config_path)?;
        let config: History = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to the default location
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let config_path = Self::get_config_path()?;
        let contents = serde_yaml::to_string(self)?;
        fs::write(config_path, contents)?;
        Ok(())
    }

    /// Verifies whether there is a record for a given repository
    pub fn has(&self, repo: &GithubRepository) -> bool {
        return self.repositories.contains_key(repo.uri().as_str());
    }

    /// Update/add a record for a given repository
    pub fn add(&mut self, repo: &GithubRepository, hash: String) {
        self.repositories.insert(repo.uri(), hash);
    }

    /// Find the last hash checked for this repository (assumes that it exists)
    pub fn find(&self, repo: &GithubRepository) -> Result<String, Box<dyn Error>> {
        self.repositories
            .get(repo.uri().as_str())
            .cloned()
            .ok_or_else(|| format!("No record for the repository {}", repo.name()).into())
    }
}

impl Default for History {
    fn default() -> Self {
        Self {
            repositories: Default::default(),
        }
    }
}
