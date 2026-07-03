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
        for yy in 0..h {
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
        let mut out = vec![0u8; n];
        match self.paint.selection_bool_op {
            // Add: union (max).
            1 => {
                for i in 0..n {
                    out[i] = base[i].max(region[i]);
                }
            }
            // Remove: base ∧ ¬region.
            2 => {
                for i in 0..n {
                    out[i] = ((u16::from(base[i]) * u16::from(255 - region[i])) / 255) as u8;
                }
            }
            // New: replace.
            _ => out.copy_from_slice(region),
        }
        self.set_selection_from_crisp(out);
    }

    /// Install a CRISP selection accumulator and derive the effective (Feathered) `selection_mask` from it.
    /// The single funnel for every op that changes WHICH texels are selected (marquee / lasso / flood /
    /// rect seed / invert / list recompose) — so Feather always re-derives from the crisp base.
    pub(super) fn set_selection_from_crisp(&mut self, crisp: Vec<u8>) {
        self.paint.selection_active = crisp.iter().any(|&v| v > 0);
        self.paint.selection_crisp = Arc::new(crisp);
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
            // Three box-blur passes ≈ a Gaussian (central-limit): a smooth, regular feather ramp.
            let mut m = box_blur(&crisp, w, h, radius);
            m = box_blur(&m, w, h, radius);
            m = box_blur(&m, w, h, radius);
            self.paint.selection_mask = Arc::new(m);
        }
        self.invalidate_composite();
    }
}

/// Separable box blur of a single-channel `w*h` coverage buffer (Feather). Two 1-D averaging passes (H
/// then V) with a `(2r+1)` window, clamping samples to the border — transcendental-free (HR-5).
fn box_blur(src: &[u8], w: usize, h: usize, r: usize) -> Vec<u8> {
    if r == 0 || w == 0 || h == 0 || src.len() != w * h {
        return src.to_vec();
    }
    let win = (2 * r + 1) as u32;
    let mut tmp = vec![0u8; w * h];
    for y in 0..h {
        let row = y * w;
        for x in 0..w {
            let mut acc = 0u32;
            for k in 0..(2 * r + 1) {
                let xx = (x + k).saturating_sub(r).min(w - 1);
                acc += u32::from(src[row + xx]);
            }
            tmp[row + x] = (acc / win) as u8;
        }
    }
    let mut out = vec![0u8; w * h];
    for x in 0..w {
        for y in 0..h {
            let mut acc = 0u32;
            for k in 0..(2 * r + 1) {
                let yy = (y + k).saturating_sub(r).min(h - 1);
                acc += u32::from(tmp[yy * w + x]);
            }
            out[y * w + x] = (acc / win) as u8;
        }
    }
    out
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
