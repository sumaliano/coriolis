//! Color themes for the UI.
//!
//! Gruvbox color palette based on the official specification:
//! <https://github.com/morhetz/gruvbox>

use crate::app::Theme;
use ratatui::style::Color;

/// Gruvbox dark color palette.
mod gruvbox_dark {
    use super::Color;

    // Background colors
    pub(super) const DARK0: Color = Color::Rgb(0x28, 0x28, 0x28); // #282828
    pub(super) const DARK1: Color = Color::Rgb(0x3c, 0x38, 0x36); // #3c3836
    #[allow(dead_code)]
    pub(super) const DARK2: Color = Color::Rgb(0x50, 0x49, 0x45); // #504945
    pub(super) const DARK3: Color = Color::Rgb(0x66, 0x5c, 0x54); // #665c54

    // Foreground colors
    #[allow(dead_code)]
    pub(super) const LIGHT0: Color = Color::Rgb(0xfb, 0xf1, 0xc7); // #fbf1c7
    pub(super) const LIGHT1: Color = Color::Rgb(0xeb, 0xdb, 0xb2); // #ebdbb2
    #[allow(dead_code)]
    pub(super) const LIGHT2: Color = Color::Rgb(0xd5, 0xc4, 0xa1); // #d5c4a1
    #[allow(dead_code)]
    pub(super) const LIGHT3: Color = Color::Rgb(0xbd, 0xae, 0x93); // #bdae93
    #[allow(dead_code)]
    pub(super) const LIGHT4: Color = Color::Rgb(0xa8, 0x99, 0x84); // #a89984

    // Accent colors (bright variants)
    pub(super) const BRIGHT_RED: Color = Color::Rgb(0xfb, 0x49, 0x34); // #fb4934
    pub(super) const BRIGHT_GREEN: Color = Color::Rgb(0xb8, 0xbb, 0x26); // #b8bb26
    pub(super) const BRIGHT_YELLOW: Color = Color::Rgb(0xfa, 0xbd, 0x2f); // #fabd2f
    #[allow(dead_code)]
    pub(super) const BRIGHT_BLUE: Color = Color::Rgb(0x83, 0xa5, 0x98); // #83a598
    #[allow(dead_code)]
    pub(super) const BRIGHT_PURPLE: Color = Color::Rgb(0xd3, 0x86, 0x9b); // #d3869b
    pub(super) const BRIGHT_AQUA: Color = Color::Rgb(0x8e, 0xc0, 0x7c); // #8ec07c
    #[allow(dead_code)]
    pub(super) const BRIGHT_ORANGE: Color = Color::Rgb(0xfe, 0x80, 0x19); // #fe8019

    // Neutral colors
    #[allow(dead_code)]
    pub(super) const NEUTRAL_RED: Color = Color::Rgb(0xcc, 0x24, 0x1d); // #cc241d
    #[allow(dead_code)]
    pub(super) const NEUTRAL_GREEN: Color = Color::Rgb(0x98, 0x97, 0x1a); // #98971a
    #[allow(dead_code)]
    pub(super) const NEUTRAL_YELLOW: Color = Color::Rgb(0xd7, 0x99, 0x21); // #d79921
    #[allow(dead_code)]
    pub(super) const NEUTRAL_BLUE: Color = Color::Rgb(0x45, 0x85, 0x88); // #458588
    #[allow(dead_code)]
    pub(super) const NEUTRAL_PURPLE: Color = Color::Rgb(0xb1, 0x62, 0x86); // #b16286
    #[allow(dead_code)]
    pub(super) const NEUTRAL_AQUA: Color = Color::Rgb(0x68, 0x9d, 0x6a); // #689d6a
    #[allow(dead_code)]
    pub(super) const NEUTRAL_ORANGE: Color = Color::Rgb(0xd6, 0x5d, 0x0e); // #d65d0e

    // Gray
    #[allow(dead_code)]
    pub(super) const GRAY: Color = Color::Rgb(0x92, 0x83, 0x74); // #928374
}

/// Gruvbox light color palette.
mod gruvbox_light {
    use super::Color;

    // Background colors (inverted from dark mode)
    pub(super) const LIGHT0: Color = Color::Rgb(0xfb, 0xf1, 0xc7); // #fbf1c7
    pub(super) const LIGHT1: Color = Color::Rgb(0xeb, 0xdb, 0xb2); // #ebdbb2
    pub(super) const LIGHT2: Color = Color::Rgb(0xd5, 0xc4, 0xa1); // #d5c4a1
    #[allow(dead_code)]
    pub(super) const LIGHT3: Color = Color::Rgb(0xbd, 0xae, 0x93); // #bdae93

    // Foreground colors (inverted from dark mode)
    #[allow(dead_code)]
    pub(super) const DARK0: Color = Color::Rgb(0x28, 0x28, 0x28); // #282828
    pub(super) const DARK1: Color = Color::Rgb(0x3c, 0x38, 0x36); // #3c3836
    #[allow(dead_code)]
    pub(super) const DARK2: Color = Color::Rgb(0x50, 0x49, 0x45); // #504945
    #[allow(dead_code)]
    pub(super) const DARK3: Color = Color::Rgb(0x66, 0x5c, 0x54); // #665c54
    #[allow(dead_code)]
    pub(super) const DARK4: Color = Color::Rgb(0x7c, 0x6f, 0x64); // #7c6f64

