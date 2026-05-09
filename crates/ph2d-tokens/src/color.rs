//! Color token system.
//!
//! Tokens (semantic) → resolved by `Theme` → concrete `Color`.
//! Widget code only ever uses tokens; literal hex values are walled
//! off in this module.
//!
//! HEX values per ADR-0023 §12. Sample-from-screenshot validation
//! pendente — direção fixada (Procreate cyan-blue accent).

use crate::theme::Theme;

/// sRGB 8-bit color with alpha. Constructed via `from_hex` for
/// readability; stored as four bytes for cheap copy + tests.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Construct from `0xRRGGBB` (alpha defaults to 0xFF / opaque).
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
            a: 0xFF,
        }
    }

    /// Construct from `0xRRGGBBAA`.
    pub const fn from_hex_alpha(hex: u32) -> Self {
        Self {
            r: ((hex >> 24) & 0xFF) as u8,
            g: ((hex >> 16) & 0xFF) as u8,
            b: ((hex >> 8) & 0xFF) as u8,
            a: (hex & 0xFF) as u8,
        }
    }

    /// Linearize sRGB channel (per WCAG 2.2 contrast formula).
    fn linearize(channel: u8) -> f64 {
        let c = channel as f64 / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    /// Relative luminance per WCAG 2.2 (alpha ignored — assume opaque
    /// over canonical surface for the contrast calculation).
    pub fn relative_luminance(&self) -> f64 {
        let r = Self::linearize(self.r);
        let g = Self::linearize(self.g);
        let b = Self::linearize(self.b);
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// WCAG contrast ratio between two opaque colors.
    /// Range: 1.0 (identical) to 21.0 (black on white).
    /// AA text minimum: 4.5. AA UI minimum: 3.0. AAA text: 7.0.
    pub fn contrast_ratio(&self, other: &Self) -> f64 {
        let l1 = self.relative_luminance();
        let l2 = other.relative_luminance();
        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }
}

/// Semantic color slot — every widget references one of these by
/// name; literal `from_hex` outside this crate is a code smell.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColorToken {
    /// Primary canvas backdrop (everything sits on this).
    Background,
    /// Panels, sidebar, toolbar.
    Surface,
    /// Popovers, tooltips, modals (slightly brighter than surface).
    SurfaceElevated,
    /// Low-contrast separator / inactive border.
    Border,
    /// Focused / hovered border, focus ring.
    BorderEmphasis,
    /// Main copy. ≥ 4.5:1 vs Surface (AA).
    TextPrimary,
    /// Labels, captions. ≥ 4.5:1 vs Surface (AA).
    TextSecondary,
    /// Explicit non-interactive. 3:1 OK per WCAG.
    TextDisabled,
    /// Active/selected primary — Procreate iconic cyan-blue.
    AccentPrimary,
    /// Selected secondary / pressed.
    AccentSecondary,
    /// Hover, focus ring tint.
    AccentTertiary,
    Success,
    Warning,
    ErrorState,
    Info,
}

