//! Watercolor **wet-on-wet diffusion** solver — the deterministic, low-resolution
//! gated diffusion-advection field behind live blooms / bleeds / backruns
//! (Curtis et al., "Computer-Generated Watercolor", SIGGRAPH 1997, with the
//! expensive Navier-Stokes momentum solver replaced by **gated diffusion** — the
//! real-time-feasible, replayable simplification used by GPU watercolor systems,
//! Van Laerhoven CAVW 2005; TAMU GPU-watercolor thesis).
//!
//! ## Why gated diffusion (not a full fluid solve)
//!
//! A live bleed does not need true incompressible momentum — it needs pigment to
//! **spread isotropically where the paper is wet, drift downhill into the paper
//! valleys, pile up at the receding water boundary, and stop dead on dry paper**.
//! That is precisely a *gated, advection-biased diffusion*, which is pure
//! arithmetic (no RNG / clock / iterative pressure projection) → HR-5 deterministic
//! and cross-OS replayable, and stable under the explicit CFL bound `D·dt ≤ 1/4`.
//!
//! ## The per-tick update (one [`DiffusionGrid::step`])
//!
//! Per cell, with wetness `w ∈ [0,1]`, pigment `p` (linear RGB), paper height
//! `h ∈ [0,1]`:
//!   1. **Wet gate** `g = smoothstep(w_lo, w_hi, w) · permeability(h)` — 0 on dry
//!      paper (crisp edges), →1 in a wet pool (fast spread); valleys are more
//!      permeable so pigment pools low.
//!   2. **Gated divergence-form diffusion** `p += D·Σ_face ½(g_c+g_n)(p_n − p_c)` —
//!      the conservative variable-coefficient 5-point Laplacian (face-averaged
//!      conductances; stencil sums to 0 ⇒ pigment mass conserved), giving the
//!      bloom.
//!   3. **Advection** along `flow = −β·∇h − λ·∇w`, gated + upwind + CFL-clamped:
//!      `−β·∇h` channels pigment into the paper valleys; `−λ·∇w` is the Curtis
//!      *FlowOutward* push — pigment carried from a wet region toward a drier one,
//!      so it strands at the receding boundary (edge darkening) and piles at a
//!      wet front invading a drying area (the backrun ring).
//!   4. **Evaporation** `w −= evap` — as `w` falls below `w_lo` the gate closes and
//!      pigment freezes in place: the bleed ends, the stroke "dries".
//!
//! [`DiffusionGrid::splat`] re-wets + deposits, so painting into a still-wet area
//! re-opens the gate and the old pigment blooms again.
//!
//! ## Budget
//!
//! Fixed low-res grid (e.g. 256² or canvas/4) stepped a few sub-steps per frame,
//! bilinear/joint-bilateral upsampled to the canvas — the only way to hit 4K live
//! (a full-res solve is ~128× the work). This module is the resolution-independent
//! **core**; the live per-frame driver + GPU port + canvas upsample are the
//! ADR-0049 (`ph2d-painter-fluid`, W15) integration layer.

/// Tunable solver coefficients. `diffusivity` already folds the time step
/// (`D·dt`), so keep it ≤ `0.24` (the explicit-Euler CFL bound for the 5-point
/// Laplacian is `D·dt ≤ 1/4`; the gate ∈[0,1] only lowers the effective rate).
#[derive(Clone, Copy, Debug)]
pub struct DiffusionParams {
    /// `D·dt` per step (≤ 0.24 for stability). Higher = faster bloom per step.
    pub diffusivity: f32,
    /// Water lost per step. As `w` crosses below `w_lo` the gate closes (drying).
    pub evaporation: f32,
    /// `β` — paper-slope advection: pigment drifts downhill into the tooth valleys.
    pub downhill: f32,
    /// `λ` — wet-gradient advection (Curtis FlowOutward): pigment carried wet→dry,
    /// stranding at the receding boundary (edge darkening) + the backrun ring.
    pub flow_outward: f32,
    /// Wet-gate smoothstep band: `g` ramps 0→1 over `w ∈ [w_lo, w_hi]`.
    pub w_lo: f32,
    pub w_hi: f32,
    /// Permeability at a tooth valley (`h=0`) vs crest (`h=1`). Valleys ≥ crests so
    /// pigment pools in the low spots (granulation-coherent).
    pub perm_valley: f32,
    pub perm_crest: f32,
    /// **Pigment-deposition layer (ADR-0078 S3, Curtis 1997 `TransferPigment`).**
    /// Base fraction of FLOWING pigment frozen into the DEPOSITED layer each step.
    /// Deposited pigment no longer diffuses/advects — it's stained into the paper.
    /// `0` ⇒ the layer is dormant (the shipped gated-diffusion look, bit-identical).
    pub deposition: f32,
    /// Extra deposition as a cell DRIES (scaled by `1 − gate`): the rim of a wash
    /// (lower water from the splat falloff) dries first, so its pigment freezes first
    /// → the dark perimeter ring (**edge-darkening**), watercolor's signature mark.
    pub deposition_dry: f32,
    /// Granulation bias: deposition scales by `1 + granulation·(1 − paper)`, so pigment
    /// settles more in the tooth VALLEYS than the crests — the grainy, mottled
    /// **granulation** of pigments like ultramarine / cerulean.
    pub granulation: f32,
}

