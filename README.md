# GitPulse

> Interactive Terminal-UI dashboard for GitHub profile analysis with **Developer Health Score**

![GitPulse Score](https://img.shields.io/badge/GitPulse%20Score-87%2F100-green?style=flat-square&logo=github)
[![CI](https://github.com/yourusername/gitpulse/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/gitpulse/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

GitPulse analyzes your GitHub profile and repositories, calculates a **Health Score (0-100)** for each repo, and gives you **concrete, actionable improvements** — all in a beautiful terminal dashboard.

```
╭─────────────────────────────────────────────────────────────────────────╮
│  👤 Profile                                                              │
│  ╭──────╮   John Doe  @johndoe                                          │
│  │(◕‿◕)│   Building open-source tools                                  │
│  ╰──────╯   📍 Berlin, Germany                                          │
│             ⭐ Stars: 1,247   🍴 Forks: 89                              │
│             👥 Followers: 342  Following: 12  Ratio: 28.5               │
╰─────────────────────────────────────────────────────────────────────────╯
╭─────────────────────────────────────────────────────────────────────────╮
│  🏥 Developer Health Score                                               │
│  87/100  Grade: A   Avg repo: 74/100                                    │
│  [████████████████░░░] 87%                                              │
╰─────────────────────────────────────────────────────────────────────────╯
```

## Features

| Feature | Description |
|---------|-------------|
| **Profile Overview** | Username, bio, location, follower ratio, account age, total stars/forks |
| **Health Score** | Per-repo score 0-100 based on 11 weighted checks |
| **Contribution Heatmap** | 52-week GitHub-style heatmap with Unicode block chars |
| **Star Charts** | Top repos by stars with horizontal bar charts |
| **Language Stats** | Language distribution across all your repos |
| **Recommendations** | Sorted by impact — highest ROI improvements first |
| **Export** | JSON report, Markdown report, shields.io Badge URL |
| **Dark/Light Theme** | Toggle with `t` |

## Installation

### From source (requires Rust 1.75+)

```bash
git clone https://github.com/yourusername/gitpulse
cd gitpulse
cargo install --path .
```

### Pre-built binaries

Download from [Releases](https://github.com/yourusername/gitpulse/releases).

## Usage

```bash
# Launch interactive TUI
gitpulse dashboard <USERNAME>

# Quick stats (no TUI)
gitpulse stats <USERNAME>

# Export reports
gitpulse export <USERNAME> --format json --output report.json
gitpulse export <USERNAME> --format md   --output report.md

# Generate badge URL
gitpulse badge <USERNAME>
```

### Authentication

Without a token you get 60 requests/hour. With a token: 5,000/hour.

```bash
# Via environment variable (recommended)
export GITHUB_TOKEN=ghp_your_token_here
gitpulse dashboard <USERNAME>

# Via flag
gitpulse -t ghp_your_token_here dashboard <USERNAME>
```

Create a token at [github.com/settings/tokens](https://github.com/settings/tokens) — read-only public access is sufficient.

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `1` – `5` | Switch tabs (Overview, Repos, Heatmap, Stars, Recommendations) |
| `↑` / `↓` or `k` / `j` | Navigate repository list |
| `PgUp` / `PgDn` | Scroll repo detail panel |
| `t` | Toggle dark / light theme |
| `r` | Refresh data from GitHub |
| `e` | Export JSON report to current directory |
| `m` | Export Markdown report to current directory |
| `b` | Print badge URL |
| `?` | Toggle help overlay |
| `q` / `Esc` | Quit |

## Health Score Breakdown

Each repository is scored out of **100 points**:

| Check | Points | Why |
|-------|--------|-----|
| README.md exists | 15 | First thing visitors see |
| LICENSE file | 10 | Required for OSS adoption |
| CI/CD (GitHub Actions) | 15 | Quality signal |
| Tests directory | 10 | Reliability signal |
| Description + Topics | 10 | Discoverability |
| Issue resolution ≥50% | 10 | Community responsiveness |
| Last commit < 30 days | 10 | Active maintenance |
| Last commit < 90 days | 5 | Recent maintenance |
| .gitignore | 5 | Best practice |
| CONTRIBUTING.md | 5 | Community health |
| CODE_OF_CONDUCT.md | 5 | Community health |
| Has at least 1 star | 5 | Community interest |

**Overall Score** = 60% average repo score + 40% profile completeness

## Export Examples

### JSON
```bash
gitpulse export octocat --format json
```

### Markdown Badge in your README
```markdown
![GitPulse Score](https://img.shields.io/badge/GitPulse%20Score-87%2F100-green?style=flat-square&logo=github)
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). PRs welcome!

## License

MIT — see [LICENSE](LICENSE).
