//! **Inflate — the Blob.** A ball whose radius follows the falloff, done as a separable parabolic dilation
//! ([`blob_dilate`], Felzenszwalb `O(N)`), plus the `(dx, dy)` source packing its advection carries.
//!
//! It is NOT memoised — its shape rides the live `amount`, which grows across the stroke — so it recomputes
//! per render in [`super::sculpt_blur::render_inflate`], which builds the peak field and calls the dilation.

/// Pack a source offset `(dx, dy)` — where the matter at a texel came FROM — into one `u32`.
///
/// `(0, 0)` packs to `0`, which is what a texel whose own tap won gets, and which the render reads as
/// *nothing moved here*. The flat interior of every stroke is all zeros, so the advection costs nothing
/// where nothing happens.
#[inline]
pub(super) fn pack_src(dx: i64, dy: i64) -> u32 {
    (((dx as i16) as u16 as u32) << 16) | ((dy as i16) as u16 as u32)
}

/// The inverse of [`pack_src`].
#[inline]
pub(super) fn unpack_src(v: u32) -> (i64, i64) {
    (
        i64::from(((v >> 16) as u16) as i16),
        i64::from((v as u16) as i16),
    )
}

/// **The Blob's engine — a separable parabolic dilation.** (Enio 2026-07-14, 3rd smoke.)
///
/// Inflate is a ball whose radius follows the falloff, and the exact spherical version is `O(area·ρ²)`,
/// which — un-memoisable, because the radius rides the live `amount` — measured **73 ms/move** on a big
/// brush. A sphere is well approximated near its top by a **parabola**, and dilation by a parabola is
/// **separable** (a pass along x, then along y) and `O(N)` by Felzenszwalb & Huttenlocher's lower-envelope
/// algorithm (*Distance Transforms of Sampled Functions*, 2004). Same rounded, fattening dome; a fraction of
/// the cost.
///
/// The peak of the parabola at source `x` is `g(x)` — set by the caller to `pre(x) ± |Depth|·amount(x)`, so
/// the falloff IS the height budget. The curvature `a = 1/(2·|Depth|·DEPTH_UNIT_PX²)` matches the sphere of
/// radius `|Depth|·DEPTH_UNIT_PX` at its apex (the two axes reconciled through the light's unit, as every
/// geometric quantity here is).
///
/// Returns, per texel of the `w × h` region, the dilated height and the **packed source offset**
/// ([`pack_src`]) — the argmax, composed through the two passes, so the paint can follow the relief.
pub(super) fn blob_dilate(g: &[f32], w: u32, h: u32, a: f32, dilate: bool) -> (Vec<f32>, Vec<u32>) {
    let (wu, hu) = (w as usize, h as usize);
    // Pass 1 — along X, per row. `h1[i]` = the dilated value, `argx[i]` = the source COLUMN it came from.
    let mut h1 = vec![0.0f32; wu * hu];
    let mut argx = vec![0u32; wu * hu];
    let mut scratch = ParabolaScratch::new(wu.max(hu));
    for y in 0..hu {
        let base = y * wu;
        scratch.transform(&g[base..base + wu], a, dilate);
        for x in 0..wu {
            h1[base + x] = scratch.out_val[x];
            argx[base + x] = scratch.out_arg[x] as u32;
        }
    }
    // Pass 2 — along Y, per column. `hf` = final height, `argy` = the source ROW.
    let mut hf = vec![0.0f32; wu * hu];
    let mut src = vec![0u32; wu * hu];
    let mut col: Vec<f32> = vec![0.0; hu];
    for x in 0..wu {
        for y in 0..hu {
            col[y] = h1[y * wu + x];
        }
        scratch.transform(&col, a, dilate);
        for y in 0..hu {
            let ry = scratch.out_arg[y]; // the source row for output (x, y)
            let cx = argx[ry * wu + x] as usize; // …and, at that row, the source column
            hf[y * wu + x] = scratch.out_val[y];
            src[y * wu + x] = pack_src(cx as i64 - x as i64, ry as i64 - y as i64);
        }
    }
    (hf, src)
}

/// Reusable buffers for the 1-D parabola lower-envelope, so a whole 2-D pass allocates once.
struct ParabolaScratch {
    v: Vec<usize>,       // the parabola locations forming the envelope
    z: Vec<f32>,         // the boundaries between them
    out_val: Vec<f32>,   // the dilated 1-D result
    out_arg: Vec<usize>, // the source index (argmax) per output
}

impl ParabolaScratch {
    fn new(n: usize) -> Self {
        Self {
            v: vec![0; n + 1],
            z: vec![0.0; n + 2],
            out_val: vec![0.0; n],
            out_arg: vec![0; n],
        }
    }

    /// One 1-D parabolic dilation (`dilate = true`) or erosion (`false`) of `f`, curvature `a`.
    ///
    /// `dilate`: `H[q] = max_p ( f[p] − a·(q−p)² )`. Via `F = −f/a`, that is `−a·` the standard **min**
    /// distance transform `min_p ( F[p] + (q−p)² )`, whose lower-envelope sweep is Felzenszwalb's — and its
    /// winner `v[k]` IS the argmax. Erosion mirrors it with `F = f/a` and `+a·`.
    fn transform(&mut self, f: &[f32], a: f32, dilate: bool) {
        let n = f.len();
        if n == 0 {
            return;
        }
        let sign = if dilate { -1.0 } else { 1.0 };
        // F[p] = sign · f[p] / a — the values the standard `min` transform runs on.
        let ff = |p: usize| -> f32 { sign * f[p] / a };
        let (v, z) = (&mut self.v, &mut self.z);
        let mut k = 0usize;
        v[0] = 0;
        z[0] = f32::NEG_INFINITY;
        z[1] = f32::INFINITY;
        for q in 1..n {
            let fq = ff(q);
            // Pop parabolas the new one has overtaken, then push it. `z[0] = −∞` and a finite `s` guarantee
            // the pop stops at `k = 0` before it can underflow, so `q` is ALWAYS pushed (the envelope keeps
            // every location, only re-ordering their reigns).
            loop {
                let p = v[k];
                // Intersection of the parabolas rooted at `q` and `p`: where `F[p]+(x−p)² = F[q]+(x−q)²`.
                let s = ((fq + (q * q) as f32) - (ff(p) + (p * p) as f32))
                    / (2.0 * q as f32 - 2.0 * p as f32);
                if s <= z[k] && k > 0 {
                    k -= 1;
                } else {
                    k += 1;
                    v[k] = q;
                    z[k] = s;
                    z[k + 1] = f32::INFINITY;
                    break;
                }
            }
        }
        let mut k = 0usize;
        for q in 0..n {
            while z[k + 1] < q as f32 {
                k += 1;
            }
            let p = v[k];
            let d = q as f32 - p as f32;
            let dt = d * d + ff(p); // the min transform's value
            // Lift back: `H = sign·a·DT`. Dilate (`sign = −1`) gives `−a·DT = max(f − a·d²)`; erode
            // (`sign = +1`) gives `+a·DT = min(f + a·d²)`. Getting this sign wrong turns the dome inside out.
            self.out_val[q] = sign * a * dt;
            self.out_arg[q] = p;
        }
    }
}
