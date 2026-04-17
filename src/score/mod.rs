use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::api::{GithubData, Repository};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoScore {
    pub repo_name: String,
    pub score: u8,
    pub max_score: u8,
    pub breakdown: Vec<ScoreItem>,
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreItem {
    pub label: String,
    pub points: i32,
    pub max_points: i32,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub severity: Severity,
    pub message: String,
    pub impact: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    Warning,
    Info,
    Good,
}

impl Severity {
    pub fn icon(&self) -> &'static str {
        match self {
            Severity::Critical => "🔴",
            Severity::Warning => "⚠️",
            Severity::Info => "ℹ️",
            Severity::Good => "✅",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileScore {
    pub overall: u8,
    pub repo_scores: Vec<RepoScore>,
    pub total_stars: u32,
    pub total_forks: u32,
    pub avg_repo_score: u8,
    pub top_repos: Vec<String>,
    pub all_recommendations: Vec<Recommendation>,
}

impl ProfileScore {
    pub fn summary_text(&self) -> String {
        let grade = match self.overall {
            90..=100 => "S",
            80..=89 => "A",
            70..=79 => "B",
            60..=69 => "C",
            50..=59 => "D",
            _ => "F",
        };
        format!(
            "Developer Health Score: {}/100 (Grade: {})\nAvg Repo Score: {}/100 | Stars: {} | Forks: {}\nTop repos: {}",
            self.overall, grade, self.avg_repo_score,
            self.total_stars, self.total_forks,
            self.top_repos.join(", ")
        )
    }
}

pub struct Scorer;

impl Scorer {
    pub fn new() -> Self {
        Self
    }

    pub fn score_repo(&self, repo: &Repository) -> RepoScore {
        let mut breakdown = Vec::new();
        let mut recs = Vec::new();
        let now = Utc::now();

        macro_rules! check {
            ($label:expr, $max:expr, $passed:expr, $rec:expr) => {{
                let pts = if $passed { $max } else { 0 };
                breakdown.push(ScoreItem {
                    label: $label.to_string(),
                    points: pts,
                    max_points: $max,
                    passed: $passed,
                });
                if !$passed {
                    recs.push($rec);
                }
            }};
        }

        check!(
            "README.md",
            15,
            repo.has_readme,
            Recommendation {
                severity: Severity::Critical,
                message: format!("Add a README.md to {} — first thing visitors see", repo.name),
                impact: 15,
            }
        );

        check!(
            "LICENSE",
            10,
            repo.license.is_some(),
            Recommendation {
                severity: Severity::Critical,
                message: format!("Add a LICENSE to {} (MIT recommended for open-source adoption)", repo.name),
                impact: 10,
            }
        );

        check!(
            ".gitignore",
            5,
            repo.has_gitignore,
            Recommendation {
                severity: Severity::Warning,
                message: format!("Add .gitignore to {} to avoid committing build artifacts", repo.name),
                impact: 5,
            }
        );

        check!(
            "CI/CD (GitHub Actions)",
            15,
            repo.has_ci,
            Recommendation {
                severity: Severity::Critical,
                message: format!("Add CI/CD to {} — create .github/workflows/ci.yml", repo.name),
                impact: 15,
            }
        );

        let has_desc_and_topics = repo.description.is_some() && !repo.topics.is_empty();
        check!(
            "Description + Topics",
            10,
            has_desc_and_topics,
            Recommendation {
                severity: Severity::Warning,
                message: format!("Add description and topics to {} for discoverability", repo.name),
                impact: 10,
            }
        );

        // Issue ratio
        let total_issues = repo.open_issues_count + repo.closed_issues_count;
        let good_ratio = total_issues == 0 || {
            let ratio = repo.closed_issues_count as f64 / total_issues as f64;
            ratio >= 0.5
        };
        check!(
            "Issue Resolution Ratio ≥50%",
            10,
            good_ratio,
            Recommendation {
                severity: Severity::Warning,
                message: format!("{} has many open issues — consider triaging and closing stale ones", repo.name),
                impact: 10,
            }
        );

        let days_since_push = repo.pushed_at.map(|p| (now - p).num_days()).unwrap_or(999);
        let recent_30 = days_since_push < 30;
        let recent_90 = days_since_push < 90;

        if recent_30 {
            breakdown.push(ScoreItem {
                label: "Last commit < 30 days".to_string(),
                points: 10,
                max_points: 10,
                passed: true,
            });
        } else if recent_90 {
            breakdown.push(ScoreItem {
                label: "Last commit < 90 days".to_string(),
                points: 5,
                max_points: 10,
                passed: true,
            });
        } else {
            breakdown.push(ScoreItem {
                label: "Recently active".to_string(),
                points: 0,
                max_points: 10,
                passed: false,
            });
            recs.push(Recommendation {
                severity: Severity::Info,
                message: format!("{} hasn't been updated in {} days — consider archiving or updating", repo.name, days_since_push),
                impact: 5,
            });
        }

        check!(
            "Tests",
            10,
            repo.has_tests,
            Recommendation {
                severity: Severity::Warning,
                message: format!("Add tests to {} — no tests/ directory found", repo.name),
                impact: 10,
            }
        );

        check!(
            "CONTRIBUTING.md",
            5,
            repo.has_contributing,
            Recommendation {
                severity: Severity::Info,
                message: format!("Add CONTRIBUTING.md to {} to guide contributors", repo.name),
                impact: 5,
            }
        );

        check!(
            "CODE_OF_CONDUCT.md",
            5,
            repo.has_code_of_conduct,
            Recommendation {
                severity: Severity::Info,
                message: format!("Add CODE_OF_CONDUCT.md to {} for community health", repo.name),
                impact: 5,
            }
        );

        check!(
            "Has stars",
            5,
            repo.stargazers_count > 0,
            Recommendation {
                severity: Severity::Info,
                message: format!("{} has no stars yet — promote it!", repo.name),
                impact: 5,
            }
        );

        let score: i32 = breakdown.iter().map(|b| b.points).sum();
        let max_score: i32 = breakdown.iter().map(|b| b.max_points).sum();

        // Sort recs by impact descending
        recs.sort_by(|a, b| b.impact.cmp(&a.impact));

        if score >= max_score * 9 / 10 {
            recs.push(Recommendation {
                severity: Severity::Good,
                message: format!("✅ {} — Health Score {}/{}: Excellent repo!", repo.name, score, max_score),
                impact: 0,
            });
        }

        RepoScore {
            repo_name: repo.name.clone(),
            score: score.clamp(0, 100) as u8,
            max_score: max_score.clamp(0, 100) as u8,
            breakdown,
            recommendations: recs,
        }
    }

    pub fn compute(&self, data: &GithubData) -> ProfileScore {
        let owned = data.owned_repos();
        let repo_scores: Vec<RepoScore> = owned.iter().map(|r| self.score_repo(r)).collect();

        let avg_repo_score = if repo_scores.is_empty() {
            0
        } else {
            (repo_scores.iter().map(|s| s.score as u32).sum::<u32>() / repo_scores.len() as u32) as u8
        };

        // Overall score: weighted avg repo score (60%) + profile completeness (40%)
        let profile_score = self.profile_completeness(&data);
        let overall = (avg_repo_score as f64 * 0.6 + profile_score as f64 * 0.4) as u8;

        let mut top_repos: Vec<_> = repo_scores.iter()
            .map(|s| (s.repo_name.clone(), s.score))
            .collect();
        top_repos.sort_by(|a, b| b.1.cmp(&a.1));
        let top_repos: Vec<String> = top_repos.into_iter().take(5).map(|(n, _)| n).collect();

        let mut all_recommendations: Vec<Recommendation> = repo_scores.iter()
            .flat_map(|s| s.recommendations.clone())
            .filter(|r| r.severity != Severity::Good)
            .collect();
        all_recommendations.sort_by(|a, b| b.impact.cmp(&a.impact));
        all_recommendations.dedup_by(|a, b| a.message == b.message);

        ProfileScore {
            overall,
            repo_scores,
            total_stars: data.total_stars,
            total_forks: data.total_forks,
            avg_repo_score,
            top_repos,
            all_recommendations,
        }
    }

    fn profile_completeness(&self, data: &GithubData) -> u8 {
        let u = &data.user;
        let mut score = 0u8;
        if u.name.is_some() { score += 20; }
        if u.bio.is_some() { score += 20; }
        if u.location.is_some() { score += 10; }
        // Follower ratio
        if u.followers > 0 {
            score += 20;
            let ratio = u.followers as f64 / (u.following.max(1)) as f64;
            if ratio >= 1.0 { score += 10; }
        }
        // Activity
        let active_days = data.contributions.iter().filter(|d| d.count > 0).count();
        if active_days > 50 { score += 20; }
        else if active_days > 20 { score += 10; }
        score.min(100)
    }
}
