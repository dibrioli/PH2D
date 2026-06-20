//! Adjustment-layer PARAM mutators — create an adjustment, then the per-kind
//! param edits (generic sliders / toggles / segments, plus the bespoke editors:
//! Curves, Gradient-Map, Channel-Mixer, Selective-Color). These route the
//! preview through the cut-cache fast lane (keep the cuts BELOW the adjustment,
//! restart from its cut) rather than the structural `invalidate_composite`.
//! `impl PainterTool` (one of several blocks in this crate). Split out of the
//! former `tool/layers.rs` god-file (pure move).

use super::super::*;

/// Max control points per Curves channel (ADR-0045 §2.6 `ControlPoints` cap).
/// The bespoke editor's add-point button stops here.
pub const MAX_CURVE_POINTS_PER_CHANNEL: usize = 8;

impl PainterTool {
    /// Create a non-destructive adjustment layer of `kind` at the top of the
    /// stack and SELECT it (highlight) WITHOUT changing the paint target — an
    /// adjustment has no pixel buffer, so the previously active raster stays the
    /// edit target (mirror of the group guard, [`Self::set_active_layer`]).
    /// No-op (`None`) mid-stroke or at the layer cap. W4 T4.3 (HSB Day-4 smoke).
    pub fn add_adjustment_layer(
        &mut self,
        kind: ph2d_painter_effects::adjustments::AdjustmentKind,
    ) -> Option<RtLayerId> {
        let undo_before = self.snapshot_model();
        let prev_active = self.layers.active();
        let id = self.layers.add_adjustment(kind)?; // LayerStack sets active = adj
        // Bespoke Curves editor: seed EVERY channel (master + R/G/B) with 5
        // evenly-spaced identity handles so the curve canvas — and each R/G/B tab —
        // opens with draggable control points. The data-model default stays empty
        // (a bit-exact identity for persisted / programmatic layers); a user-created
        // Curves layer gets editable handles. All-diagonal seeds = identity output.
        if kind == ph2d_painter_effects::adjustments::AdjustmentKind::Curves
            && let Some(adj) = self.layers.adjustment_mut(id)
            && let ph2d_painter_effects::adjustments::AdjustmentParams::Curves(c) = &mut adj.params
        {
            let identity: Vec<[f32; 2]> = (0..5)
                .map(|i| {
                    let t = i as f32 / 4.0;
                    [t, t]
                })
                .collect();
            c.points_rgb.points = identity.clone();
            c.points_r.points = identity.clone();
            c.points_g.points = identity.clone();
            c.points_b.points = identity;
        }
        // Not paintable — restore the prior raster as the edit target. The
        // canvas_rgba is untouched (add_adjustment does not flush/load), so a
        // plain `set_active` (no buffer dance) keeps it consistent.
        if let Some(p) = prev_active {
            self.layers.set_active(p);
        }
        self.reset_selection_to(id); // highlight the new adjustment row
        self.invalidate_composite();
        self.commit_structural_edit(undo_before);
        Some(id)
    }

    /// Set slider `slot` of adjustment layer `id` from a normalized `0..1`
    /// value (the panel's per-slot sliders). The per-kind mapping lives in
    /// `adjustments::set_adjustment_slider_param` (HSB / Brightness-Contrast /
    /// …). No-op mid-stroke or if `id` is not an adjustment. Invalidates the
    /// composite so the live preview re-renders.
    pub fn set_adjustment_param(&mut self, id: RtLayerId, slot: usize, slider01: f32) {
        let Some(adj) = self.layers.adjustment_mut(id) else {
            return;
        };
        ph2d_painter_effects::adjustments::set_adjustment_slider_param(
            &mut adj.params,
            slot,
            slider01,
        );
        // W5 slider-drag hot path: a param-only change leaves every layer BELOW
        // this adjustment untouched, so keep their cuts and drop only the cuts
        // ABOVE it; the next drain restarts from this adjustment's cut via
        // `composite_with_cache` instead of recomposing the whole stack. (NOT the
        // structural `invalidate_composite`, which would clear all cuts + force a
        // cold full recompose every drag frame — the exact cost we're killing.)
        self.compositor_cache.invalidate_above(id, &self.layers);
        self.composited = None;
        self.dirty_rect = None;
        self.adjustment_cache_pending = true;
        self.preview_dirty = true;
        // The adjustment's params live in the published LayerStack, so the panel
        // snapshot must republish (mirror of `invalidate_composite`'s bump).
        self.layers_revision = self.layers_revision.wrapping_add(1);
    }

