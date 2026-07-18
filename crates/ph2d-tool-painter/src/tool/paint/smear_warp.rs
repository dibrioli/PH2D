//! **The Smear as a warp** — the knife's transport, rewritten as an accumulated displacement.
//!
//! The route this replaces lifted pixels from one dab back and lerped them into place, per dab. Over a
//! stroke that is a **product** (`h·wⁿ`) and it decays to nothing everywhere except exactly on the drag
//! axis, so the knife delivered a one-texel needle and no body — Enio, twice: *"as fronteiras não são
//! vencidas. o relevo não é levado além. nada resolvido"*. The measurement, the law and the arithmetic
//! live in [`ph2d_painter_brush::smear_field`]; this module is the plumbing.
//!
//! Three things it is careful about, each of which was a named risk before it was written:
//!
//! 1. **It still rides the one dab list.** The accumulation hangs off `stamp_dabs_inner`'s list exactly
//!    where the colour blend used to, so Symmetry, Tiling, the shape editors, pressure, Jitter, **Shape**
//!    and **Grain** keep reaching the Smear for free. A warp session with geometry of its own inherits
//!    none of that, and *"Tiling doesn't work in Smear"* is how it would be discovered.
//! 2. **The body is not a second implementation.** Colour and relief are resolved from the SAME `disp` by
//!    the same door (`warp/relief.rs`), so the pigment and the thickness physically cannot disagree about
//!    where the paint went. The old `plow_dabs` was a parallel transport with its own chain; it is gone.
//! 3. **The session is per STROKE.** Deform's session spans strokes (Reconstruct needs the history); the
//!    knife has no Apply or Reset to close one, and a stroke's result must become the next stroke's
//!    baseline — the Smear edits the layer in place, as it always has.

use super::{Region, union_region};
use crate::tool::PainterTool;
use ph2d_painter_brush::{BrushSpec, Dab};
use std::sync::Arc;

