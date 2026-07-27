//! The **Blur** (soften) route: for each dab, blur the canvas neighbourhood under its footprint and
//! blend it back, weighted by the brush's full dab coverage (Shape silhouette × Grain × flatten/rotate)
//! × Strength × pressure. The soften sibling of [`super::stamp_route`]'s `stamp_dabs_smear`.
//!
//! Unlike Smear, Blur is **stationary per dab** — it needs no motion (a single click softens the
//! footprint), so there is no lift-source to chain across pointer batches. Everything else mirrors the
//! Smear route: the cached [`ph2d_painter_brush::StampMask`] for a View-static dab; the per-pixel Grain
//! path ([`ph2d_painter_brush::blur_blit_grain`]) for a canvas-fixed Grain Mapping (Tiled/Stencil) or
//! per-dab rotation (Rake / Random / Jitter Rotate); the plain round falloff otherwise. **Tiling**
//! replicates each dab across the enabled sprite edges (the neighbourhood read wraps toroidally too).

use super::{Region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::{BrushSpec, Dab};

impl PainterTool {
    /// Blur every dab's footprint into `canvas_rgba`, routed like the paint path (see the module doc).
    /// Blur amount = brush Strength × per-dab coverage (pressure). The engine snapshots each dab's
    /// neighbourhood before writing, so overlapping read/write never feeds back within a dab.
    pub(super) fn stamp_dabs_blur(&mut self, dabs: &[Dab], w: u32, h: u32) {
        if dabs.is_empty() {
            return;
        }
        let base = self.paint.brush;
        let strength = base.strength.clamp(0.0, 1.0);
        let has_shape_image = self.paint.shape_image.is_some();
        let textured = base.shape_silhouette_active(has_shape_image)
            || base.texture.is_active()
            || base.dab_flatten > 0.0;
        // Per-dab-rotating Shape/Grain (Rake / Random) or Jitter Rotate isn't scale-invariant → route it
        // (with any canvas-fixed Grain Mapping) through the per-pixel Grain path. View-static → the fast
        // cached mask; untextured → the plain round falloff.
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

        // Gather the weight source BEFORE the buffer borrow (disjoint `self.paint` fields vs
        // `self.canvas_rgba`): the cached mask (View), or the Grain/Shape images for the per-pixel path.
        // The Shape tone LUT is cloned — it borrows `self`, which the `canvas_rgba` write can't co-hold.
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
        let buf =
            crate::tool::paint::plane_fork::fork_par(&mut self.canvas_rgba, &self.undo_window);
        let mut touched: Option<Region> = None;
        for d in dabs {
            let amount = strength * d.coverage;
            // Per-dab frame for the Grain path: the Jitter-Rotate footprint + the Rake heading (`d.dir`)
            // + the Random draw (`tex_rng`), computed ONCE per dab so the wrapped Tiling copies share it.
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
            // Tiling wrap offsets (applied to the dab centre); stack buffer, no alloc.
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
            for &off in &offs[..n] {
                let c = [d.center[0] + off[0], d.center[1] + off[1]];
                let dirty = if let Some(m) = mask {
                    ph2d_painter_brush::blur_blit_stamp(
                        buf,
                        w,
                        h,
                        c,
                        d.radius_px,
                        m,
                        amount,
                        tiling,
                    )
                } else if want_grain {
                    ph2d_painter_brush::blur_blit_grain(
                        buf,
                        w,
                        h,
                        c,
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
                    ph2d_painter_brush::blur_dab(
                        buf,
                        w,
                        h,
                        c,
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
