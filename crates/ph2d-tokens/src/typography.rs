//! Typography scale tokens. Source: `docs/design/tokens.json` →
//! `typography.*`.
//!
//! Pixel sizes assumem 1.0 device pixel ratio; widget code multiplica
//! por `dpr` em render time. OS text-scaling override (até 200 %) é
//! honrado no mesmo passo.
//!
//! Famílias canônicas: **Inter** (sans variable, com `Inter Display`
//! para titles) + **JetBrains Mono** (mono). Fallbacks abaixo seguem
//! a stack do design (Apple → BlinkMacSystemFont → system-ui).

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
    /// `xl2` (`2xl` no JSON) — 32 px (section heroes).
    Xl2,
    /// `xl3` (`3xl` no JSON) — 44 px (welcome screen hero).
    Xl3,
}

impl TypeToken {
    pub const fn px(self) -> f32 {
        match self {
            Self::Xxs => 10.0,
            Self::Xs => 11.0,
            Self::Sm => 12.0,
            Self::Base => 13.0,
            Self::Md => 15.0,
            Self::Lg => 18.0,
            Self::Xl => 24.0,
            Self::Xl2 => 32.0,
            Self::Xl3 => 44.0,
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
            Self::Regular => 400,
            Self::Medium => 500,
            Self::Semibold => 600,
            Self::Bold => 700,
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
            Self::Tight => 1.15,
            Self::Snug => 1.30,
            Self::Normal => 1.45,
            Self::Loose => 1.60,
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
            Self::Tight => -0.02,
            Self::Normal => 0.0,
            Self::Wide => 0.04,
            Self::Caps => 0.08,
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
