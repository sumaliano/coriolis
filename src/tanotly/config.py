"""Configuration constants for Tanotly.

Unified theme system that consolidates:
- Textual CSS themes (for widget styling)
- Rich markup colors (for text formatting)
- Plot colors (for matplotlib/plotext visualizations)

All themes are registered through ThemeManager, making it easy to add new themes
and ensuring consistency across all UI components.
"""

from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional

from textual.theme import Theme

from .data.models import NodeType


# =============================================================================
# THEME DATA STRUCTURES
# =============================================================================

@dataclass
class ThemeColors:
    """Complete color palette for a theme.
    
    This defines all colors needed for:
    - Textual CSS theming (primary, secondary, etc.)
    - Rich markup (semantic colors like group, variable, etc.)
    - Plot rendering (background, foreground, line colors)
    """
    # Core Textual theme colors
    primary: str
    secondary: str
    accent: str
    foreground: str
    background: str
    success: str
    warning: str
    error: str
    surface: str
    panel: str
    boost: str
    
    # Text colors
    text: str
    text_muted: str
    text_disabled: str
    
    # Scrollbar colors
    scrollbar_background: str
    scrollbar_color: str
    scrollbar_color_hover: str
    scrollbar_color_active: str
    
    # Footer colors
    footer_background: str
    footer_key_foreground: str
    footer_key_background: str
    footer_description: str
    
    # Semantic colors for Rich markup
    group: str          # Groups/folders
    variable: str       # Variables/data
    dimension: str      # Dimensions
    root: str           # Root/attributes
    muted: str          # Muted/secondary text
    info: str           # Info messages
    
    # Plot colors (RGB tuples)
    plot_bg: tuple = field(default_factory=lambda: (0, 0, 0))
    plot_fg: tuple = field(default_factory=lambda: (255, 255, 255))
    plot_accent: tuple = field(default_factory=lambda: (128, 128, 128))
    plot_line: tuple = field(default_factory=lambda: (128, 128, 128))
    
    # Theme metadata
    is_dark: bool = True


@dataclass
class ThemeDefinition:
    """Complete theme definition with name and colors."""
    name: str
    display_name: str
    colors: ThemeColors


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
GRUVBOX_DARK_COLORS = ThemeColors(
    # Core Textual colors
    primary="#fabd2f",      # Bright yellow
    secondary="#b8bb26",    # Lime green
    accent="#fabd2f",       # Bright yellow
    foreground="#ebdbb2",   # Light beige
    background="#282828",   # Very dark gray
    success="#b8bb26",      # Lime green
    warning="#fe8019",      # Orange
    error="#fb4934",        # Bright red
    surface="#282828",      # Very dark gray
    panel="#3c3836",        # Dark gray
    boost="#504945",        # Medium dark gray
    
    # Text colors
    text="#ebdbb2",
    text_muted="#a89984",
    text_disabled="#665c54",
    
    # Scrollbar
    scrollbar_background="#3c3836",
    scrollbar_color="#665c54",
    scrollbar_color_hover="#928374",
    scrollbar_color_active="#a89984",
    
    # Footer
    footer_background="#3c3836",
    footer_key_foreground="#fabd2f",
    footer_key_background="#504945",
    footer_description="#a89984",
    
    # Semantic colors (Rich markup)
    group="#fabd2f",        # Yellow for groups
    variable="#83a598",     # Aqua for variables
    dimension="#b8bb26",    # Green for dimensions
    root="#d3869b",         # Purple for root/attributes
    muted="#928374",        # Gray for muted text
    info="#83a598",         # Aqua for info
    
    # Plot colors
    plot_bg=_hex_to_rgb_tuple("#282828"),
    plot_fg=_hex_to_rgb_tuple("#ebdbb2"),
    plot_accent=_hex_to_rgb_tuple("#b8bb26"),
    plot_line=_hex_to_rgb_tuple("#8ec07c"),
    
    is_dark=True,
)