    // Accent colors (faded variants for light mode)
    pub(super) const FADED_RED: Color = Color::Rgb(0x9d, 0x00, 0x06); // #9d0006
    pub(super) const FADED_GREEN: Color = Color::Rgb(0x79, 0x74, 0x0e); // #79740e
    pub(super) const FADED_YELLOW: Color = Color::Rgb(0xb5, 0x76, 0x14); // #b57614
    #[allow(dead_code)]
    pub(super) const FADED_BLUE: Color = Color::Rgb(0x07, 0x66, 0x78); // #076678
    #[allow(dead_code)]
    pub(super) const FADED_PURPLE: Color = Color::Rgb(0x8f, 0x3f, 0x71); // #8f3f71
    #[allow(dead_code)]
    pub(super) const FADED_AQUA: Color = Color::Rgb(0x42, 0x7b, 0x58); // #427b58
    pub(super) const FADED_ORANGE: Color = Color::Rgb(0xaf, 0x3a, 0x03); // #af3a03

    // Neutral colors
    #[allow(dead_code)]
    pub(super) const NEUTRAL_RED: Color = Color::Rgb(0xcc, 0x24, 0x1d); // #cc241d
    #[allow(dead_code)]
    pub(super) const NEUTRAL_GREEN: Color = Color::Rgb(0x98, 0x97, 0x1a); // #98971a
    #[allow(dead_code)]
    pub(super) const NEUTRAL_YELLOW: Color = Color::Rgb(0xd7, 0x99, 0x21); // #d79921
    #[allow(dead_code)]
    pub(super) const NEUTRAL_BLUE: Color = Color::Rgb(0x45, 0x85, 0x88); // #458588
    #[allow(dead_code)]
    pub(super) const NEUTRAL_PURPLE: Color = Color::Rgb(0xb1, 0x62, 0x86); // #b16286
    pub(super) const NEUTRAL_AQUA: Color = Color::Rgb(0x68, 0x9d, 0x6a); // #689d6a
    #[allow(dead_code)]
    pub(super) const NEUTRAL_ORANGE: Color = Color::Rgb(0xd6, 0x5d, 0x0e); // #d65d0e

    // Gray
    #[allow(dead_code)]
    pub(super) const GRAY: Color = Color::Rgb(0x92, 0x83, 0x74); // #928374
}

/// Gruvbox theme color palette using official color names.
#[derive(Debug, Clone)]
pub struct ThemeColors {
    // Background colors
    /// Primary background (dark0 or light0).
    pub bg0: Color,
    /// Secondary background (dark1 or light1).
    pub bg1: Color,
    /// Tertiary background (dark3 or light2).
    pub bg2: Color,

    // Foreground colors
    /// Primary foreground (light1 or dark1).
    pub fg0: Color,
    /// Secondary foreground (light2 or dark2).
    pub fg1: Color,

    // Accent colors
    /// Yellow accent.
    pub yellow: Color,
    /// Green accent.
    pub green: Color,
    /// Aqua accent.
    pub aqua: Color,
    /// Orange accent.
    pub orange: Color,
    /// Red accent.
    pub red: Color,
    /// Blue accent.
    #[allow(dead_code)]
    pub blue: Color,
    /// Purple accent.
    #[allow(dead_code)]
    pub purple: Color,

    // Neutral gray
    /// Gray color.
    #[allow(dead_code)]
    pub gray: Color,
}

impl ThemeColors {
    /// Create color palette from theme.
    pub fn from_theme(theme: &Theme) -> Self {
        match theme {
            Theme::GruvboxDark => Self {
                bg0: gruvbox_dark::DARK0,
                bg1: gruvbox_dark::DARK1,
                bg2: gruvbox_dark::DARK3,
                fg0: gruvbox_dark::LIGHT1,
                fg1: gruvbox_dark::LIGHT2,
                yellow: gruvbox_dark::BRIGHT_YELLOW,
                green: gruvbox_dark::BRIGHT_GREEN,
                aqua: gruvbox_dark::BRIGHT_AQUA,
                orange: gruvbox_dark::BRIGHT_ORANGE,
                red: gruvbox_dark::BRIGHT_RED,
                blue: gruvbox_dark::BRIGHT_BLUE,
                purple: gruvbox_dark::BRIGHT_PURPLE,
                gray: gruvbox_dark::GRAY,
            },
            Theme::GruvboxLight => Self {
                bg0: gruvbox_light::LIGHT0,
                bg1: gruvbox_light::LIGHT1,
                bg2: gruvbox_light::LIGHT2,
                fg0: gruvbox_light::DARK1,
                fg1: gruvbox_light::DARK2,
                yellow: gruvbox_light::FADED_YELLOW,
                green: gruvbox_light::FADED_GREEN,
                aqua: gruvbox_light::NEUTRAL_AQUA,
                orange: gruvbox_light::FADED_ORANGE,
                red: gruvbox_light::FADED_RED,
                blue: gruvbox_light::FADED_BLUE,
                purple: gruvbox_light::FADED_PURPLE,
                gray: gruvbox_light::GRAY,
            },
        }
    }
}
