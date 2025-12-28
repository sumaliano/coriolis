//! Color mapping functions for data visualization.

use crate::data_viewer::ColorPalette;
use ratatui::style::Color;

impl ColorPalette {
    /// Map a normalized value (0.0 to 1.0) to an RGB color.
    pub fn color(self, t: f64) -> Color {
        let t = t.clamp(0.0, 1.0);

        match self {
            Self::Viridis => viridis_color(t),
            Self::Plasma => plasma_color(t),
            Self::Rainbow => rainbow_color(t),
            Self::BlueRed => bluered_color(t),
        }
    }
}

/// Viridis colormap approximation.
fn viridis_color(t: f64) -> Color {
    // Simplified viridis palette using piecewise linear interpolation
    let r = if t < 0.5 {
        68.0 + t * 2.0 * (33.0 - 68.0)
    } else {
        33.0 + (t - 0.5) * 2.0 * (253.0 - 33.0)
    };

    let g = if t < 0.5 {
        1.0 + t * 2.0 * (104.0 - 1.0)
    } else {
        104.0 + (t - 0.5) * 2.0 * (231.0 - 104.0)
    };

    let b = if t < 0.5 {
        84.0 + t * 2.0 * (109.0 - 84.0)
    } else {
        109.0 + (t - 0.5) * 2.0 * (37.0 - 109.0)
    };

    Color::Rgb(r as u8, g as u8, b as u8)
}

/// Plasma colormap approximation.
fn plasma_color(t: f64) -> Color {
    let r = if t < 0.5 {
        13.0 + t * 2.0 * (180.0 - 13.0)
    } else {
        180.0 + (t - 0.5) * 2.0 * (240.0 - 180.0)
    };

    let g = if t < 0.5 {
        8.0 + t * 2.0 * (54.0 - 8.0)
    } else {
        54.0 + (t - 0.5) * 2.0 * (175.0 - 54.0)
    };

    let b = if t < 0.5 {
        135.0 + t * 2.0 * (121.0 - 135.0)
    } else {
        121.0 + (t - 0.5) * 2.0 * (12.0 - 121.0)
    };

    Color::Rgb(r as u8, g as u8, b as u8)
}

/// Rainbow/Spectral colormap.
fn rainbow_color(t: f64) -> Color {
    // HSV to RGB conversion with H varying from 240° (blue) to 0° (red)
    let h = (1.0 - t) * 240.0;
    let s = 1.0;
    let v = 1.0;

    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Color::Rgb(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

/// Blue-White-Red diverging colormap.
fn bluered_color(t: f64) -> Color {
    if t < 0.5 {
        // Blue to white
        let t2 = t * 2.0;
        let r = (t2 * 255.0) as u8;
        let g = (t2 * 255.0) as u8;
        let b = 255;
        Color::Rgb(r, g, b)
    } else {
        // White to red
        let t2 = (t - 0.5) * 2.0;
        let r = 255;
        let g = ((1.0 - t2) * 255.0) as u8;
        let b = ((1.0 - t2) * 255.0) as u8;
        Color::Rgb(r, g, b)
    }
}
