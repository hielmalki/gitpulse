use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    Terminal,
};

use crate::api::{GithubClient, GithubData};
use crate::score::{ProfileScore, Scorer};
use crate::export;

use super::theme::Theme;
use super::widgets::*;
use super::heatmap::HeatmapWidget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Overview = 0,
    Repos = 1,
    Heatmap = 2,
    Stars = 3,
    Recommendations = 4,
}

impl Tab {
    fn from_index(i: usize) -> Self {
        match i {
            0 => Tab::Overview,
            1 => Tab::Repos,
            2 => Tab::Heatmap,
            3 => Tab::Stars,
            4 => Tab::Recommendations,
            _ => Tab::Overview,
        }
    }
}

enum AppState {
    Loading(String),
    Ready { data: Box<GithubData>, score: Box<ProfileScore> },
    Error(String),
}

struct App {
    username: String,
    token: Option<String>,
    state: AppState,
    theme: Theme,
    tab: Tab,
    repo_selected: usize,
    detail_scroll: u16,
    show_help: bool,
    status_message: Option<(String, Instant)>,
}

impl App {
    fn new(username: String, token: Option<String>) -> Self {
        Self {
            username,
            token,
            state: AppState::Loading("Fetching GitHub data...".to_string()),
            theme: Theme::Dark,
            tab: Tab::Overview,
            repo_selected: 0,
            detail_scroll: 0,
            show_help: false,
            status_message: None,
        }
    }

    fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), Instant::now()));
    }

    fn repo_count(&self) -> usize {
        match &self.state {
            AppState::Ready { score, .. } => score.repo_scores.len(),
            _ => 0,
        }
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        // quit
        if matches!(code, KeyCode::Char('q') | KeyCode::Esc) {
            return true;
        }
        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
            return true;
        }

        match code {
            KeyCode::Char('?') => self.show_help = !self.show_help,
            KeyCode::Char('t') => self.theme = self.theme.toggle(),
            KeyCode::Char('1') => { self.tab = Tab::Overview; }
            KeyCode::Char('2') => { self.tab = Tab::Repos; }
            KeyCode::Char('3') => { self.tab = Tab::Heatmap; }
            KeyCode::Char('4') => { self.tab = Tab::Stars; }
            KeyCode::Char('5') => { self.tab = Tab::Recommendations; }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.repo_selected > 0 {
                    self.repo_selected -= 1;
                    self.detail_scroll = 0;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = self.repo_count().saturating_sub(1);
                if self.repo_selected < max {
                    self.repo_selected += 1;
                    self.detail_scroll = 0;
                }
            }
            KeyCode::PageDown => self.detail_scroll = self.detail_scroll.saturating_add(3),
            KeyCode::PageUp => self.detail_scroll = self.detail_scroll.saturating_sub(3),
            _ => {}
        }
        false
    }
}

pub async fn run_dashboard(username: &str, token: Option<String>) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_inner(&mut terminal, username, token).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    result
}

