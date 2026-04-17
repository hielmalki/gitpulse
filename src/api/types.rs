use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubUser {
    pub login: String,
    pub name: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub avatar_url: String,
    pub html_url: String,
    pub public_repos: u32,
    pub followers: u32,
    pub following: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub language: Option<String>,
    pub stargazers_count: u32,
    pub forks_count: u32,
    pub open_issues_count: u32,
    pub topics: Vec<String>,
    pub has_wiki: bool,
    pub has_issues: bool,
    pub fork: bool,
    pub archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pushed_at: Option<DateTime<Utc>>,
    pub license: Option<LicenseInfo>,
    // Enriched fields fetched separately
    #[serde(default)]
    pub has_readme: bool,
    #[serde(default)]
    pub has_gitignore: bool,
    #[serde(default)]
    pub has_ci: bool,
    #[serde(default)]
    pub has_tests: bool,
    #[serde(default)]
    pub has_contributing: bool,
    #[serde(default)]
    pub has_code_of_conduct: bool,
    #[serde(default)]
    pub closed_issues_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionDay {
    pub date: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubData {
    pub user: GithubUser,
    pub repos: Vec<Repository>,
    pub contributions: Vec<ContributionDay>,
    pub total_stars: u32,
    pub total_forks: u32,
    pub fetched_at: DateTime<Utc>,
}

impl GithubData {
    pub fn owned_repos(&self) -> Vec<&Repository> {
        self.repos.iter().filter(|r| !r.fork && !r.archived).collect()
    }
}
