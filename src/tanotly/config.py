"""Configuration constants for Tanotly.

Simplified theme system using Textual's Theme class with custom variables
for Rich markup colors and plot rendering.
"""

from pathlib import Path
from typing import Dict, Optional

from textual.theme import Theme

from .data.models import NodeType


# =============================================================================
# HELPER FUNCTIONS
# =============================================================================

def _hex_to_rgb(hex_color: str) -> str:
    """Convert hex color to rgb() format for Rich markup."""
    hex_color = hex_color.lstrip('#')
    r, g, b = int(hex_color[0:2], 16), int(hex_color[2:4], 16), int(hex_color[4:6], 16)
    return f"rgb({r},{g},{b})"


def _hex_to_rgb_tuple(hex_color: str) -> tuple:
    """Convert hex color to RGB tuple (0-255) for matplotlib/plotext."""
    hex_color = hex_color.lstrip('#')
    return (
        int(hex_color[0:2], 16),
        int(hex_color[2:4], 16),
        int(hex_color[4:6], 16)
    )


# =============================================================================
# THEME DEFINITIONS
# =============================================================================

# Gruvbox Dark Theme
GRUVBOX_DARK = Theme(
    name="gruvbox-dark",
    primary="#fabd2f",      # Bright yellow
    secondary="#b8bb26",    # Lime green
    accent="#fabd2f",       # Bright yellow
    foreground="#ebdbb2",   # Light beige
    background="#282828",   # Very dark gray
    success="#b8bb26",      # Lime green
    warning="#fe8019",      # Orange
    error="#fb4934",        # Bright red
    surface="#282828",      # Very dark gray (same as background)
    panel="#3c3836",        # Dark gray
    boost="#504945",        # Medium dark gray (highlighted cursor)
    dark=True,
    variables={
        # Rich markup colors (semantic colors for node types)
        "group": "#fabd2f",        # Yellow for groups (same as primary)
        "variable": "#83a598",     # Aqua for variables
        "dimension": "#b8bb26",    # Green for dimensions (same as secondary)
        "root": "#d3869b",         # Purple for root/attributes
        "muted": "#928374",        # Gray for muted text
        "info": "#83a598",         # Aqua for info (same as variable)

        # Plot colors (RGB tuples as strings for storage)
        "plot-bg": "40,40,40",      # Same as background
        "plot-fg": "235,219,178",   # Same as foreground
        "plot-line": "142,192,124", # Green-aqua for plot lines
    },
)

# Gruvbox Light Theme
GRUVBOX_LIGHT = Theme(
    name="gruvbox-light",
    primary="#b57614",      # Dark brown
    secondary="#79740e",    # Dark olive green
    accent="#b57614",       # Dark brown
    foreground="#3c3836",   # Very dark gray
    background="#fbf1c7",   # Cream
    success="#79740e",      # Dark olive green (same as secondary)
    warning="#af3a03",      # Dark red
    error="#9d0006",        # Deep red
    surface="#fbf1c7",      # Cream (same as background)
    panel="#ebdbb2",        # Light tan
    boost="#d5c4a1",        # Medium tan (highlighted cursor)
    dark=False,
    variables={
        # Rich markup colors (semantic colors for node types)
        "group": "#b57614",        # Brown for groups (same as primary)
        "variable": "#076678",     # Blue for variables
        "dimension": "#79740e",    # Green for dimensions (same as secondary)
        "root": "#8f3f71",         # Purple for root/attributes
        "muted": "#7c6f64",        # Gray for muted text
        "info": "#076678",         # Blue for info (same as variable)

        # Plot colors (RGB tuples as strings for storage)
        "plot-bg": "251,241,199",  # Same as background
        "plot-fg": "60,56,54",     # Same as foreground
        "plot-line": "66,123,88",  # Green for plot lines
    },
)


# =============================================================================
# THEME MANAGER
# =============================================================================

