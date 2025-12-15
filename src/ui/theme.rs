use crate::app::Theme;
use ratatui::style::Color;

pub struct ThemeColors {
    pub bg: Color,
    pub text: Color,
    pub heading: Color,
    pub label: Color,
    pub value: Color,
    pub border: Color,
    pub cursor_fg: Color,
    pub cursor_bg: Color,
    pub status_fg: Color,
    pub status_bg: Color,
    pub warning: Color,
    pub error: Color,
}

impl ThemeColors {
    pub fn from_theme(theme: &Theme) -> Self {
        match theme {
            Theme::GruvboxDark => Self::gruvbox_dark(),
            Theme::GruvboxLight => Self::gruvbox_light(),
        }
    }

    fn gruvbox_dark() -> Self {
        Self {
            bg: Color::Rgb(40, 40, 40),
            text: Color::Rgb(235, 219, 178),
            heading: Color::Rgb(250, 189, 47),
            label: Color::Rgb(152, 151, 26),
            value: Color::Rgb(184, 187, 38),
            border: Color::Rgb(124, 111, 100),
            cursor_fg: Color::Rgb(40, 40, 40),
            cursor_bg: Color::Rgb(250, 189, 47),
            status_fg: Color::Rgb(235, 219, 178),
            status_bg: Color::Rgb(60, 56, 54),
            warning: Color::Rgb(254, 128, 25),
            error: Color::Rgb(251, 73, 52),
        }
    }

    fn gruvbox_light() -> Self {
        Self {
            bg: Color::Rgb(251, 241, 199),
            text: Color::Rgb(60, 56, 54),
            heading: Color::Rgb(181, 118, 20),
            label: Color::Rgb(121, 116, 14),
            value: Color::Rgb(102, 92, 84),
            border: Color::Rgb(189, 174, 147),
            cursor_fg: Color::Rgb(251, 241, 199),
            cursor_bg: Color::Rgb(181, 118, 20),
            status_fg: Color::Rgb(60, 56, 54),
            status_bg: Color::Rgb(235, 219, 178),
            warning: Color::Rgb(175, 58, 3),
            error: Color::Rgb(157, 0, 6),
        }
    }
}
