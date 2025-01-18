use crate::config::AppConfig;
use crate::history::History;
use crate::repository::GithubRepository;
use crate::terminal::TerminalDisplay;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

pub struct RepositoryWatcher {
    config: AppConfig,
    history: History,
    display: TerminalDisplay,
}

impl RepositoryWatcher {
    /// Creates a new RepositoryWatcher instance
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            config: AppConfig::load()?,
            history: History::load()?,
            display: TerminalDisplay::new(),
        })
    }

    /// Starts the observation of the repositories
    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.display.config_loaded();

        // if the notification config is empty bail out quickly
        if self.config.notifications().is_empty() {
            self.display.empty_config();
            return Ok(());
        }

        // process repositories at least once
        self.process_repositories().await?;

        // if interval is set, continue monitoring
        if let Some(interval_secs) = self.config.interval() {
            loop {
                sleep(Duration::from_secs(interval_secs));
                self.process_repositories().await?;
            }
        }

        Ok(())
    }

    /// Goes over all repositories, and notifies the user for any matching rules
    async fn process_repositories(&mut self) -> Result<(), Box<dyn Error>> {
        for notification in self.config.notifications() {
            let config_rep = &notification.repository();
            let repo = match GithubRepository::new(&config_rep.owner, &config_rep.name, &self.config.token()) {
                Ok(repo) => {
                    self.display.inspect(&repo);
                    repo
                }
                Err(error) => {
                    self.display.repository_error(&error.to_string());
                    return Err(error.into());
                }
            };

            // Process head commit
            let head_commit = repo.get_head().await?;
            if !self.history.has(&repo) {
                self.history.add(&repo, head_commit.sha);
                self.history.save()?;
                self.display.repository_added(&repo)?;
                continue;
            }

            let last_sha = self.history.find(&repo)?;
            if *last_sha == head_commit.sha {
                self.display.clear_lines(1)?;
                continue;
            }

            self.display.downloading_commits();
            let mut matching_commit_found = false;

            match repo.fetch_commits_until(last_sha.as_str(), 5).await {
                Ok(commit_details) => {
                    self.history.add(&repo, commit_details.first().unwrap().sha.clone());
                    self.history.save()?;

                    for commit in commit_details {
                        if let Some(files) = commit.files {
                            for committed_file in files {
                                if !notification.files().iter().any(|file| committed_file.filename.contains(&file.path)) {
                                    continue;
                                }

                                if !matching_commit_found {
                                    matching_commit_found = true;
                                    self.display.clear_lines(1)?;
                                }

                                self.display.commit_info(
                                    &commit.html_url,
                                    &commit.sha,
                                    &committed_file.filename,
                                    committed_file.additions,
                                    committed_file.deletions,
                                );

                                if let Some(author) = &commit.author {
                                    self.display.commit_notification(
                                        &repo.uri(),
                                        &commit.sha,
                                        &author.login,
                                    )?;
                                }
                            }
                        }
                    }
                }
                Err(error) => {
                    self.display.repository_error(&error.to_string());
                    continue;
                }
            }

            if !matching_commit_found {
                self.display.clear_lines(2)?;
            }
        }
        Ok(())
    }
}