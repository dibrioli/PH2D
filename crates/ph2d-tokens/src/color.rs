//! Color token system.
//!
//! Pipeline: OKLCH design tokens (`docs/design/tokens.json`) →
//! `Color::from_oklch` → sRGB 8-bit → consumed by widgets.
//!
//! ### Why OKLCH at the source
//!
//! - Perceptually uniform (same `L` ⇒ same perceived brightness across hues).
//! - Easy theme variants (keep structure, change `H`).
//! - WCAG contrast still computed in linear sRGB space (per spec).
//!
//! Source-of-truth lives in `docs/design/tokens.json`. This module mirrors
//! those values as Rust constants/expressions — changes there must be
//! reflected here (or via future codegen).

use crate::theme::Theme;

/// sRGB 8-bit color with alpha. Constructed via `from_hex` ou
/// `from_oklch`; stored as four bytes para cheap copy + tests.
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

    /// Construct from OKLCH (L 0..1, C 0..0.4-ish, H 0..360 degrees).
    /// Alpha defaults to opaque. Out-of-gamut colors are clamped to
    /// sRGB [0,1] without warning — drawing outside the gamut is a
    /// design problem, not a function problem.
    pub fn from_oklch(l: f64, c: f64, h_deg: f64) -> Self {
        let [r, g, b] = oklch_to_srgb(l, c, h_deg);
        Self { r, g, b, a: 0xFF }
    }

    /// Construct from OKLCH with alpha (0..1).
    pub fn from_oklch_alpha(l: f64, c: f64, h_deg: f64, alpha: f64) -> Self {
        let [r, g, b] = oklch_to_srgb(l, c, h_deg);
        Self {
            r,
            g,
            b,
            a: (alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
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

/// Convert OKLCH → sRGB 8-bit per Björn Ottosson's algorithm
/// (https://bottosson.github.io/posts/oklab/). Out-of-gamut colors
/// are clamped per channel.
///
/// `l` in 0..1, `c` ~0..0.4, `h_deg` in degrees.
pub fn oklch_to_srgb(l: f64, c: f64, h_deg: f64) -> [u8; 3] {
    // OKLCH → OKLAB
    let h_rad = h_deg.to_radians();
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();

    // OKLAB → linear LMS (cube)
    let l_ = l + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
    let m_ = l - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
    let s_ = l - 0.089_484_177_5 * a - 1.291_485_548_0 * b;

    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;

    // LMS → linear sRGB
    let lr = 4.076_741_662_1 * l3 - 3.307_711_591_3 * m3 + 0.230_969_929_2 * s3;
    let lg = -1.268_438_004_6 * l3 + 2.609_757_401_1 * m3 - 0.341_319_396_5 * s3;
    let lb = -0.004_196_086_3 * l3 - 0.703_418_614_7 * m3 + 1.707_614_701_0 * s3;

    [
        linear_to_srgb_byte(lr),
        linear_to_srgb_byte(lg),
        linear_to_srgb_byte(lb),
    ]
}

fn linear_to_srgb_byte(linear: f64) -> u8 {
    let x = linear.clamp(0.0, 1.0);
    let srgb = if x <= 0.003_130_8 {
        12.92 * x
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    };
    (srgb.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Semantic color slot — every widget references one of these by name;
/// literal `from_hex`/`from_oklch` outside this crate is a code smell.
///
/// Variants map 1:1 to `color.*` keys in `docs/design/tokens.json`.
/// Adding a new slot: edit tokens.json + add a variant here + add a
/// branch in `resolve` for each of the 4 themes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColorToken {
    // ── Background scale (4 levels + elevated + scrim) ─────────────
    /// `bg-0` — base canvas backdrop.
    Bg0,
    /// `bg-1` — first elevation (panels, sidebar).
    Bg1,
    /// `bg-2` — second elevation (cards inside panels).
    Bg2,
    /// `bg-3` — third elevation (input rows, list items).
    Bg3,
    /// `bg-elev` — popovers/tooltips/floating panels (slight alpha).
    BgElev,
    /// `bg-scrim` — modal backdrop (heavy alpha).
    BgScrim,

    // ── Borders (3 levels) ─────────────────────────────────────────
    /// `border` — low-contrast separators.
    Border,
    /// `border-strong` — visible dividers.
    BorderStrong,
    /// `border-emph` — focus rings, active selection borders.
    BorderEmph,

    // ── Text scale (3 levels + disabled) ───────────────────────────
    /// `text-1` — primary copy. ≥ 4.5:1 vs Bg1 (AA).
    Text1,
    /// `text-2` — labels, captions. ≥ 4.5:1 vs Bg1 (AA).
    Text2,
    /// `text-3` — tertiary/hints.
    Text3,
    /// `text-disabled` — explicit non-interactive (3:1 OK per WCAG).
    TextDisabled,

    // ── Accent stack ───────────────────────────────────────────────
    /// `accent` — primary call-to-action / active state.
    Accent,
    /// `accent-hover` — hover state.
    AccentHover,
    /// `accent-press` — pressed state.
    AccentPress,
    /// `accent-soft` — accent at low alpha (selection tints).
    AccentSoft,
    /// `accent-fg` — foreground on accent (contrast guaranteed).
    AccentFg,

    // ── Semantic states ────────────────────────────────────────────
    Danger,
    DangerSoft,
    Success,
    SuccessSoft,
    Warn,
    WarnSoft,
    Info,
    InfoSoft,

    // ── Editor-specific ────────────────────────────────────────────
    /// `selection` — selected entity highlight.
    Selection,
    /// `focus-ring` — keyboard focus indicator.
    FocusRing,
    /// `grid-line` — minor grid stroke on canvas.
    GridLine,
    /// `grid-axis` — major axis line on canvas.
    GridAxis,
    /// `canvas` — viewport background (scene render target backdrop).
    Canvas,
}

impl ColorToken {
    /// Resolve token → concrete Color for the given Theme.
    /// Values mirror `docs/design/tokens.json`; changing here without
    /// changing there causes divergence. Tests in this crate enforce
    /// the WCAG contrast invariants.
    pub fn resolve(self, theme: Theme) -> Color {
        // Per-theme tables. Each theme is its own block — repetition is
        // deliberate for readable diffs and exhaustive compiler matching.
        match theme {
            Theme::ForgeSdf => self.resolve_forge_sdf(),
            Theme::PaintStudio => self.resolve_paint_studio(),
            Theme::Sunstone => self.resolve_sunstone(),
            Theme::Blueprint => self.resolve_blueprint(),
        }
    }

    /// `forge-sdf` (default): dark + magenta accent.
    fn resolve_forge_sdf(self) -> Color {
        match self {
            Self::Bg0 => Color::from_oklch(0.135, 0.006, 285.0),
            Self::Bg1 => Color::from_oklch(0.170, 0.007, 285.0),
            Self::Bg2 => Color::from_oklch(0.205, 0.008, 285.0),
            Self::Bg3 => Color::from_oklch(0.250, 0.010, 285.0),
            Self::BgElev => Color::from_oklch_alpha(0.220, 0.008, 285.0, 0.86),
            Self::BgScrim => Color::from_oklch_alpha(0.080, 0.005, 285.0, 0.55),

            Self::Border => Color::from_oklch(0.295, 0.011, 285.0),
            Self::BorderStrong => Color::from_oklch(0.500, 0.018, 285.0),
            Self::BorderEmph => Color::from_oklch(0.560, 0.020, 285.0),

            Self::Text1 => Color::from_oklch(0.965, 0.004, 285.0),
            Self::Text2 => Color::from_oklch(0.745, 0.007, 285.0),
            Self::Text3 => Color::from_oklch(0.560, 0.009, 285.0),
            Self::TextDisabled => Color::from_oklch(0.420, 0.008, 285.0),

            Self::Accent => Color::from_oklch(0.740, 0.160, 340.0),
            Self::AccentHover => Color::from_oklch(0.790, 0.165, 340.0),
            Self::AccentPress => Color::from_oklch(0.690, 0.155, 340.0),
            Self::AccentSoft => Color::from_oklch_alpha(0.740, 0.160, 340.0, 0.16),
            Self::AccentFg => Color::from_oklch(0.150, 0.030, 340.0),

            Self::Danger => Color::from_oklch(0.660, 0.200, 25.0),
            Self::DangerSoft => Color::from_oklch_alpha(0.660, 0.200, 25.0, 0.16),
            Self::Success => Color::from_oklch(0.745, 0.140, 155.0),
            Self::SuccessSoft => Color::from_oklch_alpha(0.745, 0.140, 155.0, 0.16),
            Self::Warn => Color::from_oklch(0.800, 0.140, 80.0),
            Self::WarnSoft => Color::from_oklch_alpha(0.800, 0.140, 80.0, 0.16),
            Self::Info => Color::from_oklch(0.720, 0.120, 235.0),
            Self::InfoSoft => Color::from_oklch_alpha(0.720, 0.120, 235.0, 0.16),

            Self::Selection => Color::from_oklch_alpha(0.740, 0.160, 340.0, 0.24),
            Self::FocusRing => Color::from_oklch_alpha(0.740, 0.160, 340.0, 0.55),
            Self::GridLine => Color::from_oklch_alpha(1.0, 0.0, 0.0, 0.04),
            Self::GridAxis => Color::from_oklch_alpha(0.740, 0.160, 340.0, 0.32),
            Self::Canvas => Color::from_oklch(0.105, 0.004, 285.0),
        }
    }

    /// `paint-studio`: inherits `forge-sdf` structure, only accent
    /// stack changes (cyan).
    fn resolve_paint_studio(self) -> Color {
        match self {
            Self::Accent => Color::from_oklch(0.780, 0.140, 205.0),
            Self::AccentHover => Color::from_oklch(0.820, 0.140, 205.0),
            Self::AccentPress => Color::from_oklch(0.730, 0.140, 205.0),
            Self::AccentSoft => Color::from_oklch_alpha(0.780, 0.140, 205.0, 0.16),
            Self::Selection => Color::from_oklch_alpha(0.780, 0.140, 205.0, 0.24),
            Self::FocusRing => Color::from_oklch_alpha(0.780, 0.140, 205.0, 0.55),
            Self::GridAxis => Color::from_oklch_alpha(0.780, 0.140, 205.0, 0.32),
            // Everything else inherits forge-sdf.
            other => other.resolve_forge_sdf(),
        }
    }

    /// `sunstone`: light + warm orange. Redefine surfaces, text, accent.
    fn resolve_sunstone(self) -> Color {
        match self {
            Self::Bg0 => Color::from_oklch(0.985, 0.006, 75.0),
            Self::Bg1 => Color::from_oklch(0.965, 0.008, 75.0),
            Self::Bg2 => Color::from_oklch(0.940, 0.010, 75.0),
            Self::Bg3 => Color::from_oklch(0.910, 0.014, 75.0),
            Self::BgElev => Color::from_oklch_alpha(0.985, 0.006, 75.0, 0.92),
            Self::BgScrim => Color::from_oklch_alpha(0.220, 0.014, 75.0, 0.40),

            Self::Border => Color::from_oklch(0.870, 0.016, 75.0),
            Self::BorderStrong => Color::from_oklch(0.640, 0.018, 75.0),
            Self::BorderEmph => Color::from_oklch(0.500, 0.020, 75.0),

            Self::Text1 => Color::from_oklch(0.220, 0.014, 75.0),
            Self::Text2 => Color::from_oklch(0.420, 0.012, 75.0),
            Self::Text3 => Color::from_oklch(0.560, 0.010, 75.0),
            Self::TextDisabled => Color::from_oklch(0.700, 0.008, 75.0),

            Self::Accent => Color::from_oklch(0.560, 0.190, 55.0),
            Self::AccentHover => Color::from_oklch(0.610, 0.195, 55.0),
            Self::AccentPress => Color::from_oklch(0.510, 0.185, 55.0),
            Self::AccentSoft => Color::from_oklch_alpha(0.560, 0.190, 55.0, 0.16),
            Self::AccentFg => Color::from_oklch(0.985, 0.030, 55.0),

            Self::Warn => Color::from_oklch(0.560, 0.160, 80.0),
            Self::Selection => Color::from_oklch_alpha(0.560, 0.190, 55.0, 0.24),
            Self::FocusRing => Color::from_oklch_alpha(0.560, 0.190, 55.0, 0.55),
            Self::GridLine => Color::from_oklch_alpha(0.0, 0.0, 0.0, 0.04),
            Self::GridAxis => Color::from_oklch_alpha(0.560, 0.190, 55.0, 0.32),
            Self::Canvas => Color::from_oklch(0.945, 0.012, 75.0),

            // Semantic states inherit forge-sdf chroma but shift L for
            // light surface. Per audit.md sunstone tunes accent/warn
            // explicitly; rest uses sensible darker variants.
            Self::Danger => Color::from_oklch(0.560, 0.200, 25.0),
            Self::DangerSoft => Color::from_oklch_alpha(0.560, 0.200, 25.0, 0.16),
            Self::Success => Color::from_oklch(0.520, 0.140, 155.0),
            Self::SuccessSoft => Color::from_oklch_alpha(0.520, 0.140, 155.0, 0.16),
            Self::WarnSoft => Color::from_oklch_alpha(0.560, 0.160, 80.0, 0.16),
            Self::Info => Color::from_oklch(0.520, 0.140, 235.0),
            Self::InfoSoft => Color::from_oklch_alpha(0.520, 0.140, 235.0, 0.16),
        }
    }

    /// `blueprint`: light + cool blue (CAD vibe). Sidebar layout flag
    /// is read from `Theme::panel_layout`, not from color resolve.
    fn resolve_blueprint(self) -> Color {
        match self {
            Self::Bg0 => Color::from_oklch(0.975, 0.008, 250.0),
            Self::Bg1 => Color::from_oklch(0.955, 0.010, 250.0),
            Self::Bg2 => Color::from_oklch(0.925, 0.014, 250.0),
            Self::Bg3 => Color::from_oklch(0.895, 0.016, 250.0),
            Self::BgElev => Color::from_oklch_alpha(0.975, 0.008, 250.0, 0.92),
            Self::BgScrim => Color::from_oklch_alpha(0.220, 0.020, 250.0, 0.40),

            Self::Border => Color::from_oklch(0.860, 0.018, 250.0),
            Self::BorderStrong => Color::from_oklch(0.620, 0.020, 250.0),
            Self::BorderEmph => Color::from_oklch(0.480, 0.022, 250.0),

            Self::Text1 => Color::from_oklch(0.220, 0.020, 250.0),
            Self::Text2 => Color::from_oklch(0.420, 0.018, 250.0),
            Self::Text3 => Color::from_oklch(0.560, 0.014, 250.0),
            Self::TextDisabled => Color::from_oklch(0.700, 0.012, 250.0),

            Self::Accent => Color::from_oklch(0.500, 0.180, 250.0),
            Self::AccentHover => Color::from_oklch(0.555, 0.185, 250.0),
            Self::AccentPress => Color::from_oklch(0.450, 0.175, 250.0),
            Self::AccentSoft => Color::from_oklch_alpha(0.500, 0.180, 250.0, 0.16),
            Self::AccentFg => Color::from_oklch(0.985, 0.020, 250.0),

            Self::Warn => Color::from_oklch(0.560, 0.160, 80.0),
            Self::Selection => Color::from_oklch_alpha(0.500, 0.180, 250.0, 0.24),
            Self::FocusRing => Color::from_oklch_alpha(0.500, 0.180, 250.0, 0.55),
            Self::GridLine => Color::from_oklch_alpha(0.0, 0.0, 0.0, 0.06),
            Self::GridAxis => Color::from_oklch_alpha(0.500, 0.180, 250.0, 0.36),
            Self::Canvas => Color::from_oklch(0.940, 0.016, 250.0),

            Self::Danger => Color::from_oklch(0.560, 0.200, 25.0),
            Self::DangerSoft => Color::from_oklch_alpha(0.560, 0.200, 25.0, 0.16),
            Self::Success => Color::from_oklch(0.520, 0.140, 155.0),
            Self::SuccessSoft => Color::from_oklch_alpha(0.520, 0.140, 155.0, 0.16),
            Self::WarnSoft => Color::from_oklch_alpha(0.560, 0.160, 80.0, 0.16),
            Self::Info => Color::from_oklch(0.520, 0.140, 235.0),
            Self::InfoSoft => Color::from_oklch_alpha(0.520, 0.140, 235.0, 0.16),
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

    #[test]
    fn oklch_white_round_trips_to_white_ish() {
        // L=1, C=0 should yield pure white in sRGB.
        let [r, g, b] = oklch_to_srgb(1.0, 0.0, 0.0);
        assert_eq!((r, g, b), (255, 255, 255));
    }

    #[test]
    fn oklch_black_round_trips_to_black() {
        // L=0, C=0 should yield pure black in sRGB.
        let [r, g, b] = oklch_to_srgb(0.0, 0.0, 0.0);
        assert_eq!((r, g, b), (0, 0, 0));
    }

    #[test]
    fn oklch_mid_gray_is_neutral() {
        // C=0 always yields R=G=B (achromatic). L=0.5 in OKLAB maps to
        // sRGB ~99 (perceptually "half of white" — not 128, because
        // OKLAB is perceptually uniform, not linear in sRGB).
        let [r, g, b] = oklch_to_srgb(0.5, 0.0, 0.0);
        assert_eq!(r, g);
        assert_eq!(g, b);
        assert!((90..=110).contains(&r), "expected ~99, got {r}");
    }

    /// **WCAG 2.2 AA gate** — text-on-bg1 contrast ≥ 4.5:1 across the 4 themes.
    #[test]
    fn text1_on_bg1_meets_aa_in_all_themes() {
        for theme in [
            Theme::ForgeSdf,
            Theme::PaintStudio,
            Theme::Sunstone,
            Theme::Blueprint,
        ] {
            let bg = ColorToken::Bg1.resolve(theme);
            let fg = ColorToken::Text1.resolve(theme);
            let ratio = bg.contrast_ratio(&fg);
            assert!(
                ratio >= 4.5,
                "{theme:?}: text-1 on bg-1 = {ratio:.2}:1, need ≥ 4.5"
            );
        }
    }

    #[test]
    fn text2_on_bg1_meets_aa_in_all_themes() {
        for theme in [
            Theme::ForgeSdf,
            Theme::PaintStudio,
            Theme::Sunstone,
            Theme::Blueprint,
        ] {
            let bg = ColorToken::Bg1.resolve(theme);
            let fg = ColorToken::Text2.resolve(theme);
            let ratio = bg.contrast_ratio(&fg);
            assert!(
                ratio >= 4.5,
                "{theme:?}: text-2 on bg-1 = {ratio:.2}:1, need ≥ 4.5"
            );
        }
    }

    /// WCAG SC 1.4.11 — non-text UI components (focus rings, borders).
    #[test]
    fn border_emph_meets_ui_aa_in_all_themes() {
        for theme in [
            Theme::ForgeSdf,
            Theme::PaintStudio,
            Theme::Sunstone,
            Theme::Blueprint,
        ] {
            let bg = ColorToken::Bg1.resolve(theme);
            let fg = ColorToken::BorderEmph.resolve(theme);
            let ratio = bg.contrast_ratio(&fg);
            assert!(
                ratio >= 3.0,
                "{theme:?}: border-emph on bg-1 = {ratio:.2}:1, need ≥ 3.0"
            );
        }
    }

    #[test]
    fn accent_meets_ui_aa_in_all_themes() {
        for theme in [
            Theme::ForgeSdf,
            Theme::PaintStudio,
            Theme::Sunstone,
            Theme::Blueprint,
        ] {
            let bg = ColorToken::Bg1.resolve(theme);
            let fg = ColorToken::Accent.resolve(theme);
            let ratio = bg.contrast_ratio(&fg);
            assert!(
                ratio >= 3.0,
                "{theme:?}: accent on bg-1 = {ratio:.2}:1, need ≥ 3.0"
            );
        }
    }
}