async fn run_inner(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    username: &str,
    token: Option<String>,
) -> Result<()> {
    let mut app = App::new(username.to_string(), token.clone());

    // Draw loading screen immediately
    terminal.draw(|f| draw(f, &app))?;

    // Fetch data
    let client = GithubClient::new(token);
    match client.fetch_all(username).await {
        Ok(data) => {
            let scorer = Scorer::new();
            let score = scorer.compute(&data);
            app.state = AppState::Ready {
                data: Box::new(data),
                score: Box::new(score),
            };
        }
        Err(e) => {
            app.state = AppState::Error(format!("{:#}", e));
        }
    }

    loop {
        terminal.draw(|f| draw(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                let quit = app.handle_key(key.code, key.modifiers);
                if quit {
                    break;
                }

                // Handle export/refresh actions that need async
                match key.code {
                    KeyCode::Char('r') => {
                        app.state = AppState::Loading("Refreshing...".to_string());
                        terminal.draw(|f| draw(f, &app))?;
                        let client2 = GithubClient::new(app.token.clone());
                        match client2.fetch_all(&app.username).await {
                            Ok(data) => {
                                let scorer = Scorer::new();
                                let score = scorer.compute(&data);
                                app.state = AppState::Ready {
                                    data: Box::new(data),
                                    score: Box::new(score),
                                };
                                app.set_status("Data refreshed successfully");
                            }
                            Err(e) => {
                                app.state = AppState::Error(format!("{:#}", e));
                            }
                        }
                    }
                    KeyCode::Char('e') => {
                        if let AppState::Ready { data, score } = &app.state {
                            match export_inline(data, score, "json") {
                                Ok(path) => app.set_status(format!("Exported to {}", path)),
                                Err(e) => app.set_status(format!("Export failed: {}", e)),
                            }
                        }
                    }
                    KeyCode::Char('m') => {
                        if let AppState::Ready { data, score } = &app.state {
                            match export_inline(data, score, "md") {
                                Ok(path) => app.set_status(format!("Exported to {}", path)),
                                Err(e) => app.set_status(format!("Export failed: {}", e)),
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Clear expired status messages
        if let Some((_, t)) = &app.status_message {
            if t.elapsed() > Duration::from_secs(4) {
                app.status_message = None;
            }
        }
    }

    Ok(())
}

fn draw(frame: &mut ratatui::Frame, app: &App) {
    let area = frame.area();

    frame.render_widget(
        ratatui::widgets::Block::default()
            .style(ratatui::style::Style::default().bg(app.theme.bg())),
        area,
    );

    // Layout: status bar (1) + content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let content_area = chunks[0];
    let status_area = chunks[1];

    // Status bar
    let status_text = if let Some((msg, _)) = &app.status_message {
        msg.clone()
    } else {
        String::new()
    };

    frame.render_widget(
        StatusBar {
            theme: app.theme,
            tab: app.tab as usize,
            username: app.username.clone(),
        },
        status_area,
    );

    match &app.state {
        AppState::Loading(msg) => {
            draw_loading(frame, content_area, app.theme, msg);
        }
        AppState::Error(err) => {
            draw_error(frame, content_area, app.theme, err);
        }
        AppState::Ready { data, score } => {
            match app.tab {
                Tab::Overview => draw_overview(frame, content_area, app, data, score),
                Tab::Repos => draw_repos(frame, content_area, app, score),
                Tab::Heatmap => draw_heatmap(frame, content_area, app, data),
                Tab::Stars => draw_stars(frame, content_area, app, data, score),
                Tab::Recommendations => draw_recommendations(frame, content_area, app, score),
            }
        }
    }

    if app.show_help {
        frame.render_widget(HelpOverlay { theme: app.theme }, area);
    }
}

fn draw_loading(
    frame: &mut ratatui::Frame,
    area: Rect,
    theme: Theme,
    msg: &str,
) {
    use ratatui::widgets::{Block, Borders, Paragraph};
    use ratatui::text::{Line, Span};
    use ratatui::style::{Style, Modifier};

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent()))
        .style(Style::default().bg(theme.panel_bg()));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let spinner = ["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"];
    let idx = (Instant::now().elapsed().subsec_millis() / 100) as usize % spinner.len();

    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            r"   _____ _ _   ____        _          ",
            Style::default().fg(theme.accent()).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            r"  / ____(_) |_|  _ \ _   _| |___  ___ ",
            Style::default().fg(theme.accent()).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            r" | |  __| | __| |_) | | | | / __|/ _ \",
            Style::default().fg(theme.accent()).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            r" | |_| | | |_|  __/| |_| | \__ \  __/",
            Style::default().fg(theme.accent()).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            r"  \_____|_|\__|_|    \__,_|_|___/\___|",
            Style::default().fg(theme.accent()).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "   GitHub Profile Analytics — Developer Health Score",
            Style::default().fg(theme.muted()),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("   {} {}...", spinner[idx], msg),
            Style::default().fg(theme.warn()),
        )),
    ];

    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(theme.panel_bg())),
        inner,
    );
}

