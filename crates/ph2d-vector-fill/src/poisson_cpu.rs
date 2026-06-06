//! CPU **multigrid** Poisson solver — the deterministic reference / Mobile-Core
//! fallback for diffusion-curve mesh gradients (ADR-0060 §2.5, file
//! `poisson_cpu.rs`: *"CPU multigrid baseline (Mobile Core fallback)"*).
//!
//! ## What it solves
//!
//! Given a set of [`DiffusionCurve`](crate::diffusion_curve::DiffusionCurve)s,
//! each injecting a colour on its two sides, the smooth fill is the function
//! that is **harmonic** (`∇²u = 0`) everywhere except at those curve "walls",
//! where it equals the authored side colour. That is a Laplace problem with
//! interior Dirichlet constraints; the domain border is **Neumann** (zero-flux,
//! reflecting) so colour diffuses cleanly to the canvas edge. Each OKLab channel
//! (`L`, `a`, `b`) is an independent scalar Laplace solve.
//!
//! ## Why multigrid (and why it's the *reference*)
//!
//! Plain Gauss-Seidel needs `O(N²)` sweeps to propagate a boundary value across
//! an `N×N` grid — low-frequency error decays glacially. A multigrid V-cycle
//! kills error at every scale (smooth → restrict residual → recurse → prolong
//! correction → smooth), converging in a handful of cycles independent of `N`.
//!
//! It is also the **golden reference** the (stochastic, noisy) Walk-on-Spheres
//! GPU path (W7 step 3) gets validated against: this solver is deterministic
//! (fixed traversal, no RNG, single-threaded), so the same input yields
//! bit-identical output every run — exactly what a cross-OS reference needs.
//!
//! ## Correctness, by construction
//!
//! The converged discrete solution is a **fixed point of the V-cycle regardless
//! of the coarse-grid mask**: at the true solution the residual is zero at every
//! free vertex, so the restricted residual is zero, the coarse correction is
//! zero, and the prolongated correction (additionally zeroed at fixed vertices)
//! perturbs nothing. The fine grid alone *defines* the answer (its smoother +
//! Dirichlet pinning); the coarse grids only *accelerate*. The inline tests pin
//! this down against analytic harmonic ground truth (`u = x`, `u = x·y` are
//! exact discrete harmonics of the 5-point stencil) and against a
//! Gauss-Seidel-to-convergence oracle.

use glam::Vec2;
use ph2d_color::OklabColor;

use crate::diffusion_curve::DiffusionCurveSet;

/// Pre-smoothing sweeps per V-cycle level (red-black Gauss-Seidel).
const PRE_SWEEPS: usize = 2;
/// Post-smoothing sweeps per V-cycle level.
const POST_SWEEPS: usize = 2;
/// Sweeps at the coarsest grid (a near-exact solve on the tiny `≤4×4` grid).
const COARSE_SWEEPS: usize = 40;
/// A solid default V-cycle count: multigrid converges in a handful of cycles
/// independent of resolution, so this is comfortable headroom for authoring.
pub const DEFAULT_VCYCLES: usize = 16;

/// Smallest axis length that can still be coarsened (`(n+1)/2 ≥ 3` ⟺ `n ≥ 5`).
const MIN_COARSENABLE: usize = 5;

// ───────────────────────────── public surface ──────────────────────────────

/// A solve resolution. Each axis must be `2^k + 1` (vertex-centred multigrid:
/// halving the interval count keeps grids aligned, so restriction/prolongation
/// are the clean textbook stencils). Use [`Resolution::new`] / [`square`] which
/// validate the shape.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Resolution {
    pub w: usize,
    pub h: usize,
}

impl Resolution {
    /// `Some` iff both axes are `2^k + 1` and `≥ 3`.
    pub fn new(w: usize, h: usize) -> Option<Self> {
        (is_pow2_plus_1(w) && is_pow2_plus_1(h)).then_some(Self { w, h })
    }

