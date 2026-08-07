//! Material Design 3 color theme for the TUI.
//!
//! Provides a palette based on MD3 tokens with both light and dark variants.

use ratatui::style::Color;

/// MD3-inspired dark theme palette.
pub struct Theme {
    // Surface colors
    pub surface_bg: Color,
    pub surface_dim_bg: Color,
    pub surface_dim_fg: Color,

    // Primary
    pub primary_bg: Color,
    pub primary_fg: Color,
    pub primary_container_bg: Color,

    // Content
    pub on_surface: Color,
    pub on_surface_variant: Color,
    pub outline: Color,

    // Accent
    pub error_bg: Color,
    pub error_fg: Color,
    pub success_fg: Color,
    pub warning_fg: Color,
}

/// Returns the default dark theme.
pub fn md3_theme() -> Theme {
    Theme {
        // Surface: dark grays
        surface_bg: Color::Rgb(28, 27, 31),        // MD3 surface
        surface_dim_bg: Color::Rgb(20, 19, 23),     // MD3 surface dim
        surface_dim_fg: Color::Rgb(148, 143, 157),  // MD3 on-surface-variant

        // Primary: soft blue-purple
        primary_bg: Color::Rgb(103, 80, 164),       // MD3 primary
        primary_fg: Color::Rgb(234, 221, 255),      // MD3 on-primary
        primary_container_bg: Color::Rgb(79, 55, 139), // MD3 primary container

        // Content
        on_surface: Color::Rgb(230, 224, 233),      // MD3 on-surface
        on_surface_variant: Color::Rgb(202, 196, 208), // MD3 on-surface-variant
        outline: Color::Rgb(147, 143, 153),         // MD3 outline

        // Accent
        error_bg: Color::Rgb(147, 0, 10),
        error_fg: Color::Rgb(255, 180, 171),
        success_fg: Color::Rgb(129, 201, 149),
        warning_fg: Color::Rgb(243, 188, 70),
    }
}

/// Returns a light theme variant.
pub fn light_theme() -> Theme {
    Theme {
        surface_bg: Color::Rgb(255, 251, 254),
        surface_dim_bg: Color::Rgb(231, 224, 236),
        surface_dim_fg: Color::Rgb(121, 116, 126),

        primary_bg: Color::Rgb(103, 80, 164),
        primary_fg: Color::Rgb(255, 255, 255),
        primary_container_bg: Color::Rgb(234, 221, 255),

        on_surface: Color::Rgb(28, 27, 31),
        on_surface_variant: Color::Rgb(73, 69, 79),
        outline: Color::Rgb(121, 116, 126),

        error_bg: Color::Rgb(255, 218, 214),
        error_fg: Color::Rgb(147, 0, 10),
        success_fg: Color::Rgb(26, 120, 45),
        warning_fg: Color::Rgb(138, 95, 0),
    }
}