    /// Flip toggle `slot` of adjustment layer `id` (the panel's per-slot switches,
    /// e.g. Photo Filter's "Preserve Luminosity"). The params are the single
    /// source of truth, so the panel forwards a bare click and the tool reads the
    /// current value + inverts it (mirror of [`Self::toggle_mask_inverted`]). The
    /// per-kind mapping lives in `adjustments::{adjustment_toggle_params,
    /// set_adjustment_toggle_param}`. No-op mid-stroke or if `id` is not an
    /// adjustment. Routes the preview through the same cut-cache fast lane as a
    /// slider edit ([`Self::set_adjustment_param`]).
    pub fn flip_adjustment_toggle(&mut self, id: RtLayerId, slot: usize) {
        let Some(adj) = self.layers.adjustment_mut(id) else {
            return;
        };
        let cur = ph2d_painter_effects::adjustments::adjustment_toggle_params(&adj.params)
            .get(slot)
            .map(|(_, on)| *on)
            .unwrap_or(false);
        ph2d_painter_effects::adjustments::set_adjustment_toggle_param(&mut adj.params, slot, !cur);
        // Same param-only hot lane as `set_adjustment_param`: keep the cuts below
        // this adjustment, restart from its cut, republish for the panel snapshot.
        self.compositor_cache.invalidate_above(id, &self.layers);
        self.composited = None;
        self.dirty_rect = None;
        self.adjustment_cache_pending = true;
        self.preview_dirty = true;
        self.layers_revision = self.layers_revision.wrapping_add(1);
    }

    /// Set weight `slot` (0 = R, 1 = G, 2 = B source, 3 = constant) of Channel
    /// Mixer `id`'s `output` row (0 = Red/Gray, 1 = Green, 2 = Blue) from a
    /// normalized `0..1` value. The bespoke mixer editor forwards this with the
    /// active output tab in the payload (the channel the generic slider rack can
    /// not carry). No-op mid-stroke or if `id` is not a Channel Mixer. Routes the
    /// preview through the same cut-cache fast lane as a slider edit.
    pub fn set_channel_mixer_weight(
        &mut self,
        id: RtLayerId,
        output: usize,
        slot: usize,
        value01: f32,
    ) {
        let Some(adj) = self.layers.adjustment_mut(id) else {
            return;
        };
        let ph2d_painter_effects::adjustments::AdjustmentParams::ChannelMixer(m) = &mut adj.params
        else {
            return;
        };
        ph2d_painter_effects::adjustments::set_channel_mixer_param(m, output, slot, value01);
        self.compositor_cache.invalidate_above(id, &self.layers);
        self.composited = None;
        self.dirty_rect = None;
        self.adjustment_cache_pending = true;
        self.preview_dirty = true;
        self.layers_revision = self.layers_revision.wrapping_add(1);
    }

    /// Gradient-Map editor mutators (W4 BATCH-2) — move / add / remove a stop +
    /// set the selected stop's RGB. Each routes the preview through the same
    /// cut-cache fast lane as a slider edit. No-op mid-stroke or for a non-Gradient
    /// layer / out-of-range stop (delegated to the brush helpers).
    pub fn set_gradient_stop_offset(&mut self, id: RtLayerId, stop: usize, offset: f32) {
        if let Some(g) = self.gradient_params_mut(id) {
            ph2d_painter_effects::adjustments::move_gradient_stop(g, stop, offset);
            self.after_curve_edit(id);
        }
    }

    /// Insert a stop on Gradient-Map `id` (midpoint of the widest gap, color on the
    /// current gradient). Returns the inserted index, or `None` (cap / non-Gradient).
    pub fn add_gradient_stop(&mut self, id: RtLayerId) -> Option<usize> {
        let g = self.gradient_params_mut(id)?;
        let idx = ph2d_painter_effects::adjustments::add_gradient_stop(g);
        if idx.is_some() {
            self.after_curve_edit(id);
        }
        idx
    }

    /// Remove stop `stop` from Gradient-Map `id` (keeps ≥2 stops).
    pub fn remove_gradient_stop(&mut self, id: RtLayerId, stop: usize) {
        if let Some(g) = self.gradient_params_mut(id) {
            ph2d_painter_effects::adjustments::remove_gradient_stop(g, stop);
            self.after_curve_edit(id);
        }
    }