    /// A square resolution; `Some` iff `n` is `2^k + 1` and `≥ 3`.
    pub fn square(n: usize) -> Option<Self> {
        Self::new(n, n)
    }
}

#[inline]
fn is_pow2_plus_1(n: usize) -> bool {
    n >= 3 && (n - 1).is_power_of_two()
}

/// The diffused result: a dense grid of **linear-light RGBA** texels, ready for
/// the renderer / a [`crate::FillNode::MeshGradient`] to sample. Row-major,
/// `w × h`, vertex-centred (texel `(x,y)` is the field at normalized uv
/// `(x/(w-1), y/(h-1))`).
#[derive(Clone, Debug, PartialEq)]
pub struct ColorField {
    pub w: usize,
    pub h: usize,
    /// `w * h` linear-light RGBA texels, row-major.
    pub texel: Vec<[f32; 4]>,
}

impl ColorField {
    /// A fully transparent field of the given size (the empty-input result).
    pub fn transparent(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            texel: vec![[0.0; 4]; w * h],
        }
    }

    /// The texel at integer grid coords (no bounds adjustment; caller in range).
    #[inline]
    pub fn at(&self, x: usize, y: usize) -> [f32; 4] {
        self.texel[y * self.w + x]
    }

    /// Bilinearly sample at normalized uv `∈ [0,1]²` (clamped to the field).
    pub fn sample(&self, uv: Vec2) -> [f32; 4] {
        let fx = (uv.x.clamp(0.0, 1.0)) * (self.w - 1) as f32;
        let fy = (uv.y.clamp(0.0, 1.0)) * (self.h - 1) as f32;
        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1).min(self.w - 1);
        let y1 = (y0 + 1).min(self.h - 1);
        let tx = fx - x0 as f32;
        let ty = fy - y0 as f32;
        let c00 = self.at(x0, y0);
        let c10 = self.at(x1, y0);
        let c01 = self.at(x0, y1);
        let c11 = self.at(x1, y1);
        let mut out = [0.0; 4];
        for k in 0..4 {
            let top = lerp(c00[k], c10[k], tx);
            let bot = lerp(c01[k], c11[k], tx);
            out[k] = lerp(top, bot, ty);
        }
        out
    }
}

/// Resolves a [`crate::FillNode::MeshGradient`]'s `gradient_id` to its solved
/// [`ColorField`]. The id→field map is owned by the host (the renderer / the doc
/// store the Coordenador wires); the CPU evaluator only needs read access at
/// sample time. [`solve_color_field`] produces the fields the host stores here.
pub trait FieldResolver {
    /// The solved field for `gradient_id`, or `None` if it isn't available
    /// (unsolved / unknown id) — in which case the evaluator renders that
    /// gradient transparent rather than failing the whole graph.
    fn resolve(&self, gradient_id: u64) -> Option<&ColorField>;
}

/// A [`FieldResolver`] that knows no fields — every `MeshGradient` resolves to
/// `None` (renders transparent). The default the bare
/// [`crate::eval::eval_color`] uses for graphs with no diffusion gradients.
pub struct NoFields;

impl FieldResolver for NoFields {
    #[inline]
    fn resolve(&self, _gradient_id: u64) -> Option<&ColorField> {
        None
    }
}

/// A concrete `gradient_id → ColorField` store — the simple host/testing
/// implementation of [`FieldResolver`]. Keyed in a [`BTreeMap`] for
/// deterministic iteration (HR-5 / ADR-0022 — no `std` `HashMap` in sim crates).
///
/// [`BTreeMap`]: std::collections::BTreeMap
#[derive(Clone, Debug, Default)]
pub struct FieldStore {
    fields: std::collections::BTreeMap<u64, ColorField>,
}

