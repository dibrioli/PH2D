//! The **Clone** (clone-stamp) route: copy canvas pixels from a sampled source at a fixed offset.
//!
//! Sibling of [`super::stamp_route`]'s `stamp_dabs_smear` — but instead of lifting from the previous
//! dab (a per-step displacement), Clone lifts from a **fixed offset** established at stroke start from
//! the sampled source anchor (`offset = source − stroke_start`), so each dab reads `canvas[dab + offset]`.
//!
//! Source UX (self-contained, mirrors the Symmetry on-canvas pick modes — no shell modifier plumbing):
//! the panel's "Set Source" button arms a pick mode; the next canvas Down records the source point and
//! consumes the click (no paint). **Aligned** keeps the offset fixed across strokes; **non-aligned**
//! re-anchors the source to the sampled point at the start of each new stroke.

use super::{PaintMode, Region, union_region};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPointer, PanelEvent, PointerPhase};
use ph2d_painter_brush::{BrushSpec, Dab};

impl PainterTool {
    /// Whether the active operation is **Clone** (drives the stamp route + the panel snapshot).
    #[must_use]
    pub fn is_clone_mode(&self) -> bool {
        matches!(self.paint.paint_mode, PaintMode::Clone)
    }

    /// Whether a clone source has been sampled (the panel shows "Set Source" vs "Source set").
    #[must_use]
    pub fn clone_has_source(&self) -> bool {
        self.paint.clone_source.is_some()
    }

    /// Whether Clone is in **Aligned** mode (offset persists across strokes).
    #[must_use]
    pub fn clone_aligned(&self) -> bool {
        self.paint.clone_aligned
    }

    /// Whether the "Set Source" pick mode is armed (the shell shows a sampling cursor; the next canvas
    /// Down sets the source instead of painting).
    #[must_use]
    pub fn clone_sample_armed(&self) -> bool {
        self.paint.clone_sample_armed
    }

    /// Arm the "Set Source" pick mode (panel button). The next canvas Down samples the source.
    pub fn arm_clone_sample(&mut self) {
        self.paint.clone_sample_armed = true;
    }

    /// Toggle **Aligned** mode (panel checkbox).
    pub fn toggle_clone_aligned(&mut self) {
        self.paint.clone_aligned = !self.paint.clone_aligned;
    }

    /// Set the clone source anchor at `pos` (image px) — the pick mode's canvas click. Clears the
    /// established offset (a fresh source re-anchors the next stroke) and disarms the pick mode.
    pub(super) fn set_clone_source(&mut self, pos: [f32; 2]) {
        self.paint.clone_source = Some(pos);
        self.paint.clone_offset = None;
        self.paint.clone_sample_armed = false;
    }

    /// Route the armed "Set Source" pick: a Down samples the source; every phase is consumed (no paint
    /// while armed). Mirrors [`PainterTool::symmetry_pick_pointer`].
    pub(super) fn clone_sample_pointer(&mut self, ev: CanvasPointer) -> bool {
        if ev.phase == PointerPhase::Down {
            self.set_clone_source(ev.pos);
        }
        true
    }

    /// Establish the source→dest offset at stroke start (from `paint_begin`). Aligned: set only once per
    /// sampled source (persists across strokes); non-aligned: re-anchored every stroke. No-op with no source.
    pub(super) fn clone_begin_offset(&mut self, stroke_start: [f32; 2]) {
        let Some(src) = self.paint.clone_source else {
            return;
        };
        if !self.paint.clone_aligned || self.paint.clone_offset.is_none() {
            self.paint.clone_offset = Some([src[0] - stroke_start[0], src[1] - stroke_start[1]]);
        }
    }

    /// Route the Clone-card panel events (Set Source, Aligned toggle). Returns `true` iff consumed.
    pub(crate) fn route_clone_event(&mut self, event: &PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        if let PanelEvent::Click(id) = event {
            if *id == core_ids::PAINTER_BRUSH_CLONE_SET_SOURCE {
                self.arm_clone_sample();
                return true;
            }
            if *id == core_ids::PAINTER_BRUSH_CLONE_ALIGNED {
                self.toggle_clone_aligned();
                return true;
            }
        }
        false
    }

