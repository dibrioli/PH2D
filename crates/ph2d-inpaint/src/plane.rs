//! [`Plane`] — an interleaved RGB float image (`w*h*3`, sRGB in `[0,1]`) with
//! clamp-to-edge sampling and box-downsample / bilinear-upsample for the
//! coarse-to-fine pyramid. The whole algorithm works in sRGB float: averaging
//! and SSD stay transcendental-free (no sRGB↔linear `pow`), which keeps the CPU
//! reference deterministic (HR-5) and byte-matchable by the GPU translation.
//! Clamp-to-edge sampling (never wraps, never reads out of bounds) mirrors a
//! GPU sampler in `ClampToEdge`, so patch reads at the image border agree.

/// Interleaved RGB float plane. `px.len() == w * h * 3`.
#[derive(Clone, Debug)]
pub struct Plane {
    pub w: usize,
    pub h: usize,
    pub px: Vec<f32>,
}

/// Clamp an integer coordinate into `[0, n)`.
#[inline]
pub(crate) fn clampi(v: i32, n: usize) -> usize {
    if v < 0 {
        0
    } else if (v as usize) >= n {
        n - 1
    } else {
        v as usize
    }
}

impl Plane {
    /// A zeroed `w×h` plane.
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            px: vec![0.0; w * h * 3],
        }
    }

    /// Read RGB at `(x, y)` with clamp-to-edge (out-of-bounds coords snap to the
    /// nearest border pixel). Never panics — the border ring extends infinitely.
    #[inline]
    pub fn get(&self, x: i32, y: i32) -> [f32; 3] {
        let cx = clampi(x, self.w);
        let cy = clampi(y, self.h);
        let i = (cy * self.w + cx) * 3;
        [self.px[i], self.px[i + 1], self.px[i + 2]]
    }

    /// Write RGB at an in-bounds `(x, y)`.
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, c: [f32; 3]) {
        let i = (y * self.w + x) * 3;
        self.px[i] = c[0];
        self.px[i + 1] = c[1];
        self.px[i + 2] = c[2];
    }

    /// Mean RGB over every pixel (used to seed the coarsest hole).
    pub fn mean(&self) -> [f32; 3] {
        let n = (self.w * self.h) as f32;
        if n == 0.0 {
            return [0.0; 3];
        }
        let mut s = [0.0f32; 3];
        for p in self.px.chunks_exact(3) {
            s[0] += p[0];
            s[1] += p[1];
            s[2] += p[2];
        }
        [s[0] / n, s[1] / n, s[2] / n]
    }

    /// Halve resolution with a clamped 2×2 box filter. Odd dimensions round up
    /// (`ceil`) so no column/row is dropped. Pure averaging — no transcendentals.
    pub fn downsample(&self) -> Plane {
        let w2 = self.w.div_ceil(2).max(1);
        let h2 = self.h.div_ceil(2).max(1);
        let mut out = Plane::new(w2, h2);
        for y in 0..h2 {
            for x in 0..w2 {
                let (sx, sy) = (x as i32 * 2, y as i32 * 2);
                let mut c = [0.0f32; 3];
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let p = self.get(sx + dx, sy + dy);
                    c[0] += p[0];
                    c[1] += p[1];
                    c[2] += p[2];
                }
                out.set(x, y, [c[0] * 0.25, c[1] * 0.25, c[2] * 0.25]);
            }
        }
        out
    }

    /// Bilinear-upsample to `(tw, th)` with clamp-to-edge. Maps target pixel
    /// centres back to source centres (`(x+0.5)*w/tw - 0.5`) so the result is
    /// aligned, not shifted by half a texel.
    pub fn upsample_to(&self, tw: usize, th: usize) -> Plane {
        let mut out = Plane::new(tw, th);
        let sx = self.w as f32 / tw as f32;
        let sy = self.h as f32 / th as f32;
        for y in 0..th {
            let fy = (y as f32 + 0.5) * sy - 0.5;
            let y0 = fy.floor();
            let ty = fy - y0;
            let y0 = y0 as i32;
            for x in 0..tw {
                let fx = (x as f32 + 0.5) * sx - 0.5;
                let x0 = fx.floor();
                let tx = fx - x0;
                let x0 = x0 as i32;
                let c00 = self.get(x0, y0);
                let c10 = self.get(x0 + 1, y0);
                let c01 = self.get(x0, y0 + 1);
                let c11 = self.get(x0 + 1, y0 + 1);
                let mut c = [0.0f32; 3];
                for k in 0..3 {
                    let top = c00[k] + (c10[k] - c00[k]) * tx;
                    let bot = c01[k] + (c11[k] - c01[k]) * tx;
                    c[k] = top + (bot - top) * ty;
                }
                out.set(x, y, c);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_reads_the_border_ring() {
        let mut p = Plane::new(2, 2);
        p.set(0, 0, [1.0, 0.0, 0.0]);
        p.set(1, 1, [0.0, 0.0, 1.0]);
        assert_eq!(p.get(-5, -5), [1.0, 0.0, 0.0]);
        assert_eq!(p.get(99, 99), [0.0, 0.0, 1.0]);
    }

    #[test]
    fn downsample_of_a_constant_plane_is_constant() {
        let mut p = Plane::new(4, 4);
        for y in 0..4 {
            for x in 0..4 {
                p.set(x, y, [0.5, 0.25, 0.75]);
            }
        }
        let d = p.downsample();
        assert_eq!((d.w, d.h), (2, 2));
        for c in d.px.chunks_exact(3) {
            assert!((c[0] - 0.5).abs() < 1e-6);
            assert!((c[1] - 0.25).abs() < 1e-6);
            assert!((c[2] - 0.75).abs() < 1e-6);
        }
    }

    #[test]
    fn upsample_of_a_constant_plane_is_constant() {
        let mut p = Plane::new(2, 2);
        for y in 0..2 {
            for x in 0..2 {
                p.set(x, y, [0.3, 0.6, 0.9]);
            }
        }
        let u = p.upsample_to(5, 5);
        assert_eq!((u.w, u.h), (5, 5));
        for c in u.px.chunks_exact(3) {
            assert!((c[0] - 0.3).abs() < 1e-6);
            assert!((c[1] - 0.6).abs() < 1e-6);
            assert!((c[2] - 0.9).abs() < 1e-6);
        }
    }

    #[test]
    fn odd_dimension_downsample_rounds_up() {
        let p = Plane::new(5, 3);
        let d = p.downsample();
        assert_eq!((d.w, d.h), (3, 2));
    }
}