impl Default for DiffusionParams {
    fn default() -> Self {
        Self {
            diffusivity: 0.2,
            evaporation: 0.012,
            downhill: 0.18,
            flow_outward: 0.35,
            w_lo: 0.05,
            w_hi: 0.4,
            perm_valley: 1.0,
            perm_crest: 0.55,
            // Deposition layer OFF by default → no change to the shipped look until a
            // brush opts in (ADR-0078 S3 ramps these for the watercolor signatures).
            deposition: 0.0,
            deposition_dry: 0.0,
            granulation: 0.0,
        }
    }
}

/// Frequency of the paper-height field sampled into the grid (cycles per grid
/// cell) — coarse enough that valleys span a few cells so pigment can pool.
const HEIGHT_FREQ: f32 = 0.13;
const HEIGHT_SEED: u32 = 0x70a9_e2c5; // shared with cpu_render paper tooth

#[inline]
fn smoothstep(lo: f32, hi: f32, x: f32) -> f32 {
    let t = ((x - lo) / (hi - lo).max(1e-6)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A low-resolution wet-on-wet diffusion field. Holds water + pigment + a static
/// paper-height map, with ping-pong scratch for the conservative passes. All
/// updates are pure arithmetic → deterministic + replayable (HR-5).
pub struct DiffusionGrid {
    width: u32,
    height: u32,
    /// Wetness ∈ [0,1] per cell.
    water: Vec<f32>,
    /// Pigment (linear RGB) per cell — mass-conserving under diffusion/advection.
    pigment: Vec<[f32; 3]>,
    /// **Deposited pigment** (linear RGB) per cell — frozen into the paper by
    /// `TransferPigment` (ADR-0078 S3); does NOT diffuse/advect. The composited colour
    /// is `pigment + deposited`. All-zero while the deposition params are off.
    deposited: Vec<[f32; 3]>,
    /// Static paper-tooth height ∈ [0,1] per cell (1 = crest, 0 = valley).
    paper: Vec<f32>,
    scratch: Vec<[f32; 3]>,
    scratch_w: Vec<f32>,
}

impl DiffusionGrid {
    /// A `width × height` grid with a deterministic paper-height field. `scale`
    /// maps grid cells → the world-space FBM frequency so the tooth is coherent
    /// with the live paper tooth (use 1.0 for a self-contained grid).
    #[must_use]
    pub fn new(width: u32, height: u32, scale: f32) -> Self {
        Self::with_paper(width, height, Self::generate_paper(width, height, scale))
    }

    /// Generate the deterministic paper-tooth height field for a `width × height` grid
    /// at world `scale` — the EXPENSIVE part of [`Self::new`] (`grain_noise` per cell,
    /// O(grid) on the CPU). It depends only on `(width, height, scale)`, so a caller
    /// painting many strokes on one canvas can compute it ONCE and reuse it via
    /// [`Self::with_paper`] instead of paying it per stroke (a ~⅓ s hitch at 4K).
    #[must_use]
    pub fn generate_paper(width: u32, height: u32, scale: f32) -> Vec<f32> {
        let n = (width as usize) * (height as usize);
        let mut paper = vec![0.0f32; n];
        for y in 0..height {
            for x in 0..width {
                paper[(y * width + x) as usize] = crate::grain_noise::grain_value(
                    crate::grain_noise::GRAIN_SIMPLEX,
                    x as f32 * HEIGHT_FREQ * scale,
                    y as f32 * HEIGHT_FREQ * scale,
                    HEIGHT_SEED,
                );
            }
        }
        paper
    }

    /// Build a grid from a PRE-COMPUTED paper field (companion to [`Self::generate_paper`]
    /// for caching). `paper.len()` must equal `width * height`. The water/pigment/scratch
    /// buffers are fresh zeroed allocations (lazy-zeroed pages — ~free until touched, and
    /// the GPU-resident path never touches them).
    #[must_use]
    pub fn with_paper(width: u32, height: u32, paper: Vec<f32>) -> Self {
        let n = (width as usize) * (height as usize);
        debug_assert_eq!(paper.len(), n, "paper length must match width*height");
        Self {
            width,
            height,
            water: vec![0.0; n],
            pigment: vec![[0.0; 3]; n],
            deposited: vec![[0.0; 3]; n],
            paper,
            scratch: vec![[0.0; 3]; n],
            scratch_w: vec![0.0; n],
        }
    }

    #[inline]
    fn idx(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    /// Deposit pigment + water in a soft disc — a brush touch wetting the paper.
    /// Re-wetting a dried area re-opens its gate so the old pigment blooms again.
    /// `water_add` raises wetness (clamped to 1); `color` (linear RGB) is added,
    /// weighted by the disc falloff.
    pub fn splat(&mut self, cx: f32, cy: f32, radius: f32, water_add: f32, color: [f32; 3]) {
        if radius <= 0.0 {
            return;
        }
        let r = radius.max(0.5);
        let x0 = ((cx - r).floor() as i32).max(0);
        let y0 = ((cy - r).floor() as i32).max(0);
        let x1 = ((cx + r).ceil() as i32).min(self.width as i32 - 1);
        let y1 = ((cy + r).ceil() as i32).min(self.height as i32 - 1);
        let inv_r = 1.0 / r;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let d = (((x as f32 - cx).powi(2)) + ((y as f32 - cy).powi(2))).sqrt() * inv_r;
                if d >= 1.0 {
                    continue;
                }
                let fall = 1.0 - d * d * (3.0 - 2.0 * d); // 1 at centre → 0 at rim
                let i = self.idx(x as u32, y as u32);
                self.water[i] = (self.water[i] + water_add * fall).min(1.0);
                self.pigment[i][0] += color[0] * fall;
                self.pigment[i][1] += color[1] * fall;
                self.pigment[i][2] += color[2] * fall;
            }
        }
    }

    /// Advance the field one tick. Gates are precomputed into the reusable scratch
    /// buffer (a `mem::take` decouples it from `&self` so the conservative passes
    /// can read fields + write `scratch` without borrow conflicts).
    pub fn step(&mut self, p: &DiffusionParams) {
        let n = (self.width as usize) * (self.height as usize);
        let mut gates = std::mem::take(&mut self.scratch_w);
        gates.clear();
        gates.reserve(n);
        for i in 0..n {
            let perm = p.perm_valley + (p.perm_crest - p.perm_valley) * self.paper[i];
            gates.push(smoothstep(p.w_lo, p.w_hi, self.water[i]) * perm);
        }
        self.diffuse(p, &gates);
        self.advect(p, &gates);
        self.scratch_w = gates;
        // Curtis TransferPigment (ADR-0078 S3): freeze a fraction of flowing pigment
        // into the deposited layer (edge-darkening + granulation). After advect so it
        // sees the just-transported pigment, before evaporate so `deposition_dry` reads
        // this step's water. No-op while the deposition params are off.
        self.transfer_pigment(p);
        // Evaporate (in place) — drying closes the gate and freezes pigment.
        for w in &mut self.water {
            *w = (*w - p.evaporation).max(0.0);
        }
    }

    /// Pass 1 — conservative gated diffusion of pigment (the bloom). Writes
    /// `scratch`, then swaps. Out-of-bounds faces carry no flux (Neumann) so total
    /// pigment is conserved.
    fn diffuse(&mut self, p: &DiffusionParams, gates: &[f32]) {
        let (w, h) = (self.width, self.height);
        let d = p.diffusivity;
        for y in 0..h {
            for x in 0..w {
                let c = self.idx(x, y);
                let gc = gates[c];
                let pc = self.pigment[c];
                let mut acc = [0.0f32; 3];
                let add = |nidx: usize, acc: &mut [f32; 3]| {
                    let cond = 0.5 * (gc + gates[nidx]);
                    let pn = self.pigment[nidx];
                    acc[0] += cond * (pn[0] - pc[0]);
                    acc[1] += cond * (pn[1] - pc[1]);
                    acc[2] += cond * (pn[2] - pc[2]);
                };
                if x > 0 {
                    add(self.idx(x - 1, y), &mut acc);
                }
                if x + 1 < w {
                    add(self.idx(x + 1, y), &mut acc);
                }
                if y > 0 {
                    add(self.idx(x, y - 1), &mut acc);
                }
                if y + 1 < h {
                    add(self.idx(x, y + 1), &mut acc);
                }
                self.scratch[c] = [pc[0] + d * acc[0], pc[1] + d * acc[1], pc[2] + d * acc[2]];
            }
        }
        std::mem::swap(&mut self.pigment, &mut self.scratch);
    }

    /// Pass 2 — gated upwind advection along `flow = −β·∇h − λ·∇w`. Mass-conserving
    /// (every gram removed from a cell is added to its downstream neighbour);
    /// CFL-clamped to ≤ 0.5 cell/step. Writes `scratch` (seeded with the current
    /// field, then net transfers applied), then swaps.
    fn advect(&mut self, p: &DiffusionParams, gates: &[f32]) {
        let (w, h) = (self.width, self.height);
        self.scratch.copy_from_slice(&self.pigment);
        let sample = |v: &[f32], x: u32, y: u32| v[(y * w + x) as usize];
        for y in 0..h {
            for x in 0..w {
                let c = self.idx(x, y);
                let g = gates[c];
                if g <= 1e-4 {
                    continue;
                }
                // Central-difference gradients (clamped at the border).
                let (xm, xp) = (x.saturating_sub(1), (x + 1).min(w - 1));
                let (ym, yp) = (y.saturating_sub(1), (y + 1).min(h - 1));
                let dhx = sample(&self.paper, xp, y) - sample(&self.paper, xm, y);
                let dhy = sample(&self.paper, x, yp) - sample(&self.paper, x, ym);
                let dwx = sample(&self.water, xp, y) - sample(&self.water, xm, y);
                let dwy = sample(&self.water, x, yp) - sample(&self.water, x, ym);
                let fx =
                    (g * (-p.downhill * 0.5 * dhx - p.flow_outward * 0.5 * dwx)).clamp(-0.5, 0.5);
                let fy =
                    (g * (-p.downhill * 0.5 * dhy - p.flow_outward * 0.5 * dwy)).clamp(-0.5, 0.5);
                let pc = self.pigment[c];
                // Upwind: push |f|·p of pigment to the downstream neighbour.
                let (nx, amx) = if fx > 0.0 && x + 1 < w {
                    (Some(c + 1), fx)
                } else if fx < 0.0 && x > 0 {
                    (Some(c - 1), -fx)
                } else {
                    (None, 0.0)
                };
                let (ny, amy) = if fy > 0.0 && y + 1 < h {
                    (Some(c + w as usize), fy)
                } else if fy < 0.0 && y > 0 {
                    (Some(c - w as usize), -fy)
                } else {
                    (None, 0.0)
                };
                for (k, &pck) in pc.iter().enumerate() {
                    if let Some(n) = nx {
                        let q = amx * pck;
                        self.scratch[c][k] -= q;
                        self.scratch[n][k] += q;
                    }
                    if let Some(n) = ny {
                        let q = amy * pck;
                        self.scratch[c][k] -= q;
                        self.scratch[n][k] += q;
                    }
                }
            }
        }
        std::mem::swap(&mut self.pigment, &mut self.scratch);
    }

    /// Pass 3 — **`TransferPigment`** (Curtis 1997 §4.2 / ADR-0078 S3): freeze a
    /// fraction of the FLOWING pigment into the DEPOSITED layer. Mass-conserving (every
    /// gram leaving `pigment` lands in `deposited`); deposited pigment is staticstained
    /// into the paper (no more diffuse/advect). The rate rises as a cell dries
    /// (`deposition_dry · (1 − gate)`) → the rim freezes first → **edge-darkening**;
    /// and in the tooth valleys (`granulation · (1 − paper)`) → **granulation**. A
    /// no-op while all deposition params are 0 (the shipped look is untouched).
    fn transfer_pigment(&mut self, p: &DiffusionParams) {
        if p.deposition <= 0.0 && p.deposition_dry <= 0.0 {
            return;
        }
        let n = (self.width as usize) * (self.height as usize);
        for i in 0..n {
            // `dry` ∈ [0,1]: 0 where wet (gate open), 1 where dry (gate shut). Same
            // smoothstep band as the gate so deposition kicks in exactly as flow stops.
            let dry = 1.0 - smoothstep(p.w_lo, p.w_hi, self.water[i]);
            let gran = 1.0 + p.granulation * (1.0 - self.paper[i]);
            let rate = ((p.deposition + p.deposition_dry * dry) * gran).clamp(0.0, 1.0);
            if rate <= 0.0 {
                continue;
            }
            for k in 0..3 {
                let moved = rate * self.pigment[i][k];
                self.pigment[i][k] -= moved;
                self.deposited[i][k] += moved;
            }
        }
    }

    /// Grid dimensions.
    #[must_use]
    pub fn dims(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Pigment field (linear RGB per cell).
    #[must_use]
    pub fn pigment(&self) -> &[[f32; 3]] {
        &self.pigment
    }

    /// Wetness field ∈ [0,1] per cell.
    #[must_use]
    pub fn water(&self) -> &[f32] {
        &self.water
    }

    /// Paper-height field ∈ [0,1] per cell (1 = crest, 0 = valley).
    #[must_use]
    pub fn paper(&self) -> &[f32] {
        &self.paper
    }

    /// Overwrite the pigment field — used to write a GPU-stepped result back into
    /// the grid (the GPU is the accelerator; this grid stays the CPU source of
    /// truth + the composite input). Panics on a length mismatch (caller sizes it).
    pub fn set_pigment_from(&mut self, p: &[[f32; 3]]) {
        self.pigment.copy_from_slice(p);
    }

    /// Overwrite the wetness field — companion to [`Self::set_pigment_from`] so the
    /// GPU's evaporated water replaces the grid's (else a re-upload would re-wet it).
    pub fn set_water_from(&mut self, w: &[f32]) {
        self.water.copy_from_slice(w);
    }

    /// Zero the pigment field — the W15.3 GPU resident path uses the grid pigment as
    /// the per-frame **dab deposit**: the shell uploads it (added to the GPU-resident
    /// `pig_a`), then clears it so next frame holds only the next frame's dabs. The
    /// bloomed pigment lives on the GPU, not here.
    pub fn clear_pigment(&mut self) {
        for p in &mut self.pigment {
            *p = [0.0; 3];
        }
    }

    /// Evaporate the water field by `amount` (clamped at 0). The W15.3 resident path
    /// keeps water CPU-side (the GPU reads it for the gate/flow but never writes it),
    /// so the CPU owns evaporation + the dry-check — no water readback (ADR-0049 §0).
    pub fn evaporate(&mut self, amount: f32) {
        for w in &mut self.water {
            *w = (*w - amount).max(0.0);
        }
    }

    /// The wettest cell (max water) — the W15.3 CPU dry-check (`< threshold` ⇒ drop
    /// the field), computed on the CPU water mirror so no GPU readback is needed.
    #[must_use]
    pub fn max_water(&self) -> f32 {
        self.water.iter().copied().fold(0.0f32, f32::max)
    }

    /// Inclusive grid-cell bbox of cells **currently** wetter than `threshold`
    /// (`None` if dry).
    ///
    /// **NOT a superset of the resident pigment** — do not composite over this alone.
    /// Water only evaporates (its bbox marches INWARD each step), while pigment is
    /// conserved AND `diffuse`/`advect` push it up to a cell PAST the wet gate. So a
    /// drying wash has pigment OUTSIDE the current water bbox; compositing over this
    /// rect hard-cuts the round dab into an axis-aligned rectangle (the W15.3
    /// "quinas retangulares" bug). The GPU path must composite over the **all-time
    /// wet envelope** (the monotonic union of these bboxes — a true upper bound on
    /// where pigment can ever be, since the gate only opens for `water > w_lo ≫
    /// threshold`); the CPU path uses the exact pigment bbox. This stays readback-free.
    #[must_use]
    pub fn water_bbox(&self, threshold: f32) -> Option<(u32, u32, u32, u32)> {
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
        let mut any = false;
        for gy in 0..self.height {
            let row = (gy * self.width) as usize;
            for gx in 0..self.width {
                if self.water[row + gx as usize] > threshold {
                    any = true;
                    x0 = x0.min(gx);
                    y0 = y0.min(gy);
                    x1 = x1.max(gx);
                    y1 = y1.max(gy);
                }
            }
        }
        any.then_some((x0, y0, x1, y1))
    }

    /// Total FLOWING pigment per channel — conserved under pure diffusion + advection
    /// (no evaporation removes pigment). With the deposition layer on, flowing pigment
    /// migrates into [`Self::deposited`]; `total_pigment + total_deposited` is the
    /// conserved sum (the invariant the deposition tests check).
    #[must_use]
    pub fn total_pigment(&self) -> [f64; 3] {
        let mut s = [0.0f64; 3];
        for px in &self.pigment {
            s[0] += px[0] as f64;
            s[1] += px[1] as f64;
            s[2] += px[2] as f64;
        }
        s
    }

    /// The deposited (frozen-into-paper) pigment field (ADR-0078 S3). The composited
    /// colour is `pigment + deposited`; all-zero while the deposition params are off.
    #[must_use]
    pub fn deposited(&self) -> &[[f32; 3]] {
        &self.deposited
    }

    /// Total deposited pigment per channel (companion to [`Self::total_pigment`] for
    /// the mass-conservation invariant `flowing + deposited == splatted`).
    #[must_use]
    pub fn total_deposited(&self) -> [f64; 3] {
        let mut s = [0.0f64; 3];
        for px in &self.deposited {
            s[0] += px[0] as f64;
            s[1] += px[1] as f64;
            s[2] += px[2] as f64;
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pigment mass is conserved by diffusion + advection (no evaporation): the
    /// per-channel total is invariant to ~1e-3 over many steps.
    #[test]
    fn pigment_mass_is_conserved() {
        let mut g = DiffusionGrid::new(48, 48, 1.0);
        // A wet, pigmented blob; flood the whole grid wet so the gate is open.
        for w in g.water.iter_mut() {
            *w = 1.0;
        }
        g.splat(24.0, 24.0, 8.0, 0.0, [0.6, 0.2, 0.1]);
        let before = g.total_pigment();
        let p = DiffusionParams {
            evaporation: 0.0, // isolate conservation
            ..Default::default()
        };
        for _ in 0..40 {
            g.step(&p);
        }
        let after = g.total_pigment();
        for k in 0..3 {
            assert!(
                (before[k] - after[k]).abs() < 1e-3 * (before[k].abs() + 1.0),
                "channel {k} mass drifted: {} -> {}",
                before[k],
                after[k]
            );
        }
    }

    /// Diffusion is DETERMINISTIC — same seed + inputs → bit-identical field (HR-5).
    #[test]
    fn diffusion_is_deterministic() {
        let run = || {
            let mut g = DiffusionGrid::new(32, 32, 1.0);
            for w in g.water.iter_mut() {
                *w = 0.8;
            }
            g.splat(16.0, 16.0, 5.0, 0.3, [0.5, 0.3, 0.7]);
            let p = DiffusionParams::default();
            for _ in 0..25 {
                g.step(&p);
            }
            g.pigment().to_vec()
        };
        assert_eq!(run(), run(), "diffusion must be deterministic (HR-5)");
    }

    /// The gate: pigment in a wet pool SPREADS (the bloom), but on DRY paper it
    /// stays put (crisp edge) — same blob, wet vs dry.
    #[test]
    fn wet_blooms_dry_stays_put() {
        let spread = |wet: f32| -> f32 {
            let mut g = DiffusionGrid::new(48, 48, 1.0);
            for w in g.water.iter_mut() {
                *w = wet;
            }
            g.splat(24.0, 24.0, 3.0, 0.0, [0.7, 0.2, 0.1]);
            let p = DiffusionParams {
                evaporation: 0.0,
                ..Default::default()
            };
            for _ in 0..30 {
                g.step(&p);
            }
            // Pigment that reached a ring ~8px from the centre = how far it spread.
            let (cx, cy) = (24.0f32, 24.0f32);
            let mut ring = 0.0f32;
            for y in 0..48u32 {
                for x in 0..48u32 {
                    let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
                    if (7.0..9.0).contains(&d) {
                        ring += g.pigment()[(y * 48 + x) as usize][0];
                    }
                }
            }
            ring
        };
        let wet = spread(0.9);
        let dry = spread(0.0);
        assert!(wet > 0.05, "wet pigment must bloom outward: ring {wet}");
        assert!(
            dry < wet * 0.1,
            "dry pigment must NOT spread: dry {dry} vs wet {wet}"
        );
    }

    /// Stability: many steps never produce NaN / Inf / runaway values.
    #[test]
    fn solver_is_stable_over_many_steps() {
        let mut g = DiffusionGrid::new(40, 40, 1.0);
        g.splat(20.0, 20.0, 10.0, 1.0, [0.9, 0.8, 0.2]);
        g.splat(10.0, 10.0, 4.0, 0.6, [0.1, 0.2, 0.9]);
        let p = DiffusionParams::default();
        for _ in 0..200 {
            g.step(&p);
        }
        for px in g.pigment() {
            for &c in px {
                assert!(
                    c.is_finite() && (0.0..10.0).contains(&c),
                    "unstable value {c}"
                );
            }
        }
    }

    /// Drying ends the bleed: after the water evaporates the gate is shut, so a
    /// further burst of steps barely moves the (now frozen) pigment.
    #[test]
    fn drying_freezes_the_pigment() {
        let mut g = DiffusionGrid::new(48, 48, 1.0);
        g.splat(24.0, 24.0, 4.0, 0.5, [0.6, 0.2, 0.1]);
        let p = DiffusionParams::default();
        // Run until dry (water starts at ≤0.5, evap 0.012 → ~42 steps to 0).
        for _ in 0..60 {
            g.step(&p);
        }
        let dried = g.pigment().to_vec();
        for _ in 0..60 {
            g.step(&p);
        }
        let mut max_move = 0.0f32;
        for (a, b) in dried.iter().zip(g.pigment().iter()) {
            max_move = max_move.max((a[0] - b[0]).abs());
        }
        assert!(
            max_move < 1e-3,
            "dried pigment must stay frozen: moved {max_move}"
        );
    }

    // ───────────────── ADR-0078 S3 — pigment-deposition layer ─────────────────

    /// DORMANT by default: with the deposition params at 0 nothing is ever deposited,
    /// so the shipped gated-diffusion look is untouched (the rest of the test suite,
    /// run with `Default`, also implicitly proves `transfer_pigment` is a no-op).
    #[test]
    fn deposition_off_is_dormant() {
        let mut g = DiffusionGrid::new(40, 40, 1.0);
        g.splat(20.0, 20.0, 8.0, 0.9, [0.6, 0.2, 0.1]);
        let p = DiffusionParams::default();
        for _ in 0..50 {
            g.step(&p);
        }
        let dep = g.total_deposited();
        assert_eq!(dep, [0.0, 0.0, 0.0], "deposition must be dormant at rate 0");
    }

    /// Mass-conserving: `TransferPigment` MOVES pigment flowing→deposited, never
    /// creates/destroys it. `total_pigment + total_deposited` equals the splatted mass
    /// (evaporation removes only water, not pigment).
    #[test]
    fn deposition_conserves_total_pigment() {
        let mut g = DiffusionGrid::new(48, 48, 1.0);
        for w in g.water.iter_mut() {
            *w = 1.0;
        }
        g.splat(24.0, 24.0, 8.0, 0.0, [0.6, 0.2, 0.1]);
        let before = g.total_pigment();
        let p = DiffusionParams {
            evaporation: 0.0, // isolate conservation (no drying)
            deposition: 0.03,
            deposition_dry: 0.05,
            granulation: 1.5,
            ..Default::default()
        };
        for _ in 0..40 {
            g.step(&p);
        }
        let flow = g.total_pigment();
        let dep = g.total_deposited();
        assert!(dep[0] > 0.01, "deposition must actually occur (dep {dep:?})");
        for k in 0..3 {
            let total = flow[k] + dep[k];
            assert!(
                (before[k] - total).abs() < 1e-3 * (before[k].abs() + 1.0),
                "channel {k}: flowing+deposited {total} != splatted {}",
                before[k]
            );
        }
    }

    /// EDGE-DARKENING: a wash's rim (lower water from the splat falloff) dries first,
    /// so `deposition_dry` freezes its pigment first → the rim ends up a HIGHER
    /// deposited fraction than the still-wet centre (watercolor's dark perimeter).
    #[test]
    fn dry_deposition_darkens_the_rim() {
        let (w, h) = (80u32, 80u32);
        let (cx, cy, r) = (40.0f32, 40.0, 16.0);
        let mut g = DiffusionGrid::new(w, h, 1.0);
        g.splat(cx, cy, r, 0.9, [0.5, 0.3, 0.2]);
        let p = DiffusionParams {
            deposition: 0.0,
            deposition_dry: 0.4, // pure dry-driven deposition isolates edge-darkening
            granulation: 0.0,
            ..Default::default()
        };
        for _ in 0..30 {
            g.step(&p);
        }
        // Mean deposited FRACTION (deposited / total) over a rim annulus vs a centre
        // disc — the fraction normalises out the splat's radial pigment falloff.
        let frac = |lo: f32, hi: f32| -> f32 {
            let (mut dep, mut tot) = (0.0f32, 0.0f32);
            for y in 0..h {
                for x in 0..w {
                    let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
                    if d >= lo && d < hi {
                        let i = (y * w + x) as usize;
                        dep += g.deposited()[i][0];
                        tot += g.deposited()[i][0] + g.pigment()[i][0];
                    }
                }
            }
            if tot > 1e-6 { dep / tot } else { 0.0 }
        };
        let rim = frac(12.0, 15.0);
        let center = frac(0.0, 4.0);
        assert!(
            rim > center + 0.1,
            "edge-darkening: rim deposited fraction {rim} must exceed centre {center}"
        );
    }

    /// GRANULATION: with `granulation > 0`, pigment settles more in the tooth VALLEYS
    /// (low paper height) than the crests — the deposited fraction is higher in valleys.
    #[test]
    fn granulation_favors_paper_valleys() {
        let (w, h) = (64u32, 64u32);
        let mut g = DiffusionGrid::new(w, h, 4.0); // scale 4 → paper varies across grid
        for wv in g.water.iter_mut() {
            *wv = 0.6;
        }
        // Broad pigment coverage so most cells carry pigment to settle.
        g.splat(32.0, 32.0, 28.0, 0.0, [0.5, 0.4, 0.3]);
        // Small base + few steps keeps the deposited fraction MID-range (not saturated
        // near 1, where the valley/crest gap would be compressed), so the granulation
        // rate difference is visible.
        let p = DiffusionParams {
            evaporation: 0.0,
            deposition: 0.01,
            deposition_dry: 0.0,
            granulation: 4.0,
            ..Default::default()
        };
        for _ in 0..18 {
            g.step(&p);
        }
        // Median paper height splits valleys vs crests; compare deposited fraction.
        let mut sorted: Vec<f32> = g.paper.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = sorted[sorted.len() / 2];
        let (mut v_dep, mut v_tot, mut c_dep, mut c_tot) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        for i in 0..(w * h) as usize {
            let tot = g.deposited()[i][0] + g.pigment()[i][0];
            if tot < 1e-4 {
                continue;
            }
            if g.paper[i] < median {
                v_dep += g.deposited()[i][0];
                v_tot += tot;
            } else {
                c_dep += g.deposited()[i][0];
                c_tot += tot;
            }
        }
        let valley = if v_tot > 1e-6 { v_dep / v_tot } else { 0.0 };
        let crest = if c_tot > 1e-6 { c_dep / c_tot } else { 0.0 };
        assert!(
            valley > crest + 0.02,
            "granulation: valley deposited fraction {valley} must exceed crest {crest}"
        );
    }
}
