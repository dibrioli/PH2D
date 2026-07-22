//! The wet session's **per-stroke style** (child of [`super`], split for the
//! workspace file-LOC cap): the wash params captured at each stroke's pen-down
//! ([`WetStrokeStyle`]) + the session table and per-pixel owner map
//! ([`WetSessionStyles`]). The field math, samplers and settle constants stay
//! in the parent — this file is the one place a wash's captured identity lives.

use super::{SPREAD_THIN_MAX, SPREAD_THIN_REF};

/// EDGE-1 **per-stroke style** (doc 13 topo, Enio 2026-07-09): the wash params captured at each
/// session stroke's pen-down. The union re-bake resolves them PER PIXEL via the owner map — an
/// already-painted wash keeps ITS Concentration (depth) / Body / Edge / water / granulation
/// instead of being re-styled by the current brush (the reported bug: any param change propagated
/// through the wet pools inside the composite's rectangular window). Values are stored
/// pre-clamped EXACTLY as the composite clamped its globals, so owner-resolved math is
/// bit-identical for a single-style session.
#[derive(Clone, Copy)]
pub(in crate::tool::paint) struct WetStrokeStyle {
    pub(in crate::tool::paint) fill: f32,
    pub(in crate::tool::paint) depth: f32,
    /// Per-owner **Opacity** (pigment body / hiding power): a baked light-pigment wash keeps ITS body
    /// when a later stroke changes Opacity, like every other wash param. `0` = pure transmittance.
    pub(in crate::tool::paint) opacity: f32,
    pub(in crate::tool::paint) edge_gain: f32,
    pub(in crate::tool::paint) wet: f32,
    pub(in crate::tool::paint) granulation: f32,
    pub(in crate::tool::paint) warp: f32,
    pub(in crate::tool::paint) pigment_mix: f32,
    pub(in crate::tool::paint) color: [u8; 3],
    /// Per-owner GEOMETRY (doc 13 "mudança no brush propaga"): thinning multiplier, feather-blur
    /// radius, Spread — a baked wash re-renders with ITS geometry, never the live brush's.
    pub(in crate::tool::paint) spread_thin: f32,
    pub(in crate::tool::paint) core_r: u16,
    pub(in crate::tool::paint) spread_px: u16,
    /// Per-owner SUBSTRATE (doc 14 #13, smoke 2026-07-10): the Paper slot + its Depth + the "Same as
    /// Paper" flag + the Grain slot. A baked wash keeps ITS paper/grain — changing the substrate for
    /// the next stroke must NOT re-texture the pool below (the "aplica a tudo" + rectangles bug). The
    /// IMAGES (loaded custom paper/grain) stay session-shared in v1; only the SETTINGS are per-owner,
    /// which covers the reported triggers (paper Kind · Same as Paper · Grain Amount).
    pub(in crate::tool::paint) paper: ph2d_painter_brush::TextureSettings,
    pub(in crate::tool::paint) paper_depth: f32,
    pub(in crate::tool::paint) granulation_use_paper: bool,
    pub(in crate::tool::paint) texture: ph2d_painter_brush::TextureSettings,
}

impl WetStrokeStyle {
    /// Capture the current brush's wash params — the composite's exact clamps, verbatim. `forced_wet`
    /// is the **Wet the layer** floor (#3): the captured Rewet is `max(brush Rewet, forced)`, so strokes
    /// made after the Wet button lift the existing paint even at brush Rewet `0` (`forced = 0` ⇒ verbatim).
    pub(in crate::tool::paint) fn capture(
        spec: &ph2d_painter_brush::BrushSpec,
        forced_wet: f32,
    ) -> Self {
        let spread_px = spec.edge_spread.round().clamp(0.0, 48.0) as usize;
        Self {
            fill: spec.fill.clamp(0.0, 1.0),
            depth: spec.depth.max(0.0),
            opacity: spec.opacity.clamp(0.0, 1.0),
            edge_gain: spec.edge_gain.max(0.0),
            wet: spec.wet_rewet.max(forced_wet).clamp(0.0, 1.0),
            granulation: spec.granulation.clamp(0.0, 1.0),
            warp: spec.warp.max(0.0),
            pigment_mix: spec.effective_pigment_mix(),
            color: [
                (spec.color[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                (spec.color[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                (spec.color[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            ],
            spread_thin: (1.0 + (spread_px as f32 - SPREAD_THIN_REF).max(0.0) / SPREAD_THIN_REF)
                .min(SPREAD_THIN_MAX),
            core_r: spread_px.min(((spec.radius_px * 0.5).round() as usize).max(1)) as u16,
            spread_px: spread_px as u16,
            paper: spec.paper,
            paper_depth: spec.paper_depth.clamp(0.0, 1.0),
            granulation_use_paper: spec.granulation_use_paper,
            texture: spec.texture,
        }
    }
}

/// The wet session's per-stroke style TABLE + per-pixel OWNER map (`0` = unowned → the current
/// brush's style; `k` = `table[k−1]`). Ownership is RECENCY — the last stroke to touch a pixel
/// styles it, matching the colour buffer's source-over. Cleared with the session.
#[derive(Default)]
pub(in crate::tool::paint) struct WetSessionStyles {
    pub(in crate::tool::paint) table: Vec<WetStrokeStyle>,
    pub(in crate::tool::paint) owner: Vec<u8>,
}

impl WetSessionStyles {
    /// Register the beginning stroke's style; the index saturates at 255 (a 256th stroke in one
    /// wet session shares the last slot — far beyond any real ~8.5 s session).
    pub(in crate::tool::paint) fn push_capture(
        &mut self,
        spec: &ph2d_painter_brush::BrushSpec,
        forced_wet: f32,
    ) {
        if self.table.len() < 255 {
            self.table.push(WetStrokeStyle::capture(spec, forced_wet));
        } else if let Some(last) = self.table.last_mut() {
            *last = WetStrokeStyle::capture(spec, forced_wet);
        }
    }

    /// The CURRENT stroke's owner byte for the coverage splat.
    pub(in crate::tool::paint) fn current_owner(&self) -> u8 {
        self.table.len().min(255) as u8
    }

    /// SESSION MAXIMA of the geometry/field knobs, seeded with the live brush's values: any stroke with
    /// water builds the fields, the widest warp/spread pads the composite window, and the blur radii are
    /// global passes — so the window/reach must be conservative across every owner (per-pixel terms still
    /// resolve the OWNER's values in the loop). Empty table ⇒ the seeds verbatim (byte-identical).
    pub(in crate::tool::paint) fn session_maxima(
        &self,
        wet: f32,
        warp: f32,
        spread: usize,
        core: usize,
    ) -> (f32, f32, usize, usize) {
        self.table
            .iter()
            .fold((wet, warp, spread, core), |(w, wa, sm, cm), s| {
                (
                    w.max(s.wet),
                    wa.max(s.warp),
                    sm.max(s.spread_px as usize),
                    cm.max(s.core_r as usize),
                )
            })
    }

    pub(in crate::tool::paint) fn clear(&mut self) {
        self.table.clear();
        self.owner = Vec::new();
    }
}
