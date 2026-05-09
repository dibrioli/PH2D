//! Spacing scale tokens. Per ADR-0023 §12: múltiplos de 4 px.

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Spacing {
    /// 0 — no gap.
    None,
    /// 4 px — tight inline padding (icon ↔ label).
    Xxs,
    /// 8 px — standard inline padding.
    Xs,
    /// 12 px — default vertical rhythm.
    Sm,
    /// 16 px — comfortable padding.
    Md,
    /// 24 px — section separation.
    Lg,
    /// 32 px — major panel margin.
    Xl,
}

impl Spacing {
    pub const fn px(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Xxs => 4.0,
            Self::Xs => 8.0,
            Self::Sm => 12.0,
            Self::Md => 16.0,
            Self::Lg => 24.0,
            Self::Xl => 32.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_is_strictly_increasing() {
        assert!(Spacing::None.px() < Spacing::Xxs.px());
        assert!(Spacing::Xxs.px() < Spacing::Xs.px());
        assert!(Spacing::Xs.px() < Spacing::Sm.px());
        assert!(Spacing::Sm.px() < Spacing::Md.px());
        assert!(Spacing::Md.px() < Spacing::Lg.px());
        assert!(Spacing::Lg.px() < Spacing::Xl.px());
    }

    #[test]
    fn all_values_are_multiples_of_four() {
        for s in [
            Spacing::None,
            Spacing::Xxs,
            Spacing::Xs,
            Spacing::Sm,
            Spacing::Md,
            Spacing::Lg,
            Spacing::Xl,
        ] {
            assert_eq!(
                s.px() % 4.0,
                0.0,
                "spacing {s:?} = {} px must be multiple of 4 (ADR-0023 §12)",
                s.px()
            );
        }
    }
}