    /// Set RGB slider `slot` of Gradient-Map `id`'s `stop` from a `0..1` value.
    pub fn set_gradient_stop_color(&mut self, id: RtLayerId, stop: usize, slot: usize, value: f32) {
        if let Some(g) = self.gradient_params_mut(id) {
            ph2d_painter_effects::adjustments::set_gradient_stop_color_param(g, stop, slot, value);
            self.after_curve_edit(id);
        }
    }

    /// `&mut GradientMapParams` of adjustment `id`, or `None` mid-stroke / for a
    /// non-Gradient-Map layer.
    fn gradient_params_mut(
        &mut self,
        id: RtLayerId,
    ) -> Option<&mut ph2d_painter_effects::adjustments::GradientMapParams> {
        let adj = self.layers.adjustment_mut(id)?;
        match &mut adj.params {
            ph2d_painter_effects::adjustments::AdjustmentParams::GradientMap(g) => Some(g),
            _ => None,
        }
    }

    /// Set CMYK slider `slot` (0 = C, 1 = M, 2 = Y, 3 = K) of Selective-Color
    /// `id`'s color group `bucket` (0..9: Reds … Blacks) from a normalized `0..1`
    /// value. The bespoke editor forwards this with the active bucket in the
    /// payload (the group the generic slider rack can not carry). No-op mid-stroke
    /// or if `id` is not a Selective Color. Same cut-cache fast lane as a slider.
    pub fn set_selective_color_value(
        &mut self,
        id: RtLayerId,
        bucket: usize,
        slot: usize,
        value01: f32,
    ) {
        let Some(adj) = self.layers.adjustment_mut(id) else {
            return;
        };
        let ph2d_painter_effects::adjustments::AdjustmentParams::SelectiveColor(s) =
            &mut adj.params
        else {
            return;
        };
        ph2d_painter_effects::adjustments::set_selective_color_param(s, bucket, slot, value01);
        self.compositor_cache.invalidate_above(id, &self.layers);
        self.composited = None;
        self.dirty_rect = None;
        self.adjustment_cache_pending = true;
        self.preview_dirty = true;
        self.layers_revision = self.layers_revision.wrapping_add(1);
    }

    /// Select option `option` of adjustment layer `id`'s segmented param (the
    /// panel's segment-button row, e.g. Color Balance's tonal range). The per-kind
    /// mapping lives in `adjustments::set_adjustment_segment_param`. No-op
    /// mid-stroke or if `id` is not an adjustment. Routes the preview through the
    /// same cut-cache fast lane as a slider edit ([`Self::set_adjustment_param`]).
    pub fn set_adjustment_segment(&mut self, id: RtLayerId, option: usize) {
        let Some(adj) = self.layers.adjustment_mut(id) else {
            return;
        };
        ph2d_painter_effects::adjustments::set_adjustment_segment_param(&mut adj.params, option);
        // Same param-only hot lane as `set_adjustment_param` (a scope change only
        // rebuilds this adjustment's transfer; layers below keep their cuts).
        self.compositor_cache.invalidate_above(id, &self.layers);
        self.composited = None;
        self.dirty_rect = None;
        self.adjustment_cache_pending = true;
        self.preview_dirty = true;
        self.layers_revision = self.layers_revision.wrapping_add(1);
    }

    /// Move control point `point_index` of adjustment `id`'s `channel` curve to
    /// normalized `(x01, y01)` (both `0..=1`). `channel`: 0 = master (RGB),
    /// 1 = R, 2 = G, 3 = B. The bespoke curve-editor UI calls this on a point
    /// drag (the Curves analogue of [`Self::set_adjustment_param`] — Curves does
    /// not fit the generic ≤6-slider rack). No-op mid-stroke, for a non-Curves
    /// layer, an out-of-range channel, or a missing point index.
    ///
    /// X is **clamped between the two neighbours** (not re-sorted): the free-2D
    /// editor binds a stable `point_index` per handle, so a sort here would make
    /// the next drag frame grab a *different* point as soon as a point crossed its
    /// neighbour. Clamping keeps the points ordered (the spline eval needs
    /// ascending x) AND the index stable for the whole gesture. Routes the preview
    /// through the same cut-point cache fast lane as a slider drag (so the GPU LUT
    /// path re-renders Curves in real time).
    pub fn set_curve_point(
        &mut self,
        id: RtLayerId,
        channel: u8,
        point_index: usize,
        x01: f32,
        y01: f32,
    ) {
        let Some(adj) = self.layers.adjustment_mut(id) else {
            return;
        };
        let ph2d_painter_effects::adjustments::AdjustmentParams::Curves(c) = &mut adj.params else {
            return;
        };
        let pts = match channel {
            0 => &mut c.points_rgb,
            1 => &mut c.points_r,
            2 => &mut c.points_g,
            3 => &mut c.points_b,
            _ => return,
        };
        let n = pts.points.len();
        if point_index >= n {
            return;
        }
        // Clamp X into the neighbours' span so the points stay ordered without a
        // sort (stable index across the drag — see the doc comment). Endpoints are
        // free to the [0,1] domain edge on their outer side.
        let left = if point_index == 0 {
            0.0
        } else {
            pts.points[point_index - 1][0]
        };
        let right = if point_index + 1 == n {
            1.0
        } else {
            pts.points[point_index + 1][0]
        };
        let p = &mut pts.points[point_index];
        p[0] = x01.clamp(0.0, 1.0).clamp(left, right);
        p[1] = y01.clamp(0.0, 1.0);
        self.after_curve_edit(id);
    }

