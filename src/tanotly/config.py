"""Configuration constants for Tanotly."""

from pathlib import Path
from typing import Dict

from .data.models import NodeType


# =============================================================================
# GRUVBOX COLOR PALETTE
# Official colors from https://github.com/morhetz/gruvbox
# =============================================================================

class GruvboxColors:
    """Official Gruvbox color palette."""
    
    # Dark mode backgrounds
    DARK_BG_HARD = "#1d2021"
    DARK_BG = "#282828"
    DARK_BG_SOFT = "#32302f"
    DARK_BG1 = "#3c3836"
    DARK_BG2 = "#504945"
    DARK_BG3 = "#665c54"
    DARK_BG4 = "#7c6f64"
    
    # Light mode backgrounds
    LIGHT_BG_HARD = "#f9f5d7"
    LIGHT_BG = "#fbf1c7"
    LIGHT_BG_SOFT = "#f2e5bc"
    LIGHT_BG1 = "#ebdbb2"
    LIGHT_BG2 = "#d5c4a1"
    LIGHT_BG3 = "#bdae93"
    LIGHT_BG4 = "#a89984"
    
    # Dark mode foregrounds
    DARK_FG = "#ebdbb2"
    DARK_FG0 = "#fbf1c7"
    DARK_FG1 = "#ebdbb2"
    DARK_FG2 = "#d5c4a1"
    DARK_FG3 = "#bdae93"
    DARK_FG4 = "#a89984"
    
    # Light mode foregrounds
    LIGHT_FG = "#3c3836"
    LIGHT_FG0 = "#282828"
    LIGHT_FG1 = "#3c3836"
    LIGHT_FG2 = "#504945"
    LIGHT_FG3 = "#665c54"
    LIGHT_FG4 = "#7c6f64"
    
    # Dark mode colors (bright)
    DARK_RED = "#fb4934"
    DARK_GREEN = "#b8bb26"
    DARK_YELLOW = "#fabd2f"
    DARK_BLUE = "#83a598"
    DARK_PURPLE = "#d3869b"
    DARK_AQUA = "#8ec07c"
    DARK_ORANGE = "#fe8019"
    DARK_GRAY = "#928374"
    
    # Light mode colors (faded)
    LIGHT_RED = "#9d0006"
    LIGHT_GREEN = "#79740e"
    LIGHT_YELLOW = "#b57614"
    LIGHT_BLUE = "#076678"
    LIGHT_PURPLE = "#8f3f71"
    LIGHT_AQUA = "#427b58"
    LIGHT_ORANGE = "#af3a03"
    LIGHT_GRAY = "#7c6f64"
    
    # Neutral colors
    NEUTRAL_RED = "#cc241d"
    NEUTRAL_GREEN = "#98971a"
    NEUTRAL_YELLOW = "#d79921"
    NEUTRAL_BLUE = "#458588"
    NEUTRAL_PURPLE = "#b16286"
    NEUTRAL_AQUA = "#689d6a"
    NEUTRAL_ORANGE = "#d65d0e"
    NEUTRAL_GRAY = "#928374"


def _hex_to_rgb(hex_color: str) -> str:
    """Convert hex color to rgb() format for Rich markup."""
    hex_color = hex_color.lstrip('#')
    r, g, b = int(hex_color[0:2], 16), int(hex_color[2:4], 16), int(hex_color[4:6], 16)
    return f"rgb({r},{g},{b})"


