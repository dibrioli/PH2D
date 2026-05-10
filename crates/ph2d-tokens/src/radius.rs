//! Border-radius tokens. Source: `docs/design/tokens.json` → `radius.*`.
//!
//! Tier-based ("soft" default across all themes); values here are the
//! soft tier. `full` (999) yields a perfect circle (avatars, pills).

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Radius {
    /// `xs` — 4 px (chips, micro-tags).
    Xs,
    /// `sm` — 6 px (compact buttons, dense inputs).
    Sm,
    /// `md` — 8 px (default cards, inner panels).
    Md,
    /// `lg` — 12 px (floating panels, modals).
    Lg,
    /// `xl` — 16 px (hero cards, splash).
    Xl,
    /// `xl2` (`2xl` in JSON) — 20 px (large surfaces).
    Xl2,
    /// `full` — 999 px (perfect circle).
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
