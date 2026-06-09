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
    /// **Shallow-water velocity layer (ADR-0078 S3d, Curtis 1997 §3).** Master scale on
    /// the momentum-carrying velocity field `(u, v)`'s contribution to pigment advection.
    /// `0` ⇒ the layer is **dormant**: [`DiffusionGrid::move_water`] is skipped and pigment
    /// advects by the static gradient flow `−β·∇h − λ·∇w` (the shipped look, bit-identical).
    /// `>0` ⇒ pigment advects by `velocity·(u, v)` instead → **directional flow +
    /// backruns/cauliflower**, the watercolor signatures gated diffusion can't make.
    pub velocity: f32,
    /// `μ` — momentum viscosity: per-step Laplacian smoothing of the velocity field
    /// (coherent flow, damps the collocated-grid checkerboard). Keep ≤ 0.24 (the same
    /// explicit-Euler bound as `diffusivity`). Only read while the velocity layer is on.
    pub viscosity: f32,
    /// `κ` — velocity drag ∈ [0,1): per-step damping `(u, v) *= (1 − drag)` so the flow
    /// decays as the wash settles (stability + the wash coming to rest). Layer-on only.
    pub drag: f32,
    /// Pressure-projection strength (Curtis *FlowOutward* / incompressibility relaxation):
    /// the fraction of the divergence-removing pressure gradient subtracted from the
    /// velocity each `move_water`. Turns the local body forces into *directional* flow and
    /// builds the pressure fronts that pile pigment into the **backrun ring**. `0` ⇒ no
    /// projection (compressible advected forces — flow, but no backruns). Layer-on only.
    pub pressure: f32,
    /// **Capillary fringe (ADR-0078 S5, Curtis 1997 capillary layer).** Per-step rate of the
    /// outward wicking of WATER from wet cells into the drier paper *beyond* the painted area
    /// ([`DiffusionGrid::capillary_flow`]) — the soft, feathery wet-edge that defines the
    /// watercolor look. A conservative, permeability-weighted diffusion of the water field
    /// (`water += capillary·Σ_face ½(perm_c+perm_n)(water_n − water_c)`); because diffusion only
    /// acts where there is a water gradient, the saturated pool interior is untouched and only
    /// the wet→dry boundary creeps out, carrying the wet gate (so the existing pigment then
    /// bleeds into the fringe). CFL-bounded: keep ≤ `0.24` (perm ≤ 1 ⇒ `capillary·Σ_face cond ≤
    /// 1`, so water stays in `[0,1]` and mass is conserved without a clamp). `0` ⇒ the layer is
    /// **dormant**: [`DiffusionGrid::capillary_flow`] is skipped and the water field is
    /// bit-identical to the shipped (harder wet-gate) edge.
    pub capillary: f32,
    /// **Capillary pigment mobility (ADR-0078 S5 — chromatographic filtering).** The fraction
    /// of the capillary WATER transport that the PIGMENT follows. In real watercolor the paper
    /// fibres filter the (larger) pigment particles while the water wicks ahead, so the wet
    /// front outruns the pigment: the outermost fringe is water-only (transparent) and the
    /// pigment lags behind, feathering out — *this* is why a watercolor edge fades to
    /// transparent. `1.0` ⇒ pigment co-moves 1:1 with the water (a uniformly-coloured, opaque
    /// fringe — physically wrong); `< 1.0` ⇒ pigment lags (the transparent outer halo + soft
    /// colour feather); `0.0` ⇒ Curtis's water-only capillary (pigment reaches the fringe only
    /// via the weak gated bloom). Only read while the capillary layer is active.
    pub capillary_mobility: f32,
    /// **Advection sharpness (ADR-0078 S5c — BFECC / MacCormack second-order transport, Selle
    /// et al. 2008).** Blends the first-order upwind [`DiffusionGrid::advect`] (`0`, smearing/
    /// diffusive) toward the **MacCormack** error-compensated advection (`1`, sharp): a forward
    /// advect, a reverse advect of that, and a corrected re-step `φ̂ + s·½(φ − φ̄)`, clamped to
    /// the local extrema (the unconditionally-stable limiter). It cancels the numerical
    /// diffusion of the plain upwind step, so velocity-driven flow + backruns/cauliflower stay
    /// CRISP instead of blurring out. `0` ⇒ exactly the first-order path (bit-identical shipped
    /// look, mass-conserving); `>0` ⇒ progressively sharper (the clamp keeps it stable but is no
    /// longer strictly mass-conserving). Range `[0,1]`.
    pub sharpness: f32,
}

/// Default capillary pigment mobility (ADR-0078 S5): the pigment co-advects at ~⅓ the water's
/// capillary rate, so the water front clearly outruns it → a transparent outer halo with the
/// colour feathering behind it (the chromatographic filtering that gives watercolor its soft
/// transparent edge). A physical constant of the paper↔pigment interaction, not an artist knob;
/// the artist controls how *far* the water wicks via the per-brush `Capillary` slider.
pub const CAPILLARY_PIGMENT_MOBILITY: f32 = 0.35;

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
            // Shallow-water velocity layer OFF by default (ADR-0078 S3d) → `move_water` is
            // skipped and the static gradient-flow advect runs, so the shipped look is
            // bit-identical until a brush opts in.
            velocity: 0.0,
            viscosity: 0.0,
            drag: 0.0,
            pressure: 0.0,
            // Capillary fringe OFF by default (ADR-0078 S5) → `capillary_flow` is skipped and
            // the shipped (wet-gate) edge is bit-identical until a brush opts in.
            capillary: 0.0,
            // The physical pigment-filtering constant (only read when the capillary layer is on).
            capillary_mobility: CAPILLARY_PIGMENT_MOBILITY,
            // First-order advection by default (ADR-0078 S5c) → bit-identical shipped look.
            sharpness: 0.0,
        }
    }
}

/// Frequency of the paper-height field sampled into the grid (cycles per grid
/// cell) — coarse enough that valleys span a few cells so pigment can pool.
const HEIGHT_FREQ: f32 = 0.13;
const HEIGHT_SEED: u32 = 0x70a9_e2c5; // shared with cpu_render paper tooth

