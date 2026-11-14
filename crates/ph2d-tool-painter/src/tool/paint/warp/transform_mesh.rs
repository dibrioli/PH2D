//! **Warp mesh** (Deform Wave 2) — Distort's big brother. A coarse lattice of draggable control points over
//! the floating patch, Catmull-Rom **subdivided** into a fine grid so the deformation bends in SMOOTH curves
//! through the points (not faceted). Reuses the [`FloatingPatch`] composite: clear the dirty rect to `base`,
//! then rasterize each FINE cell (backward-gather the patch through the cell's inverse homography, clipped to
//! the cell quad). Transcendental-free (HR-5).

use super::super::Region;
use super::transform_float::{
    bilinear_patch, for_dirty_rows_parallel, over_fast, region_clip, region_union,
};
use super::transform_geom::{Mat3, homography_from_quads};
use crate::tool::PainterTool;

/// The cached Catmull-Rom subdivision of the PRISTINE Warp grid — constant during a whole drag, but it
/// was re-subdivided every move. Self-validating: `pristine` is compared against the live mesh's control
/// points, so a reseed / undo / redo simply re-fills it (no lifecycle to forget).
#[derive(Clone)]
pub(crate) struct MeshFineCache {
    pristine: Vec<[f32; 2]>,
    pts: Vec<[f32; 2]>,
    fc: u32,
    fr: u32,
}

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
    #[cfg(test)]
    pub(crate) fn seed(b: [[f32; 2]; 4], cols: u32, rows: u32) -> Mesh {
        Mesh::seed_between(b, b, cols, rows)
    }

    /// Seed with `pristine` over the box where the patch was LIFTED (the patch's own space) and `current`
    /// over the box where it is NOW — so the mesh starts as exactly the affine the box already shows.
    pub(crate) fn seed_between(
        pristine: [[f32; 2]; 4],
        current: [[f32; 2]; 4],
        cols: u32,
        rows: u32,
    ) -> Mesh {
        Mesh {
            cols,
            rows,
            pristine: grid_over(pristine, cols, rows),
            current: grid_over(current, cols, rows),
        }
    }

    /// The overlay grid lines: only the COARSE control lines (one per control row + column), but each traced
    /// smoothly through the Catmull-Rom fine points → a clean `(cols+1)×(rows+1)` set of CURVED lines (not a
    /// dense fine grid). Purely visual — the deformation still uses the full fine subdivision.
    pub(crate) fn smooth_lines(&self) -> Vec<Vec<[f32; 2]>> {
        let (fine, fc, fr) = subdivide_grid(&self.current, self.cols, self.rows);
        let stride = (fc + 1) as usize;
        let mut lines = Vec::with_capacity((self.cols + self.rows + 2) as usize);
        // One curved line per control COLUMN (traced down the fine rows).
        for cc in 0..=self.cols {
            let fcol = (cc * SUBDIV) as usize;
            lines.push((0..=fr).map(|r| fine[r as usize * stride + fcol]).collect());
        }
        // One curved line per control ROW (traced across the fine columns).
        for cr in 0..=self.rows {
            let frow = (cr * SUBDIV) as usize;
            lines.push((0..=fc).map(|c| fine[frow * stride + c as usize]).collect());
        }
        lines
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
    /// Re-composite the floating patch through the Warp mesh. To make the deformation SMOOTH (not the
    /// faceted per-control-cell look), the coarse control grid is first Catmull-Rom **subdivided** into a
    /// fine grid — the warp then bends in smooth curves through the control points. Each FINE cell is
    /// backward-gathered through its inverse homography (clipped to the cell quad). Clears the dirty rect to
    /// `base` first (the hole), loops only the deformed mesh bbox (+ the previous frame's). No-op without a
    /// patch / mesh.
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
        // Subdivide both grids to the SAME fine resolution (smooth Catmull-Rom surface through the points).
        // The PRISTINE grid is constant during the whole drag → cached; only `current` re-subdivides.
        if self
            .paint
            .deform
            .xform_mesh_fine
            .as_ref()
            .is_none_or(|c| c.pristine != mesh.pristine)
        {
            let (pts, fc, fr) = subdivide_grid(&mesh.pristine, mesh.cols, mesh.rows);
            self.paint.deform.xform_mesh_fine = Some(MeshFineCache {
                pristine: mesh.pristine.clone(),
                pts,
                fc,
                fr,
            });
        }
        let (fine_src, fc, fr) = {
            let c = self
                .paint
                .deform
                .xform_mesh_fine
                .as_ref()
                .expect("just filled");
            (c.pts.clone(), c.fc, c.fr)
        };
        let (fine_dst, _, _) = subdivide_grid(&mesh.current, mesh.cols, mesh.rows);
        let pstride = (fc + 1) as usize;
        let fine_cell = |grid: &[[f32; 2]], r: u32, c: u32| -> [[f32; 2]; 4] {
            let i = |rr: u32, cc: u32| grid[(rr as usize) * pstride + cc as usize];
            [i(r, c), i(r, c + 1), i(r + 1, c + 1), i(r + 1, c)]
        };

        // Precompute each fine cell ONCE per move (inverse homography + quads + reach).
        let mut cells = Vec::with_capacity((fr * fc) as usize);
        let mut affected = points_bbox(&fine_dst, w, h);
        for r in 0..fr {
            for c in 0..fc {
                let src = fine_cell(&fine_src, r, c);
                let dst = fine_cell(&fine_dst, r, c);
                let outer = [r == 0, c + 1 == fc, r + 1 == fr, c == 0];
                let bbox = region_clip(cell_reach(&src, &dst, outer, w, h), w, h);
                if bbox.w == 0 || bbox.h == 0 {
                    continue;
                }
                let Some(cell) = MeshCell::new(src, dst, outer, bbox) else {
                    continue;
                };
                affected = region_union(affected, bbox);
                cells.push(cell);
            }
        }
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
        let buf = crate::tool::paint::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            Some(dirty),
        );
        let stride = (w as usize) * 4;
        // Whole-image Warp makes `dirty` the entire canvas — split the rows into disjoint bands across
        // the cores (each band overlays the cells in the same (r,c) order, so the result is bit-identical
        // to the serial loop).
        for_dirty_rows_parallel(buf, dirty, w, |band, band_y0| {
            let rows = (band.len() / stride) as u32;
            let band_y1 = band_y0 + rows;
            let (d0, d1) = ((dirty.x as usize) * 4, ((dirty.x + dirty.w) as usize) * 4);
            // 1) Clear the band's dirty span to the base (the hole the patch floats over).
            for ry in 0..rows {
                let brow = ((band_y0 + ry) as usize) * stride;
                let row = &mut band[(ry as usize) * stride..(ry as usize + 1) * stride];
                row[d0..d1].copy_from_slice(&base[brow + d0..brow + d1]);
            }
            // 2) Overlay each FINE deformed cell clipped to this band (backward-gather through its
            //    inverse homography; ownership tested at the SAME point that is sampled).
            for cell in &cells {
                let y0 = cell.bbox.y.max(band_y0);
                let y1 = (cell.bbox.y + cell.bbox.h).min(band_y1);
                for y in y0..y1 {
                    let ry = (y - band_y0) as usize;
                    let row = &mut band[ry * stride..(ry + 1) * stride];
                    let brow = (y as usize) * stride;
                    for x in cell.bbox.x..cell.bbox.x + cell.bbox.w {
                        let Some(sp) = cell.locate(x, y) else {
                            continue;
                        };
                        let px = bilinear_patch(patch, w, h, sp[0], sp[1]);
                        let o = (x as usize) * 4;
                        let b = [
                            base[brow + o],
                            base[brow + o + 1],
                            base[brow + o + 2],
                            base[brow + o + 3],
                        ];
                        row[o..o + 4].copy_from_slice(&over_fast(px, b));
                    }
                }
            }
        });
        // …e o CORPO pela mesma malha, sobre a mesma janela, com a mesma posse por célula.
        #[cfg(test)]
        let corpo = !super::prancha_b06b_tests::antes();
        #[cfg(not(test))]
        let corpo = true;
        if corpo {
            self.composite_mesh_relief(&cells, dirty);
        }
        self.paint.deform.xform_last_bbox = Some(affected);
        self.mark_dirty(dirty);
    }
}

