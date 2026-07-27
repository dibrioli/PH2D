//! **Floating-patch Transform** (Deform Wave 2, Procreate model). When Transform begins with an active
//! selection, the selected pixels are LIFTED into a floating `patch` (the marquee disappears — the selection
//! is "captured" by the transform), the rest of the layer becomes `base` with a hole where the patch was,
//! and the gizmo moves/scales/rotates the patch freely OVER the base (it can leave the original area,
//! revealing the hole). With no selection the whole layer is the patch (base empty) → a plain layer
//! transform. Byte-identical at `M = I` (patch composited over base at its origin reconstructs the source).
//!
//! Rendered by backward-gather composite over only the patch's source + destination bbox (dirty-rect), so a
//! small selection costs a small loop — never the whole canvas. One structural undo entry per whole
//! transform (committed when it ends), like Procreate's single-step Transform.

use super::super::Region;
use super::transform_geom::{Affine2, Mat3, TransformFrame};
use crate::tool::PainterTool;
use std::sync::Arc;

/// Bilinear sample of the patch with a TRANSPARENT border — a source coordinate outside the canvas
/// contributes zero, so a patch pulled/shrunk away from the edge vacates to transparent instead of
/// smearing the edge texels (which is what clamping did on a whole-image transform of an opaque
/// layer: `bilinear_clamped` duplicated the border colour over everything the patch left behind).
/// In-bounds samples are byte-identical to the clamped variant (identical corners and weights).
pub(super) fn bilinear_patch(canvas: &[u8], w: u32, h: u32, x: f32, y: f32) -> [u8; 4] {
    if w == 0 || h == 0 {
        return [0, 0, 0, 0];
    }
    let x0f = x.floor();
    let y0f = y.floor();
    let fx = x - x0f;
    let fy = y - y0f;
    let (wi, hi) = (w as i64, h as i64);
    let (x0, y0) = (x0f as i64, y0f as i64);
    let stride = w as usize * 4;
    // A corner as PREMULTIPLIED `[r·a, g·a, b·a, a]` (rgb on the `0..=255` scale, `a` raw); outside
    // the canvas it is fully transparent.
    let corner = |xi: i64, yi: i64| -> [f32; 4] {
        if xi < 0 || xi >= wi || yi < 0 || yi >= hi {
            return [0.0, 0.0, 0.0, 0.0];
        }
        let b = yi as usize * stride + xi as usize * 4;
        let a = f32::from(canvas[b + 3]);
        let s = a / 255.0;
        [
            f32::from(canvas[b]) * s,
            f32::from(canvas[b + 1]) * s,
            f32::from(canvas[b + 2]) * s,
            a,
        ]
    };
    let lerp = |p: [f32; 4], q: [f32; 4], t: f32| {
        [
            p[0] + (q[0] - p[0]) * t,
            p[1] + (q[1] - p[1]) * t,
            p[2] + (q[2] - p[2]) * t,
            p[3] + (q[3] - p[3]) * t,
        ]
    };
    // Identical arithmetic to `bilinear_clamped` from here down, so in-bounds samples stay
    // byte-for-byte equal.
    let top = lerp(corner(x0, y0), corner(x0 + 1, y0), fx);
    let bot = lerp(corner(x0, y0 + 1), corner(x0 + 1, y0 + 1), fx);
    let m = lerp(top, bot, fy); // [premR, premG, premB, A]
    let a = m[3];
    if a <= 0.0 {
        return [0, 0, 0, 0];
    }
    let inv = 255.0 / a; // un-premultiply
    [
        (m[0] * inv).round().clamp(0.0, 255.0) as u8,
        (m[1] * inv).round().clamp(0.0, 255.0) as u8,
        (m[2] * inv).round().clamp(0.0, 255.0) as u8,
        a.round().clamp(0.0, 255.0) as u8,
    ]
}

/// The lifted selection (or whole layer) being transformed, plus the remainder it floats over.
#[derive(Clone)]
pub(crate) struct FloatingPatch {
    /// Canvas-sized straight RGBA: the lifted pixels (selected content), transparent elsewhere.
    pub patch: Arc<Vec<u8>>,
    /// Canvas-sized straight RGBA: the source with the lifted region cleared (the hole the patch floats over).
    pub base: Arc<Vec<u8>>,
    /// The patch's source bbox (selection bbox, or opaque-content bbox when whole-layer).
    pub src: Region,
}