impl FieldStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert (or replace) the solved field for `gradient_id`.
    pub fn insert(&mut self, gradient_id: u64, field: ColorField) {
        self.fields.insert(gradient_id, field);
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

impl FieldResolver for FieldStore {
    #[inline]
    fn resolve(&self, gradient_id: u64) -> Option<&ColorField> {
        self.fields.get(&gradient_id)
    }
}

/// Solve a [`DiffusionCurveSet`] into a [`ColorField`] at `res`, using
/// [`DEFAULT_VCYCLES`] V-cycles. Empty input → a transparent field.
pub fn solve_color_field(set: &DiffusionCurveSet, res: Resolution) -> ColorField {
    solve_color_field_cycles(set, res, DEFAULT_VCYCLES)
}

/// As [`solve_color_field`], with an explicit V-cycle count (tests / tier
/// tuning). The three OKLab channels are solved independently, then recombined
/// and converted to linear RGBA.
pub fn solve_color_field_cycles(
    set: &DiffusionCurveSet,
    res: Resolution,
    cycles: usize,
) -> ColorField {
    let (w, h) = (res.w, res.h);
    if set.is_empty() {
        return ColorField::transparent(w, h);
    }

    let c = rasterize_constraints(set, w, h);
    // Warm-start free vertices with each channel's constraint mean — a constant
    // start is already correct for a constant field and a good guess otherwise,
    // so the first cycle has less low-frequency error to chew through.
    let means = c.channel_means();
    let sol_l = solve_channel(&c.fixed, &c.init_channel(0, means[0]), w, h, cycles);
    let sol_a = solve_channel(&c.fixed, &c.init_channel(1, means[1]), w, h, cycles);
    let sol_b = solve_channel(&c.fixed, &c.init_channel(2, means[2]), w, h, cycles);

    let mut texel = vec![[0.0; 4]; w * h];
    for (i, t) in texel.iter_mut().enumerate() {
        *t = OklabColor::new(sol_l[i], sol_a[i], sol_b[i], 1.0)
            .to_linear()
            .as_array();
    }
    ColorField { w, h, texel }
}

// ───────────────────────── constraint rasterization ─────────────────────────

/// The Dirichlet boundary condition for one solve: which vertices are pinned and
/// the OKLab value pinned there (channel 0 = `L`, 1 = `a`, 2 = `b`).
struct Constraints {
    fixed: Vec<bool>,
    lab: [Vec<f32>; 3],
}

impl Constraints {
    /// Per-channel mean over the pinned vertices (`0` if none are pinned).
    fn channel_means(&self) -> [f32; 3] {
        let n = self.fixed.iter().filter(|f| **f).count().max(1) as f32;
        let mut sums = [0.0_f32; 3];
        for i in 0..self.fixed.len() {
            if self.fixed[i] {
                for (sum, lab) in sums.iter_mut().zip(&self.lab) {
                    *sum += lab[i];
                }
            }
        }
        [sums[0] / n, sums[1] / n, sums[2] / n]
    }

    /// An initial-guess grid for `channel`: the pinned value at fixed vertices,
    /// `fill` (the channel mean) everywhere else.
    fn init_channel(&self, channel: usize, fill: f32) -> Vec<f32> {
        let src = &self.lab[channel];
        (0..self.fixed.len())
            .map(|i| if self.fixed[i] { src[i] } else { fill })
            .collect()
    }
}

/// Stamp each curve's two side-colours onto the grid as Dirichlet sources. For
/// every sample along a curve we pin the nearest vertex `δ` off each side
/// (`δ ≈ 1.5` cells, so the red/blue walls don't collide in one cell), tagged
/// with that side's OKLab colour at the sample's arc-length parameter.
fn rasterize_constraints(set: &DiffusionCurveSet, w: usize, h: usize) -> Constraints {
    let mut fixed = vec![false; w * h];
    let mut lab = [vec![0.0; w * h], vec![0.0; w * h], vec![0.0; w * h]];

    // One grid cell in normalized fill space (use the finer axis so neither is
    // under-sampled), and the off-curve stamp offset.
    let cell = 1.0 / (w.max(h) - 1) as f32;
    let delta = 1.5 * cell;

    let mut stamp = |p: Vec2, col: OklabColor| {
        let x = (p.x * (w - 1) as f32).round().clamp(0.0, (w - 1) as f32) as usize;
        let y = (p.y * (h - 1) as f32).round().clamp(0.0, (h - 1) as f32) as usize;
        let idx = y * w + x;
        fixed[idx] = true;
        lab[0][idx] = col.l;
        lab[1][idx] = col.a;
        lab[2][idx] = col.b;
    };

    for curve in &set.curves {
        if !curve.is_valid() {
            continue;
        }
        let total = curve.arc_length().max(f32::EPSILON);
        let mut acc = 0.0_f32;
        for seg in curve.points.windows(2) {
            let (a, b) = (seg[0], seg[1]);
            let seg_vec = b - a;
            let seg_len = seg_vec.length();
            if seg_len < f32::EPSILON {
                continue;
            }
            let tangent = seg_vec / seg_len;
            let normal = Vec2::new(-tangent.y, tangent.x);
            // Sample at ≤ half a cell so no vertex along the wall is skipped.
            let steps = ((seg_len / (0.5 * cell)).ceil() as usize).max(1);
            for s in 0..=steps {
                let f = s as f32 / steps as f32;
                let p = a + seg_vec * f;
                let t = (acc + seg_len * f) / total;
                stamp(p + normal * delta, curve.left_color_at(t));
                stamp(p - normal * delta, curve.right_color_at(t));
            }
            acc += seg_len;
        }
    }

    Constraints { fixed, lab }
}

// ──────────────────────────── multigrid core ───────────────────────────────

/// Solve one scalar Laplace channel (`∇²u = 0`, interior Dirichlet from
/// `fixed`/`init`, Neumann border) with `cycles` multigrid V-cycles. `init`
/// holds the pinned values at fixed vertices and the initial guess elsewhere.
fn solve_channel(fixed: &[bool], init: &[f32], w: usize, h: usize, cycles: usize) -> Vec<f32> {
    let mut u = init.to_vec();
    let f = vec![0.0_f32; w * h]; // Laplace: zero RHS.
    // Nominal spacing; cancels exactly for the f = 0 Laplace case, kept for a
    // consistent operator scaling should a Poisson RHS ever be threaded in.
    let s = 1.0 / (w.max(h) - 1) as f32;
    let h2 = s * s;
    for _ in 0..cycles {
        vcycle(&mut u, &f, fixed, w, h, h2);
    }
    u
}

/// One V-cycle on a grid: pre-smooth → (if coarsenable) restrict residual,
/// recurse for a correction, prolong + add → post-smooth. Smoothing counts are
/// the module [`PRE_SWEEPS`]/[`POST_SWEEPS`] constants (identical at every level,
/// so they are not threaded as arguments).
fn vcycle(u: &mut [f32], f: &[f32], fixed: &[bool], w: usize, h: usize, h2: f32) {
    smooth_redblack(u, f, fixed, w, h, h2, PRE_SWEEPS);

    if w < MIN_COARSENABLE || h < MIN_COARSENABLE {
        // Coarsest grid — smooth hard for a near-exact solve and return.
        smooth_redblack(u, f, fixed, w, h, h2, COARSE_SWEEPS);
        return;
    }

    // Residual on the fine grid (zero at pinned vertices by construction).
    let r = residual(u, f, fixed, w, h, h2);
    let (mut rc, cw, ch) = restrict_full_weighting(&r, w, h);
    let fc = restrict_mask(fixed, w, h);
    // The coarse error is homogeneous-Dirichlet at coarse-fixed vertices.
    for i in 0..rc.len() {
        if fc[i] {
            rc[i] = 0.0;
        }
    }

    let mut ec = vec![0.0_f32; cw * ch];
    vcycle(&mut ec, &rc, &fc, cw, ch, h2 * 4.0);

    // Prolong the correction and add it to the free vertices only.
    let e = prolongate_bilinear(&ec, cw, ch, w, h);
    for i in 0..u.len() {
        if !fixed[i] {
            u[i] += e[i];
        }
    }

    smooth_redblack(u, f, fixed, w, h, h2, POST_SWEEPS);
}

/// Red-black Gauss-Seidel for `(4u − Σneighbours)/h² = f`, i.e. the update
/// `u = (Σneighbours + h²·f)/4`. Fixed vertices are never written. Red-black
/// ordering (update `(x+y)` even, then odd) is deterministic *and* the natural
/// shape the GPU WoS/Jacobi path mirrors.
fn smooth_redblack(
    u: &mut [f32],
    f: &[f32],
    fixed: &[bool],
    w: usize,
    h: usize,
    h2: f32,
    sweeps: usize,
) {
    for _ in 0..sweeps {
        for parity in 0..2 {
            for y in 0..h {
                for x in 0..w {
                    if (x + y) & 1 != parity {
                        continue;
                    }
                    let idx = y * w + x;
                    if fixed[idx] {
                        continue;
                    }
                    u[idx] = (neighbour_sum(u, w, h, x, y) + h2 * f[idx]) * 0.25;
                }
            }
        }
    }
}

/// Sum of the four axis neighbours with a reflecting (zero-flux Neumann) border:
/// an out-of-domain neighbour mirrors its in-domain partner across the edge
/// vertex, so a left-edge vertex effectively sees `2·east + north + south`.
#[inline]
fn neighbour_sum(u: &[f32], w: usize, h: usize, x: usize, y: usize) -> f32 {
    let xm = if x == 0 { 1 } else { x - 1 };
    let xp = if x == w - 1 { w - 2 } else { x + 1 };
    let ym = if y == 0 { 1 } else { y - 1 };
    let yp = if y == h - 1 { h - 2 } else { y + 1 };
    u[y * w + xm] + u[y * w + xp] + u[ym * w + x] + u[yp * w + x]
}

/// Residual `r = f − L u` at free vertices; `0` at fixed vertices (they are
/// exact, so they contribute no correction).
fn residual(u: &[f32], f: &[f32], fixed: &[bool], w: usize, h: usize, h2: f32) -> Vec<f32> {
    let mut r = vec![0.0_f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            if fixed[idx] {
                continue;
            }
            let lu = (4.0 * u[idx] - neighbour_sum(u, w, h, x, y)) / h2;
            r[idx] = f[idx] - lu;
        }
    }
    r
}