/// One fine cell of the Warp mesh, precomputed once per move. The homography and the ownership test live
/// in LOCAL coordinates (each quad relative to its first corner): built from canvas coordinates in f32, two
/// neighbouring homographies far from the origin disagree by more than a texel on their shared edge.
pub(super) struct MeshCell {
    hinv: Mat3,
    /// The source quad relative to `src_o`.
    src: [[f32; 2]; 4],
    src_o: [f32; 2],
    dst_o: [f32; 2],
    /// Which edges (`top, right, bottom, left`) are the OUTER border of the whole mesh.
    outer: [bool; 4],
    bbox: Region,
}

impl MeshCell {
    fn new(
        src: [[f32; 2]; 4],
        dst: [[f32; 2]; 4],
        outer: [bool; 4],
        bbox: Region,
    ) -> Option<MeshCell> {
        #[cfg(test)]
        let (src_o, dst_o) = if super::prancha_b06b_tests::antes() {
            ([0.0; 2], [0.0; 2])
        } else {
            (src[0], dst[0])
        };
        #[cfg(not(test))]
        let (src_o, dst_o) = (src[0], dst[0]);
        let local = |q: [[f32; 2]; 4], o: [f32; 2]| q.map(|p| [p[0] - o[0], p[1] - o[1]]);
        let src = local(src, src_o);
        let hinv = homography_from_quads(&src, &local(dst, dst_o)).and_then(|m| m.inverse())?;
        Some(MeshCell {
            hinv,
            src,
            src_o,
            dst_o,
            outer,
            bbox,
        })
    }

