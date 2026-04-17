use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Widget},
};

use crate::api::ContributionDay;
use super::theme::Theme;

pub struct HeatmapWidget<'a> {
    pub contributions: &'a [ContributionDay],
    pub theme: Theme,
    pub block: Option<Block<'a>>,
}

impl<'a> HeatmapWidget<'a> {
    pub fn new(contributions: &'a [ContributionDay], theme: Theme) -> Self {
        Self { contributions, theme, block: None }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    fn intensity(count: u32) -> usize {
        match count {
            0 => 0,
            1..=2 => 1,
            3..=5 => 2,
            6..=9 => 3,
            _ => 4,
        }
    }
}

impl<'a> Widget for HeatmapWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = match &self.block {
            Some(b) => {
                let inner = b.inner(area);
                b.clone().render(area, buf);
                inner
            }
            None => area,
        };

        if inner.height < 9 || inner.width < 53 {
            return;
        }

        let colors = self.theme.heatmap_colors();
        let blocks = ['░', '▒', '▓', '▓', '█'];

        // Day labels on the left (Mon, Wed, Fri)
        let day_labels = ["   ", "Mon", "   ", "Wed", "   ", "Fri", "   "];

        // We display 52 weeks × 7 days
        let days: Vec<&ContributionDay> = self.contributions.iter().rev().take(364).rev().collect();
        // Pad to exactly 364 items
        let weeks = 52usize;

        // Month labels row
        // Calculate which columns are month boundaries
        let mut month_label_row = vec![' '; weeks * 2 + 3];
        {
            let today_offset = days.len() as i64 - 1;
            let mut last_month = 99u32;
            let month_names = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
            for (i, day) in days.iter().enumerate() {
                if let Ok(d) = chrono::NaiveDate::parse_from_str(&day.date, "%Y-%m-%d") {
                    let m = d.month0() as usize;
                    let week_col = (i / 7) * 2 + 3;
                    if d.month() != last_month && week_col + 3 < month_label_row.len() {
                        let name = month_names[m];
                        for (ci, ch) in name.chars().enumerate() {
                            if week_col + ci < month_label_row.len() {
                                month_label_row[week_col + ci] = ch;
                            }
                        }
                        last_month = d.month();
                    }
                }
                let _ = today_offset;
            }
        }
        let month_str: String = month_label_row.iter().collect();

        // Render month labels
        if inner.y < buf.area.bottom() {
            let x = inner.x;
            let y = inner.y;
            let style = Style::default().fg(self.theme.muted());
            for (i, ch) in month_str.chars().enumerate() {
                let cx = x + i as u16;
                if cx < inner.x + inner.width {
                    buf[(cx, y)].set_char(ch).set_style(style);
                }
            }
        }

        // Render day rows
        for row in 0..7usize {
            let y = inner.y + 1 + row as u16;
            if y >= inner.y + inner.height {
                break;
            }
            // Day label
            let label = day_labels[row];
            let label_style = Style::default().fg(self.theme.muted());
            for (i, ch) in label.chars().enumerate() {
                let cx = inner.x + i as u16;
                if cx < inner.x + inner.width {
                    buf[(cx, y)].set_char(ch).set_style(label_style);
                }
            }

            // Week columns
            for week in 0..weeks {
                let idx = week * 7 + row;
                let count = days.get(idx).map(|d| d.count).unwrap_or(0);
                let intensity = Self::intensity(count);
                let color = colors[intensity];
                let ch = blocks[intensity];

                let cx = inner.x + 3 + week as u16 * 2;
                if cx < inner.x + inner.width {
                    buf[(cx, y)]
                        .set_char(ch)
                        .set_style(Style::default().fg(color).bg(self.theme.panel_bg()));
                }
            }
        }

        // Legend
        let legend_y = inner.y + 9;
        if legend_y < inner.y + inner.height {
            let legend_x = inner.x + 3;
            let label = "Less ";
            for (i, ch) in label.chars().enumerate() {
                buf[(legend_x + i as u16, legend_y)]
                    .set_char(ch)
                    .set_style(Style::default().fg(self.theme.muted()));
            }
            for (i, (&color, &ch)) in colors.iter().zip(blocks.iter()).enumerate() {
                let cx = legend_x + label.len() as u16 + i as u16 * 2;
                if cx < inner.x + inner.width {
                    buf[(cx, legend_y)]
                        .set_char(ch)
                        .set_style(Style::default().fg(color).bg(self.theme.panel_bg()));
                }
            }
            let more_label = " More";
            let cx = legend_x + label.len() as u16 + colors.len() as u16 * 2;
            for (i, ch) in more_label.chars().enumerate() {
                let x = cx + i as u16;
                if x < inner.x + inner.width {
                    buf[(x, legend_y)]
                        .set_char(ch)
                        .set_style(Style::default().fg(self.theme.muted()));
                }
            }
        }
    }
}