    /// **Clone** route: copy `canvas[dab + offset]` into each dab, weighted by the brush's full dab
    /// coverage (Shape × Grain × flatten) × Strength × pressure. Routed like the paint path (cached mask
    /// / per-pixel Grain / round). No-op until a source is sampled (`clone_offset` set at stroke start).
    /// **Tiling** replicates each dab across the sprite edges (the lift wraps toroidally too).
    pub(super) fn stamp_dabs_clone(&mut self, dabs: &[Dab], w: u32, h: u32) {
        if dabs.is_empty() {
            return;
        }
        let Some(offset) = self.paint.clone_offset else {
            return; // no source sampled yet → nothing to clone
        };
        let base = self.paint.brush;
        let strength = base.strength.clamp(0.0, 1.0);
        let has_shape_image = self.paint.shape_image.is_some();
        let textured = base.shape_silhouette_active(has_shape_image)
            || base.texture.is_active()
            || base.dab_flatten > 0.0;
        let per_dab_rotation =
            base.has_per_dab_rotation() || base.shape_has_per_dab_rotation(has_shape_image);
        // Shape / Grain Colour Ramps act as B&W coverage TONES here (no colour painted); the cached mask
        // can't carry them, so an active ramp tone forces the per-pixel Grain path.
        self.ensure_shape_ramp_lut();
        let grain_tone = self.grain_tone_lut();
        let ramp_tone_active = self.shape_tone_lut_slice().is_some() || grain_tone.is_some();
        let want_mask = textured
            && base.dab_mask_cacheable(has_shape_image)
            && !per_dab_rotation
            && !ramp_tone_active;
        let want_grain = textured && !want_mask;
        let grain_active = base.texture.is_active();
        let shape_active = base.shape_silhouette_active(has_shape_image);
        let tiling = self.paint.tiling;
        let tiled = tiling[0] || tiling[1];
        let source_size = self.source_size;

        if want_mask {
            let max_r = dabs.iter().map(|d| d.radius_px).fold(0.0_f32, f32::max);
            self.ensure_stamp_cache(&base, super::stamp_cache::mask_size_for(max_r));
        }
        let mask = want_mask
            .then(|| self.paint.stamp_cache.as_ref().map(|(m, _)| m))
            .flatten();
        let grain_img = want_grain
            .then(|| self.paint.texture_image.as_ref().map(|i| i.as_mask()))
            .flatten();
        let shape_img = want_grain
            .then(|| self.paint.shape_image.as_ref().map(|i| i.as_mask()))
            .flatten();
        let shape_lut: Option<Vec<f32>> = if want_grain {
            self.shape_tone_lut_slice().map(|s| s.to_vec())
        } else {
            None
        };

        let mut tex_rng = self.paint.tex_rng;
        let buf = crate::tool::paint::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            None,
        );
        let mut touched: Option<Region> = None;
        for d in dabs {
            let amount = strength * d.coverage;
            let rotor = base.dab_rotor(d);
            let fp = base.dab_footprint(rotor);
            let sbasis = (want_grain && shape_active).then(|| {
                ph2d_painter_brush::texture::shape_basis(
                    &base.shape,
                    &mut tex_rng,
                    [w as f32, h as f32],
                    fp,
                    ph2d_painter_brush::texture::ShapeFrame::Stroke {
                        arc_len: d.arc_len,
                        unit_px: d.stroke_radius_px,
                    },
                )
            });
            let gbasis = (want_grain && grain_active).then(|| {
                ph2d_painter_brush::texture::dab_basis(
                    &base.texture,
                    &mut tex_rng,
                    [w as f32, h as f32],
                    fp,
                )
            });
            let mut offs = [[0.0f32; 2]; 9];
            let n = if tiled {
                super::tiling::tiled_offsets_into(
                    d.center,
                    d.radius_px,
                    source_size,
                    tiling,
                    &mut offs,
                )
            } else {
                1
            };
            for &woff in &offs[..n] {
                let c = [d.center[0] + woff[0], d.center[1] + woff[1]];
                let dirty = if let Some(m) = mask {
                    ph2d_painter_brush::clone_blit_stamp(
                        buf,
                        w,
                        h,
                        c,
                        offset,
                        d.radius_px,
                        m,
                        amount,
                        tiling,
                    )
                } else if want_grain {
                    ph2d_painter_brush::clone_blit_grain(
                        buf,
                        w,
                        h,
                        c,
                        offset,
                        d.radius_px,
                        &base,
                        fp,
                        gbasis.as_ref(),
                        sbasis.as_ref(),
                        grain_img.as_ref(),
                        shape_img.as_ref(),
                        shape_lut.as_deref(),
                        grain_tone.as_deref(),
                        amount,
                        tiling,
                    )
                } else {
                    ph2d_painter_brush::clone_dab(
                        buf,
                        w,
                        h,
                        c,
                        offset,
                        &BrushSpec {
                            radius_px: d.radius_px,
                            ..base
                        },
                        amount,
                        tiling,
                    )
                };
                if let Some(r) = dirty {
                    let rect = Region {
                        x: r.x,
                        y: r.y,
                        w: r.w,
                        h: r.h,
                    };
                    touched = Some(touched.map_or(rect, |acc| union_region(acc, rect)));
                }
            }
        }
        self.paint.tex_rng = tex_rng;
        self.declare_wrote(touched);
        if let Some(rect) = touched {
            self.mark_dirty(rect);
        }
    }
}