# Gruvbox Light Theme
GRUVBOX_LIGHT_COLORS = ThemeColors(
    # Core Textual colors
    primary="#b57614",      # Dark brown
    secondary="#79740e",    # Dark olive green
    accent="#b57614",       # Dark brown
    foreground="#3c3836",   # Very dark gray
    background="#fbf1c7",   # Cream
    success="#79740e",      # Dark olive green
    warning="#af3a03",      # Dark red
    error="#9d0006",        # Deep red
    surface="#fbf1c7",      # Cream
    panel="#ebdbb2",        # Light tan
    boost="#d5c4a1",        # Medium tan
    
    # Text colors
    text="#3c3836",
    text_muted="#7c6f64",
    text_disabled="#bdae93",
    
    # Scrollbar
    scrollbar_background="#ebdbb2",
    scrollbar_color="#bdae93",
    scrollbar_color_hover="#a89984",
    scrollbar_color_active="#7c6f64",
    
    # Footer
    footer_background="#ebdbb2",
    footer_key_foreground="#b57614",
    footer_key_background="#d5c4a1",
    footer_description="#7c6f64",
    
    # Semantic colors (Rich markup)
    group="#b57614",        # Brown for groups
    variable="#076678",     # Blue for variables
    dimension="#79740e",    # Green for dimensions
    root="#8f3f71",         # Purple for root/attributes
    muted="#7c6f64",        # Gray for muted text
    info="#076678",         # Blue for info
    
    # Plot colors
    plot_bg=_hex_to_rgb_tuple("#fbf1c7"),
    plot_fg=_hex_to_rgb_tuple("#3c3836"),
    plot_accent=_hex_to_rgb_tuple("#79740e"),
    plot_line=_hex_to_rgb_tuple("#427b58"),
    
    is_dark=False,
)


# =============================================================================
# THEME MANAGER
# =============================================================================