fn draw_error(frame: &mut ratatui::Frame, area: Rect, theme: Theme, err: &str) {
    use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
    use ratatui::text::{Line, Span};

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("❌ Error fetching GitHub data:", ratatui::style::Style::default()
                .fg(theme.danger()).add_modifier(ratatui::style::Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled(err, ratatui::style::Style::default().fg(theme.fg()))),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'r' to retry or 'q' to quit. Make sure GITHUB_TOKEN is set for higher rate limits.",
                ratatui::style::Style::default().fg(theme.muted()),
            )),
        ])
        .block(Block::default().borders(Borders::ALL)
            .border_style(ratatui::style::Style::default().fg(theme.danger())))
        .wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_overview(
    frame: &mut ratatui::Frame,
    area: Rect,
    app: &App,
    data: &GithubData,
    score: &ProfileScore,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(area);

    frame.render_widget(ProfilePanel { data, score, theme: app.theme }, chunks[0]);
    frame.render_widget(ScorePanel { score, theme: app.theme }, chunks[1]);

    // Bottom: top recommendations preview
    frame.render_widget(
        RecommendationsPanel { score, theme: app.theme, scroll: 0 },
        chunks[2],
    );
}

fn draw_repos(
    frame: &mut ratatui::Frame,
    area: Rect,
    app: &App,
    score: &ProfileScore,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    frame.render_widget(
        RepoListPanel {
            scores: &score.repo_scores,
            selected: app.repo_selected,
            theme: app.theme,
        },
        chunks[0],
    );

    if let Some(rs) = score.repo_scores.get(app.repo_selected) {
        frame.render_widget(
            RepoDetailPanel {
                repo_score: rs,
                theme: app.theme,
                scroll: app.detail_scroll,
            },
            chunks[1],
        );
    }
}

fn draw_heatmap(
    frame: &mut ratatui::Frame,
    area: Rect,
    app: &App,
    data: &GithubData,
) {
    use ratatui::widgets::{Block, Borders, Paragraph};
    use ratatui::text::{Line, Span};
    use ratatui::style::Style;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(14), Constraint::Min(0)])
        .split(area);

    let block = Block::default()
        .title(Span::styled(" 📅 Contribution Heatmap (last 52 weeks) ", app.theme.title_style()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border()))
        .style(Style::default().bg(app.theme.panel_bg()));

    frame.render_widget(
        HeatmapWidget::new(&data.contributions, app.theme).block(block),
        chunks[0],
    );

    // Stats below heatmap
    let active = data.contributions.iter().filter(|d| d.count > 0).count();
    let total_events: u32 = data.contributions.iter().map(|d| d.count).sum();
    let max_day = data.contributions.iter().max_by_key(|d| d.count)
        .map(|d| format!("{} ({} events)", d.date, d.count))
        .unwrap_or_default();

    let stats = vec![
        Line::from(vec![
            Span::styled("  Active days: ", Style::default().fg(app.theme.muted())),
            Span::styled(format!("{}/364", active), Style::default().fg(app.theme.highlight())),
            Span::styled("   Total events: ", Style::default().fg(app.theme.muted())),
            Span::styled(format!("{}", total_events), Style::default().fg(app.theme.accent())),
            Span::styled("   Best day: ", Style::default().fg(app.theme.muted())),
            Span::styled(max_day, Style::default().fg(app.theme.warn())),
        ]),
    ];

    frame.render_widget(
        Paragraph::new(stats)
            .block(Block::default().borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border()))
                .style(Style::default().bg(app.theme.panel_bg()))),
        chunks[1],
    );
}

fn draw_stars(
    frame: &mut ratatui::Frame,
    area: Rect,
    app: &App,
    data: &GithubData,
    score: &ProfileScore,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    frame.render_widget(
        StarVelocityPanel { data, score, theme: app.theme },
        chunks[0],
    );

    // Language distribution
    draw_language_breakdown(frame, chunks[1], app, data);
}

