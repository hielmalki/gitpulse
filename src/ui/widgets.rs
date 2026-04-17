use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Gauge, List, ListItem, Paragraph, Sparkline, Widget, Wrap,
    },
};

use crate::api::GithubData;
use crate::score::{ProfileScore, RepoScore, Severity};
use super::theme::Theme;

// ─── Profile Panel ──────────────────────────────────────────────────────────

pub struct ProfilePanel<'a> {
    pub data: &'a GithubData,
    pub score: &'a ProfileScore,
    pub theme: Theme,
}

impl<'a> Widget for ProfilePanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(Span::styled(" 👤 Profile ", self.theme.title_style()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border()))
            .style(Style::default().bg(self.theme.panel_bg()));
        let inner = block.inner(area);
        block.render(area, buf);

        let u = &self.data.user;
        let name = u.name.as_deref().unwrap_or(&u.login);
        let bio = u.bio.as_deref().unwrap_or("No bio");
        let location = u.location.as_deref().unwrap_or("—");

        let account_age_days = (chrono::Utc::now() - u.created_at).num_days();
        let account_years = account_age_days / 365;
        let account_months = (account_age_days % 365) / 30;

        let ratio = if u.following > 0 {
            format!("{:.1}", u.followers as f64 / u.following as f64)
        } else {
            "∞".to_string()
        };

        // ASCII avatar placeholder
        let avatar_lines = vec![
            "  ╭──────╮  ",
            "  │ (◕‿◕) │  ",
            "  ╰──────╯  ",
        ];

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(14), Constraint::Min(0)])
            .split(inner);

        // Avatar
        for (i, line) in avatar_lines.iter().enumerate() {
            let y = chunks[0].y + i as u16;
            if y < chunks[0].y + chunks[0].height {
                let para = Paragraph::new(*line)
                    .style(Style::default().fg(self.theme.accent()));
                para.render(Rect::new(chunks[0].x, y, chunks[0].width, 1), buf);
            }
        }

        // Stats
        let lines = vec![
            Line::from(vec![
                Span::styled(name, Style::default().fg(self.theme.accent()).add_modifier(Modifier::BOLD)),
                Span::styled(format!("  @{}", u.login), Style::default().fg(self.theme.muted())),
            ]),
            Line::from(Span::styled(bio, Style::default().fg(self.theme.fg()))),
            Line::from(Span::styled(format!("📍 {}", location), Style::default().fg(self.theme.muted()))),
            Line::from(Span::raw("")),
            Line::from(vec![
                Span::styled("⭐ Stars: ", Style::default().fg(self.theme.warn())),
                Span::styled(format!("{}", self.data.total_stars), Style::default().fg(self.theme.fg()).add_modifier(Modifier::BOLD)),
                Span::styled("  🍴 Forks: ", Style::default().fg(self.theme.muted())),
                Span::styled(format!("{}", self.data.total_forks), Style::default().fg(self.theme.fg())),
            ]),
            Line::from(vec![
                Span::styled("👥 Followers: ", Style::default().fg(self.theme.muted())),
                Span::styled(format!("{}", u.followers), Style::default().fg(self.theme.fg())),
                Span::styled("  Following: ", Style::default().fg(self.theme.muted())),
                Span::styled(format!("{}  Ratio: {}", u.following, ratio), Style::default().fg(self.theme.fg())),
            ]),
            Line::from(vec![
                Span::styled("📦 Repos: ", Style::default().fg(self.theme.muted())),
                Span::styled(format!("{}", u.public_repos), Style::default().fg(self.theme.fg())),
                Span::styled("  🗓 Age: ", Style::default().fg(self.theme.muted())),
                Span::styled(
                    format!("{}y {}m", account_years, account_months),
                    Style::default().fg(self.theme.fg()),
                ),
            ]),
        ];

        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .render(chunks[1], buf);
    }
}

// ─── Health Score Gauge ──────────────────────────────────────────────────────

pub struct ScorePanel<'a> {
    pub score: &'a ProfileScore,
    pub theme: Theme,
}

impl<'a> Widget for ScorePanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(Span::styled(" 🏥 Developer Health Score ", self.theme.title_style()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border()))
            .style(Style::default().bg(self.theme.panel_bg()));
        let inner = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Length(1), Constraint::Min(0)])
            .split(inner);

        let grade = match self.score.overall {
            90..=100 => "S", 80..=89 => "A", 70..=79 => "B",
            60..=69 => "C", 50..=59 => "D", _ => "F",
        };

        let score_color = self.theme.score_color(self.score.overall);

        let header = Line::from(vec![
            Span::styled(
                format!("{}/100", self.score.overall),
                Style::default().fg(score_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  Grade: {}  ", grade),
                Style::default().fg(self.theme.accent()).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("Avg repo: {}/100", self.score.avg_repo_score),
                Style::default().fg(self.theme.muted()),
            ),
        ]);
        Paragraph::new(header).render(chunks[0], buf);

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(score_color).bg(self.theme.border()))
            .percent(self.score.overall as u16)
            .label(format!("{}%", self.score.overall));
        gauge.render(chunks[1], buf);
    }
}

