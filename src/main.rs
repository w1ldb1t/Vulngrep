use std::fs;
use std::error::Error;
use serde::{Deserialize, Serialize};
use repository::GithubRepository;

mod repository;

#[derive(Debug, Serialize, Deserialize)]
struct File {
    path: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Repository {
    name: String,
    owner: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Notification {
    repository: Repository,
    files: Vec<File>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    interval: String,
    github_token: String,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
        let config_rep = notification.repository;
        let repo : GithubRepository;

        match GithubRepository::new(config_rep.owner, config_rep.name, &config.github_token) {
            Ok(github_repo) =>  {
                repo = github_repo;
                println!("Repository: {}", format!("{}/{}", repo.owner(), repo.name()));
            },
            Err(error) =>  {
                eprintln!("{}", error);
                std::process::exit(1);
            },
        }

        let fetched_commits = repo.fetch_commits_until("HASH", 10).await;

        match fetched_commits {
            Ok(commit_details) =>  {
                for commit in commit_details {
                    for committed_file in commit.files.unwrap() {
                        let mut pattern_satisfied = false;
                        for file in &notification.files {
                            if committed_file.filename.contains(&file.path) {
                                pattern_satisfied = true;
                                continue;
                            }
                        }
                        if !pattern_satisfied {
                            continue;
                        }
            
                        println!("-----");
                        println!("Commit SHA: {}", commit.sha);
                        println!(
                            "File: {file}, Additions: {additions}, Deletions: {deletions}",
                            file = committed_file.filename,
                            additions = committed_file.additions,
                            deletions = committed_file.deletions,
                        );
                        println!("-----");
                    }
                }
            },
            Err(error) =>  {
                eprintln!("{}", error);
            },
        }
    }

    Ok(())
}