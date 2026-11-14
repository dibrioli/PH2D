//! Selection **rasterization** (ADR-0103) — turning a gesture (rectangle / ellipse marquee, lasso polygon,
//! Automatic flood) into a canvas-sized coverage buffer, combining it into the mask by the boolean op, and
//! deriving the Feathered effective mask from the crisp accumulator. Split from `selection` for the LOC cap.

use super::PainterTool;
use std::sync::Arc;

/// Largest Feather radius (image px) the `0..1` Feather slider maps to.
pub(super) const FEATHER_MAX_PX: usize = 64;

impl PainterTool {
    /// Rasterize a rectangle / ellipse marquee (image px) into a canvas-sized coverage buffer (0 / 255).
    pub(super) fn raster_marquee(&self, a: [f32; 2], b: [f32; 2], ellipse: bool) -> Vec<u8> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let mut cov = vec![0u8; w * h];
        if w == 0 || h == 0 {
            return cov;
        }
        let x0 = a[0].min(b[0]).floor().clamp(0.0, w as f32) as usize;
        let x1 = a[0].max(b[0]).ceil().clamp(0.0, w as f32) as usize;
        let y0 = a[1].min(b[1]).floor().clamp(0.0, h as f32) as usize;
        let y1 = a[1].max(b[1]).ceil().clamp(0.0, h as f32) as usize;
        if ellipse {
            let cx = (x0 as f32 + x1 as f32) * 0.5;
            let cy = (y0 as f32 + y1 as f32) * 0.5;
            let rx = ((x1.saturating_sub(x0)) as f32 * 0.5).max(0.5);
            let ry = ((y1.saturating_sub(y0)) as f32 * 0.5).max(0.5);
            for yy in y0..y1 {
                for xx in x0..x1 {
                    let dx = (xx as f32 + 0.5 - cx) / rx;
                    let dy = (yy as f32 + 0.5 - cy) / ry;
                    if dx * dx + dy * dy <= 1.0 {
                        cov[yy * w + xx] = 255;
                    }
                }
            }
        } else {
            for yy in y0..y1 {
                for xx in x0..x1 {
                    cov[yy * w + xx] = 255;
                }
            }
        }
        cov
    }

    /// Rasterize a closed lasso polygon (image px) into a canvas-sized coverage buffer via an even-odd
    /// scanline fill. Needs ≥3 points.
    pub(super) fn raster_lasso(&self, pts: &[[f32; 2]]) -> Vec<u8> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let mut cov = vec![0u8; w * h];
        if w == 0 || h == 0 || pts.len() < 3 {
            return cov;
        }
        // Only rows inside the polygon's y-extent can cross an edge (NaN vertices never cross).
        let (ymin, ymax) = pts
            .iter()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), p| {
                (a.min(p[1]), b.max(p[1]))
            });
        let y0 = (ymin - 1.0).floor().clamp(0.0, h as f32) as usize;
        let y1 = (ymax + 1.0).ceil().clamp(0.0, h as f32) as usize;
        for yy in y0..y1 {
            let yc = yy as f32 + 0.5;
            let mut xs: Vec<f32> = Vec::new();
            for i in 0..pts.len() {
                let p = pts[i];
                let q = pts[(i + 1) % pts.len()];
                let (py, qy) = (p[1], q[1]);
                if (py <= yc && yc < qy) || (qy <= yc && yc < py) {
                    let t = (yc - py) / (qy - py);
                    xs.push(p[0] + t * (q[0] - p[0]));
                }
            }
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mut i = 0;
            while i + 1 < xs.len() {
                let xa = xs[i].max(0.0).round() as usize;
                let xb = (xs[i + 1].min(w as f32).round() as usize).min(w);
                for xx in xa..xb {
                    cov[yy * w + xx] = 255;
                }
                i += 2;
            }
        }
        cov
    }

    /// Rasterize an Automatic flood-select from `seed` at the current threshold into a coverage buffer.
    pub(super) fn raster_flood(&self, seed: [f32; 2]) -> Vec<u8> {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let mut cov = vec![0u8; w * h];
        if w == 0 || h == 0 || self.canvas_rgba.len() != w * h * 4 {
            return cov;
        }
        let sx = (seed[0].floor() as i32).clamp(0, w as i32 - 1) as usize;
        let sy = (seed[1].floor() as i32).clamp(0, h as i32 - 1) as usize;
        let tol = (self.paint.selection_threshold.clamp(0.0, 1.0) * 255.0).round() as u8;
        flood_coverage(&self.canvas_rgba, w, h, (sx, sy), tol, &mut cov);
        cov
    }

    /// Combine `region` (canvas-sized coverage) into the selection mask using the current boolean op,
    /// against the gesture's base selection. Updates `selection_active`.
    pub(super) fn apply_selection_region(&mut self, region: &[u8]) {
        let n = (self.source_size.0 as usize) * (self.source_size.1 as usize);
        if n == 0 || region.len() != n {
            return;
        }
        let base = if self.paint.selection_base.len() == n {
            Arc::clone(&self.paint.selection_base)
        } else {
            Arc::new(vec![0u8; n])
        };
        if self.paint.selection_bool_op == 0 {
            self.reset_selection_offset(); // a New gesture (marquee / flood) starts from a clean offset
        }
        // ⚠️ **UMA lei, dois consumidores.** Isto aqui era uma SEGUNDA cópia do `combine_into` — o
        // mesmo `max` / `base ∧ ¬region` / replace escritos de novo —, e as duas divergiram no dia em
        // que o Intersect chegou: a lista de formas o compunha e este caminho VIVO caía no `_`, que
        // REPLACE, então o gesto mostrava a segunda faixa inteira enquanto o resultado guardado era a
        // interseção. Byte-idêntico para New/Add/Remove: `out` começa na `base`, que é exatamente o
        // `crisp` que o `combine_into` recebe.
        let mut out = base.as_slice().to_vec();
        super::selection_shapes::combine_into(&mut out, region, self.paint.selection_bool_op);
        self.set_selection_from_crisp(out);
    }

    /// Install a CRISP selection accumulator and derive the effective (Feathered) `selection_mask` from it.
    /// The single funnel for every op that changes WHICH texels are selected (marquee / lasso / flood /
    /// rect seed / invert / list recompose) — so Feather always re-derives from the crisp base.
    pub(super) fn set_selection_from_crisp(&mut self, crisp: Vec<u8>) {
        self.set_selection_from_crisp_arc(Arc::new(crisp));
    }

    /// [`Self::set_selection_from_crisp`] sharing an already-immutable buffer (the offset source, a cached
    /// offset level) instead of copying it (B40.A3). The buffer is never mutated in place: a new content is
    /// always a new `Arc`, which is what the marching-ants overlay keys on.
    pub(super) fn set_selection_from_crisp_arc(&mut self, crisp: Arc<Vec<u8>>) {
        self.paint.selection_active = crisp.iter().any(|&v| v > 0);
        self.paint.selection_crisp = crisp;
        self.derive_effective();
    }

    /// Recompute `selection_mask` = Feather(`selection_crisp`). Feather is a separable box blur (HR-5
    /// transcendental-free) whose radius scales with the Feather amount; `0` copies the crisp mask.
    pub(super) fn derive_effective(&mut self) {
        let crisp = Arc::clone(&self.paint.selection_crisp);
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let radius =
            (self.paint.selection_feather.clamp(0.0, 1.0) * FEATHER_MAX_PX as f32).round() as usize;
        if radius == 0 || w == 0 || h == 0 || crisp.len() != w * h {
            self.paint.selection_mask = crisp;
        } else {
            self.paint.selection_mask = Arc::new(feather(&crisp, w, h, radius));
        }
        // NB: a selection-mask change does NOT touch the layer composite — the compositor is provably
        // selection-independent (zero selection refs), the marquee/hatching is a per-frame shell OVERLAY, and
        // painting reads the mask directly. So we DON'T `invalidate_composite()` here: doing so dropped the
        // whole composite cache + forced a FULL-canvas GPU re-upload on EVERY gizmo move (the "serious FPS
        // drop with N boolean shapes"). The one-shot create/clear/commit sites still invalidate where a
        // genuine structural change warrants it (perf audit 2026-07-04, lens F2).
    }
}

