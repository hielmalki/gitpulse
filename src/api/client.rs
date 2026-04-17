use anyhow::{Context, Result};
use reqwest::{Client, header};
use serde_json::Value;
use std::time::Duration;
use chrono::Utc;

use super::types::*;

const BASE: &str = "https://api.github.com";

pub struct GithubClient {
    client: Client,
}

impl GithubClient {
    pub fn new(token: Option<String>) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/vnd.github.v3+json"),
        );
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("gitpulse/0.1.0"),
        );
        if let Some(t) = token {
            if let Ok(v) = header::HeaderValue::from_str(&format!("Bearer {t}")) {
                headers.insert(header::AUTHORIZATION, v);
            }
        }
        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .use_rustls_tls()
            .build()
            .expect("failed to build HTTP client");
        Self { client }
    }

    async fn get_json(&self, url: &str) -> Result<Value> {
        let resp = self.client.get(url).send().await
            .with_context(|| format!("GET {url}"))?;
        let status = resp.status();
        if status == 404 {
            return Ok(Value::Null);
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error {status} for {url}: {body}");
        }
        resp.json().await.with_context(|| format!("parsing JSON from {url}"))
    }

    pub async fn fetch_user(&self, username: &str) -> Result<GithubUser> {
        let url = format!("{BASE}/users/{username}");
        let v: GithubUser = serde_json::from_value(self.get_json(&url).await?)?;
        Ok(v)
    }

    pub async fn fetch_repos(&self, username: &str) -> Result<Vec<Repository>> {
        let mut all = Vec::new();
        let mut page = 1u32;
        loop {
            let url = format!(
                "{BASE}/users/{username}/repos?sort=updated&per_page=100&page={page}"
            );
            let v = self.get_json(&url).await?;
            let batch: Vec<Repository> = serde_json::from_value(v)?;
            if batch.is_empty() {
                break;
            }
            all.extend(batch);
            page += 1;
            if page > 5 {
                break; // cap at 500 repos
            }
        }
        Ok(all)
    }

    /// Check if a path exists in a repo (using Contents API — returns 200 or 404)
    async fn path_exists(&self, full_name: &str, path: &str) -> bool {
        let url = format!("{BASE}/repos/{full_name}/contents/{path}");
        matches!(self.get_json(&url).await, Ok(v) if !v.is_null())
    }

    /// Check if .github/workflows dir has any files
    async fn has_workflows(&self, full_name: &str) -> bool {
        let url = format!("{BASE}/repos/{full_name}/contents/.github/workflows");
        match self.get_json(&url).await {
            Ok(v) => v.as_array().map(|a| !a.is_empty()).unwrap_or(false),
            Err(_) => false,
        }
    }

    async fn count_closed_issues(&self, full_name: &str) -> u32 {
        let url = format!("{BASE}/repos/{full_name}/issues?state=closed&per_page=1");
        // GitHub returns Link header with last page — we parse total from that
        // Simpler: just fetch one page and check if any exist
        match self.client.get(&url).send().await {
            Ok(resp) => {
                // Extract last page from Link header if present
                if let Some(link) = resp.headers().get("link").and_then(|h| h.to_str().ok()) {
                    if let Some(n) = parse_last_page(link) {
                        return n;
                    }
                }
                let body: Value = resp.json().await.unwrap_or(Value::Null);
                body.as_array().map(|a| a.len() as u32).unwrap_or(0)
            }
            Err(_) => 0,
        }
    }

    pub async fn enrich_repo(&self, repo: &mut Repository) {
        let fn_ = &repo.full_name;
        // Run checks concurrently
        let (readme, gitignore, ci, tests_dir, tests_file, contributing, coc) = tokio::join!(
            self.path_exists(fn_, "README.md"),
            self.path_exists(fn_, ".gitignore"),
            self.has_workflows(fn_),
            self.path_exists(fn_, "tests"),
            self.path_exists(fn_, "test"),
            self.path_exists(fn_, "CONTRIBUTING.md"),
            self.path_exists(fn_, "CODE_OF_CONDUCT.md"),
        );
        repo.has_readme = readme;
        repo.has_gitignore = gitignore;
        repo.has_ci = ci;
        repo.has_tests = tests_dir || tests_file;
        repo.has_contributing = contributing;
        repo.has_code_of_conduct = coc;
        repo.closed_issues_count = self.count_closed_issues(fn_).await;
    }

    /// Fetch contribution activity from the events API as a proxy for contributions
    pub async fn fetch_contributions(&self, username: &str) -> Result<Vec<ContributionDay>> {
        // GitHub doesn't expose the contribution graph via REST API without GraphQL.
        // We use the public events API and synthesize per-day counts for the last 90 days.
        let mut days: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
        for page in 1..=3u32 {
            let url = format!("{BASE}/users/{username}/events/public?per_page=100&page={page}");
            let v = self.get_json(&url).await?;
            let events = match v.as_array() {
                Some(a) => a.clone(),
                None => break,
            };
            if events.is_empty() {
                break;
            }
            for ev in &events {
                if let Some(date_str) = ev["created_at"].as_str() {
                    let date = &date_str[..10];
                    *days.entry(date.to_string()).or_insert(0) += 1;
                }
            }
        }
        // Fill last 364 days
        let today = Utc::now().date_naive();
        let mut result = Vec::new();
        for offset in (0..364i64).rev() {
            let d = today - chrono::Duration::days(offset);
            let key = d.format("%Y-%m-%d").to_string();
            let count = days.get(&key).copied().unwrap_or(0);
            result.push(ContributionDay { date: key, count });
        }
        Ok(result)
    }

    pub async fn fetch_all(&self, username: &str) -> Result<GithubData> {
        let (user, mut repos, contributions) = tokio::try_join!(
            self.fetch_user(username),
            self.fetch_repos(username),
            self.fetch_contributions(username),
        )?;

        // Enrich top-20 non-fork repos (rate-limit friendly)
        let to_enrich: Vec<usize> = repos
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.fork && !r.archived)
            .map(|(i, _)| i)
            .take(20)
            .collect();

        // Enrich in batches of 5 concurrently
        for chunk in to_enrich.chunks(5) {
            let futs: Vec<_> = chunk.iter().map(|&i| {
                let fn_ = repos[i].full_name.clone();
                async move {
                    let mut tmp = repos[i].clone();
                    self.enrich_repo(&mut tmp).await;
                    (i, tmp)
                }
            }).collect();
            let results = futures_batch(futs).await;
            for (i, enriched) in results {
                repos[i] = enriched;
            }
        }

        let total_stars = repos.iter().map(|r| r.stargazers_count).sum();
        let total_forks = repos.iter().map(|r| r.forks_count).sum();

        Ok(GithubData {
            user,
            repos,
            contributions,
            total_stars,
            total_forks,
            fetched_at: Utc::now(),
        })
    }
}

async fn futures_batch<T: Send + 'static>(
    futs: Vec<impl std::future::Future<Output = T> + Send + 'static>,
) -> Vec<T> {
    let handles: Vec<_> = futs.into_iter().map(tokio::spawn).collect();
    let mut results = Vec::new();
    for h in handles {
        if let Ok(r) = h.await {
            results.push(r);
        }
    }
    results
}

fn parse_last_page(link: &str) -> Option<u32> {
    // Link: <url?page=N>; rel="last"
    for part in link.split(',') {
        if part.contains(r#"rel="last""#) {
            if let Some(start) = part.find("page=") {
                let s = &part[start + 5..];
                let end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
                return s[..end].parse().ok();
            }
        }
    }
    None
}