impl PainterTool {
    /// Lift the floating patch from the current canvas and start a Transform. Uses the selection coverage as
    /// the cut mask (whole layer when nothing is selected), consumes the selection marquee, and frames the
    /// gizmo on the lifted region. Captures the undo baseline (committed when the transform ends). No-op when
    /// already lifted or the canvas is unsized. Returns `true` when a patch was lifted.
    pub(super) fn begin_transform(&mut self) -> bool {
        if self.paint.deform.xform_patch.is_some() {
            return true;
        }
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if n == 0 || self.canvas_rgba.len() != n * 4 {
            return false;
        }
        let before = self.snapshot_model();
        let restrict = self.deform_restricts_to_selection();
        let cur = Arc::clone(&self.canvas_rgba);
        let mut patch = vec![0u8; n * 4];
        let mut base = (*cur).clone();
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
        let mut any = false;
        for y in 0..h {
            for x in 0..w {
                let m = if restrict {
                    f32::from(self.selection_coverage_at(x, y)) / 255.0
                } else {
                    1.0
                };
                if m <= 0.0 {
                    continue;
                }
                let i = ((y * w + x) as usize) * 4;
                let a = f32::from(cur[i + 3]);
                // Split the source alpha: the patch takes the `m` fraction, the base keeps `1 − m` (a clean
                // partition → additive/over reconstruct the source at identity for a hard mask).
                patch[i] = cur[i];
                patch[i + 1] = cur[i + 1];
                patch[i + 2] = cur[i + 2];
                patch[i + 3] = (a * m).round() as u8;
                base[i + 3] = (a * (1.0 - m)).round() as u8;
                if patch[i + 3] > 0 {
                    any = true;
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
            }
        }
        if !any {
            return false; // nothing to lift (empty layer / empty selection)
        }
        let src = Region {
            x: x0,
            y: y0,
            w: x1 - x0 + 1,
            h: y1 - y0 + 1,
        };
        let frame = frame_from_region(src);
        self.paint.deform.xform_patch = Some(FloatingPatch {
            patch: Arc::new(patch),
            base: Arc::new(base),
            src,
        });
        self.paint.deform.xform = Some(super::transform::Xform {
            pristine: frame,
            current: frame,
            corners: None,
        });
        self.paint.deform.xform_before = Some(before);
        self.paint.deform.xform_last_bbox = None;
        self.paint.deform.xform_redo.clear();
        self.paint.deform.xform_relift = None;
        self.paint.deform.xform_undo.clear();
        self.paint.deform.xform_moved = false;
        // Consume the selection: the marquee disappears (Procreate) — the region now lives in the patch. Done
        // WITHOUT its own undo (the transform's single `before` snapshot, captured above, still holds it).
        if restrict {
            self.consume_selection_into_transform();
        }
        // Canvas is unchanged (identity composite == source) → byte-identical until the first drag.
        true
    }

    /// Re-composite the floating patch under the projective map `m` (affine for Uniform/Free, a homography
    /// for Distort): `out = over(patch(m⁻¹·dst), base)` inside the affected bbox, `base` where the patch
    /// vacated. Loops only the patch's source + destination bbox (+ the previous frame's), never the whole
    /// canvas. No-op without a lifted patch or a singular `m`.
    pub(super) fn composite_transform(&mut self, m: Mat3) {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let Some(fp) = self.paint.deform.xform_patch.clone() else {
            return;
        };
        if self.canvas_rgba.len() != n * 4 {
            return;
        }
        let Some(minv) = m.inverse() else {
            return;
        };
        let dest = transform_aabb(&m, fp.src, w, h);
        let affected = region_union(fp.src, dest);
        let dirty = region_clip(
            region_union(
                affected,
                self.paint.deform.xform_last_bbox.unwrap_or(affected),
            ),
            w,
            h,
        );
        let patch = &fp.patch;
        let base = &fp.base;
        let buf =
            crate::tool::paint::plane_fork::fork_par(&mut self.canvas_rgba, &self.undo_window);
        let stride = (w as usize) * 4;
        // Whole-image transforms make `dirty` the entire canvas (~4.2M px @ 2048²) — a serial per-texel
        // gather was ~190 ms/move (measured, `perf_transform_whole_image_table`). Split the rows into
        // disjoint bands across the cores (bit-identical to serial), turn the strips OUTSIDE the affected
        // bbox into straight row copies of `base`, and only run the generic `over` on antialiased edges.
        for_dirty_rows_parallel(buf, dirty, w, |band, band_y0| {
            let rows = band.len() / stride;
            let (d0, d1) = ((dirty.x as usize) * 4, ((dirty.x + dirty.w) as usize) * 4);
            for ry in 0..rows {
                let y = band_y0 + ry as u32;
                let row = &mut band[ry * stride..(ry + 1) * stride];
                let brow = (y as usize) * stride;
                // The affected x-span on this row (empty when the row is outside the affected bbox).
                let (ax0, ax1) = if y >= affected.y && y < affected.y + affected.h {
                    let x0 = affected.x.max(dirty.x);
                    let x1 = (affected.x + affected.w).min(dirty.x + dirty.w);
                    (x0, x1.max(x0))
                } else {
                    (dirty.x, dirty.x)
                };
                let (a0, a1) = ((ax0 as usize) * 4, (ax1 as usize) * 4);
                // The patch left these texels → show the base (the hole): straight copies.
                row[d0..a0].copy_from_slice(&base[brow + d0..brow + a0]);
                row[a1..d1].copy_from_slice(&base[brow + a1..brow + d1]);
                for x in ax0..ax1 {
                    let o = (x as usize) * 4;
                    let sp = minv.apply([x as f32, y as f32]);
                    let p = bilinear_patch(patch, w, h, sp[0], sp[1]);
                    let b = [
                        base[brow + o],
                        base[brow + o + 1],
                        base[brow + o + 2],
                        base[brow + o + 3],
                    ];
                    row[o..o + 4].copy_from_slice(&over_fast(p, b));
                }
            }
        });
        self.paint.deform.xform_last_bbox = Some(affected);
        self.mark_dirty(dirty);
    }

    /// Reset the live transform to identity (the patch snaps back to its source) without ending it. Drops
    /// any free geometry (Distort corners / Warp mesh).
    pub(super) fn reset_transform(&mut self) {
        let Some(x) = self.paint.deform.xform else {
            return;
        };
        self.paint.deform.xform = Some(super::transform::Xform {
            pristine: x.pristine,
            current: x.pristine,
            corners: None,
        });
        self.paint.deform.xform_grab = None;
        self.paint.deform.xform_mesh = None;
        self.paint.deform.xform_redo.clear();
        self.paint.deform.xform_relift = None;
        self.paint.deform.xform_undo.clear();
        self.paint.deform.xform_moved = false;
        self.composite_transform(Mat3::from_affine(Affine2::IDENTITY));
    }

    /// End the Transform: bake the composited pixels (already in `canvas`) and drop the float. When `commit`,
    /// records the whole transform as ONE structural undo entry (from the lift baseline). Returns whether a
    /// transform was active.
    pub(crate) fn end_transform(&mut self, commit: bool) -> bool {
        if self.paint.deform.xform_patch.take().is_none() {
            self.paint.deform.xform = None;
            self.paint.deform.xform_grab = None;
            self.paint.deform.xform_mesh = None;
            self.paint.deform.xform_before = None;
            self.paint.deform.xform_last_bbox = None;
            self.paint.deform.xform_redo.clear();
            self.paint.deform.xform_relift = None;
            self.paint.deform.xform_undo.clear();
            self.paint.deform.xform_moved = false;
            return false;
        }
        let before = self.paint.deform.xform_before.take();
        self.paint.deform.xform = None;
        self.paint.deform.xform_grab = None;
        self.paint.deform.xform_mesh = None;
        self.paint.deform.xform_last_bbox = None;
        self.paint.deform.xform_redo.clear();
        self.paint.deform.xform_relift = None;
        self.paint.deform.xform_undo.clear();
        self.paint.deform.xform_moved = false;
        if commit && let Some(b) = before {
            self.commit_structural_edit(b);
        }
        true
    }

    /// Clear the selection marquee inline (no separate undo) when the transform captures it.
    fn consume_selection_into_transform(&mut self) {
        self.paint.selection_edit_mode = false;
        self.paint.selection_grab = None;
        self.paint.selection_active = false;
        self.paint.selection_mask = Arc::new(Vec::new());
        self.paint.selection_crisp = Arc::new(Vec::new());
        self.paint.selection_shapes.clear();
        self.reset_selection_offset();
        self.invalidate_composite();
    }
}

/// [`over`] with the three exact fast paths — transparent src (keep dst), opaque src / transparent dst
/// (take src). The generic blend then only runs on antialiased patch edges. Each path returns exactly
/// what `over` computes for it (the float round-trip `(v·a)/a` is within ~2 ulp of the integer `v`, far
/// from any `.round()` boundary), so the composite stays byte-identical to the plain-`over` loop.
#[inline]
pub(super) fn over_fast(src: [u8; 4], dst: [u8; 4]) -> [u8; 4] {
    if src[3] == 0 {
        return dst;
    }
    if src[3] == 255 || dst[3] == 0 {
        return src;
    }
    over(src, dst)
}

/// Run `f` over the dirty rect's rows in disjoint full-width row bands — in parallel across the cores
/// for large rects (mirrors `parallel_band_stamp` in `ph2d-painter-brush`). `f(band, band_y0)` receives
/// the band's FULL-width row slice whose first row is `band_y0`. Disjoint bands ⇒ bit-identical to the
/// serial loop regardless of band count (HR-5); small rects (a typical selection drag) stay serial so
/// the thread spawn never costs more than it saves.
pub(super) fn for_dirty_rows_parallel<F>(buf: &mut [u8], dirty: Region, w: u32, f: F)
where
    F: Fn(&mut [u8], u32) + Sync,
{
    if dirty.w == 0 || dirty.h == 0 {
        return;
    }
    let stride = (w as usize) * 4;
    let region = &mut buf[(dirty.y as usize) * stride..((dirty.y + dirty.h) as usize) * stride];
    let rows = dirty.h as usize;
    /// Below this dirty-rect area (px) the serial loop wins over spawning scoped threads.
    const PARALLEL_MIN_AREA: usize = 1 << 17;
    if (rows * dirty.w as usize) < PARALLEL_MIN_AREA || rows <= 1 {
        f(region, dirty.y);
        return;
    }
    let threads = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(1)
        .clamp(1, rows);
    let rpb = rows.div_ceil(threads);
    let f = &f;
    std::thread::scope(|s| {
        let handles: Vec<_> = region
            .chunks_mut(rpb * stride)
            .enumerate()
            .map(|(bi, chunk)| {
                let band_y0 = dirty.y + (bi * rpb) as u32;
                s.spawn(move || f(chunk, band_y0))
            })
            .collect();
        for h in handles {
            h.join().expect("transform composite band panicked");
        }
    });
}

/// Straight-alpha src-over: `src` over `dst`.
pub(super) fn over(src: [u8; 4], dst: [u8; 4]) -> [u8; 4] {
    let sa = f32::from(src[3]) / 255.0;
    let da = f32::from(dst[3]) / 255.0;
    let oa = sa + da * (1.0 - sa);
    if oa <= 0.0 {
        return [0, 0, 0, 0];
    }
    let mut out = [0u8; 4];
    for c in 0..3 {
        let s = f32::from(src[c]);
        let d = f32::from(dst[c]);
        out[c] = ((s * sa + d * da * (1.0 - sa)) / oa)
            .round()
            .clamp(0.0, 255.0) as u8;
    }
    out[3] = (oa * 255.0).round().clamp(0.0, 255.0) as u8;
    out
}

/// The axis-aligned frame of a region (centre + half-extents, axis-aligned).
fn frame_from_region(r: Region) -> TransformFrame {
    TransformFrame {
        center: [r.x as f32 + r.w as f32 * 0.5, r.y as f32 + r.h as f32 * 0.5],
        u: [1.0, 0.0],
        hx: (r.w as f32 * 0.5).max(1.0),
        hy: (r.h as f32 * 0.5).max(1.0),
    }
}

/// The AABB (clamped to the canvas) that region `r` maps to under the projective map `m`.
fn transform_aabb(m: &Mat3, r: Region, w: u32, h: u32) -> Region {
    let (x0, y0) = (r.x as f32, r.y as f32);
    let (x1, y1) = ((r.x + r.w) as f32, (r.y + r.h) as f32);
    let corners = [
        m.apply([x0, y0]),
        m.apply([x1, y0]),
        m.apply([x0, y1]),
        m.apply([x1, y1]),
    ];
    let mut minx = f32::INFINITY;
    let mut miny = f32::INFINITY;
    let mut maxx = f32::NEG_INFINITY;
    let mut maxy = f32::NEG_INFINITY;
    for c in corners {
        minx = minx.min(c[0]);
        miny = miny.min(c[1]);
        maxx = maxx.max(c[0]);
        maxy = maxy.max(c[1]);
    }
    let ix0 = (minx.floor().max(0.0) as u32).min(w);
    let iy0 = (miny.floor().max(0.0) as u32).min(h);
    let ix1 = (maxx.ceil().max(0.0) as u32).min(w);
    let iy1 = (maxy.ceil().max(0.0) as u32).min(h);
    Region {
        x: ix0,
        y: iy0,
        w: ix1.saturating_sub(ix0),
        h: iy1.saturating_sub(iy0),
    }
}

/// The bounding union of two regions.
pub(super) fn region_union(a: Region, b: Region) -> Region {
    if a.w == 0 || a.h == 0 {
        return b;
    }
    if b.w == 0 || b.h == 0 {
        return a;
    }
    let x0 = a.x.min(b.x);
    let y0 = a.y.min(b.y);
    let x1 = (a.x + a.w).max(b.x + b.w);
    let y1 = (a.y + a.h).max(b.y + b.h);
    Region {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    }
}

/// Clamp a region to the canvas bounds.
pub(super) fn region_clip(r: Region, w: u32, h: u32) -> Region {
    let x0 = r.x.min(w);
    let y0 = r.y.min(h);
    let x1 = (r.x + r.w).min(w);
    let y1 = (r.y + r.h).min(h);
    Region {
        x: x0,
        y: y0,
        w: x1.saturating_sub(x0),
        h: y1.saturating_sub(y0),
    }
}