impl ColorToken {
    /// Resolve token → concrete Color for the given Theme.
    /// Values per ADR-0023 §12; sample-from-screenshot validation
    /// pendente.
    pub const fn resolve(self, theme: Theme) -> Color {
        match (theme, self) {
            // Dark Mode (Procreate-inspired charcoal)
            (Theme::Dark, Self::Background) => Color::from_hex(0x161616),
            (Theme::Dark, Self::Surface) => Color::from_hex(0x1F1F1F),
            (Theme::Dark, Self::SurfaceElevated) => Color::from_hex(0x2A2A2A),
            (Theme::Dark, Self::Border) => Color::from_hex(0x3A3A3A),
            (Theme::Dark, Self::BorderEmphasis) => Color::from_hex(0x6E6E6E),
            (Theme::Dark, Self::TextPrimary) => Color::from_hex(0xE8E8E8),
            (Theme::Dark, Self::TextSecondary) => Color::from_hex(0xB5B5B5),
            (Theme::Dark, Self::TextDisabled) => Color::from_hex(0x6E6E6E),
            (Theme::Dark, Self::AccentPrimary) => Color::from_hex(0x0AB4FF),
            (Theme::Dark, Self::AccentSecondary) => Color::from_hex(0x0786BF),
            (Theme::Dark, Self::AccentTertiary) => Color::from_hex(0x04566F),
            (Theme::Dark, Self::Success) => Color::from_hex(0x4CAF50),
            (Theme::Dark, Self::Warning) => Color::from_hex(0xFFB300),
            (Theme::Dark, Self::ErrorState) => Color::from_hex(0xEF5350),
            (Theme::Dark, Self::Info) => Color::from_hex(0x29B6F6),

            // Light Mode (Procreate-inspired warm off-white)
            (Theme::Light, Self::Background) => Color::from_hex(0xF5F5F5),
            (Theme::Light, Self::Surface) => Color::from_hex(0xFFFFFF),
            (Theme::Light, Self::SurfaceElevated) => Color::from_hex(0xFFFFFF),
            (Theme::Light, Self::Border) => Color::from_hex(0xD0D0D0),
            (Theme::Light, Self::BorderEmphasis) => Color::from_hex(0x707070),
            (Theme::Light, Self::TextPrimary) => Color::from_hex(0x1A1A1A),
            (Theme::Light, Self::TextSecondary) => Color::from_hex(0x595959),
            (Theme::Light, Self::TextDisabled) => Color::from_hex(0x9E9E9E),
            (Theme::Light, Self::AccentPrimary) => Color::from_hex(0x0078D4),
            (Theme::Light, Self::AccentSecondary) => Color::from_hex(0x005A9E),
            (Theme::Light, Self::AccentTertiary) => Color::from_hex(0x003D6B),
            (Theme::Light, Self::Success) => Color::from_hex(0x2E7D32),
            (Theme::Light, Self::Warning) => Color::from_hex(0xE65100),
            (Theme::Light, Self::ErrorState) => Color::from_hex(0xC62828),
            (Theme::Light, Self::Info) => Color::from_hex(0x0277BD),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_white_contrast_is_21() {
        let black = Color::from_hex(0x000000);
        let white = Color::from_hex(0xFFFFFF);
        let ratio = black.contrast_ratio(&white);
        assert!((ratio - 21.0).abs() < 0.01, "expected 21.0, got {ratio}");
    }

    #[test]
    fn from_hex_alpha_round_trips() {
        let c = Color::from_hex_alpha(0x12_34_56_78);
        assert_eq!(c.r, 0x12);
        assert_eq!(c.g, 0x34);
        assert_eq!(c.b, 0x56);
        assert_eq!(c.a, 0x78);
    }

    #[test]
    fn from_hex_defaults_alpha_opaque() {
        let c = Color::from_hex(0xFF0000);
        assert_eq!(c.a, 0xFF);
    }

    /// **WCAG 2.2 Level AA gate** — text-on-surface contrast must
    /// be ≥ 4.5:1 in BOTH themes. This is the headline a11y promise
    /// of ADR-0023; if anyone tweaks a hex and breaks it, this test
    /// is the immediate bouncer.
    #[test]
    fn text_primary_on_surface_meets_aa_dark() {
        let bg = ColorToken::Surface.resolve(Theme::Dark);
        let fg = ColorToken::TextPrimary.resolve(Theme::Dark);
        let ratio = bg.contrast_ratio(&fg);
        assert!(
            ratio >= 4.5,
            "Dark text-primary on surface = {ratio:.2}:1, need ≥ 4.5"
        );
    }

    #[test]
    fn text_primary_on_surface_meets_aa_light() {
        let bg = ColorToken::Surface.resolve(Theme::Light);
        let fg = ColorToken::TextPrimary.resolve(Theme::Light);
        let ratio = bg.contrast_ratio(&fg);
        assert!(
            ratio >= 4.5,
            "Light text-primary on surface = {ratio:.2}:1, need ≥ 4.5"
        );
    }

    #[test]
    fn text_secondary_meets_aa_in_both_themes() {
        for theme in [Theme::Dark, Theme::Light] {
            let bg = ColorToken::Surface.resolve(theme);
            let fg = ColorToken::TextSecondary.resolve(theme);
            let ratio = bg.contrast_ratio(&fg);
            assert!(
                ratio >= 4.5,
                "{theme:?} text-secondary = {ratio:.2}:1, need ≥ 4.5"
            );
        }
    }

    #[test]
    fn border_emphasis_meets_aa_ui_in_both_themes() {
        // BorderEmphasis is used as focus ring. WCAG SC 1.4.11
        // (Non-text Contrast) = 3.0 minimum for UI components.
        for theme in [Theme::Dark, Theme::Light] {
            let bg = ColorToken::Surface.resolve(theme);
            let fg = ColorToken::BorderEmphasis.resolve(theme);
            let ratio = bg.contrast_ratio(&fg);
            assert!(
                ratio >= 3.0,
                "{theme:?} border-emphasis = {ratio:.2}:1, need ≥ 3.0 (WCAG 1.4.11)"
            );
        }
    }

    #[test]
    fn accent_primary_meets_ui_aa_in_both_themes() {
        for theme in [Theme::Dark, Theme::Light] {
            let bg = ColorToken::Surface.resolve(theme);
            let fg = ColorToken::AccentPrimary.resolve(theme);
            let ratio = bg.contrast_ratio(&fg);
            assert!(
                ratio >= 3.0,
                "{theme:?} accent-primary on surface = {ratio:.2}:1, need ≥ 3.0"
            );
        }
    }

    #[test]
    fn semantic_colors_meet_ui_aa() {
        for theme in [Theme::Dark, Theme::Light] {
            for token in [
                ColorToken::Success,
                ColorToken::Warning,
                ColorToken::ErrorState,
                ColorToken::Info,
            ] {
                let bg = ColorToken::Surface.resolve(theme);
                let fg = token.resolve(theme);
                let ratio = bg.contrast_ratio(&fg);
                assert!(
                    ratio >= 3.0,
                    "{theme:?} {token:?} = {ratio:.2}:1, need ≥ 3.0"
                );
            }
        }
    }
}