// ─── Repo List Panel ─────────────────────────────────────────────────────────

pub struct RepoListPanel<'a> {
    pub scores: &'a [RepoScore],
    pub selected: usize,
    pub theme: Theme,
}

impl<'a> Widget for RepoListPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(Span::styled(" 📦 Repositories (↑↓ navigate) ", self.theme.title_style()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border()))
            .style(Style::default().bg(self.theme.panel_bg()));
        let inner = block.inner(area);
        block.render(area, buf);

        let items: Vec<ListItem> = self.scores.iter().enumerate().map(|(i, rs)| {
            let color = self.theme.score_color(rs.score);
            let bar_filled = (rs.score as usize * 10 / 100).min(10);
            let bar = format!("{}{}", "█".repeat(bar_filled), "░".repeat(10 - bar_filled));

            let selected = i == self.selected;
            let style = if selected {
                Style::default().bg(self.theme.selected_bg()).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if selected { "▶ " } else { "  " };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(self.theme.accent())),
                Span::styled(
                    format!("{:<30}", truncate(&rs.repo_name, 28)),
                    style.fg(self.theme.fg()),
                ),
                Span::styled(bar, Style::default().fg(color)),
                Span::styled(
                    format!(" {:>3}/100", rs.score),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
            ]))
        }).collect();

        List::new(items)
            .style(Style::default().bg(self.theme.panel_bg()))
            .render(inner, buf);
    }
}

// ─── Repo Detail Panel ───────────────────────────────────────────────────────

pub struct RepoDetailPanel<'a> {
    pub repo_score: &'a RepoScore,
    pub theme: Theme,
    pub scroll: u16,
}

impl<'a> Widget for RepoDetailPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(Span::styled(
                format!(" 🔍 {} — {}/100 ", self.repo_score.repo_name, self.repo_score.score),
                self.theme.title_style(),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border()))
            .style(Style::default().bg(self.theme.panel_bg()));
        let inner = block.inner(area);
        block.render(area, buf);

        let mut lines = Vec::new();

        lines.push(Line::from(Span::styled("Score Breakdown:", Style::default()
            .fg(self.theme.accent()).add_modifier(Modifier::BOLD))));
        lines.push(Line::from(""));

        for item in &self.repo_score.breakdown {
            let icon = if item.passed { "✅" } else { "❌" };
            let color = if item.passed { self.theme.highlight() } else { self.theme.danger() };
            lines.push(Line::from(vec![
                Span::raw(format!("  {} ", icon)),
                Span::styled(format!("{:<28}", item.label), Style::default().fg(self.theme.fg())),
                Span::styled(
                    format!("{:>2}/{}", item.points, item.max_points),
                    Style::default().fg(color),
                ),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Recommendations:", Style::default()
            .fg(self.theme.accent()).add_modifier(Modifier::BOLD))));
        lines.push(Line::from(""));

        for rec in &self.repo_score.recommendations {
            let color = match rec.severity {
                Severity::Critical => self.theme.danger(),
                Severity::Warning => self.theme.warn(),
                Severity::Info => self.theme.accent(),
                Severity::Good => self.theme.highlight(),
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} {} ", rec.severity.icon(), rec.message),
                    Style::default().fg(color),
                ),
                Span::styled(
                    if rec.impact > 0 { format!("[+{}pts]", rec.impact) } else { String::new() },
                    Style::default().fg(self.theme.muted()),
                ),
            ]));
        }

        Paragraph::new(lines)
            .scroll((self.scroll, 0))
            .wrap(Wrap { trim: false })
            .render(inner, buf);
    }
}

// ─── Recommendations Panel ───────────────────────────────────────────────────

pub struct RecommendationsPanel<'a> {
    pub score: &'a ProfileScore,
    pub theme: Theme,
    pub scroll: u16,
}

impl<'a> Widget for RecommendationsPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(Span::styled(" 💡 Top Recommendations ", self.theme.title_style()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border()))
            .style(Style::default().bg(self.theme.panel_bg()));
        let inner = block.inner(area);
        block.render(area, buf);

        let items: Vec<ListItem> = self.score.all_recommendations.iter().take(50).map(|rec| {
            let color = match rec.severity {
                Severity::Critical => self.theme.danger(),
                Severity::Warning => self.theme.warn(),
                Severity::Info => self.theme.accent(),
                Severity::Good => self.theme.highlight(),
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  {} {} ", rec.severity.icon(), rec.message),
                    Style::default().fg(color),
                ),
                Span::styled(
                    if rec.impact > 0 { format!("+{}pts", rec.impact) } else { String::new() },
                    Style::default().fg(self.theme.muted()),
                ),
            ]))
        }).collect();

        List::new(items)
            .style(Style::default().bg(self.theme.panel_bg()))
            .render(inner, buf);
    }
}

