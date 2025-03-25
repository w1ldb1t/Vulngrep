#![allow(unused)]

use octocrab::models::repos::RepoCommit;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum GithubRepositoryError {
    #[error("Failed to initialize GitHub client")]
    InitializationFailed,
    #[error("Invalid GitHub token")]
    InvalidToken,
    #[error("Repository not found")]
    InvalidRepository,
    #[error("Failed to fetch commits")]
    FetchCommitsFailed,
    #[error("Invalid commit hash")]
    InvalidCommitHash,
}

#[derive(Debug, Clone)]
pub struct GithubRepository {
    owner: String,
    name: String,
    client: octocrab::Octocrab,
}

impl GithubRepository {
    /// Creates a new GithubRepository instance
    pub async fn new(
        owner: impl Into<String>,
        name: impl Into<String>,
        token: &str,
    ) -> Result<Self, GithubRepositoryError> {
        let client = octocrab::OctocrabBuilder::new()
            .personal_token(token.to_string())
            .build()
            .map_err(|_| GithubRepositoryError::InitializationFailed)?;

        // check if the GitHub token is valid or not
        let user = client.current().user().await;
        match user {
            Ok(_) => (),
            Err(_) => return Err(GithubRepositoryError::InvalidToken.into()),
        }

        let (owner, name) = (owner.into(), name.into());

        let repo = client.repos(&owner, &name).get().await;
        match repo {
            Ok(_) => {}
            Err(_) => return Err(GithubRepositoryError::InvalidRepository.into()),
        }

        Ok(Self {
            owner,
            name,
            client,
        })
    }

    /// Fetches all commits up to a specific hash
    pub async fn fetch_commits_until(
        &self,
        target_hash: &str,
        per_page: u8,
    ) -> Result<Vec<RepoCommit>, GithubRepositoryError> {
        let mut all_commits = Vec::new();
        let mut page = 1u32;

        loop {
            let commits = self
                .client
                .repos(&self.owner, &self.name)
                .list_commits()
                .per_page(per_page)
                .page(page)
                .send()
                .await
                .map_err(|_| GithubRepositoryError::FetchCommitsFailed)?;

            if commits.items.is_empty() {
                break;
            }

            // process commits
            for commit in commits.items {
                if commit.sha == target_hash {
                    return Ok(all_commits);
                }

                let sha = &commit.sha;
                match self.client.commits(&self.owner, &self.name).get(sha).await {
                    Ok(commit_details) => {
                        // add the detailed commit info to the list
                        all_commits.push(commit_details);
                    }
                    Err(_) => {
                        // request failed, but the commit is actually valid
                        all_commits.push(commit);
                    }
                }
            }

            page += 1;
        }

        if all_commits.is_empty() {
            Err(GithubRepositoryError::InvalidCommitHash)
        } else {
            Ok(all_commits)
        }
    }

    /// Gets the HEAD commit of the current repository
    pub async fn get_head(&self) -> Result<RepoCommit, GithubRepositoryError> {
        let commits = self
            .client
            .repos(&self.owner, &self.name)
            .list_commits()
            .per_page(1)
            .page(1u32)
            .send()
            .await
            .map_err(|_| GithubRepositoryError::FetchCommitsFailed)?;
        let head_commit = commits.items.first().unwrap();
        Ok(head_commit.clone())
    }

    /// Gets repository owner
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Gets repository name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets owner/name
    pub fn uri(&self) -> String {
        format!("{}/{}", self.owner(), self.name())
    }
}