/// Feather: three box-blur passes ≈ a Gaussian (central-limit), a smooth, regular ramp. Runs in the
/// window = bbox(crisp) ⊕ 3r: outside it the three passes are 0, and inside it every clamped sample past
/// the window edge is 0 too (pass k is nonzero only within bbox ⊕ k·r), so it equals the whole-canvas blur.
fn feather(crisp: &[u8], w: usize, h: usize, r: usize) -> Vec<u8> {
    let mut out = vec![0u8; w * h];
    let Some([bx0, by0, bx1, by1]) = nonzero_bbox(crisp, w, h) else {
        return out;
    };
    let m = 3 * r;
    let (x0, y0) = (bx0.saturating_sub(m), by0.saturating_sub(m));
    let (x1, y1) = ((bx1 + m).min(w), (by1 + m).min(h));
    let (ww, wh) = (x1 - x0, y1 - y0);
    let mut a = vec![0u8; ww * wh];
    for (y, row) in a.chunks_exact_mut(ww).enumerate() {
        row.copy_from_slice(&crisp[(y0 + y) * w + x0..][..ww]);
    }
    let mut tmp = vec![0u8; ww * wh];
    // Row bands on the pool (ADR-0202): every byte is its own exact sum, so no band count changes it.
    let threads = crate::compositor::parallel_threads(ww * wh, wh);
    for _ in 0..3 {
        crate::compositor::em_faixas_de_linhas(&mut tmp, ww, threads, |band, y| {
            blur_rows(&a[y * ww..][..band.len()], band, ww, r);
        });
        crate::compositor::em_faixas_de_linhas(&mut a, ww, threads, |band, y| {
            blur_cols(&tmp, band, ww, wh, r, y);
        });
    }
    for (y, row) in a.chunks_exact(ww).enumerate() {
        out[(y0 + y) * w + x0..][..ww].copy_from_slice(row);
    }
    out
}