impl PainterTool {
    /// Accumulate one batch of Smear dabs into the session displacement, then re-render what moved.
    ///
    /// Returns `false` when there is no session to accumulate into (unsized canvas), so the caller can
    /// fall back rather than silently do nothing.
    pub(super) fn smear_dabs_field(&mut self, dabs: &[Dab], w: u32, h: u32) -> bool {
        if dabs.is_empty() {
            return true;
        }
        // The knife's session opens on the first dab of the stroke and is closed by `close_stroke`.
        // `ensure_warp_session` is idempotent, so later batches in the same stroke reuse it — which is
        // precisely what makes the transport a sum across the whole gesture rather than per batch.
        if !self.ensure_warp_session() {
            return false;
        }
        // The knife's Plow decides how much of the body comes along — through the ONE door, never a
        // second transport. Re-read every batch so the slider stays live within a stroke.
        self.paint.warp.relief_disp_scale =
            self.paint.brush.effective_impasto_plow().clamp(0.0, 1.0);
        let base = self.paint.brush;
        // The Smear's fold has never included Flow (`amount = strength · coverage`), and `walk_dab` folds
        // `coverage × flow × strength`. Neutralising Flow here keeps the knife's response to its sliders
        // exactly what it has always been — turning an inert slider live is not this fix's business.
        let spec_base = BrushSpec { flow: 1.0, ..base };

        // Resolve each dab's frames exactly as the colour route does — same Shape basis, same Grain
        // frame, same order, same RNG discipline (a COPY: this pass must not advance the stream).
        self.ensure_shape_ramp_lut();
        let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        let grain_image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let shape_ramp_lut = (self.paint.shape_color_ramp_enabled
            && self.paint.shape_color_ramp_bw)
            .then_some(self.paint.shape_ramp_lut.clone());
        let shape_active = base.shape_silhouette_active(shape_image.is_some());
        let grain_active = base.texture.is_active();
        let groups = self.paint.dab_groups.clone();
        let mut dab_rng = super::tiling::DabRng::new(self.paint.tex_rng);
        // The Selection attenuates each dab AS IT LANDS (never the running total — see
        // `accumulate_dab_sculpt`: attenuating the total compounds once per pointer batch, and a Feather
        // makes that visible).
        let mask: Option<Arc<Vec<u8>>> = self
            .selection_restricts_paint()
            .then(|| Arc::clone(&self.paint.selection_mask));

        let tiling = self.paint.tiling;
        let tiled = tiling[0] || tiling[1];
        let source_size = self.source_size;

        let mut disp = std::mem::take(Arc::make_mut(&mut self.paint.warp.disp));
        let mut scratch = std::mem::take(&mut self.paint.smear_scratch);
        let mut from = self.paint.last_smear_pos;
        let mut touched: Option<Region> = None;
        for (di, d) in dabs.iter().enumerate() {
            let tex_rng = dab_rng.enter(&groups, di);
            if let Some(prev) = from {
                let spec = BrushSpec {
                    radius_px: d.radius_px,
                    ..spec_base
                };
                let fp = spec.footprint_deform().rotated_by(d.rotation);
                let shape_basis = shape_active.then(|| {
                    ph2d_painter_brush::texture::dab_basis(
                        &spec.shape,
                        d.dir,
                        &mut *tex_rng,
                        [w as f32, h as f32],
                        [1.0, 0.0],
                        fp,
                    )
                });
                let grain_basis = grain_active.then(|| {
                    ph2d_painter_brush::texture::dab_basis(
                        &spec.texture,
                        d.dir,
                        &mut *tex_rng,
                        [w as f32, h as f32],
                        [1.0, 0.0],
                        fp,
                    )
                });
                // This dab's motion, in canvas px and NOT rounded to whole texels: a displacement is
                // resampled bilinearly, so the integer quantisation the lift-and-blend kernel needed
                // (it indexed source pixels directly) is pure loss here.
                let step = [d.center[0] - prev[0], d.center[1] - prev[1]];
                // Tiling: the wrapped copies each accumulate at their own place, with the same step —
                // the same offsets the colour blend used to walk.
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
                    let hd = ph2d_painter_brush::height::HeightDab {
                        center: [d.center[0] + off[0], d.center[1] + off[1]],
                        radius: d.radius_px,
                        coverage: d.coverage,
                        footprint: fp,
                        // No sweep: like a sculpt dab, a smear dab marks where it IS. The field sums, so
                        // consecutive dabs blend by construction and there is no bead-seam to hide.
                        prev_center: None,
                        shape: shape_basis
                            .as_ref()
                            .map(|sb| ph2d_painter_brush::ShapeInput {
                                basis: sb,
                                image: shape_image.as_ref(),
                                ramp_lut: shape_ramp_lut.as_deref(),
                            }),
                        grain: grain_basis.as_ref(),
                        grain_image: grain_image.as_ref(),
                    };
                    if let Some(r) = ph2d_painter_brush::accumulate_dab_smear(
                        ph2d_painter_brush::SmearOut {
                            disp: &mut disp,
                            scratch: &mut scratch,
                        },
                        step,
                        mask.as_ref().map(|m| m.as_slice()),
                        w,
                        h,
                        &spec,
                        &hd,
                    ) {
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
            from = Some(d.center);
        }
        *Arc::make_mut(&mut self.paint.warp.disp) = disp;
        self.paint.smear_scratch = scratch;
        self.paint.last_smear_pos = from;
        self.paint.tex_rng = dab_rng.finish();
        if let Some(rect) = touched {
            // One resample of the frozen source over everything that moved — colour and body together.
            self.warp_render_from_session(rect);
            self.mark_dirty(rect);
        }
        true
    }

    /// Close the knife's per-stroke session. Called from `close_stroke`; a no-op in every other mode
    /// because only the Smear opens a session there.
    pub(super) fn end_smear_session(&mut self) {
        if matches!(self.paint.paint_mode, super::PaintMode::Smear) {
            self.end_warp_session();
        }
    }
}
