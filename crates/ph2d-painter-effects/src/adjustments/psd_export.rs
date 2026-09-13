//! A **classificação de exportação PSD** congelada (ADR-0045 §2.8) — o [`PsdExport`] e o
//! `AdjustmentKind::psd_export` —, irmã de `adjustments/mod.rs` por tecto de LOC. O caminho público
//! não muda: `mod.rs` re-exporta o `PsdExport`.
//!
//! Corte mecânico: os dois itens saíram inteiros, verbatim.

use super::*;

/// PSD export classification — frozen mapping table (ADR-0045 §2.8). A
/// [`PsdExport::Layered`] kind round-trips as a native PSD adjustment layer
/// (the 4-char type key); a [`PsdExport::Baked`] kind has no PSD layer
/// equivalent and is rasterized into the pixel data on export (W16).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PsdExport {
    /// 1:1 native PSD adjustment layer with this 4-char type key.
    Layered(&'static str),
    /// No PSD layer equivalent — baked into pixels on export.
    Baked,
}

impl AdjustmentKind {
    /// The frozen PSD interop mapping (§2.8). Every kind maps to exactly one
    /// [`PsdExport`]; v1 = 16 layered (1:1) + 8 baked.
    #[must_use]
    pub fn psd_export(self) -> PsdExport {
        use AdjustmentKind::*;
        match self {
            // Tier 1 — 5 layered, 7 baked.
            HueSaturationBrightness => PsdExport::Layered("hsbr"),
            ColorBalance => PsdExport::Layered("cobl"),
            Curves => PsdExport::Layered("curv"),
            GradientMap => PsdExport::Layered("grdm"),
            BrightnessContrast => PsdExport::Layered("brit"),
            GaussianBlur | MotionBlur | Bloom | Noise | Sharpen | Halftone
            | ChromaticAberration => PsdExport::Baked,
            // Tier 2 — 11 layered, 1 baked.
            Vibrance => PsdExport::Layered("vibA"),
            ColorLookupLut => PsdExport::Layered("clrL"),
            PhotoFilter => PsdExport::Layered("phfl"),
            Posterize => PsdExport::Layered("post"),
            Threshold => PsdExport::Layered("thrs"),
            Invert => PsdExport::Layered("nvrt"),
            Levels => PsdExport::Layered("levl"),
            SelectiveColor => PsdExport::Layered("selc"),
            ChannelMixer => PsdExport::Layered("mixr"),
            Exposure => PsdExport::Layered("expA"),
            ShadowsHighlights => PsdExport::Baked,
            BlackAndWhite => PsdExport::Layered("blwh"),
        }
    }
}
