//! Typography scale tokens. Source: `docs/design/tokens.json` →
//! `typography.*`.
//!
//! Pixel sizes assume 1.0 device pixel ratio; widget code multiplies
//! by `dpr` at render time. OS text-scaling override (up to 200 %) is
//! honored at the same step.
//!
//! Canonical families: **Inter** (variable sans, with `Inter Display`
//! for titles) + **JetBrains Mono** (mono). Fallbacks below follow
//! the design stack (Apple → BlinkMacSystemFont → system-ui).

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypeToken {
    /// `xxs` — 10 px (badges, micro-tooltips).
    Xxs,
    /// `xs` — 11 px (status bar, mini chips).
    Xs,
    /// `sm` — 12 px (sidebar labels, dense lists).
    Sm,
    /// `base` — 13 px (default body, inputs).
    Base,
    /// `md` — 15 px (panel section headers).
    Md,
    /// `lg` — 18 px (page subheaders).
    Lg,
    /// `xl` — 24 px (page headers).
    Xl,
    /// `xl2` (`2xl` in JSON) — 32 px (section heroes).
    Xl2,
    /// `xl3` (`3xl` in JSON) — 44 px (welcome screen hero).
    Xl3,
}

impl TypeToken {
    pub const fn px(self) -> f32 {
        match self {
            Self::Xxs => crate::generated::TYPOGRAPHY_SIZE_XXS,
            Self::Xs => crate::generated::TYPOGRAPHY_SIZE_XS,
            Self::Sm => crate::generated::TYPOGRAPHY_SIZE_SM,
            Self::Base => crate::generated::TYPOGRAPHY_SIZE_BASE,
            Self::Md => crate::generated::TYPOGRAPHY_SIZE_MD,
            Self::Lg => crate::generated::TYPOGRAPHY_SIZE_LG,
            Self::Xl => crate::generated::TYPOGRAPHY_SIZE_XL,
            Self::Xl2 => crate::generated::TYPOGRAPHY_SIZE_XL2,
            Self::Xl3 => crate::generated::TYPOGRAPHY_SIZE_XL3,
        }
    }

    /// Token id (matches JSON key).
    pub const fn id(self) -> &'static str {
        match self {
            Self::Xxs => "xxs",
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Base => "base",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
            Self::Xl2 => "2xl",
            Self::Xl3 => "3xl",
        }
    }
}

/// Font weight tokens (matches JSON `weight.*`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FontWeight {
    Regular,
    Medium,
    Semibold,
    Bold,
}

impl FontWeight {
    pub const fn value(self) -> u16 {
        match self {
            Self::Regular => crate::generated::TYPOGRAPHY_WEIGHT_REGULAR,
            Self::Medium => crate::generated::TYPOGRAPHY_WEIGHT_MEDIUM,
            Self::Semibold => crate::generated::TYPOGRAPHY_WEIGHT_SEMIBOLD,
            Self::Bold => crate::generated::TYPOGRAPHY_WEIGHT_BOLD,
        }
    }
}

/// Line-height multipliers (matches JSON `line.*`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LineHeight {
    Tight,
    Snug,
    Normal,
    Loose,
}

impl LineHeight {
    pub const fn ratio(self) -> f32 {
        match self {
            Self::Tight => crate::generated::TYPOGRAPHY_LINE_TIGHT,
            Self::Snug => crate::generated::TYPOGRAPHY_LINE_SNUG,
            Self::Normal => crate::generated::TYPOGRAPHY_LINE_NORMAL,
            Self::Loose => crate::generated::TYPOGRAPHY_LINE_LOOSE,
        }
    }
}

/// Letter-spacing tokens (matches JSON `track.*`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LetterSpacing {
    Tight,
    Normal,
    Wide,
    Caps,
}

impl LetterSpacing {
    /// Em units (matches CSS `letter-spacing: Xem`).
    pub const fn em(self) -> f32 {
        match self {
            Self::Tight => crate::generated::TYPOGRAPHY_TRACK_TIGHT,
            Self::Normal => crate::generated::TYPOGRAPHY_TRACK_NORMAL,
            Self::Wide => crate::generated::TYPOGRAPHY_TRACK_WIDE,
            Self::Caps => crate::generated::TYPOGRAPHY_TRACK_CAPS,
        }
    }
}

/// Font family chains. Strings constantes para uso direto em parley
/// font stack lookups (parley aceita comma-separated fallback chain).
pub const FONT_SANS: &str =
    "Inter, -apple-system, BlinkMacSystemFont, 'SF Pro Text', system-ui, sans-serif";
pub const FONT_DISPLAY: &str = "'Inter Display', Inter, system-ui, sans-serif";
pub const FONT_MONO: &str = "'JetBrains Mono', ui-monospace, 'SF Mono', Menlo, monospace";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_scale_strictly_increasing() {
        let scale = [
            TypeToken::Xxs,
            TypeToken::Xs,
            TypeToken::Sm,
            TypeToken::Base,
            TypeToken::Md,
            TypeToken::Lg,
            TypeToken::Xl,
            TypeToken::Xl2,
            TypeToken::Xl3,
        ];
        for w in scale.windows(2) {
            assert!(w[0].px() < w[1].px(), "{:?} → {:?}", w[0], w[1]);
        }
    }

    #[test]
    fn weight_values_match_css_standard() {
        assert_eq!(FontWeight::Regular.value(), 400);
        assert_eq!(FontWeight::Bold.value(), 700);
    }

    #[test]
    fn line_height_ratios_in_typography_norm() {
        assert!(LineHeight::Tight.ratio() < LineHeight::Normal.ratio());
        assert!(LineHeight::Normal.ratio() < LineHeight::Loose.ratio());
    }
}
