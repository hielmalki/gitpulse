use gitpulse::api::{Repository, LicenseInfo};
use gitpulse::score::Scorer;
use chrono::Utc;

fn make_repo(name: &str) -> Repository {
    Repository {
        id: 1,
        name: name.to_string(),
        full_name: format!("user/{}", name),
        description: None,
        html_url: String::new(),
        language: None,
        stargazers_count: 0,
        forks_count: 0,
        open_issues_count: 0,
        topics: vec![],
        has_wiki: false,
        has_issues: true,
        fork: false,
        archived: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        pushed_at: Some(Utc::now()),
        license: None,
        has_readme: false,
        has_gitignore: false,
        has_ci: false,
        has_tests: false,
        has_contributing: false,
        has_code_of_conduct: false,
        closed_issues_count: 0,
    }
}

#[test]
fn empty_repo_scores_zero() {
    let repo = make_repo("empty");
    let scorer = Scorer::new();
    let result = scorer.score_repo(&repo);
    assert_eq!(result.score, 0, "empty repo should score 0");
}

#[test]
fn perfect_repo_scores_high() {
    let mut repo = make_repo("perfect");
    repo.has_readme = true;
    repo.license = Some(LicenseInfo { key: "mit".into(), name: "MIT".into() });
    repo.has_gitignore = true;
    repo.has_ci = true;
    repo.description = Some("Great project".into());
    repo.topics = vec!["rust".into(), "cli".into()];
    repo.open_issues_count = 1;
    repo.closed_issues_count = 10;
    repo.pushed_at = Some(Utc::now());
    repo.has_tests = true;
    repo.has_contributing = true;
    repo.has_code_of_conduct = true;
    repo.stargazers_count = 5;

    let scorer = Scorer::new();
    let result = scorer.score_repo(&repo);
    assert!(result.score >= 95, "perfect repo should score ≥95, got {}", result.score);
}

#[test]
fn readme_adds_15_points() {
    let mut base = make_repo("test");
    let scorer = Scorer::new();
    let base_score = scorer.score_repo(&base).score;
    base.has_readme = true;
    let with_readme = scorer.score_repo(&base).score;
    assert_eq!(with_readme - base_score, 15);
}

#[test]
fn license_adds_10_points() {
    let mut base = make_repo("test");
    let scorer = Scorer::new();
    let base_score = scorer.score_repo(&base).score;
    base.license = Some(LicenseInfo { key: "mit".into(), name: "MIT".into() });
    let with_license = scorer.score_repo(&base).score;
    assert_eq!(with_license - base_score, 10);
}

#[test]
fn ci_adds_15_points() {
    let mut base = make_repo("test");
    let scorer = Scorer::new();
    let base_score = scorer.score_repo(&base).score;
    base.has_ci = true;
    let with_ci = scorer.score_repo(&base).score;
    assert_eq!(with_ci - base_score, 15);
}

#[test]
fn stale_repo_gets_zero_recency_points() {
    let mut repo = make_repo("stale");
    repo.pushed_at = Some(Utc::now() - chrono::Duration::days(200));
    let scorer = Scorer::new();
    let result = scorer.score_repo(&repo);
    let recency = result.breakdown.iter().find(|b| b.label.contains("active") || b.label.contains("30") || b.label.contains("90"));
    if let Some(r) = recency {
        assert_eq!(r.points, 0, "stale repo should get 0 recency points");
    }
}

#[test]
fn recommendations_sorted_by_impact() {
    let repo = make_repo("test");
    let scorer = Scorer::new();
    let result = scorer.score_repo(&repo);
    let impacts: Vec<i32> = result.recommendations.iter()
        .filter(|r| r.impact > 0)
        .map(|r| r.impact)
        .collect();
    let mut sorted = impacts.clone();
    sorted.sort_by(|a, b| b.cmp(a));
    assert_eq!(impacts, sorted, "recommendations must be sorted by impact descending");
}
