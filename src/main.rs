use config::AppConfig;
use history::History;
use open;
use repository::GithubRepository;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use std::{env, time};

mod config;
mod history;
mod repository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // parse the yaml config & repo history
    let config = AppConfig::load()?;
    let mut history = History::load()?;

    // parse any console argument
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        let command = &args[1];
        if command == "config" {
            let config_path: std::path::PathBuf = AppConfig::get_config_path()?;
            open::that(config_path.as_os_str())?;
        } else if command == "help" {
            println!("github-notify [config]")
        }
        return Ok(())
    }

    // bail out quickly if there are no desired notifications
    if config.notifications().len() == 0 {
        println!("No notifications found!");
        return Ok(())
    }

    loop {
        let _ = perform_search(&config, &mut history).await;
        sleep(Duration::from_secs(config.interval()));
    }
}

async fn perform_search(config: &AppConfig, history: &mut History) -> Result<(), Box<dyn Error>> {
    for notification in config.notifications() {
        let config_rep = &notification.repository();
        let repo: GithubRepository;

        match GithubRepository::new(&config_rep.owner, &config_rep.name, &config.token()) {
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
                        for file in notification.files() {
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
