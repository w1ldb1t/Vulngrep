#![allow(unused)]

use anyhow::{Result, Context};
use octocrab::models::repos::RepoCommit;

#[derive(Debug, Clone)]
pub struct GithubRepository {
    owner: String,
    name: String,
    client: octocrab::Octocrab,
}

impl GithubRepository {
    /// Creates a new GithubRepository instance
    pub fn new(owner: impl Into<String>, name: impl Into<String>, token: &str) -> Result<Self> {
        let client = octocrab::OctocrabBuilder::new()
            .personal_token(token.to_string())
            .build()
            .context("Failed to initialize GitHub client")?;

        Ok(Self {
            owner: owner.into(),
            name: name.into(),
            client,
        })
    }

    /// Fetches all commits up to a specific hash
    pub async fn fetch_commits_until(
        &self,
        target_hash: &str,
        per_page: u8,
    ) -> Result<Vec<RepoCommit>> {
        let mut all_commits = Vec::new();
        let mut page = 1u32;
        
        loop {
            let commits = self.client
                .repos(&self.owner, &self.name)
                .list_commits()
                .per_page(per_page)
                .page(page)
                .send()
                .await
                .with_context(|| format!("Failed to fetch commits from {}/{}", self.owner, self.name))?;

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
                    Ok(commit_details) =>  {
                        // add the detailed commit info to the list
                        all_commits.push(commit_details);
                    },
                    Err(_) => {
                        // request failed, but the commit is actually valid
                        all_commits.push(commit);
                    },
                }
            }

            page += 1;
        }

        if all_commits.is_empty() {
            anyhow::bail!("No commits found or invalid commit hash: {}", target_hash);
        } else {
            Ok(all_commits)
        }
    }

    // Gets the HEAD commit of the current repository
    pub async fn get_head(&self) -> Result<RepoCommit> {
        let commits = self.client
                .repos(&self.owner, &self.name)
                .list_commits()
                .per_page(1)
                .page(1u32)
                .send()
                .await
                .with_context(|| format!("Failed to fetch commits from {}/{}", self.owner, self.name))?;
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
