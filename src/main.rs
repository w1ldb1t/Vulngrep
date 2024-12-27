use config::AppConfig;
use history::History;
use repository::GithubRepository;
use std::error::Error;

mod config;
mod history;
mod repository;

fn parse_interval(interval_str: &str) -> Result<u64, String> {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // parse the yaml config
    let config = AppConfig::load()?;
    let mut history = History::load()?;

    // convert config str to interval
    let interval_seconds = parse_interval(&config.interval)?;
    println!("Interval: {} seconds", interval_seconds);

    // bail out quickly if there are no desired notifications
    if config.notifications.len() == 0 {
        println!("No notifications found!");
        return Ok(())
    }

    for notification in &config.notifications {
        let config_rep = &notification.repository;
        let repo: GithubRepository;

        match GithubRepository::new(&config_rep.owner, &config_rep.name, &config.github_token) {
            Ok(github_repo) => {
                repo = github_repo;
                println!(
                    "Repository: {}",
                    format!("{}/{}", repo.owner(), repo.name())
                );
            }
            Err(error) => {
                eprintln!("{}", error);
                std::process::exit(1);
            }
        }

        // get the head commit
        let head_commit = repo.get_head().await?;
        if !history.has(&repo) {
            println!("Initializing history of repo {} ...", repo.uri());
            // set the current HEAD commit as the last checked
            history.add(&repo, head_commit.sha);
            // save the history information
            history.save()?;
            // skip further processing, we are the top
            continue;
        }

        let last_sha = history.find(&repo)?;
        if *last_sha == head_commit.sha {
            println!("No new commits found!");
            continue;
        }

        println!("Fetching new commits for {} ...", repo.uri());
        let fetched_commits = repo.fetch_commits_until(last_sha.as_str(), 5).await;

        match fetched_commits {
            Ok(commit_details) => {
                // update the history record for this repo
                history.add(&repo, commit_details.first().unwrap().sha.clone());
                history.save()?;

                // process all new commits for this repo
                for commit in commit_details {
                    // the commit does details do not mention file changes
                    if !commit.files.is_some() {
                        continue;
                    }

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
                        println!("{}", commit.html_url);
                    }
                }
            }
            Err(error) => {
                eprintln!("{}", error);
            }
        }
    }

    Ok(())
}