class ThemeManager:
    """Centralized theme management for all UI components.
    
    This class provides a single source of truth for themes, ensuring
    consistency between Textual CSS, Rich markup, and plot colors.
    
    Usage:
        # Register themes on app startup
        ThemeManager.register_all_themes(app)
        
        # Switch themes
        ThemeManager.set_theme("gruvbox-dark")
        
        # Get colors for Rich markup
        color = ThemeManager.get_color("variable")
        
        # Get plot colors
        plot_colors = ThemeManager.get_plot_colors()
    """
    
    _themes: Dict[str, ThemeDefinition] = {}
    _current_theme: str = "gruvbox-dark"
    
    @classmethod
    def register_theme(cls, name: str, display_name: str, colors: ThemeColors) -> None:
        """Register a new theme.
        
        Args:
            name: Internal theme name (e.g., "gruvbox-dark")
            display_name: Human-readable name (e.g., "Gruvbox Dark")
            colors: ThemeColors instance with all color definitions
        """
        cls._themes[name] = ThemeDefinition(
            name=name,
            display_name=display_name,
            colors=colors
        )
    
    @classmethod
    def get_textual_theme(cls, name: str) -> Theme:
        """Get a Textual Theme object for registration with the app.
        
        Args:
            name: Theme name
            
        Returns:
            Textual Theme object
        """
        theme_def = cls._themes.get(name)
        if not theme_def:
            raise ValueError(f"Unknown theme: {name}")
        
        colors = theme_def.colors
        return Theme(
            name=name,
            primary=colors.primary,
            secondary=colors.secondary,
            accent=colors.accent,
            foreground=colors.foreground,
            background=colors.background,
            success=colors.success,
            warning=colors.warning,
            error=colors.error,
            surface=colors.surface,
            panel=colors.panel,
            boost=colors.boost,
            dark=colors.is_dark,
            variables={
                "text": colors.text,
                "text-muted": colors.text_muted,
                "text-disabled": colors.text_disabled,
                "scrollbar-background": colors.scrollbar_background,
                "scrollbar-color": colors.scrollbar_color,
                "scrollbar-color-hover": colors.scrollbar_color_hover,
                "scrollbar-color-active": colors.scrollbar_color_active,
                "footer-background": colors.footer_background,
                "footer-key-foreground": colors.footer_key_foreground,
                "footer-key-background": colors.footer_key_background,
                "footer-description": colors.footer_description,
            },
        )
    
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
    def get_current_colors(cls) -> ThemeColors:
        """Get the current theme's colors."""
        theme_def = cls._themes.get(cls._current_theme)
        if not theme_def:
            # Fallback to gruvbox-dark colors
            return GRUVBOX_DARK_COLORS
        return theme_def.colors
    
    @classmethod
    def is_dark(cls) -> bool:
        """Check if the current theme is dark."""
        return cls.get_current_colors().is_dark
    
    @classmethod
    def get_color(cls, color_key: str) -> str:
        """Get a semantic color for Rich markup.
        
        Args:
            color_key: Color name (group, variable, dimension, etc.)
            
        Returns:
            RGB color string for Rich markup (e.g., "rgb(250,189,47)")
        """
        colors = cls.get_current_colors()
        hex_color = getattr(colors, color_key, colors.text)
        return _hex_to_rgb(hex_color)
    
    @classmethod
    def get_plot_colors(cls) -> Dict[str, tuple]:
        """Get plot colors for the current theme.
        
        Returns:
            Dictionary with 'bg', 'fg', 'accent', 'line' RGB tuples
        """
        colors = cls.get_current_colors()
        return {
            "bg": colors.plot_bg,
            "fg": colors.plot_fg,
            "accent": colors.plot_accent,
            "line": colors.plot_line,
        }
    
    @classmethod
    def get_theme_names(cls) -> List[str]:
        """Get list of all registered theme names."""
        return list(cls._themes.keys())
    
    @classmethod
    def get_display_name(cls, name: str) -> str:
        """Get the display name for a theme."""
        theme_def = cls._themes.get(name)
        return theme_def.display_name if theme_def else name
    
    @classmethod
    def register_all_themes(cls, app) -> None:
        """Register all themes with a Textual app.
        
        This should be called during app.on_mount() to register
        all available themes with the Textual framework.
        
        Args:
            app: Textual App instance
        """
        for name in cls._themes:
            app.register_theme(cls.get_textual_theme(name))


# =============================================================================
# REGISTER DEFAULT THEMES
# =============================================================================

# Register Gruvbox themes
ThemeManager.register_theme("gruvbox-dark", "Gruvbox Dark", GRUVBOX_DARK_COLORS)
ThemeManager.register_theme("gruvbox-light", "Gruvbox Light", GRUVBOX_LIGHT_COLORS)


# =============================================================================
# COLORS CLASS (Backward-compatible API)
# =============================================================================

class Colors:
    """Theme-aware color accessor for Rich markup.
    
    This class provides a backward-compatible API that delegates
    to ThemeManager for actual color lookups.
    
    Usage:
        # Get colors for Rich markup
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
# BACKWARD-COMPATIBLE FUNCTIONS
# =============================================================================

def get_plot_colors(theme_name: Optional[str] = None) -> Dict[str, tuple]:
    """Get plot colors for a specific theme.
    
    Args:
        theme_name: Theme name (uses current theme if None)
        
    Returns:
        Dictionary with 'bg', 'fg', 'accent', 'line' RGB tuples
    """
    if theme_name:
        # Temporarily get colors for a specific theme
        theme_def = ThemeManager._themes.get(theme_name)
        if theme_def:
            colors = theme_def.colors
            return {
                "bg": colors.plot_bg,
                "fg": colors.plot_fg,
                "accent": colors.plot_accent,
                "line": colors.plot_line,
            }
    return ThemeManager.get_plot_colors()


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
# PATHS
# =============================================================================

CSS_PATH = Path(__file__).parent / "app.tcss"
