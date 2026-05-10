//! Border-radius tokens. Source: `docs/design/tokens.json` → `radius.*`.
//!
//! Tier-based ("soft" default em todos os themes); valores aqui são o
//! tier soft. `full` (999) é círculo perfeito (avatares, pílulas).

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Radius {
    /// `xs` — 4 px (chips, micro-tags).
    Xs,
    /// `sm` — 6 px (buttons compactos, inputs densos).
    Sm,
    /// `md` — 8 px (default cards, panels internos).
    Md,
    /// `lg` — 12 px (floating panels, modais).
    Lg,
    /// `xl` — 16 px (hero cards, splash).
    Xl,
    /// `xl2` (`2xl` no JSON) — 20 px (large surfaces).
    Xl2,
    /// `full` — 999 px (círculo perfeito).
    Full,
}

impl Radius {
    pub const fn px(self) -> f32 {
        match self {
            Self::Xs => 4.0,
            Self::Sm => 6.0,
            Self::Md => 8.0,
            Self::Lg => 12.0,
            Self::Xl => 16.0,
            Self::Xl2 => 20.0,
            Self::Full => 999.0,
        }
    }

    pub const fn id(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
            Self::Xl2 => "2xl",
            Self::Full => "full",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_strictly_increasing_until_full() {
        let scale = [
            Radius::Xs,
            Radius::Sm,
            Radius::Md,
            Radius::Lg,
            Radius::Xl,
            Radius::Xl2,
        ];
        for w in scale.windows(2) {
            assert!(w[0].px() < w[1].px(), "{:?} → {:?}", w[0], w[1]);
        }
        assert!(Radius::Full.px() > Radius::Xl2.px());
    }
}