class ThemeColors:
    """Theme-aware color accessor for the application."""
    
    _is_dark: bool = True
    
    @classmethod
    def set_dark_mode(cls, is_dark: bool) -> None:
        """Set the current theme mode."""
        cls._is_dark = is_dark
    
    @classmethod
    def is_dark(cls) -> bool:
        """Check if dark mode is active."""
        return cls._is_dark
    
    # Semantic colors for the application
    @classmethod
    def group(cls) -> str:
        """Color for group nodes (yellow)."""
        color = GruvboxColors.DARK_YELLOW if cls._is_dark else GruvboxColors.LIGHT_YELLOW
        return _hex_to_rgb(color)
    
    @classmethod
    def variable(cls) -> str:
        """Color for variable nodes (aqua/cyan)."""
        color = GruvboxColors.DARK_BLUE if cls._is_dark else GruvboxColors.LIGHT_BLUE
        return _hex_to_rgb(color)
    
    @classmethod
    def dimension(cls) -> str:
        """Color for dimension info (green)."""
        color = GruvboxColors.DARK_GREEN if cls._is_dark else GruvboxColors.LIGHT_GREEN
        return _hex_to_rgb(color)
    
    @classmethod
    def root(cls) -> str:
        """Color for root node (purple)."""
        color = GruvboxColors.DARK_PURPLE if cls._is_dark else GruvboxColors.LIGHT_PURPLE
        return _hex_to_rgb(color)
    
    @classmethod
    def muted(cls) -> str:
        """Color for muted/secondary text (gray)."""
        color = GruvboxColors.DARK_GRAY if cls._is_dark else GruvboxColors.LIGHT_GRAY
        return _hex_to_rgb(color)
    
    @classmethod
    def accent(cls) -> str:
        """Primary accent color (yellow)."""
        color = GruvboxColors.DARK_YELLOW if cls._is_dark else GruvboxColors.LIGHT_YELLOW
        return _hex_to_rgb(color)
    
    @classmethod
    def success(cls) -> str:
        """Success/positive color (green)."""
        color = GruvboxColors.DARK_GREEN if cls._is_dark else GruvboxColors.LIGHT_GREEN
        return _hex_to_rgb(color)
    
    @classmethod
    def error(cls) -> str:
        """Error/negative color (red)."""
        color = GruvboxColors.DARK_RED if cls._is_dark else GruvboxColors.LIGHT_RED
        return _hex_to_rgb(color)
    
    @classmethod
    def warning(cls) -> str:
        """Warning color (orange)."""
        color = GruvboxColors.DARK_ORANGE if cls._is_dark else GruvboxColors.LIGHT_ORANGE
        return _hex_to_rgb(color)
    
    @classmethod
    def info(cls) -> str:
        """Info color (blue)."""
        color = GruvboxColors.DARK_BLUE if cls._is_dark else GruvboxColors.LIGHT_BLUE
        return _hex_to_rgb(color)
    
    @classmethod
    def foreground(cls) -> str:
        """Primary foreground/text color."""
        color = GruvboxColors.DARK_FG if cls._is_dark else GruvboxColors.LIGHT_FG
        return _hex_to_rgb(color)
    
    @classmethod
    def background(cls) -> str:
        """Primary background color."""
        color = GruvboxColors.DARK_BG if cls._is_dark else GruvboxColors.LIGHT_BG
        return _hex_to_rgb(color)


# =============================================================================
# NODE ICONS
# =============================================================================

NODE_ICONS = {
    NodeType.ROOT: "🏠",
    NodeType.GROUP: "📂",
    NodeType.VARIABLE: "🌡️",
    NodeType.DIMENSION: "📏",
    NodeType.ATTRIBUTE: "🏷️",
}


# =============================================================================
# PLOT COLORS
# =============================================================================

def _hex_to_rgb_tuple(hex_color: str) -> tuple:
    """Convert hex color to RGB tuple for matplotlib."""
    hex_color = hex_color.lstrip('#')
    return (
        int(hex_color[0:2], 16),
        int(hex_color[2:4], 16),
        int(hex_color[4:6], 16)
    )


PLOT_COLORS = {
    "dark": {
        "bg": _hex_to_rgb_tuple(GruvboxColors.DARK_BG),
        "fg": _hex_to_rgb_tuple(GruvboxColors.DARK_FG),
        "accent": _hex_to_rgb_tuple(GruvboxColors.DARK_GREEN),
        "line": _hex_to_rgb_tuple(GruvboxColors.DARK_AQUA),
    },
    "light": {
        "bg": _hex_to_rgb_tuple(GruvboxColors.LIGHT_BG),
        "fg": _hex_to_rgb_tuple(GruvboxColors.LIGHT_FG),
        "accent": _hex_to_rgb_tuple(GruvboxColors.LIGHT_GREEN),
        "line": _hex_to_rgb_tuple(GruvboxColors.LIGHT_AQUA),
    }
}


def get_plot_colors(is_dark: bool = True) -> dict:
    """Get plot colors based on theme mode."""
    return PLOT_COLORS["dark" if is_dark else "light"]


# =============================================================================
# PATHS
# =============================================================================

CSS_PATH = Path(__file__).parent / "app.tcss"
