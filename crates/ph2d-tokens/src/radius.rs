//! Border-radius tokens. Source: `docs/design/tokens.json` → `radius.*`.
//!
//! Tier-based ("soft" default across all themes); values here are the
//! soft tier. `full` (999) yields a perfect circle (avatars, pills).
//!
//! Wave 4 stage A: values now come from `crate::generated::RADIUS_*`
//! consts (codegen'd by build.rs).

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
            Self::Xs => crate::generated::RADIUS_XS,
            Self::Sm => crate::generated::RADIUS_SM,
            Self::Md => crate::generated::RADIUS_MD,
            Self::Lg => crate::generated::RADIUS_LG,
            Self::Xl => crate::generated::RADIUS_XL,
            Self::Xl2 => crate::generated::RADIUS_XL2,
            Self::Full => crate::generated::RADIUS_FULL,
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
