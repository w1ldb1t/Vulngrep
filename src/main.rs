use config::AppConfig;
use console::style;
use console::Term;
use history::History;
use open;
use repository::GithubRepository;
use std::env;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

mod config;
mod history;
mod repository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // parse the yaml config & repo history
    let config = AppConfig::load()?;
    let mut history = History::load()?;
    let term = Term::stdout();

    println!(
        "{} Configuration loaded successfully",
        style("[✓]").green().bold()
    );

    // parse any console argument
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 || args.len() != 1 {
        let command = &args[1];
        if command == "config" {
            let config_path: std::path::PathBuf = AppConfig::get_config_path()?;
            open::that(config_path.as_os_str())?;
        } else {
            println!("Usage: github-notify [config]");
        }
        return Ok(());
    }

    // bail out quickly if there are no desired notifications
    if config.notifications().len() == 0 {
        println!("{} No notifications found!", style("[!]").yellow().bold());
        return Ok(());
    }

    loop {
        let _ = perform_search(&term, &config, &mut history).await;
        sleep(Duration::from_secs(config.interval()));
    }
}

async fn perform_search(
    term: &Term,
    config: &AppConfig,
    history: &mut History,
) -> Result<(), Box<dyn Error>> {
    for notification in config.notifications() {
        let config_rep = &notification.repository();
        let repo: GithubRepository;

        // setup the communication with GitHub's API for this repo
        match GithubRepository::new(&config_rep.owner, &config_rep.name, &config.token()) {
            Ok(github_repo) => {
                repo = github_repo;
                let header = format!(
                    "Inspecting repository {} ...",
                    style(repo.uri()).white().underlined()
                );
                println!("{} {}", style("[*]").blue().bold(), header);
            }
            Err(error) => {
                eprintln!("{} {}", style("✗").red().bold(), error);
                std::process::exit(1);
            }
        }

        // get the head commit
        let head_commit = repo.get_head().await?;
        if !history.has(&repo) {
            // set the current HEAD commit as the last checked
            history.add(&repo, head_commit.sha);
            // save the history information
            history.save()?;
            // inform the user
            term.clear_last_lines(1)?;
            println!(
                "{} Repository {} has been added to the database",
                style("[*]").blue().bold(),
                style(repo.uri()).white().underlined()
            );
            // skip further processing, we are the top
            continue;
        }

        let last_sha = history.find(&repo)?;
        if *last_sha == head_commit.sha {
            // no new commits found, remove prior logs
            term.clear_last_lines(1)?;
            continue;
        }

        // inform the user that new commits will now be downloaded
        println!(
            "{:>4}{} Downloading new commits ...",
            "",
            style("[*]").blue().bold()
        );
        let fetched_commits = repo.fetch_commits_until(last_sha.as_str(), 5).await;

        let mut matching_commit_found = false;
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
                    // go over all the files that this commit refers to
                    for committed_file in commit.files.unwrap() {
                        let mut pattern_satisfied = false;
                        for file in notification.files() {
                            if committed_file.filename.contains(&file.path) {
                                pattern_satisfied = true;
                                continue;
                            }
                        }
                        // if no user-defined pattern matched, bail out
                        if !pattern_satisfied {
                            continue;
                        }
                        // remove the "Downloading new commits" msg above
                        if !matching_commit_found {
                            matching_commit_found = true;
                            term.clear_last_lines(1)?;
                        }
                        // make the commit's hash a clickable link to the official github page
                        let commit_to_link = format!(
                            "\x1B]8;;{}\x07{}\x1B]8;;\x07",
                            &commit.html_url,
                            commit.sha.as_str()
                        );
                        // print the information of the relevant commits
                        println!(
                            "{:>4}{} Commit SHA: {}",
                            "",
                            style("[!]").yellow().bold(),
                            style(commit_to_link).blue().underlined(),
                        );
                        println!(
                            "{:>7} File: {file}, Additions: {additions}, Deletions: {deletions}",
                            "",
                            file = style(committed_file.filename).white().bold(),
                            additions = style(committed_file.additions).green().underlined(),
                            deletions = style(committed_file.deletions).red().underlined(),
                        );
                    }
                }
            }
            Err(error) => {
                eprintln!("{} {}", style("✗").red().bold(), error);
                continue;
            }
        }
        // if there were new commits, remove prior logs
        if !matching_commit_found {
            term.clear_last_lines(2)?;
        }
    }
    Ok(())
}