    /// The patch-space point texel `(x, y)` samples, when this cell owns it. The test is made in SOURCE
    /// space, at the very point the sampler reads — on the outer border it reaches one source texel past
    /// the edge (the bilinear's support), so the patch alpha draws the antialiased rim, as the affine does.
    pub(super) fn locate(&self, x: u32, y: u32) -> Option<[f32; 2]> {
        let (px, py) = (x as f32 - self.dst_o[0], y as f32 - self.dst_o[1]);
        let lp = self.hinv.apply([px, py]);
        let sp = [lp[0] + self.src_o[0], lp[1] + self.src_o[1]];
        #[cfg(test)]
        if super::prancha_b06b_tests::antes() {
            let centro = self.hinv.apply([px + 0.5, py + 0.5]);
            return in_cell(&self.src, centro, [false; 4]).then_some(sp);
        }
        in_cell(&self.src, lp, self.outer).then_some(sp)
    }

    pub(super) fn bbox(&self) -> Region {
        self.bbox
    }
}

/// The bilinear sampler's support past an outer edge, in SOURCE texels.
const SRC_RIM_TEXELS: f32 = 1.0;

/// The canvas reach of a cell: its dst bbox, grown on the outer border by the rim mapped to dst
/// (`SRC_RIM_TEXELS` × the cell's largest edge stretch, + 1 for the floor/ceil).
fn cell_reach(
    src: &[[f32; 2]; 4],
    dst: &[[f32; 2]; 4],
    outer: [bool; 4],
    w: u32,
    h: u32,
) -> Region {
    let b = points_bbox(dst, w, h);
    if !outer.iter().any(|o| *o) {
        return b;
    }
    let len = |q: &[[f32; 2]; 4], i: usize| {
        let (a, c) = (q[i], q[(i + 1) % 4]);
        ((c[0] - a[0]) * (c[0] - a[0]) + (c[1] - a[1]) * (c[1] - a[1])).sqrt()
    };
    let stretch = (0..4)
        .filter(|i| len(src, *i) > 0.0)
        .map(|i| len(dst, i) / len(src, i))
        .fold(1.0f32, f32::max);
    let m = (stretch * SRC_RIM_TEXELS).ceil() as u32 + 1;
    let x0 = b.x.saturating_sub(m);
    let y0 = b.y.saturating_sub(m);
    let x1 = (b.x + b.w + m).min(w);
    let y1 = (b.y + b.h + m).min(h);
    Region {
        x: x0,
        y: y0,
        w: x1.saturating_sub(x0),
        h: y1.saturating_sub(y0),
    }
}