/// `[x0, y0, x1, y1)` of the nonzero texels, `None` when all are 0.
fn nonzero_bbox(v: &[u8], w: usize, h: usize) -> Option<[usize; 4]> {
    let (mut x0, mut x1) = (w, 0);
    let (mut y0, mut y1) = (h, 0);
    for (y, row) in v.chunks_exact(w).enumerate() {
        let Some(a) = row.iter().position(|&p| p != 0) else {
            continue;
        };
        let b = row.iter().rposition(|&p| p != 0).unwrap_or(a);
        (x0, x1, y0, y1) = (x0.min(a), x1.max(b + 1), y0.min(y), y + 1);
    }
    (y1 > 0).then_some([x0, y0, x1, y1])
}

/// Horizontal box pass: a `(2r+1)` window clamped to the row ends, as a sliding u32 sum (O(1)/texel;
/// the same integer sum as summing the window, so `/win` gives the same byte).
fn blur_rows(src: &[u8], dst: &mut [u8], w: usize, r: usize) {
    let win = (2 * r + 1) as u32;
    let last = w - 1;
    for (s, d) in src.chunks_exact(w).zip(dst.chunks_exact_mut(w)) {
        let mut acc: u32 = (0..=2 * r)
            .map(|k| u32::from(s[k.saturating_sub(r).min(last)]))
            .sum();
        for (x, o) in d.iter_mut().enumerate() {
            *o = (acc / win) as u8;
            acc = acc + u32::from(s[(x + r + 1).min(last)]) - u32::from(s[x.saturating_sub(r)]);
        }
    }
}

/// Vertical box pass over the band of `dst` that starts at row `y0`, row by row: one running sum per
/// column (reads follow the stride, no column walk); the band seeds its own sum, so bands are independent.
fn blur_cols(src: &[u8], dst: &mut [u8], w: usize, h: usize, r: usize, y0: usize) {
    let win = (2 * r + 1) as u32;
    let last = h - 1;
    let row = |y: usize| &src[y * w..][..w];
    let mut acc = vec![0u32; w];
    for k in 0..=2 * r {
        for (a, &v) in acc
            .iter_mut()
            .zip(row((y0 + k).saturating_sub(r).min(last)))
        {
            *a += u32::from(v);
        }
    }
    for (i, d) in dst.chunks_exact_mut(w).enumerate() {
        let y = y0 + i;
        for (o, &a) in d.iter_mut().zip(&acc) {
            *o = (a / win) as u8;
        }
        let (add, sub) = (row((y + r + 1).min(last)), row(y.saturating_sub(r)));
        for ((a, &p), &q) in acc.iter_mut().zip(add).zip(sub) {
            *a = *a + u32::from(p) - u32::from(q);
        }
    }
}

/// Scanline flood — like [`super::fill::flood_fill`] but WRITES a coverage buffer (255 in-region) instead
/// of painting, reading `px` (RGBA8) read-only. Matches the 4-connected region within `tol` of the seed.
fn flood_coverage(px: &[u8], w: usize, h: usize, seed: (usize, usize), tol: u8, cov: &mut [u8]) {
    if w == 0 || h == 0 || px.len() != w * h * 4 || cov.len() != w * h {
        return;
    }
    let (sx, sy) = seed;
    if sx >= w || sy >= h {
        return;
    }
    let si = (sy * w + sx) * 4;
    let seed_c = [px[si], px[si + 1], px[si + 2], px[si + 3]];
    let matches = |idx: usize| -> bool {
        let o = idx * 4;
        px[o]
            .abs_diff(seed_c[0])
            .max(px[o + 1].abs_diff(seed_c[1]))
            .max(px[o + 2].abs_diff(seed_c[2]))
            .max(px[o + 3].abs_diff(seed_c[3]))
            <= tol
    };
    let mut visited = vec![false; w * h];
    let mut stack: Vec<(usize, usize)> = vec![seed];
    while let Some((x, y)) = stack.pop() {
        if visited[y * w + x] {
            continue;
        }
        let mut lx = x;
        while lx > 0 && !visited[y * w + lx - 1] && matches(y * w + lx - 1) {
            lx -= 1;
        }
        let mut rx = x;
        while rx + 1 < w && !visited[y * w + rx + 1] && matches(y * w + rx + 1) {
            rx += 1;
        }
        for xx in lx..=rx {
            let idx = y * w + xx;
            visited[idx] = true;
            cov[idx] = 255;
        }
        for ny in [y.checked_sub(1), (y + 1 < h).then_some(y + 1)]
            .into_iter()
            .flatten()
        {
            for xx in lx..=rx {
                if !visited[ny * w + xx] && matches(ny * w + xx) {
                    stack.push((xx, ny));
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "selection_raster_tests.rs"]
mod tests;
