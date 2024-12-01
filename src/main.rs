use serde::{Deserialize, Serialize};
use std::fs;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct File {
    path: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Notification {
    repository: String,
    files: Vec<File>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    interval: String,
    notifications: Vec<Notification>,
}

fn parse_interval(interval_str: &str) -> Result<u64, String> {
    let (value, unit) = interval_str.split_at(interval_str.len() - 1);
    
    let numeric_value: u64 = match value.parse() {
        Ok(num) => num,
        Err(_) => return Err(format!("Invalid numeric value in interval: {}", value)),
    };

    match unit {
        "h" => Ok(numeric_value * 3600), // hours to seconds
        "m" => Ok(numeric_value * 60),   // minutes to seconds
        _ => Err(format!("Invalid time unit: {}. Use 'h' for hours or 'm' for minutes.", unit)),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // read the yaml config
    let contents = fs::read_to_string("config.yaml")?;
    
    // parse the yaml config
    let config: Config = serde_yaml::from_str(&contents)?;

    // convert config str to interval
    let interval_seconds = parse_interval(&config.interval)?;

    // print the config
    println!("Interval: {} seconds", interval_seconds);
    println!("Notifications:");
    for notification in config.notifications {
        println!("Repository: {}", notification.repository);
        for repo in notification.files {
            println!("      {}", repo.path);
        }
    }

    Ok(())
}