/// Full-weighting restriction (fine → coarse), `[1 2 1; 2 4 2; 1 2 1]/16`. The
/// coarse vertex `(X,Y)` coincides with fine `(2X,2Y)`; neighbours reflect at
/// the border. Returns the coarse grid and its dims `(2^{k-1}+1)`.
fn restrict_full_weighting(r: &[f32], w: usize, h: usize) -> (Vec<f32>, usize, usize) {
    let cw = w.div_ceil(2);
    let ch = h.div_ceil(2);
    let mut rc = vec![0.0_f32; cw * ch];
    for cy in 0..ch {
        for cx in 0..cw {
            let x = (2 * cx) as isize;
            let y = (2 * cy) as isize;
            let center = sample_reflect(r, w, h, x, y);
            let edges = sample_reflect(r, w, h, x - 1, y)
                + sample_reflect(r, w, h, x + 1, y)
                + sample_reflect(r, w, h, x, y - 1)
                + sample_reflect(r, w, h, x, y + 1);
            let corners = sample_reflect(r, w, h, x - 1, y - 1)
                + sample_reflect(r, w, h, x + 1, y - 1)
                + sample_reflect(r, w, h, x - 1, y + 1)
                + sample_reflect(r, w, h, x + 1, y + 1);
            rc[cy * cw + cx] = (4.0 * center + 2.0 * edges + corners) / 16.0;
        }
    }
    (rc, cw, ch)
}