/// How far past a shared edge, in SOURCE texels, both neighbours still claim a seam texel: covers the f32
/// rounding of two neighbouring homographies far from the origin (a double claim is harmless — each cell
/// writes `over(patch, base)` at ~the same point).
const SEAM_SLACK_TEXELS: f32 = 1e-3;

/// Whether `p` is inside the quad `q = [TL, TR, BR, BL]` (all edge distances share a sign, up to
/// [`SEAM_SLACK_TEXELS`]), with the OUTER edges pushed out by [`SRC_RIM_TEXELS`] (along the quad's orientation).
fn in_cell(q: &[[f32; 2]; 4], p: [f32; 2], outer: [bool; 4]) -> bool {
    let area2: f32 = (0..4)
        .map(|i| {
            let (a, b) = (q[i], q[(i + 1) % 4]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum();
    let s = if area2 > 0.0 {
        1.0
    } else if area2 < 0.0 {
        -1.0
    } else {
        0.0
    };
    let mut pos = false;
    let mut neg = false;
    for i in 0..4 {
        let a = q[i];
        let b = q[(i + 1) % 4];
        let cross = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
        let len = ((b[0] - a[0]) * (b[0] - a[0]) + (b[1] - a[1]) * (b[1] - a[1])).sqrt();
        // Signed distance to the edge line, in source texels (the slack must not shrink with the edge).
        let mut d = if len > 0.0 { cross / len } else { cross };
        if outer[i] {
            d += s * SRC_RIM_TEXELS;
        }
        if d > SEAM_SLACK_TEXELS {
            pos = true;
        } else if d < -SEAM_SLACK_TEXELS {
            neg = true;
        }
    }
    !(pos && neg)
}

/// How finely each control cell is subdivided for the smooth Catmull-Rom render (per axis). Higher = smoother
/// curves (and more fine cells). `5` turns a 3×3 control grid into a 15×15 fine grid.
const SUBDIV: u32 = 5;

/// Catmull-Rom subdivide a `cols × rows`-cell control grid (row-major `(cols+1)×(rows+1)` points) into a
/// smooth fine grid, separably (rows then columns). Returns `(fine_points, fine_cols, fine_rows)` in cells.
/// The spline passes THROUGH every control point (interpolating), so dragging a point stays under the cursor.
pub(crate) fn subdivide_grid(pts: &[[f32; 2]], cols: u32, rows: u32) -> (Vec<[f32; 2]>, u32, u32) {
    let cc = (cols + 1) as usize;
    let rc = (rows + 1) as usize;
    // Step 1: subdivide each control row along the columns.
    let row_fine: Vec<Vec<[f32; 2]>> = (0..rc)
        .map(|r| {
            let row: Vec<[f32; 2]> = (0..cc).map(|c| pts[r * cc + c]).collect();
            catmull_rom(&row, SUBDIV)
        })
        .collect();
    let fcp = row_fine[0].len(); // fine columns (points)
    // Step 2: subdivide each fine column along the rows.
    let col_fine: Vec<Vec<[f32; 2]>> = (0..fcp)
        .map(|c| {
            let col: Vec<[f32; 2]> = (0..rc).map(|r| row_fine[r][c]).collect();
            catmull_rom(&col, SUBDIV)
        })
        .collect();
    let frp = col_fine[0].len(); // fine rows (points)
    let mut out = Vec::with_capacity(frp * fcp);
    // Assemble row-major from the column-major `col_fine` (a transpose → indexed access is the clear form).
    for r in 0..frp {
        for col in &col_fine {
            out.push(col[r]);
        }
    }
    (out, (fcp - 1) as u32, (frp - 1) as u32)
}

/// Uniform Catmull-Rom interpolation of a point sequence with `sub` samples per segment (endpoints linearly
/// extrapolated for the boundary tangents). Interpolates through every input point.
fn catmull_rom(pts: &[[f32; 2]], sub: u32) -> Vec<[f32; 2]> {
    let n = pts.len();
    if n < 2 || sub == 0 {
        return pts.to_vec();
    }
    let get = |i: i64| -> [f32; 2] {
        if i < 0 {
            [2.0 * pts[0][0] - pts[1][0], 2.0 * pts[0][1] - pts[1][1]]
        } else if i as usize >= n {
            [
                2.0 * pts[n - 1][0] - pts[n - 2][0],
                2.0 * pts[n - 1][1] - pts[n - 2][1],
            ]
        } else {
            pts[i as usize]
        }
    };
    let mut out = Vec::with_capacity((n - 1) * sub as usize + 1);
    for i in 0..n - 1 {
        let (p0, p1, p2, p3) = (
            get(i as i64 - 1),
            get(i as i64),
            get(i as i64 + 1),
            get(i as i64 + 2),
        );
        for s in 0..sub {
            out.push(cr(p0, p1, p2, p3, s as f32 / sub as f32));
        }
    }
    out.push(pts[n - 1]);
    out
}

/// One Catmull-Rom sample at parameter `t ∈ [0, 1]` on the segment `p1 → p2`.
fn cr(p0: [f32; 2], p1: [f32; 2], p2: [f32; 2], p3: [f32; 2], t: f32) -> [f32; 2] {
    let t2 = t * t;
    let t3 = t2 * t;
    let comp = |a: f32, b: f32, c: f32, d: f32| {
        0.5 * (2.0 * b
            + (-a + c) * t
            + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2
            + (-a + 3.0 * b - 3.0 * c + d) * t3)
    };
    [
        comp(p0[0], p1[0], p2[0], p3[0]),
        comp(p0[1], p1[1], p2[1], p3[1]),
    ]
}

/// The bilinear `(cols+1)×(rows+1)` grid over a box `b = [TL, TR, BR, BL]` (row-major).
fn grid_over(b: [[f32; 2]; 4], cols: u32, rows: u32) -> Vec<[f32; 2]> {
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
    pts
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a regular `cols×rows`-cell grid over `[0,span]²` (row-major control points).
    fn regular(cols: u32, rows: u32, span: f32) -> Vec<[f32; 2]> {
        let mut v = Vec::new();
        for r in 0..=rows {
            for c in 0..=cols {
                v.push([span * c as f32 / cols as f32, span * r as f32 / rows as f32]);
            }
        }
        v
    }

    #[test]
    fn subdivision_passes_through_the_control_points() {
        // The Catmull-Rom surface INTERPOLATES: every coarse control point appears exactly in the fine grid
        // (at the `SUBDIV` multiples), so dragging a control point keeps it under the cursor.
        let (cols, rows) = (3u32, 3u32);
        let ctrl = regular(cols, rows, 30.0);
        let (fine, fc, _fr) = subdivide_grid(&ctrl, cols, rows);
        let fstride = (fc + 1) as usize;
        for r in 0..=rows {
            for c in 0..=cols {
                let ctrl_p = ctrl[(r * (cols + 1) + c) as usize];
                let fine_p = fine[(r * SUBDIV) as usize * fstride + (c * SUBDIV) as usize];
                assert!(
                    (ctrl_p[0] - fine_p[0]).abs() < 1e-3 && (ctrl_p[1] - fine_p[1]).abs() < 1e-3,
                    "control ({r},{c}) {ctrl_p:?} != fine {fine_p:?}"
                );
            }
        }
    }

    #[test]
    fn regular_grid_subdivides_to_a_regular_grid() {
        // A flat (un-deformed) grid must subdivide to an evenly-spaced fine grid — no ripples from the
        // spline (Catmull-Rom of collinear points is collinear), so the identity warp stays clean.
        let (cols, rows) = (3u32, 3u32);
        let (fine, fc, fr) = subdivide_grid(&regular(cols, rows, 30.0), cols, rows);
        let step = 30.0 / fc as f32;
        for r in 0..=fr {
            for c in 0..=fc {
                let p = fine[(r * (fc + 1) + c) as usize];
                let want = [step * c as f32, step * r as f32];
                assert!(
                    (p[0] - want[0]).abs() < 1e-2 && (p[1] - want[1]).abs() < 1e-2,
                    "fine ({r},{c}) {p:?} != {want:?}"
                );
            }
        }
    }
}