// ─── Star Velocity Chart ─────────────────────────────────────────────────────

pub struct StarVelocityPanel<'a> {
    pub data: &'a GithubData,
    pub score: &'a ProfileScore,
    pub theme: Theme,
}

impl<'a> Widget for StarVelocityPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(Span::styled(" ⭐ Top Repositories by Stars ", self.theme.title_style()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border()))
            .style(Style::default().bg(self.theme.panel_bg()));
        let inner = block.inner(area);
        block.render(area, buf);

        let mut repos: Vec<_> = self.data.repos.iter()
            .filter(|r| !r.fork && r.stargazers_count > 0)
            .collect();
        repos.sort_by(|a, b| b.stargazers_count.cmp(&a.stargazers_count));
        repos.truncate(8);

        if repos.is_empty() {
            Paragraph::new("No starred repositories yet.")
                .style(Style::default().fg(self.theme.muted()))
                .render(inner, buf);
            return;
        }

        let max_stars = repos[0].stargazers_count.max(1);
        let bar_width = inner.width.saturating_sub(36) as usize;

        for (i, repo) in repos.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.y + inner.height {
                break;
            }
            let filled = (repo.stargazers_count as usize * bar_width / max_stars as usize).min(bar_width);
            let empty = bar_width - filled;
            let color = self.theme.score_color(((repo.stargazers_count * 100 / max_stars) as u8).min(100));

            let line = Line::from(vec![
                Span::styled(
                    format!("{:<22}", truncate(&repo.name, 20)),
                    Style::default().fg(self.theme.fg()),
                ),
                Span::styled("│", Style::default().fg(self.theme.border())),
                Span::styled("█".repeat(filled), Style::default().fg(color)),
                Span::styled("░".repeat(empty), Style::default().fg(self.theme.border())),
                Span::styled("│", Style::default().fg(self.theme.border())),
                Span::styled(
                    format!(" {:>5}⭐", repo.stargazers_count),
                    Style::default().fg(self.theme.warn()).add_modifier(Modifier::BOLD),
                ),
            ]);
            Paragraph::new(line).render(Rect::new(inner.x, y, inner.width, 1), buf);
        }
    }
}

// ─── Status Bar ──────────────────────────────────────────────────────────────

pub struct StatusBar {
    pub theme: Theme,
    pub tab: usize,
    pub username: String,
}

impl Widget for StatusBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let tabs = ["[1] Overview", "[2] Repos", "[3] Heatmap", "[4] Stars", "[5] Recs"];
        let mut spans = Vec::new();

        for (i, tab) in tabs.iter().enumerate() {
            let style = if i == self.tab {
                Style::default()
                    .fg(self.theme.bg())
                    .bg(self.theme.accent())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(self.theme.muted())
            };
            spans.push(Span::styled(*tab, style));
            spans.push(Span::raw(" "));
        }

        spans.push(Span::styled(
            format!("  @{}  ", self.username),
            Style::default().fg(self.theme.fg()),
        ));
        spans.push(Span::styled(
            " [t]heme  [r]efresh  [q]uit  [?]help ",
            Style::default().fg(self.theme.muted()),
        ));

        Paragraph::new(Line::from(spans))
            .style(Style::default().bg(self.theme.panel_bg()))
            .render(area, buf);
    }
}

// ─── Help Overlay ────────────────────────────────────────────────────────────

pub struct HelpOverlay {
    pub theme: Theme,
}

impl Widget for HelpOverlay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Center a 50×18 box
        let w = 54u16;
        let h = 20u16;
        let x = area.x + area.width.saturating_sub(w) / 2;
        let y = area.y + area.height.saturating_sub(h) / 2;
        let popup = Rect::new(x, y, w.min(area.width), h.min(area.height));

        let block = Block::default()
            .title(Span::styled(" ⌨  Keyboard Shortcuts ", Style::default()
                .fg(self.theme.accent()).add_modifier(Modifier::BOLD)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.accent()))
            .style(Style::default().bg(self.theme.panel_bg()));
        let inner = block.inner(popup);
        block.render(popup, buf);

        let shortcuts = vec![
            ("1-5", "Switch tabs"),
            ("↑/↓  k/j", "Navigate repos"),
            ("PgUp/PgDn", "Scroll detail"),
            ("t", "Toggle dark/light theme"),
            ("r", "Refresh data"),
            ("e", "Export JSON report"),
            ("m", "Export Markdown report"),
            ("b", "Show badge URL"),
            ("q / Esc", "Quit"),
            ("?", "Toggle this help"),
        ];

        for (i, (key, desc)) in shortcuts.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.y + inner.height {
                break;
            }
            let line = Line::from(vec![
                Span::styled(format!("  {:>12}  ", key), Style::default()
                    .fg(self.theme.accent()).add_modifier(Modifier::BOLD)),
                Span::styled(*desc, Style::default().fg(self.theme.fg())),
            ]);
            Paragraph::new(line).render(Rect::new(inner.x, y, inner.width, 1), buf);
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
