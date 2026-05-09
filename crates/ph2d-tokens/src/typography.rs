//! Typography scale tokens. Per ADR-0023 §12.
//!
//! Pixel sizes assume 1.0 device pixel ratio; widget code multiplies
//! by `dpr` at render time. OS text-scaling override (up to 200 %)
//! is honored at the same multiplication step.

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypeToken {
    /// 11 px — badges, micro-tooltips.
    Xs,
    /// 13 px — sidebar labels, secondary text.
    Sm,
    /// 14 px — default body.
    Base,
    /// 16 px — primary text, inputs.
    Md,
    /// 20 px — panel headers.
    Lg,
    /// 24 px — page headers.
    Xl,
    /// 13 px monospace — paths, IDs, code.
    Mono,
}

impl TypeToken {
    pub const fn px(self) -> f32 {
        match self {
            Self::Xs => 11.0,
            Self::Sm => 13.0,
            Self::Base => 14.0,
            Self::Md => 16.0,
            Self::Lg => 20.0,
            Self::Xl => 24.0,
            Self::Mono => 13.0,
        }
    }

    /// True when the token defaults to a monospace font family.
    pub const fn is_mono(self) -> bool {
        matches!(self, Self::Mono)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_is_strictly_increasing() {
        // Xs < Sm < Base < Md < Lg < Xl. Mono shares Sm size; not in
        // the monotone chain.
        assert!(TypeToken::Xs.px() < TypeToken::Sm.px());
        assert!(TypeToken::Sm.px() < TypeToken::Base.px());
        assert!(TypeToken::Base.px() < TypeToken::Md.px());
        assert!(TypeToken::Md.px() < TypeToken::Lg.px());
        assert!(TypeToken::Lg.px() < TypeToken::Xl.px());
    }

    #[test]
    fn mono_is_thirteen_px_and_flagged() {
        assert_eq!(TypeToken::Mono.px(), 13.0);
        assert!(TypeToken::Mono.is_mono());
        assert!(!TypeToken::Sm.is_mono());
    }
}