class ThemeManager:
    """Centralized theme management using Textual Theme objects.

    Provides a unified interface for:
    - Textual CSS theming (via Theme registration)
    - Rich markup colors (via theme variables)
    - Plot rendering colors (via theme variables)

    Usage:
        # Register themes on app startup
        ThemeManager.register_all_themes(app)

        # Switch themes
        ThemeManager.set_theme("gruvbox-dark")
        app.theme = "gruvbox-dark"

        # Get colors for Rich markup
        color = ThemeManager.get_color("variable")

        # Get plot colors
        plot_colors = ThemeManager.get_plot_colors()
    """

    _themes: Dict[str, Theme] = {}
    _current_theme: str = "gruvbox-dark"

    @classmethod
    def register_theme(cls, theme: Theme) -> None:
        """Register a new theme.

        Args:
            theme: Textual Theme object
        """
        cls._themes[theme.name] = theme

    @classmethod
    def set_theme(cls, name: str) -> None:
        """Set the current active theme.

        Args:
            name: Theme name to activate
        """
        if name not in cls._themes:
            raise ValueError(f"Unknown theme: {name}")
        cls._current_theme = name

    @classmethod
    def get_current_theme(cls) -> str:
        """Get the current theme name."""
        return cls._current_theme

    @classmethod
    def get_theme(cls, name: Optional[str] = None) -> Theme:
        """Get a theme by name.

        Args:
            name: Theme name (uses current theme if None)

        Returns:
            Theme object
        """
        theme_name = name or cls._current_theme
        theme = cls._themes.get(theme_name)
        if not theme:
            # Fallback to gruvbox-dark
            return GRUVBOX_DARK
        return theme

    @classmethod
    def is_dark(cls) -> bool:
        """Check if the current theme is dark."""
        theme = cls.get_theme()
        return theme.dark

    @classmethod
    def get_color(cls, color_key: str, theme_name: Optional[str] = None) -> str:
        """Get a color for Rich markup.

        Args:
            color_key: Color name (group, variable, dimension, primary, etc.)
            theme_name: Theme name (uses current theme if None)

        Returns:
            RGB color string for Rich markup (e.g., "rgb(250,189,47)")
        """
        theme = cls.get_theme(theme_name)

        # First check custom variables
        if color_key in theme.variables:
            hex_color = theme.variables[color_key]
            return _hex_to_rgb(hex_color)

        # Then check standard theme properties
        color_map = {
            "primary": theme.primary,
            "secondary": theme.secondary,
            "accent": theme.accent,
            "success": theme.success,
            "warning": theme.warning,
            "error": theme.error,
            "foreground": theme.foreground,
            "background": theme.background,
        }

        hex_color = color_map.get(color_key, theme.foreground)
        return _hex_to_rgb(hex_color)

    @classmethod
    def get_plot_colors(cls, theme_name: Optional[str] = None) -> Dict[str, tuple]:
        """Get plot colors for the current theme.

        Args:
            theme_name: Theme name (uses current theme if None)

        Returns:
            Dictionary with 'bg', 'fg', 'accent', 'line' RGB tuples
        """
        theme = cls.get_theme(theme_name)

        # Parse RGB tuples from theme variables
        def parse_rgb(rgb_str: str) -> tuple:
            r, g, b = rgb_str.split(',')
            return (int(r), int(g), int(b))

        return {
            "bg": parse_rgb(theme.variables.get("plot-bg", "0,0,0")),
            "fg": parse_rgb(theme.variables.get("plot-fg", "255,255,255")),
            "accent": _hex_to_rgb_tuple(theme.secondary),
            "line": parse_rgb(theme.variables.get("plot-line", "128,128,128")),
        }

    @classmethod
    def get_theme_names(cls) -> list[str]:
        """Get list of all registered theme names."""
        return list(cls._themes.keys())

    @classmethod
    def get_display_name(cls, name: str) -> str:
        """Get the display name for a theme."""
        # Convert kebab-case to Title Case
        return name.replace('-', ' ').title()

    @classmethod
    def register_all_themes(cls, app) -> None:
        """Register all themes with a Textual app.

        This should be called during app.on_mount() to register
        all available themes with the Textual framework.

        Args:
            app: Textual App instance
        """
        for theme in cls._themes.values():
            app.register_theme(theme)


# =============================================================================
# REGISTER DEFAULT THEMES
# =============================================================================

ThemeManager.register_theme(GRUVBOX_DARK)
ThemeManager.register_theme(GRUVBOX_LIGHT)


# =============================================================================
# COLORS CLASS (Backward-compatible API)
# =============================================================================

class Colors:
    """Theme-aware color accessor for Rich markup.

    Provides a convenient API for getting colors in Rich markup format.

    Usage:
        f"[{Colors.variable()}]text[/]"
    """

    @classmethod
    def set_theme(cls, theme_name: str) -> None:
        """Set the current theme by name."""
        ThemeManager.set_theme(theme_name)

    @classmethod
    def is_dark(cls) -> bool:
        """Check if current theme is dark."""
        return ThemeManager.is_dark()

    @classmethod
    def group(cls) -> str:
        return ThemeManager.get_color("group")

    @classmethod
    def variable(cls) -> str:
        return ThemeManager.get_color("variable")

    @classmethod
    def dimension(cls) -> str:
        return ThemeManager.get_color("dimension")

    @classmethod
    def root(cls) -> str:
        return ThemeManager.get_color("root")

    @classmethod
    def muted(cls) -> str:
        return ThemeManager.get_color("muted")

    @classmethod
    def accent(cls) -> str:
        return ThemeManager.get_color("accent")

    @classmethod
    def success(cls) -> str:
        return ThemeManager.get_color("success")

    @classmethod
    def error(cls) -> str:
        return ThemeManager.get_color("error")

    @classmethod
    def warning(cls) -> str:
        return ThemeManager.get_color("warning")

    @classmethod
    def info(cls) -> str:
        return ThemeManager.get_color("info")


# =============================================================================
# NODE ICONS
# =============================================================================

# Using simple ASCII/Unicode characters that render well in all terminals
# Alternative styles available:
# - Box drawing: ┌─┐ ├─┤ └─┘
# - Geometric: ■ □ ▪ ▫ ● ○ ◆ ◇ ▲ △
# - ASCII-art: [+] [-] [*] [>] [#]

NODE_ICONS = {
    NodeType.ROOT: "■",      # Solid square for root/file
    NodeType.GROUP: "▶",     # Right-pointing triangle for expandable groups
    NodeType.VARIABLE: "●",  # Solid circle for data variables
    NodeType.DIMENSION: "│",  # Vertical line for dimensions (measure/axis)
    NodeType.ATTRIBUTE: "▫",  # Small hollow square for metadata/attributes
}


# =============================================================================
# PATHS
# =============================================================================

CSS_PATH = Path(__file__).parent / "app.tcss"