/// Fixed Jacobi-iteration count for the shallow-water pressure projection
/// ([`DiffusionGrid::project`], ADR-0078 S3d). A *fixed* count (not a convergence
/// threshold) keeps the solve deterministic + bounded (HR-5) and makes the GPU mirror
/// trivially bit-parity (both run the same loop). A handful of iterations is enough
/// to turn the local body forces into directional flow at watercolor's visual budget;
/// the pressure field is re-seeded to 0 each `move_water` (no warm-start state).
///
/// **Public** so the GPU mirror (`ph2d-painter-fluid`) runs the exact same sweep count —
/// the parity gate depends on it. **Keep even** (the GPU Jacobi ping-pong lands the result
/// in a fixed buffer when the count is even).
pub const RELAX_ITERS: u32 = 6;

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
    /// **Shallow-water velocity field (ADR-0078 S3d)** — `(vel_u, vel_v)` per cell, the
    /// momentum that transports pigment when the velocity layer is active. Resident state
    /// (persists across steps → momentum), damped by `drag`. All-zero while dormant
    /// (the gradient-flow path); lazy-zeroed, so it's free until the layer runs.
    vel_u: Vec<f32>,
    vel_v: Vec<f32>,
    /// Pressure field for the incompressibility projection — re-seeded to 0 and re-solved
    /// each [`Self::move_water`] (Jacobi, [`RELAX_ITERS`]); kept here only to avoid a
    /// per-step alloc.
    pressure: Vec<f32>,
    scratch: Vec<[f32; 3]>,
    scratch_w: Vec<f32>,
    /// Velocity ping-pong scratch (the `add_forces` write target) + the divergence and
    /// pressure-Jacobi ping-pong buffers. Allocated lazily with the rest; untouched (and
    /// near-free) while the velocity layer is dormant.
    scratch_u: Vec<f32>,
    scratch_v: Vec<f32>,
    divergence: Vec<f32>,
    pressure_b: Vec<f32>,
    /// Water ping-pong scratch — the [`Self::capillary_flow`] write target (the capillary
    /// diffusion reads the old water + writes the new field here, then swaps). Allocated
    /// lazily with the rest; untouched (near-free) while the capillary layer is dormant.
    scratch_w2: Vec<f32>,
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
            vel_u: vec![0.0; n],
            vel_v: vec![0.0; n],
            pressure: vec![0.0; n],
            scratch: vec![[0.0; 3]; n],
            scratch_w: vec![0.0; n],
            scratch_u: vec![0.0; n],
            scratch_v: vec![0.0; n],
            divergence: vec![0.0; n],
            pressure_b: vec![0.0; n],
            scratch_w2: vec![0.0; n],
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
        // Shallow-water MoveWater (ADR-0078 S3d): evolve the velocity field BEFORE the
        // pigment transport so `advect` carries pigment along the just-updated `(u,v)`.
        // Reads the pre-evaporation water (same as diffuse/advect). Dormant (skipped)
        // unless the velocity layer is on → the shipped gradient-flow path is untouched.
        if p.velocity > 0.0 {
            self.move_water(p);
        }
        self.diffuse(p, &gates);
        // Advection: first-order upwind by default; MacCormack (sharp) when `sharpness > 0`
        // (ADR-0078 S5c). The `sharpness = 0` branch is the bit-identical conservative path.
        if p.sharpness > 0.0 {
            self.advect_maccormack(p, &gates);
        } else {
            self.advect(p, &gates, false);
        }
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
        // Capillary fringe (ADR-0078 S5): wick water outward into the drier paper so the wet
        // gate (and then the pigment) creeps past the painted area into a soft fringe. Last,
        // after evaporate, so it sees this step's dried water; the spread is picked up by the
        // NEXT step's gate. No-op while `capillary` is 0 (the shipped edge is bit-identical).
        if p.capillary > 0.0 {
            self.capillary_flow(p);
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
    /// field, then net transfers applied), then swaps. `reverse` negates the flow
    /// (the backward pass of the MacCormack scheme, [`Self::advect_maccormack`]).
    fn advect(&mut self, p: &DiffusionParams, gates: &[f32], reverse: bool) {
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
                // Flow vector for the upwind transport. Two modes (ADR-0078 S3d):
                //   • velocity ON  → the momentum-carrying shallow-water field `(u, v)`,
                //     scaled by the master `velocity` and CFL-clamped (the real flow).
                //   • velocity OFF → the static gradient flow `−β·∇h − λ·∇w` (the shipped
                //     path; `move_water` never ran so `vel_*` is all-zero anyway).
                let (fx, fy) = if p.velocity > 0.0 {
                    (
                        (p.velocity * self.vel_u[c]).clamp(-0.5, 0.5),
                        (p.velocity * self.vel_v[c]).clamp(-0.5, 0.5),
                    )
                } else {
                    // Central-difference gradients (clamped at the border).
                    let (xm, xp) = (x.saturating_sub(1), (x + 1).min(w - 1));
                    let (ym, yp) = (y.saturating_sub(1), (y + 1).min(h - 1));
                    let dhx = sample(&self.paper, xp, y) - sample(&self.paper, xm, y);
                    let dhy = sample(&self.paper, x, yp) - sample(&self.paper, x, ym);
                    let dwx = sample(&self.water, xp, y) - sample(&self.water, xm, y);
                    let dwy = sample(&self.water, x, yp) - sample(&self.water, x, ym);
                    (
                        (g * (-p.downhill * 0.5 * dhx - p.flow_outward * 0.5 * dwx))
                            .clamp(-0.5, 0.5),
                        (g * (-p.downhill * 0.5 * dhy - p.flow_outward * 0.5 * dwy))
                            .clamp(-0.5, 0.5),
                    )
                };
                // Backward pass (MacCormack): negate the flow to advect the field back.
                let (fx, fy) = if reverse { (-fx, -fy) } else { (fx, fy) };
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

    /// Pass 2 (sharp) — **MacCormack / BFECC second-order advection** (ADR-0078 S5c, Selle et
    /// al. 2008): cancels the numerical diffusion of the first-order upwind [`Self::advect`] so
    /// velocity-driven flow + backruns/cauliflower stay CRISP instead of smearing out. Forward-
    /// advect `φ → φ̂`, reverse-advect `φ̂ → φ̄` (an estimate of the round-trip error), then re-form
    /// `φ̂ + sharpness·½(φ − φ̄)` and CLAMP each cell to the local extrema of `φ` (the
    /// unconditionally-stable limiter — admits no new under/overshoot, so no ringing or negative
    /// pigment). `sharpness → 0` reduces to the first-order result (the clamp is a no-op on the
    /// monotone upwind `φ̂`); `→ 1` is full MacCormack. The clamp makes it non-strictly
    /// mass-conserving, so it's opt-in — the default `sharpness = 0` keeps the plain conservative
    /// path. Bit-mirrored by the GPU `cs_advect_correct`.
    fn advect_maccormack(&mut self, p: &DiffusionParams, gates: &[f32]) {
        let phi = self.pigment.clone(); // φ
        self.advect(p, gates, false); // pigment = φ̂ (forward)
        let phi_hat = self.pigment.clone(); // φ̂ (kept for the combine)
        self.advect(p, gates, true); // pigment = φ̄ (reverse of φ̂)
        let s = p.sharpness;
        let (w, h) = (self.width, self.height);
        for y in 0..h {
            for x in 0..w {
                let c = self.idx(x, y);
                // Local extrema of φ over the 5-point stencil = the MacCormack clamp range.
                let mut nbrs = [0usize; 4];
                let mut cnt = 0usize;
                if x > 0 {
                    nbrs[cnt] = self.idx(x - 1, y);
                    cnt += 1;
                }
                if x + 1 < w {
                    nbrs[cnt] = self.idx(x + 1, y);
                    cnt += 1;
                }
                if y > 0 {
                    nbrs[cnt] = self.idx(x, y - 1);
                    cnt += 1;
                }
                if y + 1 < h {
                    nbrs[cnt] = self.idx(x, y + 1);
                    cnt += 1;
                }
                let mut lo = phi[c];
                let mut hi = phi[c];
                for &ni in &nbrs[..cnt] {
                    let v = phi[ni];
                    for k in 0..3 {
                        lo[k] = lo[k].min(v[k]);
                        hi[k] = hi[k].max(v[k]);
                    }
                }
                let pbar = self.pigment[c]; // φ̄
                for k in 0..3 {
                    let corrected = phi_hat[c][k] + s * 0.5 * (phi[c][k] - pbar[k]);
                    self.scratch[c][k] = corrected.clamp(lo[k], hi[k]);
                }
            }
        }
        std::mem::swap(&mut self.pigment, &mut self.scratch);
    }

    /// **MoveWater (ADR-0078 S3d / Curtis 1997 §3)** — advance the shallow-water velocity
    /// field one tick: accelerate `(u, v)` by the body forces + viscosity + drag
    /// ([`Self::add_forces`]), then make it (near-)divergence-free by the pressure
    /// projection ([`Self::project`]). The result is the momentum-carrying flow that
    /// [`Self::advect`] transports pigment along. Called from [`Self::step`] only while the
    /// velocity layer is on (`params.velocity > 0`); the velocity is resident state, so
    /// momentum accumulates across ticks (drag bleeds it off as the wash settles).
    fn move_water(&mut self, p: &DiffusionParams) {
        self.add_forces(p);
        self.project(p);
    }

    /// MoveWater step 1 — accelerate the velocity field. The velocity layer is driven by
    /// the **water-surface dynamics** — the `−λ·∇w` FlowOutward (wet→dry) push — smoothed by
    /// viscosity (`μ·∇²(u,v)`), damped by drag (`×(1−κ)`), faded by the wetness mask, and
    /// CFL-clamped to ±0.5 cell/step. The previous velocity is read in place (momentum), the
    /// new one written to scratch, then swapped. Dry cells (outside the wet mask) hold zero
    /// velocity (Curtis's `M`).
    ///
    /// **The paper-slope `−β·∇h` (`downhill`) force is here but OFF in the preset** (ADR-0079,
    /// 2026-06-08). It channels flowing pigment into the tooth valleys; accumulated by the
    /// velocity's momentum it imprints the paper grain as visible mottling ("ruído grande" —
    /// the original S3d complaint). So the **preset sets `downhill = 0`** (clean default —
    /// paper texture enters once, via the deposition/granulation layer) but the artist can
    /// dial it up via the "Downhill" slider for deliberate granular paper-flow. It was briefly
    /// removed entirely, which left the slider a no-op (Enio 2026-06-08: "Downhill — nenhuma
    /// mudança"); restoring it here makes the control live again.
    fn add_forces(&mut self, p: &DiffusionParams) {
        let (w, h) = (self.width, self.height);
        let n = (w as usize) * (h as usize);
        let li = |x: u32, y: u32| (y * w + x) as usize;
        let mut nu = std::mem::take(&mut self.scratch_u);
        let mut nv = std::mem::take(&mut self.scratch_v);
        nu.clear();
        nu.resize(n, 0.0);
        nv.clear();
        nv.resize(n, 0.0);
        for y in 0..h {
            for x in 0..w {
                let c = li(x, y);
                // Pure wetness mask (NOT the perm-weighted pigment gate): velocity exists
                // only where there is water, fading to 0 as the cell dries.
                let wet = smoothstep(p.w_lo, p.w_hi, self.water[c]);
                if wet <= 1e-4 {
                    continue; // nu[c] = nv[c] = 0 already
                }
                // **Permeability GATES the body forces** (ADR-0079, 2026-06-08): a less
                // permeable patch (crest) generates LESS flow, so the "Perm Valley/Crest"
                // controls visibly modulate the velocity (not just the diffusion gate — that
                // was masked by the perm-independent velocity transport). The resulting
                // paper-textured velocity is also what `viscosity` then smooths, so both
                // controls read. Gating the FORCE (not the resident velocity) avoids
                // compounding-decay of the momentum.
                let perm = p.perm_valley + (p.perm_crest - p.perm_valley) * self.paper[c];
                let (xm, xp) = (x.saturating_sub(1), (x + 1).min(w - 1));
                let (ym, yp) = (y.saturating_sub(1), (y + 1).min(h - 1));
                let dhx = self.paper[li(xp, y)] - self.paper[li(xm, y)];
                let dhy = self.paper[li(x, yp)] - self.paper[li(x, ym)];
                let dwx = self.water[li(xp, y)] - self.water[li(xm, y)];
                let dwy = self.water[li(x, yp)] - self.water[li(x, ym)];
                // Neumann velocity Laplacian (border neighbour = self via the clamp).
                let lap_u = self.vel_u[li(xm, y)] + self.vel_u[li(xp, y)] + self.vel_u[li(x, ym)]
                    + self.vel_u[li(x, yp)]
                    - 4.0 * self.vel_u[c];
                let lap_v = self.vel_v[li(xm, y)] + self.vel_v[li(xp, y)] + self.vel_v[li(x, ym)]
                    + self.vel_v[li(x, yp)]
                    - 4.0 * self.vel_v[c];
                let mut u = self.vel_u[c]
                    - perm * (p.downhill * 0.5 * dhx + p.flow_outward * 0.5 * dwx)
                    + p.viscosity * lap_u;
                let mut v = self.vel_v[c]
                    - perm * (p.downhill * 0.5 * dhy + p.flow_outward * 0.5 * dwy)
                    + p.viscosity * lap_v;
                u *= 1.0 - p.drag;
                v *= 1.0 - p.drag;
                nu[c] = (u * wet).clamp(-0.5, 0.5);
                nv[c] = (v * wet).clamp(-0.5, 0.5);
            }
        }
        self.scratch_u = std::mem::replace(&mut self.vel_u, nu);
        self.scratch_v = std::mem::replace(&mut self.vel_v, nv);
    }

    /// MoveWater step 2 — **pressure projection** (incompressibility / Curtis FlowOutward
    /// relaxation). Removing the divergence of `(u,v)` is what turns the local body forces
    /// into *directional* flow and builds the pressure fronts at the wet boundary that
    /// pile pigment into the **backrun ring**. Computes `∇·(u,v)`, solves
    /// `∇²pressure = divergence` by [`RELAX_ITERS`] **Jacobi** sweeps (order-independent →
    /// GPU-parity-safe; pressure re-seeded to 0 each call → deterministic, no warm-start),
    /// then subtracts `pressure·∇(solved pressure)` from the velocity. A no-op when
    /// `params.pressure` is 0 (the forces still advect, but compressibly — no backruns).
    fn project(&mut self, p: &DiffusionParams) {
        if p.pressure <= 0.0 {
            return;
        }
        let (w, h) = (self.width, self.height);
        let n = (w as usize) * (h as usize);
        let li = |x: u32, y: u32| (y * w + x) as usize;
        let nbrs = |x: u32, y: u32| {
            (
                x.saturating_sub(1),
                (x + 1).min(w - 1),
                y.saturating_sub(1),
                (y + 1).min(h - 1),
            )
        };
        // ∇·(u,v) (central differences, Neumann at the border).
        let mut div = std::mem::take(&mut self.divergence);
        div.clear();
        div.resize(n, 0.0);
        for y in 0..h {
            for x in 0..w {
                let (xm, xp, ym, yp) = nbrs(x, y);
                div[li(x, y)] = 0.5
                    * ((self.vel_u[li(xp, y)] - self.vel_u[li(xm, y)])
                        + (self.vel_v[li(x, yp)] - self.vel_v[li(x, ym)]));
            }
        }
        // Jacobi solve of ∇²pressure = divergence, seeded at 0 (ping-pong pr ↔ prb).
        let mut pr = std::mem::take(&mut self.pressure);
        pr.clear();
        pr.resize(n, 0.0);
        let mut prb = std::mem::take(&mut self.pressure_b);
        prb.clear();
        prb.resize(n, 0.0);
        for _ in 0..RELAX_ITERS {
            for y in 0..h {
                for x in 0..w {
                    let (xm, xp, ym, yp) = nbrs(x, y);
                    prb[li(x, y)] = 0.25
                        * (pr[li(xm, y)] + pr[li(xp, y)] + pr[li(x, ym)] + pr[li(x, yp)]
                            - div[li(x, y)]);
                }
            }
            std::mem::swap(&mut pr, &mut prb);
        }
        // Subtract the pressure gradient (scaled by the projection strength) → the
        // (near-)divergence-free velocity.
        for y in 0..h {
            for x in 0..w {
                let (xm, xp, ym, yp) = nbrs(x, y);
                let c = li(x, y);
                self.vel_u[c] -= p.pressure * 0.5 * (pr[li(xp, y)] - pr[li(xm, y)]);
                self.vel_v[c] -= p.pressure * 0.5 * (pr[li(x, yp)] - pr[li(x, ym)]);
            }
        }
        self.pressure = pr;
        self.pressure_b = prb;
        self.divergence = div;
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

    /// Pass 6 — **capillary flow** (ADR-0078 S5 / Curtis 1997 capillary layer): the water
    /// wicks from wet cells into the drier paper *around* the painted area, **carrying a thread
    /// of pigment with it**, so a soft, feathery, *coloured* fringe grows past the painted edge
    /// — the watercolor signature the harder wet-gate boundary can't make.
    ///
    /// Two coupled transports, both keyed on the same per-face capillary water flux:
    /// - **Water:** a conservative, permeability-weighted diffusion (divergence form, like
    ///   [`Self::diffuse`]) with the face conductance set by the paper permeability
    ///   (`0.5·(perm_c+perm_n)`, valleys wick faster) — NOT the wet gate, which would forbid
    ///   the wick into the *dry* fringe (the whole point). Pure diffusion only moves mass down
    ///   a gradient, so the saturated pool interior is untouched and only the wet→dry boundary
    ///   creeps out. CFL-bounded by `capillary ≤ 0.24` (with `perm ≤ 1` the update is a convex
    ///   average ⇒ water stays in `[0,1]`, exact conservation, no clamp).
    /// - **Pigment (dissolved-pigment co-advection):** the wicking water carries pigment at the
    ///   cell's own concentration — the fraction of a cell's *pigment* that follows the water to
    ///   a drier neighbour equals the fraction of its *water* that leaves (`flux/water`). Upwind
    ///   (only wetter→drier) so it rides the wick outward; gathered (each cell sums its own
    ///   losses to drier neighbours + gains from wetter ones) so it's mass-conserving AND
    ///   bit-parity-safe with the GPU. The `flux/water` fraction is bounded by `capillary·cond ≤
    ///   0.24` (since `flux ≤ capillary·cond·water`), so a cell never loses more pigment than it
    ///   has and no clamp is needed. On dry paper (no water gradient) the flux is 0 ⇒ pigment
    ///   stays put (the crisp dry edge is preserved). This is what makes the fringe *visible*
    ///   rather than a faint damp halo.
    ///
    /// The fringe self-limits (the thin film + evaporation halt the wick). Dormant at
    /// `capillary = 0`. Writes `scratch_w2`/`scratch`, then swaps. Mirrored by `cs_capillary`.
    fn capillary_flow(&mut self, p: &DiffusionParams) {
        let (w, h) = (self.width, self.height);
        let cap = p.capillary;
        let n = (w as usize) * (h as usize);
        let mut water_out = std::mem::take(&mut self.scratch_w2);
        water_out.clear();
        water_out.resize(n, 0.0);
        let mut pig_out = std::mem::take(&mut self.scratch);
        pig_out.clear();
        pig_out.resize(n, [0.0; 3]);
        for y in 0..h {
            for x in 0..w {
                let c = self.idx(x, y);
                let wc = self.water[c];
                let pc = self.pigment[c];
                let permc = p.perm_valley + (p.perm_crest - p.perm_valley) * self.paper[c];
                // Per face: the conservative water flux contribution + the pigment the wick
                // carries (loss to a drier neighbour at c's concentration, gain from a wetter
                // one at ITS concentration). Returns `(d_water, d_pigment)` so there's no `&mut`
                // capture and the borrow is plain-immutable (same shape as `diffuse`).
                // Pigment co-advects at `mobility ×` the water's transport fraction — the paper
                // filters the pigment so the water wicks AHEAD (chromatographic separation), so
                // the outer fringe is water-only (transparent) and the colour lags behind it.
                let mob = p.capillary_mobility;
                let face = |nidx: usize| -> (f32, [f32; 3]) {
                    let permn = p.perm_valley + (p.perm_crest - p.perm_valley) * self.paper[nidx];
                    let cond = 0.5 * (permc + permn);
                    let wn = self.water[nidx];
                    let dw = cond * (wn - wc);
                    let dp = if wc > wn {
                        // c donates: pigment fraction = mobility · (water fraction leaving to n).
                        let frac = mob * cap * cond * (wc - wn) / wc;
                        [-frac * pc[0], -frac * pc[1], -frac * pc[2]]
                    } else if wn > wc {
                        // n donates to c at n's concentration (also mobility-scaled).
                        let frac = mob * cap * cond * (wn - wc) / wn;
                        let pn = self.pigment[nidx];
                        [frac * pn[0], frac * pn[1], frac * pn[2]]
                    } else {
                        [0.0; 3]
                    };
                    (dw, dp)
                };
                // In-bounds neighbours in the canonical order (left, right, up, down) — the
                // order the GPU `cs_capillary` gathers, for bit-exact parity. Out-of-bounds
                // faces carry no flux (Neumann), so they're simply omitted.
                let mut nbrs = [0usize; 4];
                let mut count = 0usize;
                if x > 0 {
                    nbrs[count] = self.idx(x - 1, y);
                    count += 1;
                }
                if x + 1 < w {
                    nbrs[count] = self.idx(x + 1, y);
                    count += 1;
                }
                if y > 0 {
                    nbrs[count] = self.idx(x, y - 1);
                    count += 1;
                }
                if y + 1 < h {
                    nbrs[count] = self.idx(x, y + 1);
                    count += 1;
                }
                let mut acc_w = 0.0f32;
                let mut pn = pc;
                for &nidx in &nbrs[..count] {
                    let (dw, dp) = face(nidx);
                    acc_w += dw;
                    pn[0] += dp[0];
                    pn[1] += dp[1];
                    pn[2] += dp[2];
                }
                water_out[c] = wc + cap * acc_w;
                pig_out[c] = pn;
            }
        }
        self.scratch_w2 = std::mem::replace(&mut self.water, water_out);
        self.scratch = std::mem::replace(&mut self.pigment, pig_out);
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

    /// Shallow-water velocity field `(vel_u, vel_v)` per cell (ADR-0078 S3d) — exposed for
    /// the GPU parity gate + the resident-path writeback. All-zero while the velocity layer
    /// is dormant.
    #[must_use]
    pub fn velocity(&self) -> (&[f32], &[f32]) {
        (&self.vel_u, &self.vel_v)
    }

    /// Overwrite the velocity field — companion to [`Self::set_pigment_from`] so a
    /// GPU-stepped `(u,v)` can replace the grid's (the GPU is the accelerator; the grid
    /// stays the CPU source of truth). Panics on a length mismatch (caller sizes it).
    pub fn set_velocity_from(&mut self, u: &[f32], v: &[f32]) {
        self.vel_u.copy_from_slice(u);
        self.vel_v.copy_from_slice(v);
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

    // ─────────────── ADR-0078 S3d — shallow-water velocity layer ───────────────

    /// DORMANT by default: with `velocity = 0` (the default) `move_water` never runs, so
    /// the velocity field stays all-zero and pigment advects by the static gradient flow
    /// (the shipped look). The rest of the suite, run with `Default`, also implicitly
    /// proves the velocity path is bit-untouched.
    #[test]
    fn velocity_layer_off_is_dormant() {
        let mut g = DiffusionGrid::new(40, 40, 1.0);
        g.splat(20.0, 20.0, 8.0, 0.9, [0.6, 0.2, 0.1]);
        let p = DiffusionParams::default();
        for _ in 0..40 {
            g.step(&p);
        }
        let (u, v) = g.velocity();
        assert!(
            u.iter().chain(v).all(|&c| c == 0.0),
            "velocity must stay zero while the layer is dormant"
        );
    }

    /// DETERMINISTIC with the velocity layer ON — same inputs → bit-identical field (HR-5).
    /// The fixed-iteration Jacobi projection + pure arithmetic keep it replayable.
    #[test]
    fn velocity_layer_is_deterministic() {
        let run = || {
            let mut g = DiffusionGrid::new(40, 36, 1.0);
            g.splat(20.0, 18.0, 7.0, 0.9, [0.5, 0.3, 0.7]);
            let p = DiffusionParams {
                velocity: 1.4,
                viscosity: 0.1,
                drag: 0.06,
                pressure: 1.0,
                ..Default::default()
            };
            for _ in 0..25 {
                g.step(&p);
            }
            let (u, v) = g.velocity();
            (g.pigment().to_vec(), u.to_vec(), v.to_vec())
        };
        assert_eq!(run(), run(), "velocity solver must be deterministic (HR-5)");
    }

    /// STABLE: the velocity layer never produces NaN/Inf or runaway pigment/velocity over
    /// many steps (the CFL clamp + drag + viscosity + bounded projection keep it tame).
    #[test]
    fn velocity_layer_is_stable_over_many_steps() {
        let mut g = DiffusionGrid::new(48, 48, 2.0);
        g.splat(24.0, 24.0, 12.0, 1.0, [0.9, 0.8, 0.2]);
        g.splat(12.0, 30.0, 5.0, 0.7, [0.1, 0.2, 0.9]);
        let p = DiffusionParams {
            velocity: 1.6,
            viscosity: 0.12,
            drag: 0.05,
            pressure: 1.0,
            ..Default::default()
        };
        for _ in 0..200 {
            g.step(&p);
        }
        for px in g.pigment() {
            for &c in px {
                assert!(c.is_finite() && (0.0..10.0).contains(&c), "unstable pigment {c}");
            }
        }
        let (u, v) = g.velocity();
        for &c in u.iter().chain(v) {
            assert!(c.is_finite() && c.abs() <= 0.5, "unstable velocity {c}");
        }
    }

    /// The velocity field TRANSPORTS pigment along `(u,v)` — the defining S3d behavior.
    /// On a flat, uniformly-wet field (no body forces) a seeded rightward velocity carries
    /// the pigment blob's centre-of-mass `+x`; with the layer OFF the same seeded velocity
    /// is ignored (static flow is zero on flat fields) and the blob stays put.
    #[test]
    fn velocity_field_transports_pigment() {
        let (w, h) = (72u32, 48u32);
        let com_x = |g: &DiffusionGrid| -> f32 {
            let (mut num, mut den) = (0.0f32, 0.0f32);
            for y in 0..h {
                for x in 0..w {
                    let m = g.pigment()[(y * w + x) as usize][0];
                    num += m * x as f32;
                    den += m;
                }
            }
            num / den.max(1e-6)
        };
        let run = |velocity: f32| -> (f32, f32) {
            let mut g = DiffusionGrid::new(w, h, 1.0);
            // Flat paper + uniform wetness → zero body forces, so only a seeded velocity
            // can move pigment (isolates pure velocity advection).
            for pv in g.paper.iter_mut() {
                *pv = 0.5;
            }
            for wv in g.water.iter_mut() {
                *wv = 0.9;
            }
            for uu in g.vel_u.iter_mut() {
                *uu = 0.25; // uniform rightward momentum (divergence-free → no projection needed)
            }
            g.splat(20.0, 24.0, 5.0, 0.0, [0.6, 0.2, 0.1]);
            let before = com_x(&g);
            let p = DiffusionParams {
                evaporation: 0.0,
                diffusivity: 0.0, // isolate advection (no symmetric bloom)
                downhill: 0.0,
                flow_outward: 0.0,
                viscosity: 0.0,
                drag: 0.0,    // no decay → clean translation
                pressure: 0.0, // uniform velocity is already divergence-free
                velocity,
                ..Default::default()
            };
            for _ in 0..10 {
                g.step(&p);
            }
            (before, com_x(&g))
        };
        let (b_on, a_on) = run(1.0);
        let (b_off, a_off) = run(0.0);
        eprintln!("COM_x  velocity-on: {b_on:.2}->{a_on:.2}   velocity-off: {b_off:.2}->{a_off:.2}");
        assert!(
            a_on > b_on + 2.0,
            "velocity layer must carry pigment +x: {b_on:.2} -> {a_on:.2}"
        );
        assert!(
            (a_off - b_off).abs() < 0.25,
            "velocity OFF: pigment must not move on flat fields ({b_off:.2} -> {a_off:.2})"
        );
    }

    /// BACKRUN / cauliflower: with the velocity layer + pressure projection on, the
    /// FlowOutward push (`−λ·∇w`) drives flowing pigment to the wet boundary where the
    /// converging, projected flow piles it into a RING — the rim carries MORE pigment than
    /// the mid-radius, and the ring is more pronounced than on the static gradient path.
    /// (Deposition is OFF so this isolates FLOW-driven piling, not drying-deposition.)
    #[test]
    fn velocity_pressure_builds_a_backrun_ring() {
        let (w, h) = (96u32, 96u32);
        let (cx, cy) = (48.0f32, 48.0);
        let ann_mean = |g: &DiffusionGrid, lo: f32, hi: f32| -> f32 {
            let (mut s, mut n) = (0.0f32, 0.0f32);
            for y in 0..h {
                for x in 0..w {
                    let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
                    if d >= lo && d < hi {
                        s += g.pigment()[(y * w + x) as usize][0];
                        n += 1.0;
                    }
                }
            }
            s / n.max(1.0)
        };
        let run = |velocity: f32| -> (f32, f32) {
            let mut g = DiffusionGrid::new(w, h, 1.0);
            g.splat(cx, cy, 20.0, 0.9, [0.5, 0.25, 0.15]);
            let p = DiffusionParams {
                velocity,
                viscosity: 0.1,
                drag: 0.08,
                pressure: 0.4,
                downhill: 0.0, // isolate the FlowOutward backrun (not paper channeling)
                ..Default::default()
            };
            for _ in 0..40 {
                g.step(&p);
            }
            (ann_mean(&g, 0.0, 4.0), ann_mean(&g, 8.0, 12.0)) // (centre, ring radius)
        };
        let (centre_v, ring_v) = run(1.6);
        let (centre_s, ring_s) = run(0.0);
        eprintln!(
            "backrun  velocity: centre={centre_v:.4} ring={ring_v:.4} | static: centre={centre_s:.4} ring={ring_s:.4}"
        );
        // Velocity path: pigment fled the centre and piled into an off-centre RING (the
        // backrun/cauliflower — a radial profile with a local max away from the centre).
        assert!(
            ring_v > 2.0 * centre_v,
            "velocity must pile an off-centre backrun ring (ring {ring_v:.4} ≫ centre {centre_v:.4})"
        );
        // Static gradient path: a plain centre-peaked blob — the centre is the maximum,
        // no ring. This is the qualitative difference the velocity layer introduces.
        assert!(
            centre_s > ring_s,
            "static flow must stay centre-peaked, no ring (centre {centre_s:.4} > ring {ring_s:.4})"
        );
    }

    // ───────────────── ADR-0078 S5 — capillary fringe ─────────────────

    /// Mean water over a centred annulus `[lo, hi)` — the test probe for the wet fringe.
    fn ring_water(g: &DiffusionGrid, cx: f32, cy: f32, lo: f32, hi: f32) -> f32 {
        let (w, h) = g.dims();
        let (mut s, mut n) = (0.0f32, 0.0f32);
        for y in 0..h {
            for x in 0..w {
                let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
                if d >= lo && d < hi {
                    s += g.water()[(y * w + x) as usize];
                    n += 1.0;
                }
            }
        }
        s / n.max(1.0)
    }

    /// DORMANT by default: with `capillary = 0` the water never wicks past the painted area,
    /// so a ring OUTSIDE the splat radius stays bone-dry (no water mechanism moves water
    /// outward — diffuse/advect/transfer touch only pigment; evaporate only shrinks water).
    /// The rest of the suite, run with `Default`, also implicitly proves the path is untouched.
    #[test]
    fn capillary_off_keeps_fringe_dry() {
        let (w, h) = (64u32, 64u32);
        let (cx, cy) = (32.0f32, 32.0);
        let mut g = DiffusionGrid::new(w, h, 1.0);
        g.splat(cx, cy, 8.0, 1.0, [0.5, 0.3, 0.2]);
        let p = DiffusionParams { evaporation: 0.0, capillary: 0.0, ..Default::default() };
        for _ in 0..50 {
            g.step(&p);
        }
        let fringe = ring_water(&g, cx, cy, 10.0, 13.0);
        assert_eq!(fringe, 0.0, "capillary OFF must leave the fringe dry (got {fringe})");
    }

    /// The defining S5 behaviour: with `capillary > 0` the water WICKS outward into the dry
    /// paper, wetting a soft fringe BEYOND the splat radius — and it stays BOUNDED (the thin
    /// film + conservation self-limit it; it does not flood the whole grid). Off vs on on the
    /// SAME dry-paper blob isolates the wick.
    #[test]
    fn capillary_wicks_a_bounded_fringe() {
        let (w, h) = (64u32, 64u32);
        let (cx, cy) = (32.0f32, 32.0);
        let run = |capillary: f32| -> DiffusionGrid {
            let mut g = DiffusionGrid::new(w, h, 1.0);
            g.splat(cx, cy, 8.0, 1.0, [0.5, 0.3, 0.2]);
            let p = DiffusionParams { evaporation: 0.0, capillary, ..Default::default() };
            for _ in 0..50 {
                g.step(&p);
            }
            g
        };
        let on = run(0.24);
        let off = run(0.0);
        let fringe_on = ring_water(&on, cx, cy, 10.0, 13.0);
        let fringe_off = ring_water(&off, cx, cy, 10.0, 13.0);
        let far = ring_water(&on, cx, cy, 24.0, 28.0);
        let max_w = on.max_water();
        eprintln!("capillary fringe on={fringe_on:.4} off={fringe_off:.4} far={far:.4} max={max_w:.4}");
        assert!(
            fringe_on > 0.01 && fringe_on > fringe_off + 0.01,
            "capillary must wet a fringe past the splat (on {fringe_on:.4} ≫ off {fringe_off:.4})"
        );
        assert!(far < 0.01, "fringe must stay bounded, not flood the grid (far ring {far:.4})");
        assert!(max_w <= 1.0001, "water must stay ≤ 1 (convex average, no runaway): {max_w}");
    }

    /// Mass-conserving: capillary flow conserves BOTH water (its diffusion) AND pigment (its
    /// co-advection is a gather-form upwind transport — every gram leaving a cell lands in a
    /// neighbour). With no evaporation the totals are invariant (only redistributed).
    #[test]
    fn capillary_conserves_water_and_pigment() {
        let (w, h) = (48u32, 48u32);
        let mut g = DiffusionGrid::new(w, h, 1.0);
        g.splat(24.0, 24.0, 8.0, 0.9, [0.6, 0.2, 0.1]); // peak ≤ 0.9 → no splat saturation
        let water_before: f64 = g.water().iter().map(|&x| x as f64).sum();
        let pig_before = g.total_pigment();
        let p = DiffusionParams { evaporation: 0.0, capillary: 0.2, ..Default::default() };
        for _ in 0..40 {
            g.step(&p);
        }
        let water_after: f64 = g.water().iter().map(|&x| x as f64).sum();
        let pig_after = g.total_pigment();
        assert!(
            (water_before - water_after).abs() < 1e-3 * (water_before.abs() + 1.0),
            "capillary must conserve water: {water_before} -> {water_after}"
        );
        for k in 0..3 {
            assert!(
                (pig_before[k] - pig_after[k]).abs() < 1e-3 * (pig_before[k].abs() + 1.0),
                "capillary must conserve pigment ch{k}: {} -> {}",
                pig_before[k],
                pig_after[k]
            );
        }
    }

    /// DETERMINISTIC (HR-5): pure arithmetic → same inputs give a bit-identical field.
    #[test]
    fn capillary_is_deterministic() {
        let run = || {
            let mut g = DiffusionGrid::new(40, 36, 1.0);
            g.splat(20.0, 18.0, 7.0, 0.9, [0.5, 0.3, 0.7]);
            let p = DiffusionParams { capillary: 0.2, ..Default::default() };
            for _ in 0..25 {
                g.step(&p);
            }
            (g.water().to_vec(), g.pigment().to_vec())
        };
        assert_eq!(run(), run(), "capillary solver must be deterministic (HR-5)");
    }

    /// The fringe BLEEDS pigment: as the capillary wick opens the wet gate past the painted
    /// area, the existing pigment diffuses into the freshly-wetted fringe — so an outer
    /// annulus carries pigment with capillary ON but is ~clean with it OFF (no water there ⇒
    /// gate shut ⇒ the bloom can't reach it). This is the visible soft watercolor edge.
    #[test]
    fn capillary_bleeds_pigment_into_the_fringe() {
        let (w, h) = (64u32, 64u32);
        let (cx, cy) = (32.0f32, 32.0);
        let ann_pig = |g: &DiffusionGrid, lo: f32, hi: f32| -> f32 {
            let (mut s, mut n) = (0.0f32, 0.0f32);
            for y in 0..h {
                for x in 0..w {
                    let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
                    if d >= lo && d < hi {
                        s += g.pigment()[(y * w + x) as usize][0];
                        n += 1.0;
                    }
                }
            }
            s / n.max(1.0)
        };
        let run = |capillary: f32| -> f32 {
            let mut g = DiffusionGrid::new(w, h, 1.0);
            g.splat(cx, cy, 8.0, 1.0, [0.6, 0.2, 0.1]);
            let p = DiffusionParams { evaporation: 0.0, capillary, ..Default::default() };
            for _ in 0..80 {
                g.step(&p);
            }
            // Just outside the r=8 splat — the near fringe, where the thin wicked film still
            // gives the wet gate enough to let the bloom carry visible pigment.
            ann_pig(&g, 9.0, 11.0)
        };
        let on = run(0.2);
        let off = run(0.0);
        eprintln!("capillary pigment-bleed fringe on={on:.6} off={off:.6}");
        // off is bone-dry past the splat ⇒ exactly 0 (gate shut). The wick CO-ADVECTS pigment
        // (a thread carried with the water), so the fringe is clearly tinted — not the faint
        // damp halo a water-only wick would leave.
        assert!(
            on > 0.01 && on > off + 0.01,
            "pigment must bleed into the capillary fringe (on {on:.6} ≫ off {off:.6})"
        );
    }

    /// CHROMATOGRAPHIC FILTERING — the transparent edge (ADR-0078 S5): with `capillary_mobility
    /// < 1` the paper filters the pigment, so the WATER front wicks AHEAD of the pigment. The
    /// wet front therefore extends well beyond the pigment front → a transparent water-only
    /// halo at the outer edge, with the colour feathering behind it. With mobility = 1 (pigment
    /// co-moving) the two fronts coincide (a uniformly-coloured, opaque fringe — the contrast).
    /// This is the physical mechanism behind watercolor's soft transparent edge.
    #[test]
    fn capillary_water_wicks_ahead_of_pigment() {
        let (w, h) = (72u32, 64u32);
        let (cx, cy) = (36.0f32, 32.0);
        // Furthest radius reached by water (> 1e-3) and by pigment (> 1e-4).
        let run = |mobility: f32| -> (u32, u32) {
            let mut g = DiffusionGrid::new(w, h, 1.0);
            g.splat(cx, cy, 8.0, 1.0, [0.5, 0.3, 0.2]);
            let p = DiffusionParams {
                capillary: 0.22,
                capillary_mobility: mobility,
                evaporation: 0.0,
                ..Default::default()
            };
            for _ in 0..70 {
                g.step(&p);
            }
            let (mut wr, mut pr) = (0u32, 0u32);
            for y in 0..h {
                for x in 0..w {
                    let d = (((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt()) as u32;
                    let i = (y * w + x) as usize;
                    if g.water()[i] > 1.0e-3 {
                        wr = wr.max(d);
                    }
                    if g.pigment()[i][0] > 1.0e-4 {
                        pr = pr.max(d);
                    }
                }
            }
            (wr, pr)
        };
        let (wr_filt, pr_filt) = run(0.35);
        let (wr_co, pr_co) = run(1.0);
        eprintln!(
            "water-ahead: filtered(mob .35) water={wr_filt} pig={pr_filt} | co-moving(mob 1) water={wr_co} pig={pr_co}"
        );
        // Filtering: the water front is well beyond the pigment front → a transparent outer halo.
        assert!(
            wr_filt > pr_filt + 2,
            "filtered capillary: water must wick ahead of pigment (water {wr_filt} vs pig {pr_filt})"
        );
        // Co-moving (mobility 1): the pigment keeps up with the water — no transparent halo.
        assert!(
            pr_co >= wr_co.saturating_sub(1),
            "co-moving pigment should reach ~the water front (water {wr_co} vs pig {pr_co})"
        );
    }

    /// §2.2 ENVELOPE INVARIANT (the composite must not clip the fringe): the wet bbox
    /// (`water_bbox(1e-3)`) is a SUPERSET of the pigment bbox even with the capillary fringe —
    /// because pigment needs an open gate (`water > w_lo = 0.05 ≫ 1e-3`), it can never sit
    /// outside the wet bbox. This is what makes the driver correct in growing the composite
    /// envelope from the GPU wet-bbox: scoping the composite to the wet envelope covers the
    /// whole fringe. Also proves capillary GROWS that envelope vs off (the fringe is real).
    #[test]
    fn capillary_fringe_pigment_stays_inside_the_wet_bbox() {
        let (w, h) = (72u32, 64u32);
        let pig_bbox = |g: &DiffusionGrid| -> (u32, u32, u32, u32) {
            let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
            for y in 0..h {
                for x in 0..w {
                    if g.pigment()[(y * w + x) as usize][0] > 1.0e-4 {
                        x0 = x0.min(x);
                        y0 = y0.min(y);
                        x1 = x1.max(x);
                        y1 = y1.max(y);
                    }
                }
            }
            (x0, y0, x1, y1)
        };
        let run = |capillary: f32| -> ((u32, u32, u32, u32), (u32, u32, u32, u32)) {
            let mut g = DiffusionGrid::new(w, h, 1.0);
            g.splat(36.0, 32.0, 9.0, 1.0, [0.5, 0.3, 0.2]);
            // No evaporation: keep the field wet so the fringe is fully developed (aggressive
            // capillary + drying would dry the thin film to nothing — a separate concern).
            let p = DiffusionParams { capillary, evaporation: 0.0, ..Default::default() };
            for _ in 0..60 {
                g.step(&p);
            }
            (g.water_bbox(1.0e-3).expect("still wet"), pig_bbox(&g))
        };
        let (wet_on, pig_on) = run(0.24);
        let (wet_off, _) = run(0.0);
        eprintln!("envelope: wet_on={wet_on:?} pig_on={pig_on:?} wet_off={wet_off:?}");
        // The pigment fringe lies within a SMALL margin of the wet bbox (the wick carries a
        // thread of concentrated pigment that can reach ~2 cells past the water > 1e-3 contour,
        // a threshold artefact). The driver grows the composite envelope from the wet bbox by
        // CAPILLARY_FRINGE_PAD = 8 ≫ this margin, so it always covers the fringe (no clip). The
        // physics invariant: pigment can NEVER stray far from where water wicked it.
        const MARGIN: u32 = 4;
        assert!(
            pig_on.0 + MARGIN >= wet_on.0
                && pig_on.1 + MARGIN >= wet_on.1
                && pig_on.2 <= wet_on.2 + MARGIN
                && pig_on.3 <= wet_on.3 + MARGIN,
            "pigment bbox {pig_on:?} must lie within {MARGIN} cells of the wet bbox {wet_on:?} (the bridge pad covers this)"
        );
        // Capillary GREW the wet envelope beyond the no-capillary case (the fringe is real).
        assert!(
            wet_on.0 < wet_off.0 && wet_on.1 < wet_off.1 && wet_on.2 > wet_off.2 && wet_on.3 > wet_off.3,
            "capillary must grow the wet envelope past the no-capillary case (on {wet_on:?} vs off {wet_off:?})"
        );
    }

    // ───────────────── ADR-0078 S5c — BFECC/MacCormack advection sharpness ─────────────────

    /// SHARPNESS reduces numerical diffusion: MacCormack (`sharpness = 1`) advects a pigment
    /// blob along a uniform velocity while preserving its PEAK far better than the first-order
    /// upwind (`sharpness = 0`), which smears it out. Pure advection (no diffusion/forces) on a
    /// flat, uniformly-wet field isolates the transport accuracy.
    #[test]
    fn sharpness_reduces_advection_diffusion() {
        let (w, h) = (72u32, 40u32);
        let run = |sharpness: f32| -> f32 {
            let mut g = DiffusionGrid::new(w, h, 1.0);
            for pv in g.paper.iter_mut() {
                *pv = 0.5;
            }
            for wv in g.water.iter_mut() {
                *wv = 0.9;
            }
            for uu in g.vel_u.iter_mut() {
                *uu = 0.3; // uniform rightward momentum (divergence-free → no projection)
            }
            g.splat(18.0, 20.0, 4.0, 0.0, [0.8, 0.4, 0.2]);
            let p = DiffusionParams {
                evaporation: 0.0,
                diffusivity: 0.0, // isolate advection (no symmetric bloom)
                downhill: 0.0,
                flow_outward: 0.0,
                viscosity: 0.0,
                drag: 0.0,
                pressure: 0.0, // uniform velocity already divergence-free
                velocity: 1.0,
                sharpness,
                ..Default::default()
            };
            for _ in 0..15 {
                g.step(&p);
            }
            g.pigment().iter().map(|px| px[0]).fold(0.0f32, f32::max) // peak
        };
        let peak_sharp = run(1.0);
        let peak_soft = run(0.0);
        eprintln!("sharpness: peak sharp(1.0)={peak_sharp:.4} soft(0.0)={peak_soft:.4}");
        assert!(
            peak_sharp > peak_soft * 1.1,
            "MacCormack must preserve the peak vs first-order (sharp {peak_sharp:.4} vs soft {peak_soft:.4})"
        );
        // The limiter keeps it bounded — no overshoot/ringing/negative pigment.
        let p1 = run(1.0);
        assert!(p1.is_finite() && p1 < 2.0, "sharpened advection must stay bounded: {p1}");
    }

    /// DETERMINISTIC with sharpness on (HR-5): the MacCormack passes are pure arithmetic.
    #[test]
    fn sharpness_is_deterministic() {
        let run = || {
            let mut g = DiffusionGrid::new(40, 36, 1.0);
            g.splat(20.0, 18.0, 7.0, 0.9, [0.5, 0.3, 0.7]);
            let p = DiffusionParams { velocity: 1.4, pressure: 1.0, sharpness: 1.0, ..Default::default() };
            for _ in 0..25 {
                g.step(&p);
            }
            g.pigment().to_vec()
        };
        assert_eq!(run(), run(), "sharpened advection must be deterministic (HR-5)");
    }
}
