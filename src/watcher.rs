use crate::config::AppConfig;
use crate::history::History;
use crate::repository::GithubRepository;
use crate::terminal::TerminalDisplay;
use std::error::Error;
use std::time::Duration;
use wildmatch::WildMatch;

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

        // if interval is set, continue monitoring with countdown
        if let Some(interval_secs) = self.config.interval() {
            let interval = Duration::from_secs(interval_secs);
            loop {
                // show countdown
                self.display.show_countdown(interval)?;

                // process repositories again
                self.process_repositories().await?;
            }
        }

        Ok(())
    }

    // Creates a wildcard out of the string, and makes it inclusive
    fn make_pattern(&self, pattern: &String) -> WildMatch {
        let inclusive_pattern = format!("*{}*", pattern);
        WildMatch::new(&inclusive_pattern.as_str())
    }

    /// Goes over all repositories, and notifies the user for any matching rules
    async fn process_repositories(&mut self) -> Result<(), Box<dyn Error>> {
        for notification in self.config.notifications() {
            let config_rep = &notification.repository();
            let repo = match GithubRepository::new(
                &config_rep.owner,
                &config_rep.name,
                &self.config.token(),
            ) {
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
            let head_commit = match repo.get_head().await {
                Ok(commit) => commit,
                Err(err) => {
                    self.display
                        .repository_error(&err.to_string());
                    continue;
                }
            };

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
                    self.history
                        .add(&repo, commit_details.first().unwrap().sha.clone());
                    self.history.save()?;

                    for commit in commit_details {
                        // Figure out if a commit is of interest. A commit is of interest if:
                        // 1) It has a matching repository-wide pattern
                        // 2) It has a matching file path and no patterns attached to it
                        // 3) It has a matching file path and at least one matching file pattern
                        let mut is_commit_of_interest = false;

                        if let Some(patterns) = notification.patterns() {
                            for pattern in patterns {
                                let wild_pattern = self.make_pattern(&pattern);
                                if wild_pattern.matches(&commit.commit.message) {
                                    if !matching_commit_found {
                                        matching_commit_found = true;
                                        self.display.clear_lines(1)?;
                                    }

                                    self.display.commit_info(
                                        &commit.html_url,
                                        &commit.sha,
                                        &pattern,
                                    );

                                    if let Some(author) = &commit.author {
                                        self.display.commit_notification(
                                            &repo.uri(),
                                            &commit.sha,
                                            &author.login,
                                        )?;
                                    }

                                    is_commit_of_interest = true;
                                    break;
                                }
                            }
                        }
                        if is_commit_of_interest {
                            continue;
                        }

                        if let Some(files) = commit.files {
                            for committed_file in files {
                                let mut patterns_responsible_for_hit: Vec<String> = Vec::new();

                                for file in notification.files() {
                                    let file_path_matches = self.make_pattern(&file.path)
                                        .matches(&committed_file.filename);
                                    if !file_path_matches {
                                        break;
                                    }

                                    // Is there a matching file path with no patterns?
                                    if file_path_matches && file.pattern.is_none() {
                                        is_commit_of_interest = true;
                                        break;
                                    }

                                    // Is there a matching file-wide or repository-wide pattern?
                                    if let Some(patch) = &committed_file.patch {
                                        let patterns_list =
                                            [&file.pattern, &notification.patterns()];

                                        for patterns in patterns_list {
                                            if let Some(patterns) = patterns {
                                                if let Some(pattern) =
                                                    patterns.iter().find(|pattern| {
                                                        self.make_pattern(pattern).matches(patch)
                                                    })
                                                {
                                                    patterns_responsible_for_hit
                                                        .push(pattern.to_string());
                                                    is_commit_of_interest = true;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    if is_commit_of_interest {
                                        break;
                                    }
                                }

                                // Bail out if the commit is not of interest
                                if !is_commit_of_interest {
                                    continue;
                                }

                                if !matching_commit_found {
                                    matching_commit_found = true;
                                    self.display.clear_lines(1)?;
                                }

                                self.display.full_commit_info(
                                    &commit.html_url,
                                    &commit.sha,
                                    &committed_file.filename,
                                    committed_file.additions,
                                    committed_file.deletions,
                                    patterns_responsible_for_hit,
                                );

                                if let Some(author) = &commit.author {
                                    self.display.commit_notification(
                                        &repo.uri(),
                                        &commit.sha,
                                        &author.login,
                                    )?;
                                }

                                is_commit_of_interest = false;
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