/// Coarsen the Dirichlet mask: a coarse vertex is fixed if **any** fine vertex
/// in its `3×3` footprint is fixed. Over-marking only thickens the boundary the
/// coarse correction sees (slightly slower convergence near walls) — it cannot
/// change the converged answer, which the fine grid alone defines.
fn restrict_mask(fixed: &[bool], w: usize, h: usize) -> Vec<bool> {
    let cw = w.div_ceil(2);
    let ch = h.div_ceil(2);
    let mut fc = vec![false; cw * ch];
    for cy in 0..ch {
        for cx in 0..cw {
            let x = (2 * cx) as isize;
            let y = (2 * cy) as isize;
            let mut any = false;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let xi = (x + dx).clamp(0, w as isize - 1) as usize;
                    let yi = (y + dy).clamp(0, h as isize - 1) as usize;
                    if fixed[yi * w + xi] {
                        any = true;
                    }
                }
            }
            fc[cy * cw + cx] = any;
        }
    }
    fc
}

/// Bilinear prolongation (coarse → fine). Even fine vertices copy their
/// coincident coarse vertex; odd ones interpolate. Coarse vertex `X` lives at
/// fine `2X`, so fine `x` reads coarse coordinate `x/2`.
fn prolongate_bilinear(ec: &[f32], cw: usize, ch: usize, w: usize, h: usize) -> Vec<f32> {
    let mut e = vec![0.0_f32; w * h];
    for y in 0..h {
        let fy = y as f32 * 0.5;
        let y0 = (fy.floor() as usize).min(ch - 1);
        let y1 = (y0 + 1).min(ch - 1);
        let ty = fy - y0 as f32;
        for x in 0..w {
            let fx = x as f32 * 0.5;
            let x0 = (fx.floor() as usize).min(cw - 1);
            let x1 = (x0 + 1).min(cw - 1);
            let tx = fx - x0 as f32;
            let c00 = ec[y0 * cw + x0];
            let c10 = ec[y0 * cw + x1];
            let c01 = ec[y1 * cw + x0];
            let c11 = ec[y1 * cw + x1];
            e[y * w + x] = lerp(lerp(c00, c10, tx), lerp(c01, c11, tx), ty);
        }
    }
    e
}

