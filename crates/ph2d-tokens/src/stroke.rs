//! Stroke-width tokens. Source: `docs/design/tokens.json` → `stroke.*`.
//!
//! Wave 4 stage A — values codegen'd via `build.rs` into
//! `crate::generated::STROKE_*` consts. Designer edits the JSON;
//! widget code reaches widths via `StrokeToken::X.px()` (never
//! `Stroke::new(1.5)` literal in widget/screens — that path is
//! gated by `crates/ph2d-editor/tests/no_magic_numeric.rs`).
//!
//! Kurbo's `Stroke::new` takes `f64`; `StrokeToken::px()` returns
//! `f32`. Use `.px() as f64` cast at call sites:
//!
//! ```ignore
//! use ph2d_tokens::StrokeToken;
//! use ph2d_vector::Stroke;
//!
//! let s = Stroke::new(StrokeToken::Default.px() as f64);
//! ```

/// Semantic stroke-width slot. Variants map 1:1 to `stroke.*` keys
/// in `docs/design/tokens.json`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StrokeToken {
    /// `hairline` — 0.5 px (sub-pixel dividers; antialias-aware).
    Hairline,
    /// `thin` — 1.0 px (default border, separator).
    Thin,
    /// `default` — 1.5 px (button outline, focus ring).
    Default,
    /// `thick` — 2.0 px (selection emphasis).
    Thick,
    /// `heavy` — 3.0 px (drag overlay, pressed state).
    Heavy,
}

impl StrokeToken {
    /// Stroke width in CSS pixels (1.0 device pixel ratio assumed;
    /// renderer multiplies by `dpr`).
    pub const fn px(self) -> f32 {
        match self {
            Self::Hairline => crate::generated::STROKE_HAIRLINE,
            Self::Thin => crate::generated::STROKE_THIN,
            Self::Default => crate::generated::STROKE_DEFAULT,
            Self::Thick => crate::generated::STROKE_THICK,
            Self::Heavy => crate::generated::STROKE_HEAVY,
        }
    }

    /// Token id (matches JSON key).
    pub const fn id(self) -> &'static str {
        match self {
            Self::Hairline => "hairline",
            Self::Thin => "thin",
            Self::Default => "default",
            Self::Thick => "thick",
            Self::Heavy => "heavy",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_strictly_increasing() {
        let scale = [
            StrokeToken::Hairline,
            StrokeToken::Thin,
            StrokeToken::Default,
            StrokeToken::Thick,
            StrokeToken::Heavy,
        ];
        for w in scale.windows(2) {
            assert!(w[0].px() < w[1].px(), "{:?} → {:?}", w[0], w[1]);
        }
    }

    #[test]
    fn ids_match_tokens_json() {
        assert_eq!(StrokeToken::Hairline.id(), "hairline");
        assert_eq!(StrokeToken::Default.id(), "default");
        assert_eq!(StrokeToken::Heavy.id(), "heavy");
    }
}
