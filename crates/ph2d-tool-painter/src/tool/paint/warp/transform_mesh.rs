//! **Warp mesh** (Deform Wave 2) — Distort's big brother. A lattice of control points over the floating
//! patch; each point drags independently and every grid CELL warps by its own 4-point homography (a C0
//! piecewise-projective warp — cells meet exactly at the shared control points). Reuses the [`FloatingPatch`]
//! composite: clear the dirty rect to `base`, then rasterize each deformed cell (backward-gather the patch
//! through the cell's inverse homography, clipped to the cell quad). Transcendental-free (HR-5).

use super::super::Region;
use super::apply::bilinear_clamped;
use super::transform_float::{over, region_clip, region_union};
use super::transform_geom::homography_from_quads;
use crate::tool::PainterTool;
use std::sync::Arc;

/// A `cols × rows` cell lattice (so `(cols+1) × (rows+1)` control points, row-major). `pristine` is the
/// undeformed grid (the `M = I` reference); `current` is the dragged grid.
#[derive(Clone, Debug)]
pub(crate) struct Mesh {
    pub cols: u32,
    pub rows: u32,
    pub pristine: Vec<[f32; 2]>,
    pub current: Vec<[f32; 2]>,
}

impl Mesh {
    /// Seed an evenly-spaced grid over the box corners `b = [TL, TR, BR, BL]` (bilinear), pristine == current.
    pub(crate) fn seed(b: [[f32; 2]; 4], cols: u32, rows: u32) -> Mesh {
        let mut pts = Vec::with_capacity(((cols + 1) * (rows + 1)) as usize);
        for r in 0..=rows {
            let v = r as f32 / rows as f32;
            for c in 0..=cols {
                let u = c as f32 / cols as f32;
                let top = lerp(b[0], b[1], u); // TL→TR
                let bot = lerp(b[3], b[2], u); // BL→BR
                pts.push(lerp(top, bot, v));
            }
        }
        Mesh {
            cols,
            rows,
            pristine: pts.clone(),
            current: pts,
        }
    }

    #[inline]
    fn idx(&self, r: u32, c: u32) -> usize {
        (r * (self.cols + 1) + c) as usize
    }

    /// The `[TL, TR, BR, BL]` corners of cell `(r, c)` in `grid`.
    fn cell(&self, grid: &[[f32; 2]], r: u32, c: u32) -> [[f32; 2]; 4] {
        [
            grid[self.idx(r, c)],
            grid[self.idx(r, c + 1)],
            grid[self.idx(r + 1, c + 1)],
            grid[self.idx(r + 1, c)],
        ]
    }

    /// The index of the control point within `tol` of `pos` (nearest), or `None`.
    pub(crate) fn nearest_point(&self, pos: [f32; 2], tol: f32) -> Option<usize> {
        let tol2 = tol * tol;
        let mut best = None;
        let mut bestd = tol2;
        for (i, p) in self.current.iter().enumerate() {
            let dx = p[0] - pos[0];
            let dy = p[1] - pos[1];
            let d = dx * dx + dy * dy;
            if d <= bestd {
                bestd = d;
                best = Some(i);
            }
        }
        best
    }
}

impl PainterTool {
    /// Re-composite the floating patch through the Warp mesh: clear the dirty rect to `base`, then rasterize
    /// each deformed cell (backward-gather the patch through the cell's inverse homography, clipped to the
    /// cell quad). Loops only the deformed mesh bbox (+ the previous frame's). No-op without a patch / mesh.
    pub(super) fn composite_mesh(&mut self) {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let (Some(fp), Some(mesh)) = (
            self.paint.deform.xform_patch.clone(),
            self.paint.deform.xform_mesh.clone(),
        ) else {
            return;
        };
        if self.canvas_rgba.len() != n * 4 {
            return;
        }
        // The deformed mesh's overall bbox (union of every current control point).
        let affected = points_bbox(&mesh.current, w, h);
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
        // 1) Clear the dirty rect to the base (the hole the patch floats over).
        for y in dirty.y..dirty.y + dirty.h {
            for x in dirty.x..dirty.x + dirty.w {
                let i = ((y * w + x) as usize) * 4;
                buf[i..i + 4].copy_from_slice(&[base[i], base[i + 1], base[i + 2], base[i + 3]]);
            }
        }
        // 2) Overlay each deformed cell (backward-gather through its inverse homography, clipped to the quad).
        for r in 0..mesh.rows {
            for c in 0..mesh.cols {
                let src = mesh.cell(&mesh.pristine, r, c);
                let dst = mesh.cell(&mesh.current, r, c);
                let Some(hinv) = homography_from_quads(&src, &dst).and_then(|m| m.inverse()) else {
                    continue;
                };
                let cell_bbox = region_clip(points_bbox(&dst, w, h), w, h);
                for y in cell_bbox.y..cell_bbox.y + cell_bbox.h {
                    for x in cell_bbox.x..cell_bbox.x + cell_bbox.w {
                        let p = [x as f32 + 0.5, y as f32 + 0.5];
                        if !point_in_quad(&dst, p) {
                            continue;
                        }
                        let sp = hinv.apply([x as f32, y as f32]);
                        let px = bilinear_clamped(patch, w, h, sp[0], sp[1]);
                        let i = ((y * w + x) as usize) * 4;
                        let b = [base[i], base[i + 1], base[i + 2], base[i + 3]];
                        buf[i..i + 4].copy_from_slice(&over(px, b));
                    }
                }
            }
        }
        self.paint.deform.xform_last_bbox = Some(affected);
        self.mark_dirty(dirty);
    }
}

/// Linear interpolation of two points.
fn lerp(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

/// The AABB (clamped to the canvas) enclosing a point set (+1 px margin so edge texels are covered).
fn points_bbox(pts: &[[f32; 2]], w: u32, h: u32) -> Region {
    let mut minx = f32::INFINITY;
    let mut miny = f32::INFINITY;
    let mut maxx = f32::NEG_INFINITY;
    let mut maxy = f32::NEG_INFINITY;
    for p in pts {
        minx = minx.min(p[0]);
        miny = miny.min(p[1]);
        maxx = maxx.max(p[0]);
        maxy = maxy.max(p[1]);
    }
    if !minx.is_finite() {
        return Region {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        };
    }
    let x0 = (minx.floor().max(0.0) as u32).min(w);
    let y0 = (miny.floor().max(0.0) as u32).min(h);
    let x1 = ((maxx.ceil() + 1.0).max(0.0) as u32).min(w);
    let y1 = ((maxy.ceil() + 1.0).max(0.0) as u32).min(h);
    Region {
        x: x0,
        y: y0,
        w: x1.saturating_sub(x0),
        h: y1.saturating_sub(y0),
    }
}

/// Whether `p` is inside the (convex) quad `q = [TL, TR, BR, BL]` — all edge cross-products share a sign.
/// A tiny epsilon keeps shared cell edges from dropping seam pixels (both adjacent cells claim them).
fn point_in_quad(q: &[[f32; 2]; 4], p: [f32; 2]) -> bool {
    let mut pos = false;
    let mut neg = false;
    for i in 0..4 {
        let a = q[i];
        let b = q[(i + 1) % 4];
        let cross = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
        if cross > 1e-3 {
            pos = true;
        } else if cross < -1e-3 {
            neg = true;
        }
    }
    !(pos && neg)
}