    /// Cut-cache restart + republish after any curve edit (move/add/remove a
    /// point): a curve change is param-only relative to the layers BELOW the
    /// adjustment, so keep their cuts and restart from this adjustment's cut via
    /// `composite_with_cache` (the GPU LUT path then re-renders in real time —
    /// same hot-path lane as `set_adjustment_param`).
    fn after_curve_edit(&mut self, id: RtLayerId) {
        self.compositor_cache.invalidate_above(id, &self.layers);
        self.composited = None;
        self.dirty_rect = None;
        self.adjustment_cache_pending = true;
        self.preview_dirty = true;
        self.layers_revision = self.layers_revision.wrapping_add(1);
    }

    /// Insert a control point on `channel`'s curve of adjustment `id` at the
    /// midpoint of its widest X-gap, with Y sampled ON the current curve (so the
    /// rendered output is unchanged until the new point is dragged). Returns the
    /// inserted index, or `None` (no-op) mid-stroke, for a non-Curves layer, an
    /// out-of-range channel, a degenerate (<2-point) curve, or at the ≤8-point cap.
    pub fn add_curve_point(&mut self, id: RtLayerId, channel: u8) -> Option<usize> {
        let adj = self.layers.adjustment_mut(id)?;
        let ph2d_painter_effects::adjustments::AdjustmentParams::Curves(c) = &mut adj.params else {
            return None;
        };
        let pts = match channel {
            0 => &mut c.points_rgb,
            1 => &mut c.points_r,
            2 => &mut c.points_g,
            3 => &mut c.points_b,
            _ => return None,
        };
        let n = pts.points.len();
        if !(2..MAX_CURVE_POINTS_PER_CHANNEL).contains(&n) {
            return None;
        }
        let mut best_gap = -1.0_f32;
        let mut new_x = 0.5_f32;
        let mut insert_at = n;
        for i in 0..n - 1 {
            let gap = pts.points[i + 1][0] - pts.points[i][0];
            if gap > best_gap {
                best_gap = gap;
                new_x = (pts.points[i][0] + pts.points[i + 1][0]) * 0.5;
                insert_at = i + 1;
            }
        }
        let new_y = ph2d_painter_effects::adjustments::curve_value_at(&pts.points, new_x);
        pts.points.insert(insert_at, [new_x, new_y]);
        self.after_curve_edit(id);
        Some(insert_at)
    }

    /// Remove control point `index` of `channel`'s curve of adjustment `id`. No-op
    /// mid-stroke, for a non-Curves layer, an out-of-range channel/index, or when
    /// only the two endpoints remain (a curve needs ≥2 points).
    pub fn remove_curve_point(&mut self, id: RtLayerId, channel: u8, index: usize) {
        let Some(adj) = self.layers.adjustment_mut(id) else {
            return;
        };
        let ph2d_painter_effects::adjustments::AdjustmentParams::Curves(c) = &mut adj.params else {
            return;
        };
        let pts = match channel {
            0 => &mut c.points_rgb,
            1 => &mut c.points_r,
            2 => &mut c.points_g,
            3 => &mut c.points_b,
            _ => return,
        };
        if pts.points.len() <= 2 || index >= pts.points.len() {
            return;
        }
        pts.points.remove(index);
        self.after_curve_edit(id);
    }
}