/// Fetch with single-step reflection across border vertices (Neumann); the
/// `clamp` is a belt-and-braces guard for any multi-step access.
#[inline]
fn sample_reflect(g: &[f32], w: usize, h: usize, x: isize, y: isize) -> f32 {
    g[reflect(y, h) * w + reflect(x, w)]
}

#[inline]
fn reflect(i: isize, n: usize) -> usize {
    let n = n as isize;
    let r = if i < 0 {
        -i
    } else if i >= n {
        2 * (n - 1) - i
    } else {
        i
    };
    r.clamp(0, n - 1) as usize
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

// ──────────────────────────────── tests ────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diffusion_curve::DiffusionCurve;
    use ph2d_color::OklchColor;

    /// Max |error| of `u` against an analytic field `g(x,y)` over the interior.
    fn max_err(u: &[f32], w: usize, h: usize, g: impl Fn(f32, f32) -> f32) -> f32 {
        let mut m = 0.0_f32;
        for y in 0..h {
            for x in 0..w {
                let gx = x as f32 / (w - 1) as f32;
                let gy = y as f32 / (h - 1) as f32;
                m = m.max((u[y * w + x] - g(gx, gy)).abs());
            }
        }
        m
    }

    /// Dirichlet-pin the whole border to `g`, leave the interior free.
    fn border_problem(w: usize, h: usize, g: impl Fn(f32, f32) -> f32) -> (Vec<bool>, Vec<f32>) {
        let mut fixed = vec![false; w * h];
        let mut init = vec![0.0_f32; w * h];
        for y in 0..h {
            for x in 0..w {
                if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
                    let gx = x as f32 / (w - 1) as f32;
                    let gy = y as f32 / (h - 1) as f32;
                    fixed[y * w + x] = true;
                    init[y * w + x] = g(gx, gy);
                }
            }
        }
        (fixed, init)
    }

    #[test]
    fn harmonic_linear_reproduced() {
        // u(x,y) = x is an exact discrete harmonic of the 5-point stencil.
        let (w, h) = (65, 65);
        let g = |x: f32, _y: f32| x;
        let (fixed, init) = border_problem(w, h, g);
        let u = solve_channel(&fixed, &init, w, h, 30);
        assert!(
            max_err(&u, w, h, g) < 1e-3,
            "err = {}",
            max_err(&u, w, h, g)
        );
    }

    #[test]
    fn harmonic_bilinear_xy_reproduced() {
        // u(x,y) = x·y: ∇²(xy)=0 continuously AND the 5-point Laplacian of a
        // bilinear function is exactly zero — so it must be reproduced.
        let (w, h) = (65, 65);
        let g = |x: f32, y: f32| x * y;
        let (fixed, init) = border_problem(w, h, g);
        let u = solve_channel(&fixed, &init, w, h, 40);
        assert!(
            max_err(&u, w, h, g) < 1e-3,
            "err = {}",
            max_err(&u, w, h, g)
        );
    }

    #[test]
    fn neumann_single_constraint_is_constant() {
        // One interior fixed vertex = c, Neumann everywhere else → field ≡ c.
        let (w, h) = (33, 33);
        let mut fixed = vec![false; w * h];
        let mut init = vec![0.0_f32; w * h];
        let c = 0.7_f32;
        let mid = (h / 2) * w + (w / 2);
        fixed[mid] = true;
        init[mid] = c;
        let u = solve_channel(&fixed, &init, w, h, 30);
        let err = u.iter().map(|v| (v - c).abs()).fold(0.0_f32, f32::max);
        assert!(err < 1e-4, "constant-field err = {err}");
    }

    #[test]
    fn residual_converges_fast() {
        // Multigrid drives the residual down geometrically until it hits the
        // float floor: the 5-point operator is scaled by 1/h² (≈4096 at 65²),
        // so it amplifies f32 granularity to ≈1e-3 — below that the max-norm
        // residual jitters by a few ULP and is meaningless. We therefore assert
        // (a) strict per-cycle decrease while comfortably *above* that floor and
        // (b) convergence below a small absolute tolerance within a few cycles —
        // which is what distinguishes multigrid (≤ handful of cycles) from plain
        // Gauss-Seidel (thousands of sweeps for the same grid).
        let (w, h) = (65, 65);
        let g = |x: f32, y: f32| x * y;
        let (fixed, init) = border_problem(w, h, g);
        let f = vec![0.0_f32; w * h];
        let s = 1.0 / (w - 1) as f32;
        let h2 = s * s;
        let floor = 4e-3_f32;
        let mut u = init.clone();
        let mut prev = f32::INFINITY;
        let mut converged = false;
        for cycle in 0..12 {
            vcycle(&mut u, &f, &fixed, w, h, h2);
            let r = residual(&u, &f, &fixed, w, h, h2);
            let max_r = r.iter().map(|v| v.abs()).fold(0.0_f32, f32::max);
            if max_r < 2e-3 {
                converged = true;
                break;
            }
            if prev > floor {
                assert!(
                    max_r < prev,
                    "cycle {cycle}: residual {max_r} !< {prev} (above floor)"
                );
            }
            prev = max_r;
        }
        assert!(converged, "did not converge below 2e-3 within 12 cycles");
    }

    #[test]
    fn multigrid_matches_gauss_seidel_oracle() {
        // A deterministic interior constraint pattern; the multigrid solution
        // must equal a Gauss-Seidel-to-convergence solve of the same system.
        let (w, h) = (33, 33);
        let mut fixed = vec![false; w * h];
        let mut init = vec![0.5_f32; w * h];
        for k in 0..w {
            // top edge ramps 0→1, bottom edge constant 0.2 (interior-ish walls).
            fixed[k] = true;
            init[k] = k as f32 / (w - 1) as f32;
            let b = (h - 1) * w + k;
            fixed[b] = true;
            init[b] = 0.2;
        }
        let mg = solve_channel(&fixed, &init, w, h, 30);

        // Oracle: pure red-black Gauss-Seidel, many sweeps.
        let f = vec![0.0_f32; w * h];
        let s = 1.0 / (w - 1) as f32;
        let h2 = s * s;
        let mut gs = init.clone();
        smooth_redblack(&mut gs, &f, &fixed, w, h, h2, 8000);

        let diff = mg
            .iter()
            .zip(&gs)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f32, f32::max);
        assert!(diff < 1e-3, "multigrid vs GS oracle diff = {diff}");
    }

    #[test]
    fn solve_is_bit_deterministic() {
        let set = DiffusionCurveSet::from_curves([DiffusionCurve::straight(
            Vec2::new(0.5, 0.0),
            Vec2::new(0.5, 1.0),
            OklchColor::opaque(0.63, 0.26, 29.0),
            OklchColor::opaque(0.45, 0.31, 264.0),
        )]);
        let res = Resolution::square(65).unwrap();
        let a = solve_color_field(&set, res);
        let b = solve_color_field(&set, res);
        assert_eq!(a.texel, b.texel, "solver must be bit-identical across runs");
    }

    #[test]
    fn straight_red_blue_curve_splits_field() {
        // A full-height wall at x=0.5: left = red, right = blue. The Neumann
        // border + the wall make each half converge to its (near-constant)
        // side colour, with a smooth seam.
        let red = OklchColor::opaque(0.63, 0.26, 29.0);
        let blue = OklchColor::opaque(0.45, 0.31, 264.0);
        let set = DiffusionCurveSet::from_curves([DiffusionCurve::straight(
            Vec2::new(0.5, 0.0),
            Vec2::new(0.5, 1.0),
            red,
            blue,
        )]);
        let res = Resolution::square(65).unwrap();
        let field = solve_color_field_cycles(&set, res, 24);

        let red_lin = red.to_linear().as_array();
        let blue_lin = blue.to_linear().as_array();
        let mid = res.h / 2;
        let left = field.at(2, mid);
        let right = field.at(res.w - 3, mid);

        // Far-left ≈ red, far-right ≈ blue (linear-light, generous tol for the
        // diffusion seam / corner leakage).
        let close = |a: [f32; 4], b: [f32; 4], tol: f32| (0..3).all(|k| (a[k] - b[k]).abs() < tol);
        assert!(
            close(left, red_lin, 0.03),
            "left {left:?} != red {red_lin:?}"
        );
        assert!(
            close(right, blue_lin, 0.03),
            "right {right:?} != blue {blue_lin:?}"
        );

        // The red (R) channel must decrease left→right; blue (B) must increase.
        assert!(
            left[0] > right[0],
            "R should fall L→R: {} vs {}",
            left[0],
            right[0]
        );
        assert!(
            left[2] < right[2],
            "B should rise L→R: {} vs {}",
            left[2],
            right[2]
        );
    }

    #[test]
    fn empty_set_is_transparent() {
        let set = DiffusionCurveSet::new();
        let res = Resolution::square(33).unwrap();
        let field = solve_color_field(&set, res);
        assert!(field.texel.iter().all(|t| *t == [0.0; 4]));
    }

    #[test]
    fn resolution_rejects_non_pow2_plus_1() {
        assert!(Resolution::square(64).is_none());
        assert!(Resolution::square(65).is_some());
        assert!(Resolution::new(65, 129).is_some());
        assert!(Resolution::new(65, 100).is_none());
        assert!(Resolution::square(2).is_none());
        assert!(Resolution::square(3).is_some());
    }
}
