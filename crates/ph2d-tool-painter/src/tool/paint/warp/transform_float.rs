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
use super::apply::bilinear_clamped;
use super::transform_geom::{Affine2, Mat3, TransformFrame};
use crate::tool::PainterTool;
use std::sync::Arc;

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
        let buf = Arc::make_mut(&mut self.canvas_rgba);
        for y in dirty.y..dirty.y + dirty.h {
            for x in dirty.x..dirty.x + dirty.w {
                let i = ((y * w + x) as usize) * 4;
                let out = if region_contains(affected, x, y) {
                    let sp = minv.apply([x as f32, y as f32]);
                    let p = bilinear_clamped(patch, w, h, sp[0], sp[1]);
                    let b = [base[i], base[i + 1], base[i + 2], base[i + 3]];
                    over(p, b)
                } else {
                    // The patch left this texel → show the base (the hole).
                    [base[i], base[i + 1], base[i + 2], base[i + 3]]
                };
                buf[i..i + 4].copy_from_slice(&out);
            }
        }
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

/// Whether `(x, y)` is inside region `r`.
fn region_contains(r: Region, x: u32, y: u32) -> bool {
    x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h
}
