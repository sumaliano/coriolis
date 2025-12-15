//! Color themes for the UI.

use crate::app::Theme;
use ratatui::style::Color;

/// Theme color palette.
#[derive(Debug, Clone)]
pub struct ThemeColors {
    /// Background color.
    pub bg: Color,
    /// Primary text color.
    pub text: Color,
    /// Heading text color.
    pub heading: Color,
    /// Label text color.
    pub label: Color,
    /// Value text color.
    pub value: Color,
    /// Border color.
    pub border: Color,
    /// Cursor foreground color.
    pub cursor_fg: Color,
    /// Cursor background color.
    pub cursor_bg: Color,
    /// Status bar foreground color.
    pub status_fg: Color,
    /// Status bar background color.
    pub status_bg: Color,
    /// Warning color (reserved for future use).
    #[allow(dead_code)]
    pub warning: Color,
    /// Error color (reserved for future use).
    #[allow(dead_code)]
    pub error: Color,
}

impl ThemeColors {
    /// Create color palette from theme.
    pub fn from_theme(theme: &Theme) -> Self {
        match theme {
            Theme::GruvboxDark => Self {
                bg: Color::Rgb(40, 40, 40),
                text: Color::Rgb(235, 219, 178),
                heading: Color::Rgb(251, 184, 108),
                label: Color::Rgb(184, 187, 38),
                value: Color::Rgb(142, 192, 124),
                border: Color::Rgb(102, 92, 84),
                cursor_fg: Color::Rgb(40, 40, 40),
                cursor_bg: Color::Rgb(251, 184, 108),
                status_fg: Color::Rgb(235, 219, 178),
                status_bg: Color::Rgb(60, 56, 54),
                warning: Color::Rgb(250, 189, 47),
                error: Color::Rgb(251, 73, 52),
            },
            Theme::GruvboxLight => Self {
                bg: Color::Rgb(251, 245, 234),
                text: Color::Rgb(60, 56, 54),
                heading: Color::Rgb(175, 58, 3),
                label: Color::Rgb(121, 116, 14),
                value: Color::Rgb(102, 123, 3),
                border: Color::Rgb(213, 196, 161),
                cursor_fg: Color::Rgb(251, 245, 234),
                cursor_bg: Color::Rgb(175, 58, 3),
                status_fg: Color::Rgb(60, 56, 54),
                status_bg: Color::Rgb(235, 219, 178),
                warning: Color::Rgb(181, 118, 20),
                error: Color::Rgb(157, 0, 6),
            },
        }
    }
}
