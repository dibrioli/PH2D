//! Parameters + algorithm enum + sampled color type.
//!
//! Mirrors the legacy `BgRemovalSettings` 1:1 in semantics. Ranges are
//! documented per field so the Integrator can build any UI (slider /
//! number-input / spinner) that converts to the canonical scale here.

/// Which masking algorithm to run.
///
/// `Auto` resolves to `ColorKey` with auto-detected border colors;
/// kept as a distinct variant so the Integrator can surface it as a
/// "smart default" option in the UI.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum BgRemovalAlgorithm {
    #[default]
    Auto,
    ColorKey,
    EdgeAware,
    Luminance,
}

impl BgRemovalAlgorithm {
    /// String tag stable for serialization, RadioGroup options, and
    /// UI bindings. Lowercase, no spaces — matches the legacy TS API
    /// so existing project files port without churn.
    pub fn tag(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::ColorKey => "colorkey",
            Self::EdgeAware => "edge",
            Self::Luminance => "luminance",
        }
    }

    /// Inverse of `tag()`. Unknown tags fall back to `Auto` (matches
    /// legacy "be lenient at the boundary" UX).
    pub fn from_tag(tag: &str) -> Self {
        match tag {
            "colorkey" => Self::ColorKey,
            "edge" => Self::EdgeAware,
            "luminance" => Self::Luminance,
            _ => Self::Auto,
        }
    }
}

/// 8-bit sRGB color sample. Used for eyedropper samples and as the
/// canonical wire-format input to k-means border detection.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// All knobs that govern one `apply()` invocation. Field semantics
/// kept identical to the legacy engine so port artifacts match.
#[derive(Clone, Debug, PartialEq)]
pub struct BgRemovalParams {
    pub algorithm: BgRemovalAlgorithm,

    /// Color distance threshold. Range 0..=100 (0 = exact match only,
    /// 100 = everything matches). Maps to a perceptual distance in
    /// OKLab (see `colorkey`) — not raw sRGB Euclidean.
    pub tolerance: f32,

    /// Edge sensitivity for `EdgeAware`. Range 0..=100 (higher =
    /// flood fill stops on weaker edges).
    pub edge_threshold: f32,

    /// Feather width in pixels. Range 0..=20 (0 = no feathering).
    pub feather_width: f32,

    /// Feather strength. Range 0..=100 (0 = no effect, 100 = full
    /// smoothstep falloff).
    pub feather_strength: f32,

    /// Guided-filter smoothing iterations. Range 0..=10 (0 = off).
    /// Edge-preserving — does not blur the silhouette.
    pub smooth_amount: f32,

    /// Morphological mask expand (+) / contract (-). Range -5..=5.
    pub mask_expand: f32,

    /// Auto opening+closing pre-pass against salt-and-pepper noise.
    /// Off by default (matches legacy behavior); enabling cleans up
    /// noisy sources without the user touching `mask_expand`.
    pub auto_clean: bool,

    /// Swap foreground/background.
    pub invert_mask: bool,

    /// User-sampled background colors (eyedropper output). Empty →
    /// k-means auto-detect on border band.
    pub sampled_colors: Vec<RgbColor>,
}

impl Default for BgRemovalParams {
    fn default() -> Self {
        Self {
            algorithm: BgRemovalAlgorithm::Auto,
            tolerance: 30.0,
            edge_threshold: 50.0,
            feather_width: 2.0,
            feather_strength: 100.0,
            smooth_amount: 0.0,
            mask_expand: 0.0,
            auto_clean: false,
            invert_mask: false,
            sampled_colors: Vec::new(),
        }
    }
}

impl BgRemovalParams {
    /// Clamp every numeric field into its documented range. Cheap to
    /// call repeatedly — the Integrator can invoke after every UI
    /// event without bookkeeping.
    pub fn clamp(&mut self) {
        self.tolerance = self.tolerance.clamp(0.0, 100.0);
        self.edge_threshold = self.edge_threshold.clamp(0.0, 100.0);
        self.feather_width = self.feather_width.clamp(0.0, 20.0);
        self.feather_strength = self.feather_strength.clamp(0.0, 100.0);
        self.smooth_amount = self.smooth_amount.clamp(0.0, 10.0);
        self.mask_expand = self.mask_expand.clamp(-5.0, 5.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algorithm_tag_roundtrip() {
        for a in [
            BgRemovalAlgorithm::Auto,
            BgRemovalAlgorithm::ColorKey,
            BgRemovalAlgorithm::EdgeAware,
            BgRemovalAlgorithm::Luminance,
        ] {
            assert_eq!(BgRemovalAlgorithm::from_tag(a.tag()), a);
        }
    }

    #[test]
    fn unknown_tag_falls_back_to_auto() {
        assert_eq!(
            BgRemovalAlgorithm::from_tag("nope"),
            BgRemovalAlgorithm::Auto
        );
    }

    #[test]
    fn defaults_match_legacy_baseline() {
        let p = BgRemovalParams::default();
        assert_eq!(p.algorithm, BgRemovalAlgorithm::Auto);
        assert_eq!(p.tolerance, 30.0);
        assert_eq!(p.edge_threshold, 50.0);
        assert_eq!(p.feather_width, 2.0);
        assert_eq!(p.feather_strength, 100.0);
        assert_eq!(p.smooth_amount, 0.0);
        assert_eq!(p.mask_expand, 0.0);
        assert!(!p.invert_mask);
        assert!(!p.auto_clean);
        assert!(p.sampled_colors.is_empty());
    }

    #[test]
    fn clamp_pulls_out_of_range_back_in() {
        let mut p = BgRemovalParams {
            tolerance: 200.0,
            feather_width: -5.0,
            mask_expand: 99.0,
            ..Default::default()
        };
        p.clamp();
        assert_eq!(p.tolerance, 100.0);
        assert_eq!(p.feather_width, 0.0);
        assert_eq!(p.mask_expand, 5.0);
    }
}