fn draw_language_breakdown(
    frame: &mut ratatui::Frame,
    area: Rect,
    app: &App,
    data: &GithubData,
) {
    use ratatui::widgets::{Block, Borders, Paragraph};
    use ratatui::text::{Line, Span};
    use ratatui::style::Style;
    use std::collections::HashMap;

    let mut langs: HashMap<&str, u32> = HashMap::new();
    for repo in &data.repos {
        if !repo.fork {
            if let Some(lang) = &repo.language {
                *langs.entry(lang.as_str()).or_insert(0) += 1;
            }
        }
    }
    let mut sorted: Vec<_> = langs.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    let total: u32 = sorted.iter().map(|(_, &c)| c).sum();
    let max_w = area.width.saturating_sub(30) as usize;

    let lang_colors = [
        ratatui::style::Color::Rgb(241, 224, 90),  // yellow
        ratatui::style::Color::Rgb(88, 166, 255),   // blue
        ratatui::style::Color::Rgb(255, 123, 114),  // red
        ratatui::style::Color::Rgb(63, 185, 80),    // green
        ratatui::style::Color::Rgb(188, 140, 255),  // purple
        ratatui::style::Color::Rgb(255, 193, 7),    // orange
        ratatui::style::Color::Rgb(121, 192, 255),  // light blue
        ratatui::style::Color::Rgb(255, 135, 0),    // amber
    ];

    let lines: Vec<Line> = sorted.iter().take(8).enumerate().map(|(i, (lang, &count))| {
        let pct = count * 100 / total.max(1);
        let filled = (pct as usize * max_w / 100).max(1);
        let color = lang_colors[i % lang_colors.len()];
        Line::from(vec![
            Span::styled(format!("  {:<14}", lang), Style::default().fg(app.theme.fg())),
            Span::styled("│", Style::default().fg(app.theme.border())),
            Span::styled("█".repeat(filled.min(max_w)), Style::default().fg(color)),
            Span::styled("░".repeat(max_w.saturating_sub(filled)), Style::default().fg(app.theme.border())),
            Span::styled("│", Style::default().fg(app.theme.border())),
            Span::styled(format!(" {:>3}%  {} repos", pct, count), Style::default().fg(app.theme.muted())),
        ])
    }).collect();

    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default()
                .title(Span::styled(" 🌐 Language Distribution ", app.theme.title_style()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border()))
                .style(Style::default().bg(app.theme.panel_bg()))),
        area,
    );
}

fn draw_recommendations(
    frame: &mut ratatui::Frame,
    area: Rect,
    app: &App,
    score: &ProfileScore,
) {
    frame.render_widget(
        RecommendationsPanel { score, theme: app.theme, scroll: 0 },
        area,
    );
}

fn export_inline(
    data: &GithubData,
    score: &ProfileScore,
    format: &str,
) -> Result<String> {
    let filename = format!(
        "gitpulse-{}-{}.{}",
        data.user.login,
        chrono::Utc::now().format("%Y%m%d-%H%M%S"),
        format
    );

    let content = match format {
        "json" => {
            let combined = serde_json::json!({
                "generated_at": data.fetched_at,
                "profile": {
                    "username": data.user.login,
                    "name": data.user.name,
                    "followers": data.user.followers,
                    "total_stars": data.total_stars,
                },
                "health_score": score.overall,
                "avg_repo_score": score.avg_repo_score,
                "repo_scores": score.repo_scores,
                "recommendations": score.all_recommendations,
            });
            serde_json::to_string_pretty(&combined)?
        }
        "md" => format_markdown_report(data, score),
        _ => anyhow::bail!("Unknown format"),
    };

    std::fs::write(&filename, content)?;
    Ok(filename)
}

fn format_markdown_report(data: &GithubData, score: &ProfileScore) -> String {
    format!(
        "# GitPulse Report — @{}\n\n**Health Score: {}/100**\n\nGenerated: {}\n",
        data.user.login,
        score.overall,
        data.fetched_at.format("%Y-%m-%d %H:%M UTC"),
    )
}
