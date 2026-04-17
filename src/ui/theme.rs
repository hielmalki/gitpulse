use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    pub fn toggle(self) -> Self {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }

    pub fn bg(self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(13, 17, 23),
            Theme::Light => Color::Rgb(255, 255, 255),
        }
    }

    pub fn fg(self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(201, 209, 217),
            Theme::Light => Color::Rgb(36, 41, 47),
        }
    }

    pub fn accent(self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(88, 166, 255),
            Theme::Light => Color::Rgb(3, 102, 214),
        }
    }

    pub fn highlight(self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(35, 134, 54),
            Theme::Light => Color::Rgb(40, 167, 69),
        }
    }

    pub fn warn(self) -> Color {
        Color::Rgb(255, 193, 7)
    }

    pub fn danger(self) -> Color {
        Color::Rgb(220, 53, 69)
    }

    pub fn muted(self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(110, 118, 129),
            Theme::Light => Color::Rgb(140, 149, 159),
        }
    }

    pub fn border(self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(48, 54, 61),
            Theme::Light => Color::Rgb(209, 217, 224),
        }
    }

    pub fn panel_bg(self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(22, 27, 34),
            Theme::Light => Color::Rgb(246, 248, 250),
        }
    }

    pub fn selected_bg(self) -> Color {
        match self {
            Theme::Dark => Color::Rgb(33, 38, 45),
            Theme::Light => Color::Rgb(225, 235, 245),
        }
    }

    pub fn base_style(self) -> Style {
        Style::default().fg(self.fg()).bg(self.bg())
    }

    pub fn title_style(self) -> Style {
        Style::default()
            .fg(self.accent())
            .add_modifier(Modifier::BOLD)
    }

    pub fn score_color(self, score: u8) -> Color {
        match score {
            90..=100 => self.highlight(),
            70..=89 => Color::Rgb(63, 185, 80),
            50..=69 => self.warn(),
            30..=49 => Color::Rgb(255, 140, 0),
            _ => self.danger(),
        }
    }

    pub fn heatmap_colors(self) -> [Color; 5] {
        match self {
            Theme::Dark => [
                Color::Rgb(22, 27, 34),
                Color::Rgb(14, 68, 41),
                Color::Rgb(0, 109, 50),
                Color::Rgb(38, 166, 65),
                Color::Rgb(63, 185, 80),
            ],
            Theme::Light => [
                Color::Rgb(235, 237, 240),
                Color::Rgb(155, 233, 168),
                Color::Rgb(64, 196, 99),
                Color::Rgb(48, 161, 78),
                Color::Rgb(33, 110, 57),
            ],
        }
    }
}
