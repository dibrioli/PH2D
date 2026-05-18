//! Parameters for each background-removal backend.
//!
//! All defaults are tuned for the "give me a reasonable result on the first
//! click" case. Power users tweak via the panel sliders / toggles.

/// Which primary segmentation backend to run.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum BgRemovalMode {
    /// Chroma key in Oklab + corner-auto background detection + connected
    /// flood-fill from image borders. Fast (~50-150 ms on 4k); ideal for
    /// uniform / solid backgrounds.
    #[default]
    Chroma,
    /// GrabCut (Rother 2004) on a downsampled 1024² image; mask upsampled
    /// via nearest, refined by Guided Filter post-process. Slow but robust
    /// for natural images with textured backgrounds.
    GrabCut,
}

impl BgRemovalMode {
    pub fn label(self) -> &'static str {
        match self {
            BgRemovalMode::Chroma => "Chroma",
            BgRemovalMode::GrabCut => "Smart Cut",
        }
    }

    pub fn all() -> [BgRemovalMode; 2] {
        [BgRemovalMode::Chroma, BgRemovalMode::GrabCut]
    }
}

/// Tuning parameters for `algorithm::chroma::segment`.
#[derive(Copy, Clone, Debug)]
pub struct ChromaParams {
    /// Color-distance threshold in Oklab units. Pixels within `tolerance`
    /// of the detected background color are marked as background (after
    /// the connectivity step). Typical range 0.02..0.30; default 0.10.
    pub tolerance: f32,
    /// Width of the soft-alpha transition band, in Oklab units, *added*
    /// to `tolerance`. Pixels with ΔE in `[tolerance, tolerance + feather]`
    /// receive fractional alpha. Decoupled from `tolerance` so sharpness
    /// and softness tune independently. Range 0.0..0.20; default 0.04.
    pub feather: f32,
    /// If `Some`, override corner-auto detection and use this RGB color
    /// (sRGB 8-bit) as the background reference. `None` = auto-detect
    /// from 4 corners via mini k-means (k=2).
    pub reference_color: Option<[u8; 3]>,
    /// When true, subtract the background's chroma from soft-band pixels
    /// to remove color halos common to greenscreen-style backgrounds.
    pub despill: bool,
    /// When true, seed flood-fill from image borders and only kill
    /// connected bg-similar regions (prevents bleed-through into subject
    /// interiors that share the bg color). Auto-disabled if border-bg
    /// confidence < 60% (subject-touches-border case).
    pub use_flood: bool,
}

impl Default for ChromaParams {
    fn default() -> Self {
        Self {
            tolerance: 0.10,
            feather: 0.04,
            reference_color: None,
            despill: true,
            use_flood: true,
        }
    }
}

/// Tuning parameters for `algorithm::grabcut::segment`.
#[derive(Copy, Clone, Debug)]
pub struct GrabCutParams {
    /// Inset of the assumed-foreground rectangle, as a fraction of image
    /// dimensions, per side. Default 0.05 on all sides = the foreground
    /// is the central 90×90% of the canvas.
    pub inset_top: f32,
    pub inset_right: f32,
    pub inset_bottom: f32,
    pub inset_left: f32,
    /// Maximum GrabCut iterations. Algorithm may stop earlier if the
    /// mask-flip ratio between iters falls below 0.1%. Range 1..5;
    /// default 2.
    pub max_iters: u32,
    /// When true, input pixels with alpha < 128 are treated as hard
    /// background (`GC_BGD`) — useful when the sprite already has alpha
    /// holes (e.g. cleanup pass after a previous removal).
    pub alpha_hole_as_bg: bool,
}

impl Default for GrabCutParams {
    fn default() -> Self {
        Self {
            inset_top: 0.05,
            inset_right: 0.05,
            inset_bottom: 0.05,
            inset_left: 0.05,
            max_iters: 2,
            alpha_hole_as_bg: true,
        }
    }
}

/// Tuning parameters for `algorithm::guided_filter::refine`.
#[derive(Copy, Clone, Debug)]
pub struct GuidedFilterParams {
    /// Radius of the guided-filter window, in pixels at full resolution.
    /// Mapped to the UX slider "Feather Radius" 1..100. Default 30. Set
    /// to 0 to disable refinement entirely (hard binary mask output).
    pub radius: u32,
    /// Regularization term ε. Smaller = edges held tighter to the guide;
    /// larger = smoother. He's matting paper uses 1e-7 with [0,1]-normalized
    /// inputs. Range 1e-9..1e-3 (log-scale slider); default 1e-7.
    pub epsilon: f32,
    /// When true, use the full RGB guide (per-pixel 3×3 covariance solve).
    /// When false, use a luma-only guide (~3× faster). Color guide gives
    /// meaningfully better edges where bg and fg are chromatically similar
    /// but luminance-distinct (red hair on orange skin, green shirt on
    /// grass, etc.).
    pub color_guide: bool,
    /// When true, only process pixels within ±2r of the mask boundary;
    /// interior is copied verbatim. 5-10× speedup on typical sprite masks
    /// (mostly-solid interior, thin boundary). Disable only for debugging.
    pub boundary_only: bool,
}

impl Default for GuidedFilterParams {
    fn default() -> Self {
        Self {
            radius: 30,
            epsilon: 1e-7,
            color_guide: true,
            boundary_only: true,
        }
    }
}

/// Top-level parameter bag passed into [`super::algorithm::run_pipeline`].
#[derive(Clone, Debug, Default)]
pub struct BgRemovalParams {
    pub mode: BgRemovalMode,
    pub chroma: ChromaParams,
    pub grabcut: GrabCutParams,
    pub refinement: GuidedFilterParams,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_align_with_design_doc() {
        let p = BgRemovalParams::default();
        assert_eq!(p.mode, BgRemovalMode::Chroma);
        assert!((p.chroma.tolerance - 0.10).abs() < 1e-6);
        assert!((p.chroma.feather - 0.04).abs() < 1e-6);
        assert_eq!(p.chroma.reference_color, None);
        assert!(p.chroma.despill);
        assert!(p.chroma.use_flood);
        assert_eq!(p.grabcut.max_iters, 2);
        assert!((p.grabcut.inset_top - 0.05).abs() < 1e-6);
        assert!(p.grabcut.alpha_hole_as_bg);
        assert!((p.refinement.epsilon - 1e-7).abs() < 1e-9);
        assert_eq!(p.refinement.radius, 30);
        assert!(p.refinement.color_guide);
        assert!(p.refinement.boundary_only);
    }

    #[test]
    fn mode_all_labels_nonempty() {
        for m in BgRemovalMode::all() {
            assert!(!m.label().is_empty());
        }
    }

    #[test]
    fn mode_default_is_chroma() {
        assert_eq!(BgRemovalMode::default(), BgRemovalMode::Chroma);
    }
}
