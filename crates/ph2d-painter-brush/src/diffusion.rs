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

use crate::pigment_mix::{SPECTRAL_BANDS, ks_field_color, prepare_pigment};

/// Spectral K/S bands carried per wet-field cell (ADR-0080) — pinned to the optical
/// model's [`SPECTRAL_BANDS`] (24); the field and its GPU mirror loop over this many.
pub const PIG_BANDS: usize = SPECTRAL_BANDS;
/// Index of the first `err` channel (the round-trip re-anchor `err[3]`, ADR-0080).
pub const PIG_ERR0: usize = PIG_BANDS;
/// Index of the coverage `mass` channel.
pub const PIG_MASS: usize = PIG_BANDS + 3;
/// Index of the mass-weighted **staining** accumulator (ADR-0081): `stain_acc = Σ mass_i·stain_i`,
/// `stain_acc/mass` = the staining of the pigment in the cell ∈ [0,1] (1 = permanent stain that
/// resists lifting, 0 = liftable). Transports linearly like `ks`/`err` (survives mixing); read
/// by the opt-in lift pass. 0 for raw colours (no pigment selected) → lift dormant.
pub const PIG_STAIN: usize = PIG_BANDS + 4;
/// Channels per wet-field cell: `ks[24]` (mass-weighted Kubelka–Munk per band, ADR-0080),
/// `err[3]` (round-trip re-anchor), `mass` (coverage), `stain` (ADR-0081), and 3 reserved/pad —
/// **32 = 8·4** (clean std430 vec4 packing for the GPU mirror). All transport LINEARLY and
/// identically under diffuse/advect/transfer/capillary, so the subtractive multi-pigment mix +
/// per-pigment staining emerge from the transport itself; the per-cell reduction
/// ([`DiffusionGrid::cell_color`]) ignores `stain`/pad (they're behaviour, not colour), so the
/// composited colour is unchanged. A single pigment reduces to exactly its own colour.
pub const PIG_CH: usize = PIG_BANDS + 8;
/// One wet-field cell's pigment channels (ADR-0080/0081).
pub type WetCell = [f32; PIG_CH];

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
    /// **Lift (ADR-0081)** ∈ [0,1] — rate at which WET cells re-mobilize NON-staining deposited
    /// pigment back into the flowing layer (re-wetting "lifts" dried paint; staining pigments
    /// resist). `0` ⇒ the lift pass is dormant (deposited stays frozen — the pre-ADR-0081 path is
    /// bit-identical). Read by [`DiffusionGrid::lift_pigment`].
    pub lift: f32,
    /// **Branched (fiber-channeled) capillary fringe (ADR-0082)** ∈ [0,1] — opt-in, non-destructive.
    /// CREST-GATES the capillary per-face conductance by the paper FIBRE on that face
    /// (`fiber_factor = 1 − branching·(1 − smoothstep(BRANCH_GATE_LO, BRANCH_GATE_HI, paper_face))`):
    /// crest faces keep FULL conductance while valley faces close completely at `branching = 1`,
    /// so the wick percolates along the crest network → the fringe goes lobed/dendritic (ramified)
    /// instead of a smooth ring, the watercolor fibre-to-fibre look.
    /// Suppression-only (≤ 1, never a boost) preserves the convex-average stability + conservation
    /// of [`DiffusionGrid::capillary_flow`]. `0` ⇒ `fiber_factor = 1` ⇒ the isotropic capillary is
    /// **bit-identical** to today (opt-in). Only read while the capillary layer is active.
    pub capillary_branching: f32,
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
            // Lift OFF by default (ADR-0081) → the lift pass is dormant, deposited stays frozen.
            lift: 0.0,
            // Branched capillary OFF by default (ADR-0082) → opt-in; 0 = isotropic capillary
            // bit-identical (fiber_factor = 1, the smooth ring).
            capillary_branching: 0.0,
        }
    }
}

/// Frequency of the paper-height field sampled into the grid (cycles per grid
/// cell) — coarse enough that valleys span a few cells so pigment can pool.
const HEIGHT_FREQ: f32 = 0.13;
const HEIGHT_SEED: u32 = 0x70a9_e2c5; // shared with cpu_render paper tooth

/// **Branching crest-gate band (ADR-0082, re-tuned 2026-06-09 after Enio's A/B smoke).** The
/// fiber-channeled capillary suppression is
/// `fiber_factor = 1 − branching·(1 − smoothstep(LO, HI, paper_face))`.
/// The earlier linear gain (`1 − b·2·(1−paper_face)`) suppressed EVERYTHING — at max branching
/// even the crests lost ~40% conductance, so the whole fringe just shrank slightly + waved
/// (Enio: "a diferença é muito discreta"). A THRESHOLD gate maximizes the contrast that makes
/// fingers: faces on crests (`paper_face ≥ HI`) keep **full** conductance (the fingers grow at
/// full wick speed) while valley faces (`≤ LO`) close **completely** at branching = 1 — a
/// percolation-style channel network instead of a uniform slowdown. Still **suppression-only**
/// (`gate ∈ [0,1] ⇒ fiber_factor ∈ [1−b, 1] ⊆ [0,1]`) ⇒ the convex-average CFL bound + mass
/// conservation hold (ADR-0082 §2.2 proof unchanged). `branching = 0 ⇒ fiber_factor = 1`
/// bit-identical to the isotropic ring. The band brackets the paper-tooth median (~0.5); its
/// tight width (0.2) is what makes the channels sharp.
const BRANCH_GATE_LO: f32 = 0.40;
const BRANCH_GATE_HI: f32 = 0.60;

/// **Backdrop-lift bleed-keep fraction (ADR-0084, tuned 2026-06-09).** Of the dry paint a wet brush
/// lifts off the canvas, only this fraction bleeds into the active wash — the rest is REMOVED (the
/// brush carries it away). Lifting must dilute/lighten the spot, but merging the dry paint's FULL
/// mass into the wash made the K-M glaze of one combined-mass layer read DARKER/denser than the
/// original thin layering (Enio's smoke 2026-06-09: the lifted stroke went dark/muddy instead of
/// lightening). Keeping only a fraction (+ the `lifted_frac` backdrop-alpha drop that removes the
/// paper pigment) makes the area genuinely lighten, with a little colour bleeding into the wash.
const LIFT_BLEED_KEEP: f32 = 0.25;

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

/// **ADR-0084 (paper-reveal model) — downsample the PAINT on a canvas into `gw·gh` K–M donor
/// cells.** "Paint" = how the current `backdrop` deviates from the session's original `paper`
/// canvas (the Curtis-1997 semantics: lift *desorbs deposited pigment*, revealing the substrate —
/// it must never treat the substrate itself as liftable pigment). Per pixel, the paintedness is
/// `max(|Δr|,|Δg|,|Δb|,|Δa|)` (linear light + straight alpha) — 0 on untouched pixels (bare paper
/// contributes NOTHING: no beige-mud lifting, no lift on unpainted areas), rising with how much
/// pigment the pixel visibly carries. Each grid cell paintedness-weights the backdrop colour
/// (so bare pixels in a half-painted cell don't dilute the paint hue) and gets
/// `mass = avg paintedness`. The single seed used by BOTH the CPU
/// [`DiffusionGrid::seed_lift_source_from_backdrop`] and the GPU upload path, so the backdrop-lift
/// donor is bit-identical CPU↔GPU (HR-5). `backdrop.len() == paper.len() == cw·ch·4`;
/// `paper == backdrop` ⇒ all-zero donor (lift inert).
#[must_use]
pub fn backdrop_to_lift_source(
    backdrop: &[u8],
    paper: &[u8],
    cw: u32,
    ch: u32,
    gw: u32,
    gh: u32,
) -> Vec<WetCell> {
    debug_assert_eq!(backdrop.len(), (cw as usize) * (ch as usize) * 4);
    debug_assert_eq!(paper.len(), backdrop.len());
    let mut out = vec![[0.0f32; PIG_CH]; (gw as usize) * (gh as usize)];
    for gy in 0..gh {
        // Inclusive pixel span of this grid row (box footprint), clamped to the canvas.
        let py0 = (gy * ch / gh).min(ch.saturating_sub(1));
        let py1 = (((gy + 1) * ch).div_ceil(gh)).clamp(py0 + 1, ch);
        for gx in 0..gw {
            let px0 = (gx * cw / gw).min(cw.saturating_sub(1));
            let px1 = (((gx + 1) * cw).div_ceil(gw)).clamp(px0 + 1, cw);
            let (mut sr, mut sg, mut sb) = (0.0f64, 0.0f64, 0.0f64);
            let mut swt = 0.0f64;
            let mut cnt = 0.0f64;
            for py in py0..py1 {
                for px in px0..px1 {
                    let o = ((py * cw + px) * 4) as usize;
                    let br = crate::pigment_mix::srgb8_to_linear(backdrop[o]);
                    let bg = crate::pigment_mix::srgb8_to_linear(backdrop[o + 1]);
                    let bb = crate::pigment_mix::srgb8_to_linear(backdrop[o + 2]);
                    let ba = f32::from(backdrop[o + 3]) / 255.0;
                    let pr = crate::pigment_mix::srgb8_to_linear(paper[o]);
                    let pg = crate::pigment_mix::srgb8_to_linear(paper[o + 1]);
                    let pb = crate::pigment_mix::srgb8_to_linear(paper[o + 2]);
                    let pa = f32::from(paper[o + 3]) / 255.0;
                    let painted = (br - pr)
                        .abs()
                        .max((bg - pg).abs())
                        .max((bb - pb).abs())
                        .max((ba - pa).abs());
                    let wt = f64::from(painted);
                    sr += f64::from(br) * wt;
                    sg += f64::from(bg) * wt;
                    sb += f64::from(bb) * wt;
                    swt += wt;
                    cnt += 1.0;
                }
            }
            if swt > 1.0e-9 && cnt > 0.0 {
                let avg = [(sr / swt) as f32, (sg / swt) as f32, (sb / swt) as f32];
                let mass = (swt / cnt) as f32;
                if mass > 1.0e-6 {
                    out[(gy * gw + gx) as usize] = DiffusionGrid::cell_from_color_mass(avg, mass);
                }
            }
        }
    }
    out
}

/// A low-resolution wet-on-wet diffusion field. Holds water + pigment + a static
/// paper-height map, with ping-pong scratch for the conservative passes. All
/// updates are pure arithmetic → deterministic + replayable (HR-5).
pub struct DiffusionGrid {
    width: u32,
    height: u32,
    /// Wetness ∈ [0,1] per cell.
    water: Vec<f32>,
    /// Pigment per cell — the [`PIG_CH`]-channel mass-weighted K/S accumulation (ADR-0080):
    /// `ks[24]` + `err[3]` + `mass`. Mass-conserving under diffusion/advection (every channel
    /// transports linearly), and the multi-pigment mix emerges from the transport.
    pigment: Vec<WetCell>,
    /// **Deposited pigment** ([`PIG_CH`] channels) per cell — frozen into the paper by
    /// `TransferPigment` (ADR-0078 S3); does NOT diffuse/advect. The composited pigment
    /// is `pigment + deposited`. All-zero while the deposition params are off.
    deposited: Vec<WetCell>,
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
    scratch: Vec<WetCell>,
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
    /// **Backdrop-lift donor field (ADR-0084)** — [`PIG_CH`]-channel "dry paint available to
    /// lift", seeded once per stroke from the canvas backdrop (downsampled) by
    /// [`Self::seed_lift_source_from_backdrop`] when `lift > 0`. A *static* donor (does NOT
    /// diffuse/advect, like `deposited`); [`Self::lift_pigment`] re-mobilizes a fraction into
    /// the flowing `pigment` layer in wet cells, depleting it. All-zero while the lift is off /
    /// unseeded → the backdrop-lift branch is a no-op (pre-ADR-0084 bit-identical).
    lift_source: Vec<WetCell>,
    /// **Lifted fraction ∈ [0,1] per cell (ADR-0084)** — how much of the backdrop pigment has
    /// been re-mobilized out of the paper. The compositor reads it to drop the backdrop alpha
    /// (lifted paint = less opaque). Resident across steps (accumulates); `≡ 0` while lift is
    /// off → the compositor is byte-identical.
    lifted_frac: Vec<f32>,
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
            pigment: vec![[0.0; PIG_CH]; n],
            deposited: vec![[0.0; PIG_CH]; n],
            paper,
            vel_u: vec![0.0; n],
            vel_v: vec![0.0; n],
            pressure: vec![0.0; n],
            scratch: vec![[0.0; PIG_CH]; n],
            scratch_w: vec![0.0; n],
            scratch_u: vec![0.0; n],
            scratch_v: vec![0.0; n],
            divergence: vec![0.0; n],
            pressure_b: vec![0.0; n],
            scratch_w2: vec![0.0; n],
            // ADR-0084 backdrop-lift — dormant (empty) until a stroke with `lift > 0` seeds them.
            lift_source: vec![[0.0; PIG_CH]; n],
            lifted_frac: vec![0.0; n],
        }
    }

    #[inline]
    fn idx(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    /// Build a wet-field cell's [`PIG_CH`] channels for a pigment of `color` (linear sRGB) at
    /// coverage `mass` (ADR-0080): the mass-weighted Kubelka–Munk `ks` + round-trip `err` of
    /// `color`, and the `mass` itself. Two of these added together mix subtractively (the K/S
    /// blends by mass at the per-cell reduction); a single one reduces to exactly `color`.
    #[must_use]
    pub fn cell_from_color_mass(color: [f32; 3], mass: f32) -> WetCell {
        let p = prepare_pigment(color);
        let mut c = [0.0f32; PIG_CH];
        let ks = p.ks();
        for k in 0..PIG_BANDS {
            c[k] = ks[k] * mass;
        }
        let e = p.err();
        c[PIG_ERR0] = e[0] * mass;
        c[PIG_ERR0 + 1] = e[1] * mass;
        c[PIG_ERR0 + 2] = e[2] * mass;
        c[PIG_MASS] = mass;
        c
    }

    /// Coverage `mass` of a wet-field cell (the `dens` analogue of the pre-ADR-0080 gray field).
    #[inline]
    #[must_use]
    pub fn cell_mass(c: &WetCell) -> f32 {
        c[PIG_MASS]
    }

    /// The mixed linear-sRGB colour of a wet-field cell — the K–M reduction of its mass-weighted
    /// K/S accumulation (ADR-0080). A single pigment yields exactly its picked colour.
    #[inline]
    #[must_use]
    pub fn cell_color(c: &WetCell) -> [f32; 3] {
        let ks: [f32; PIG_BANDS] = std::array::from_fn(|i| c[i]);
        let err = [c[PIG_ERR0], c[PIG_ERR0 + 1], c[PIG_ERR0 + 2]];
        ks_field_color(&ks, err, c[PIG_MASS])
    }

    /// Deposit pigment + water in a soft disc — a brush touch wetting the paper.
    /// Re-wetting a dried area re-opens its gate so the old pigment blooms again.
    /// `water_add` raises wetness (clamped to 1); a pigment of `color` (linear sRGB) at peak
    /// coverage `pigment_mass` is added, weighted by the disc falloff. `staining` ∈ [0,1]
    /// (ADR-0081) rides along mass-weighted (1 = permanent stain, resists lifting; 0 = liftable
    /// / raw colour). The pigment is the mass-weighted K/S accumulation (ADR-0080), so
    /// overlapping splats of different colours mix subtractively in the field.
    #[allow(clippy::too_many_arguments)] // a brush dab is genuinely (pos, r, water, colour, mass, stain)
    pub fn splat(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        water_add: f32,
        color: [f32; 3],
        pigment_mass: f32,
        staining: f32,
    ) {
        if radius <= 0.0 {
            return;
        }
        // The per-unit-mass pigment channels of `color`, built ONCE (the dab is one colour);
        // each cell adds `pigment_mass·fall ×` this — linear in mass, so `dab[PIG_MASS]·m = m`.
        let dab = Self::cell_from_color_mass(color, 1.0);
        let stain = staining.clamp(0.0, 1.0);
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
                let m = pigment_mass * fall;
                let cell = &mut self.pigment[i];
                for k in 0..PIG_CH {
                    cell[k] += dab[k] * m;
                }
                // Staining rides along mass-weighted (dab carries 0 here); ADR-0081.
                cell[PIG_STAIN] += stain * m;
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
        // Lift (ADR-0081): in wet areas, re-mobilize NON-staining deposited pigment back into the
        // flowing layer so it blooms/moves again — re-wetting "lifts" dried paint; staining
        // pigments resist. Runs BEFORE diffuse so the lifted pigment participates this step. A
        // no-op while `lift == 0` (deposited stays frozen — the pre-ADR-0081 path is bit-identical).
        if p.lift > 0.0 {
            self.lift_pigment(p);
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
                let mut acc = [0.0f32; PIG_CH];
                let add = |nidx: usize, acc: &mut [f32; PIG_CH]| {
                    let cond = 0.5 * (gc + gates[nidx]);
                    let pn = &self.pigment[nidx];
                    for k in 0..PIG_CH {
                        acc[k] += cond * (pn[k] - pc[k]);
                    }
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
                let mut out = pc;
                for k in 0..PIG_CH {
                    out[k] = pc[k] + d * acc[k];
                }
                self.scratch[c] = out;
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
                    for k in 0..PIG_CH {
                        lo[k] = lo[k].min(v[k]);
                        hi[k] = hi[k].max(v[k]);
                    }
                }
                let pbar = self.pigment[c]; // φ̄
                for k in 0..PIG_CH {
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
                let lap_u = self.vel_u[li(xm, y)]
                    + self.vel_u[li(xp, y)]
                    + self.vel_u[li(x, ym)]
                    + self.vel_u[li(x, yp)]
                    - 4.0 * self.vel_u[c];
                let lap_v = self.vel_v[li(xm, y)]
                    + self.vel_v[li(xp, y)]
                    + self.vel_v[li(x, ym)]
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
            for k in 0..PIG_CH {
                let moved = rate * self.pigment[i][k];
                self.pigment[i][k] -= moved;
                self.deposited[i][k] += moved;
            }
        }
    }

    /// **Lift (ADR-0081)** — the inverse of `transfer_pigment`: in WET cells, re-mobilize a
    /// fraction of the DEPOSITED (dried) pigment back into the FLOWING layer, so re-wetting dried
    /// paint reactivates it (it blooms/moves again). The rate is `lift · smoothstep(w_lo,w_hi,
    /// water) · (1 − staining)` where `staining` is the DEPOSITED pigment's own (per-cell
    /// `stain_acc/mass`) — so a staining pigment (Phthalo, Quinacridone) resists lifting while a
    /// sedimentary one (Ultramarine, Raw Sienna) lifts. Mass-conserving (every gram leaving
    /// `deposited` lands in `pigment`). A no-op while `lift == 0` (deposited stays frozen — the
    /// pre-ADR-0081 path is bit-identical).
    fn lift_pigment(&mut self, p: &DiffusionParams) {
        if p.lift <= 0.0 {
            return;
        }
        let n = (self.width as usize) * (self.height as usize);
        for i in 0..n {
            let dep_mass = self.deposited[i][PIG_MASS];
            if dep_mass <= 1.0e-6 {
                continue;
            }
            let stain = (self.deposited[i][PIG_STAIN] / dep_mass).clamp(0.0, 1.0);
            let wet = smoothstep(p.w_lo, p.w_hi, self.water[i]);
            let rate = (p.lift * wet * (1.0 - stain)).clamp(0.0, 1.0);
            if rate <= 0.0 {
                continue;
            }
            for k in 0..PIG_CH {
                let moved = rate * self.deposited[i][k];
                self.deposited[i][k] -= moved;
                self.pigment[i][k] += moved;
            }
        }
        // ADR-0084 backdrop lift — re-mobilize the DRY paint reservoir (`lift_source`, the
        // downsampled canvas backdrop) into the flowing layer in wet cells, so a wet brush over
        // already-dry paint lifts it (it lightens + bleeds via the flowing layer's diffusion).
        // Same rate law as above; `lift_source`'s own staining ratio resists (0 for a backdrop
        // seed → dry paint lifts freely). Geometric depletion (`lift_source *= 1−rate`) makes
        // `lifted_frac` track the cumulative fraction removed → the compositor drops the backdrop
        // alpha by it. A no-op while `lift_source` is unseeded (all-zero) — the pre-ADR-0084 path.
        self.lift_from_backdrop(p);
    }

    /// ADR-0084 backdrop-lift inner loop (see [`Self::lift_pigment`]). Split out so the
    /// deposited-lift (ADR-0081) stays a tight, unchanged loop.
    fn lift_from_backdrop(&mut self, p: &DiffusionParams) {
        let n = (self.width as usize) * (self.height as usize);
        for i in 0..n {
            let src_mass = self.lift_source[i][PIG_MASS];
            if src_mass <= 1.0e-6 {
                continue;
            }
            let stain = (self.lift_source[i][PIG_STAIN] / src_mass).clamp(0.0, 1.0);
            let wet = smoothstep(p.w_lo, p.w_hi, self.water[i]);
            let rate = (p.lift * wet * (1.0 - stain)).clamp(0.0, 1.0);
            if rate <= 0.0 {
                continue;
            }
            for k in 0..PIG_CH {
                let moved = rate * self.lift_source[i][k];
                self.lift_source[i][k] -= moved;
                // Only a FRACTION bleeds into the wash; the rest is removed (carried off the brush)
                // so the spot lightens instead of concentrating into a dark merged-mass glaze.
                self.pigment[i][k] += moved * LIFT_BLEED_KEEP;
            }
            // Cumulative fraction lifted (the donor depletes geometrically by `1−rate`, so this
            // "remaining" accumulation stays consistent with `1 − src_mass/initial_mass`). Drives
            // the compositor backdrop-alpha drop → the dry paint lightens by the FULL lifted amount.
            self.lifted_frac[i] += rate * (1.0 - self.lifted_frac[i]);
        }
    }

    /// **Seed the ADR-0084 backdrop-lift donor** from a canvas-res RGBA8 `backdrop` (`cw·ch·4`
    /// bytes, straight-alpha sRGB — the same buffer the compositor glazes over) and the session's
    /// original `paper` canvas (same dims): only the PAINT — the backdrop's deviation from the
    /// paper — is liftable (paper-reveal model). Call ONCE at stroke start when `lift > 0`; resets
    /// `lifted_frac` to 0. Leaving it unseeded (the default) keeps the backdrop-lift branch
    /// dormant. The downsample is [`backdrop_to_lift_source`], a free fn the GPU path reuses
    /// verbatim so the seed is bit-identical CPU↔GPU (HR-5).
    pub fn seed_lift_source_from_backdrop(
        &mut self,
        backdrop: &[u8],
        paper: &[u8],
        cw: u32,
        ch: u32,
    ) {
        self.lift_source =
            backdrop_to_lift_source(backdrop, paper, cw, ch, self.width, self.height);
        self.lifted_frac.iter_mut().for_each(|f| *f = 0.0);
    }

    /// ADR-0084 — the per-cell lifted fraction ∈ [0,1] (the compositor drops the backdrop alpha
    /// by this). All-zero unless a stroke with `lift > 0` seeded + ran the backdrop lift.
    #[must_use]
    pub fn lifted_frac(&self) -> &[f32] {
        &self.lifted_frac
    }

    /// ADR-0084 — the backdrop-lift donor field (for parity tests / inspection).
    #[must_use]
    pub fn lift_source(&self) -> &[WetCell] {
        &self.lift_source
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
        pig_out.resize(n, [0.0; PIG_CH]);
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
                let face = |nidx: usize| -> (f32, [f32; PIG_CH]) {
                    let permn = p.perm_valley + (p.perm_crest - p.perm_valley) * self.paper[nidx];
                    let cond = 0.5 * (permc + permn);
                    // Branched (fiber-channeled) capillary (ADR-0082): CREST-GATE the face
                    // conductance by the paper fibre — crests keep FULL conductance (fingers
                    // grow at full wick speed), valleys close completely at branching = 1
                    // (see `BRANCH_GATE_*`). Suppression-only ⇒ the convex-average stability +
                    // conservation hold; `capillary_branching = 0 ⇒ fcond == cond` (bit-identical
                    // isotropic). Used for BOTH the water flux `dw` and the pigment co-advect.
                    let paper_face = 0.5 * (self.paper[c] + self.paper[nidx]);
                    let gate = smoothstep(BRANCH_GATE_LO, BRANCH_GATE_HI, paper_face);
                    let fiber_factor = 1.0 - p.capillary_branching * (1.0 - gate);
                    let fcond = cond * fiber_factor;
                    let wn = self.water[nidx];
                    let dw = fcond * (wn - wc);
                    let mut dp = [0.0f32; PIG_CH];
                    if wc > wn {
                        // c donates: pigment fraction = mobility · (water fraction leaving to n).
                        let frac = mob * cap * fcond * (wc - wn) / wc;
                        for k in 0..PIG_CH {
                            dp[k] = -frac * pc[k];
                        }
                    } else if wn > wc {
                        // n donates to c at n's concentration (also mobility-scaled).
                        let frac = mob * cap * fcond * (wn - wc) / wn;
                        let pnb = &self.pigment[nidx];
                        for k in 0..PIG_CH {
                            dp[k] = frac * pnb[k];
                        }
                    }
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
                    for k in 0..PIG_CH {
                        pn[k] += dp[k];
                    }
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

    /// Pigment field — the raw [`PIG_CH`]-channel mass-weighted K/S accumulation per cell
    /// (ADR-0080). For the per-cell colour / coverage use [`Self::pigment_color`] /
    /// [`Self::pigment_mass`]; this raw view feeds the GPU upload + parity gates.
    #[must_use]
    pub fn pigment(&self) -> &[WetCell] {
        &self.pigment
    }

    /// Coverage `mass` of flowing pigment at cell `i` (the `dens` analogue of the old gray field).
    #[inline]
    #[must_use]
    pub fn pigment_mass(&self, i: usize) -> f32 {
        self.pigment[i][PIG_MASS]
    }

    /// Mixed linear-sRGB colour of flowing pigment at cell `i` (K–M reduction, ADR-0080).
    #[inline]
    #[must_use]
    pub fn pigment_color(&self, i: usize) -> [f32; 3] {
        Self::cell_color(&self.pigment[i])
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
    pub fn set_pigment_from(&mut self, p: &[WetCell]) {
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
            *p = [0.0; PIG_CH];
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

    /// Total FLOWING pigment **mass** — conserved under pure diffusion + advection (no
    /// evaporation removes pigment). With the deposition layer on, flowing pigment migrates
    /// into [`Self::deposited`]; `total_pigment + total_deposited` is the conserved sum (the
    /// invariant the deposition tests check). (ADR-0080: was per-RGB-channel; now the scalar
    /// coverage mass — use [`Self::total_channels`] for the per-band conservation check.)
    #[must_use]
    pub fn total_pigment(&self) -> f64 {
        self.pigment.iter().map(|c| c[PIG_MASS] as f64).sum()
    }

    /// Per-channel total of the flowing field (all [`PIG_CH`] channels: K/S + err + mass) —
    /// EACH is conserved under pure diffusion/advection (the strongest conservation gate,
    /// ADR-0080). Returned as `f64` for accumulation precision.
    #[must_use]
    pub fn total_channels(&self) -> [f64; PIG_CH] {
        let mut s = [0.0f64; PIG_CH];
        for px in &self.pigment {
            for k in 0..PIG_CH {
                s[k] += px[k] as f64;
            }
        }
        s
    }

    /// The deposited (frozen-into-paper) pigment field — raw [`PIG_CH`] channels per cell
    /// (ADR-0078 S3 / ADR-0080). The composited pigment is `pigment + deposited`; all-zero
    /// while the deposition params are off.
    #[must_use]
    pub fn deposited(&self) -> &[WetCell] {
        &self.deposited
    }

    /// Coverage `mass` of deposited pigment at cell `i`.
    #[inline]
    #[must_use]
    pub fn deposited_mass(&self, i: usize) -> f32 {
        self.deposited[i][PIG_MASS]
    }

    /// Total deposited pigment **mass** (companion to [`Self::total_pigment`] for the
    /// mass-conservation invariant `flowing + deposited == splatted`).
    #[must_use]
    pub fn total_deposited(&self) -> f64 {
        self.deposited.iter().map(|c| c[PIG_MASS] as f64).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test adapter: the pre-ADR-0080 5-arg splat (deposit a `color` with mass = its channel
    /// sum, the old `dens`) + scalar coverage probes, so the physics tests below read the same
    /// (mass transports identically to the old gray pigment). `pmass`/`dmass` = flowing/deposited
    /// coverage at a cell; `pcolor` = the mixed colour (for the new multi-pigment mix tests).
    trait TestField {
        fn splat5(&mut self, cx: f32, cy: f32, r: f32, water: f32, color: [f32; 3]);
        fn pmass(&self, i: usize) -> f32;
        fn dmass(&self, i: usize) -> f32;
        fn pcolor(&self, i: usize) -> [f32; 3];
    }
    impl TestField for DiffusionGrid {
        fn splat5(&mut self, cx: f32, cy: f32, r: f32, water: f32, color: [f32; 3]) {
            self.splat(cx, cy, r, water, color, color[0] + color[1] + color[2], 0.0);
        }
        fn pmass(&self, i: usize) -> f32 {
            self.pigment_mass(i)
        }
        fn dmass(&self, i: usize) -> f32 {
            self.deposited_mass(i)
        }
        fn pcolor(&self, i: usize) -> [f32; 3] {
            self.pigment_color(i)
        }
    }

    /// Pigment is conserved by diffusion + advection (no evaporation): EVERY one of the
    /// [`PIG_CH`] channels (mass-weighted K/S + err + mass) is invariant to ~1e-3 over many
    /// steps (ADR-0080 — the linear transport conserves each accumulator independently).
    #[test]
    fn pigment_mass_is_conserved() {
        let mut g = DiffusionGrid::new(48, 48, 1.0);
        // A wet, pigmented blob; flood the whole grid wet so the gate is open.
        for w in g.water.iter_mut() {
            *w = 1.0;
        }
        g.splat5(24.0, 24.0, 8.0, 0.0, [0.6, 0.2, 0.1]);
        let before = g.total_channels();
        let p = DiffusionParams {
            evaporation: 0.0, // isolate conservation
            ..Default::default()
        };
        for _ in 0..40 {
            g.step(&p);
        }
        let after = g.total_channels();
        for k in 0..PIG_CH {
            assert!(
                (before[k] - after[k]).abs() < 1e-3 * (before[k].abs() + 1.0),
                "channel {k} drifted: {} -> {}",
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
            g.splat5(16.0, 16.0, 5.0, 0.3, [0.5, 0.3, 0.7]);
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
            g.splat5(24.0, 24.0, 3.0, 0.0, [0.7, 0.2, 0.1]);
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
                        ring += g.pmass((y * 48 + x) as usize);
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
        g.splat5(20.0, 20.0, 10.0, 1.0, [0.9, 0.8, 0.2]);
        g.splat5(10.0, 10.0, 4.0, 0.6, [0.1, 0.2, 0.9]);
        let p = DiffusionParams::default();
        for _ in 0..200 {
            g.step(&p);
        }
        for px in g.pigment() {
            for &c in px {
                // ADR-0080: err channels are signed (a re-anchor correction), so bound the
                // magnitude rather than requiring ≥0 (ks + mass stay ≥0 via the convex passes).
                assert!(c.is_finite() && c.abs() < 10.0, "unstable value {c}");
            }
        }
    }

    /// Drying ends the bleed: after the water evaporates the gate is shut, so a
    /// further burst of steps barely moves the (now frozen) pigment.
    #[test]
    fn drying_freezes_the_pigment() {
        let mut g = DiffusionGrid::new(48, 48, 1.0);
        g.splat5(24.0, 24.0, 4.0, 0.5, [0.6, 0.2, 0.1]);
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
        g.splat5(20.0, 20.0, 8.0, 0.9, [0.6, 0.2, 0.1]);
        let p = DiffusionParams::default();
        for _ in 0..50 {
            g.step(&p);
        }
        let dep = g.total_deposited();
        assert_eq!(dep, 0.0, "deposition must be dormant at rate 0");
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
        g.splat5(24.0, 24.0, 8.0, 0.0, [0.6, 0.2, 0.1]);
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
        assert!(dep > 0.01, "deposition must actually occur (dep {dep})");
        let total = flow + dep;
        assert!(
            (before - total).abs() < 1e-3 * (before.abs() + 1.0),
            "flowing+deposited mass {total} != splatted {before}"
        );
    }

    /// EDGE-DARKENING: a wash's rim (lower water from the splat falloff) dries first,
    /// so `deposition_dry` freezes its pigment first → the rim ends up a HIGHER
    /// deposited fraction than the still-wet centre (watercolor's dark perimeter).
    #[test]
    fn dry_deposition_darkens_the_rim() {
        let (w, h) = (80u32, 80u32);
        let (cx, cy, r) = (40.0f32, 40.0, 16.0);
        let mut g = DiffusionGrid::new(w, h, 1.0);
        g.splat5(cx, cy, r, 0.9, [0.5, 0.3, 0.2]);
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
                        dep += g.dmass(i);
                        tot += g.dmass(i) + g.pmass(i);
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
        g.splat5(32.0, 32.0, 28.0, 0.0, [0.5, 0.4, 0.3]);
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
            let tot = g.dmass(i) + g.pmass(i);
            if tot < 1e-4 {
                continue;
            }
            if g.paper[i] < median {
                v_dep += g.dmass(i);
                v_tot += tot;
            } else {
                c_dep += g.dmass(i);
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
        g.splat5(20.0, 20.0, 8.0, 0.9, [0.6, 0.2, 0.1]);
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
            g.splat5(20.0, 18.0, 7.0, 0.9, [0.5, 0.3, 0.7]);
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
        g.splat5(24.0, 24.0, 12.0, 1.0, [0.9, 0.8, 0.2]);
        g.splat5(12.0, 30.0, 5.0, 0.7, [0.1, 0.2, 0.9]);
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
                // ADR-0080: err channels are signed; bound magnitude (ks + mass stay ≥0).
                assert!(c.is_finite() && c.abs() < 10.0, "unstable pigment {c}");
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
                    let m = g.pmass((y * w + x) as usize);
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
            g.splat5(20.0, 24.0, 5.0, 0.0, [0.6, 0.2, 0.1]);
            let before = com_x(&g);
            let p = DiffusionParams {
                evaporation: 0.0,
                diffusivity: 0.0, // isolate advection (no symmetric bloom)
                downhill: 0.0,
                flow_outward: 0.0,
                viscosity: 0.0,
                drag: 0.0,     // no decay → clean translation
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
        eprintln!(
            "COM_x  velocity-on: {b_on:.2}->{a_on:.2}   velocity-off: {b_off:.2}->{a_off:.2}"
        );
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
                        s += g.pmass((y * w + x) as usize);
                        n += 1.0;
                    }
                }
            }
            s / n.max(1.0)
        };
        let run = |velocity: f32| -> (f32, f32) {
            let mut g = DiffusionGrid::new(w, h, 1.0);
            g.splat5(cx, cy, 20.0, 0.9, [0.5, 0.25, 0.15]);
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
        g.splat5(cx, cy, 8.0, 1.0, [0.5, 0.3, 0.2]);
        let p = DiffusionParams {
            evaporation: 0.0,
            capillary: 0.0,
            ..Default::default()
        };
        for _ in 0..50 {
            g.step(&p);
        }
        let fringe = ring_water(&g, cx, cy, 10.0, 13.0);
        assert_eq!(
            fringe, 0.0,
            "capillary OFF must leave the fringe dry (got {fringe})"
        );
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
            g.splat5(cx, cy, 8.0, 1.0, [0.5, 0.3, 0.2]);
            let p = DiffusionParams {
                evaporation: 0.0,
                capillary,
                ..Default::default()
            };
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
        eprintln!(
            "capillary fringe on={fringe_on:.4} off={fringe_off:.4} far={far:.4} max={max_w:.4}"
        );
        assert!(
            fringe_on > 0.01 && fringe_on > fringe_off + 0.01,
            "capillary must wet a fringe past the splat (on {fringe_on:.4} ≫ off {fringe_off:.4})"
        );
        assert!(
            far < 0.01,
            "fringe must stay bounded, not flood the grid (far ring {far:.4})"
        );
        assert!(
            max_w <= 1.0001,
            "water must stay ≤ 1 (convex average, no runaway): {max_w}"
        );
    }

    /// Mass-conserving: capillary flow conserves BOTH water (its diffusion) AND pigment (its
    /// co-advection is a gather-form upwind transport — every gram leaving a cell lands in a
    /// neighbour). With no evaporation the totals are invariant (only redistributed).
    #[test]
    fn capillary_conserves_water_and_pigment() {
        let (w, h) = (48u32, 48u32);
        let mut g = DiffusionGrid::new(w, h, 1.0);
        g.splat5(24.0, 24.0, 8.0, 0.9, [0.6, 0.2, 0.1]); // peak ≤ 0.9 → no splat saturation
        let water_before: f64 = g.water().iter().map(|&x| x as f64).sum();
        let pig_before = g.total_pigment();
        let p = DiffusionParams {
            evaporation: 0.0,
            capillary: 0.2,
            ..Default::default()
        };
        for _ in 0..40 {
            g.step(&p);
        }
        let water_after: f64 = g.water().iter().map(|&x| x as f64).sum();
        let pig_after = g.total_pigment();
        assert!(
            (water_before - water_after).abs() < 1e-3 * (water_before.abs() + 1.0),
            "capillary must conserve water: {water_before} -> {water_after}"
        );
        assert!(
            (pig_before - pig_after).abs() < 1e-3 * (pig_before.abs() + 1.0),
            "capillary must conserve pigment mass: {pig_before} -> {pig_after}"
        );
    }

    // ───────────────── ADR-0082 — branched (fiber-channeled) capillary fringe ─────────────────

    /// NON-DESTRUCTIVE (ADR-0082 §2.1): `capillary_branching = 0` is BIT-IDENTICAL to today's
    /// isotropic capillary (`fiber_factor = 1`). Run the capillary layer on the SAME splatted
    /// blob with branching explicitly 0 vs the `Default` (also 0) — every water + pigment field
    /// value must match bit-for-bit, proving the opt-in parameter never touches the shipped path.
    #[test]
    fn branching_off_is_bit_identical() {
        let (w, h) = (48u32, 48u32);
        let run = |branching: f32| -> DiffusionGrid {
            let mut g = DiffusionGrid::new(w, h, 1.0);
            g.splat5(24.0, 24.0, 8.0, 0.9, [0.6, 0.2, 0.1]);
            let p = DiffusionParams {
                evaporation: 0.0,
                capillary: 0.2,
                capillary_branching: branching,
                ..Default::default()
            };
            for _ in 0..30 {
                g.step(&p);
            }
            g
        };
        let zero = run(0.0); // explicit 0
        let default = run(DiffusionParams::default().capillary_branching); // the Default (also 0)
        assert_eq!(
            zero.water(),
            default.water(),
            "capillary_branching = 0 must leave the water field bit-identical to the default"
        );
        // Pigment is `[f32; PIG_CH]` per cell → compare the raw channel arrays exactly.
        assert_eq!(
            zero.pigment(),
            default.pigment(),
            "capillary_branching = 0 must leave the pigment field bit-identical to the default"
        );
    }

    /// The defining ADR-0082 behaviour: with `capillary_branching > 0` the fringe goes
    /// LOBED/dendritic (the wick is suppressed in the low-paper valleys, ~full on the crests)
    /// instead of a smooth ring. Branching is suppression-only, so it lowers the OVERALL wicked
    /// water (smaller absolute values) — the lobing shows as a higher *relative* unevenness, i.e.
    /// a higher squared coefficient of variation `var / mean²` of the fringe water (some angular
    /// sectors wick far on the crests, others barely in the valleys), vs the near-uniform smooth
    /// ring. A `scale = 4` paper makes the tooth fine enough that the fringe lobes within the
    /// probe annulus. Capillary CONSERVES water + pigment regardless of branching (suppression-only
    /// keeps the convex-average mass-conservation), checked here too.
    #[test]
    fn branching_makes_fringe_uneven() {
        let (w, h) = (64u32, 64u32);
        let (cx, cy) = (32.0f32, 32.0);
        // Squared coefficient of variation `var / mean²` of the water over the fringe annulus
        // `[lo, hi)` — the magnitude-independent lobing probe (a smooth ring → ~0; lobes → high).
        let fringe_cv2 = |g: &DiffusionGrid, lo: f32, hi: f32| -> f64 {
            let (gw, gh) = g.dims();
            let (mut sum, mut sumsq, mut n) = (0.0f64, 0.0f64, 0.0f64);
            for y in 0..gh {
                for x in 0..gw {
                    let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
                    if d >= lo && d < hi {
                        let v = g.water()[(y * gw + x) as usize] as f64;
                        sum += v;
                        sumsq += v * v;
                        n += 1.0;
                    }
                }
            }
            let mean = sum / n.max(1.0);
            let var = (sumsq / n.max(1.0) - mean * mean).max(0.0);
            var / (mean * mean).max(1e-12)
        };
        let run = |branching: f32| -> DiffusionGrid {
            // scale = 4 → a finer paper tooth (lobes within the annulus).
            let mut g = DiffusionGrid::new(w, h, 4.0);
            g.splat5(cx, cy, 8.0, 1.0, [0.10, 0.20, 0.70]);
            let p = DiffusionParams {
                evaporation: 0.0,
                capillary: 0.24,
                capillary_branching: branching,
                ..Default::default()
            };
            for _ in 0..50 {
                g.step(&p);
            }
            g
        };
        let smooth = run(0.0);
        let lobed = run(0.9);
        let cv2_smooth = fringe_cv2(&smooth, 9.0, 13.0);
        let cv2_lobed = fringe_cv2(&lobed, 9.0, 13.0);
        eprintln!("branching fringe cv²: smooth = {cv2_smooth:.6e}, lobed = {cv2_lobed:.6e}");
        assert!(
            cv2_lobed > cv2_smooth * 1.1,
            "branched capillary fringe must be MORE uneven (lobed) than the smooth ring: lobed cv² {cv2_lobed:.6e} vs smooth cv² {cv2_smooth:.6e}"
        );
        // Conservation still holds with branching ON (suppression-only ⇒ convex average) — a
        // separate unsaturated splat (peak ≤ 0.9) so there's no splat saturation, no evaporation.
        let mut g = DiffusionGrid::new(48, 48, 4.0);
        g.splat5(24.0, 24.0, 8.0, 0.9, [0.6, 0.2, 0.1]); // peak ≤ 0.9 → no splat saturation
        let wb: f64 = g.water().iter().map(|&x| x as f64).sum();
        let pb = g.total_pigment();
        let p = DiffusionParams {
            evaporation: 0.0,
            capillary: 0.2,
            capillary_branching: 0.9,
            ..Default::default()
        };
        for _ in 0..40 {
            g.step(&p);
        }
        let wa: f64 = g.water().iter().map(|&x| x as f64).sum();
        let pa = g.total_pigment();
        assert!(
            (wb - wa).abs() < 1e-3 * (wb.abs() + 1.0),
            "branched capillary must still conserve water: {wb} -> {wa}"
        );
        assert!(
            (pb - pa).abs() < 1e-3 * (pb.abs() + 1.0),
            "branched capillary must still conserve pigment mass: {pb} -> {pa}"
        );
    }

    /// DETERMINISTIC (HR-5): pure arithmetic → same inputs give a bit-identical field.
    #[test]
    fn capillary_is_deterministic() {
        let run = || {
            let mut g = DiffusionGrid::new(40, 36, 1.0);
            g.splat5(20.0, 18.0, 7.0, 0.9, [0.5, 0.3, 0.7]);
            let p = DiffusionParams {
                capillary: 0.2,
                ..Default::default()
            };
            for _ in 0..25 {
                g.step(&p);
            }
            (g.water().to_vec(), g.pigment().to_vec())
        };
        assert_eq!(
            run(),
            run(),
            "capillary solver must be deterministic (HR-5)"
        );
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
                        s += g.pmass((y * w + x) as usize);
                        n += 1.0;
                    }
                }
            }
            s / n.max(1.0)
        };
        let run = |capillary: f32| -> f32 {
            let mut g = DiffusionGrid::new(w, h, 1.0);
            g.splat5(cx, cy, 8.0, 1.0, [0.6, 0.2, 0.1]);
            let p = DiffusionParams {
                evaporation: 0.0,
                capillary,
                ..Default::default()
            };
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
            g.splat5(cx, cy, 8.0, 1.0, [0.5, 0.3, 0.2]);
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
                    if g.pmass(i) > 1.0e-4 {
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
        type Bbox = (u32, u32, u32, u32);
        let (w, h) = (72u32, 64u32);
        let pig_bbox = |g: &DiffusionGrid| -> Bbox {
            let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
            for y in 0..h {
                for x in 0..w {
                    if g.pmass((y * w + x) as usize) > 1.0e-4 {
                        x0 = x0.min(x);
                        y0 = y0.min(y);
                        x1 = x1.max(x);
                        y1 = y1.max(y);
                    }
                }
            }
            (x0, y0, x1, y1)
        };
        let run = |capillary: f32| -> (Bbox, Bbox) {
            let mut g = DiffusionGrid::new(w, h, 1.0);
            g.splat5(36.0, 32.0, 9.0, 1.0, [0.5, 0.3, 0.2]);
            // No evaporation: keep the field wet so the fringe is fully developed (aggressive
            // capillary + drying would dry the thin film to nothing — a separate concern).
            let p = DiffusionParams {
                capillary,
                evaporation: 0.0,
                ..Default::default()
            };
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
            wet_on.0 < wet_off.0
                && wet_on.1 < wet_off.1
                && wet_on.2 > wet_off.2
                && wet_on.3 > wet_off.3,
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
            g.splat5(18.0, 20.0, 4.0, 0.0, [0.8, 0.4, 0.2]);
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
            g.pigment()
                .iter()
                .map(|c| c[PIG_MASS])
                .fold(0.0f32, f32::max) // peak (mass)
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
        assert!(
            p1.is_finite() && p1 < 2.0,
            "sharpened advection must stay bounded: {p1}"
        );
    }

    /// DETERMINISTIC with sharpness on (HR-5): the MacCormack passes are pure arithmetic.
    #[test]
    fn sharpness_is_deterministic() {
        let run = || {
            let mut g = DiffusionGrid::new(40, 36, 1.0);
            g.splat5(20.0, 18.0, 7.0, 0.9, [0.5, 0.3, 0.7]);
            let p = DiffusionParams {
                velocity: 1.4,
                pressure: 1.0,
                sharpness: 1.0,
                ..Default::default()
            };
            for _ in 0..25 {
                g.step(&p);
            }
            g.pigment().to_vec()
        };
        assert_eq!(
            run(),
            run(),
            "sharpened advection must be deterministic (HR-5)"
        );
    }

    // ───────────────── ADR-0080 — multi-pigment wet-field mixing ─────────────────

    /// **THE P1 mixing gate (ADR-0080):** two overlapping wet blobs of DIFFERENT colours mix
    /// SUBTRACTIVELY in the field — blue (left) + yellow (right) → a green-dominant colour in
    /// the overlap, NOT the muddy grey a linear/coverage average gives. This is the wet-on-wet
    /// "magic" the whole ADR exists for, proven at the field level (composite gate is separate).
    #[test]
    fn field_mixes_blue_and_yellow_to_green() {
        let (w, h) = (64u32, 48u32);
        let mut g = DiffusionGrid::new(w, h, 1.0);
        for wv in g.water.iter_mut() {
            *wv = 0.9; // flood wet so both blobs bloom + overlap at the centre
        }
        // Two equal discs meeting at the centre column (x=32, equidistant from both centres).
        g.splat(26.0, 24.0, 12.0, 0.0, [0.0, 0.0, 1.0], 1.0, 0.0);
        g.splat(38.0, 24.0, 12.0, 0.0, [1.0, 1.0, 0.0], 1.0, 0.0);
        let p = DiffusionParams {
            evaporation: 0.0,
            ..Default::default()
        };
        for _ in 0..30 {
            g.step(&p);
        }
        let i = (24 * w + 32) as usize;
        assert!(
            g.pmass(i) > 1e-3,
            "overlap must carry pigment: mass {}",
            g.pmass(i)
        );
        let c = g.pcolor(i);
        assert!(
            c[1] > c[0] && c[1] > c[2],
            "overlap mixes to green-dominant (not mud): {c:?}"
        );
        assert!(c[1] > 0.12, "overlap is a real green, not dark mud: {c:?}");
    }

    /// **THE P1 single-pigment gate (ADR-0080 §2.3):** a single-colour wash reduces to EXACTLY
    /// its picked colour at every cell that carries pigment, at any coverage — this is what
    /// preserves the validated single-colour look (the composite of one colour is unchanged).
    #[test]
    fn field_single_color_reduces_to_picked_color() {
        let (w, h) = (48u32, 48u32);
        let col = [0.15, 0.35, 0.8];
        let mut g = DiffusionGrid::new(w, h, 1.0);
        for wv in g.water.iter_mut() {
            *wv = 0.9;
        }
        g.splat(24.0, 24.0, 10.0, 0.0, col, 0.8, 0.0);
        let p = DiffusionParams {
            evaporation: 0.0,
            ..Default::default()
        };
        for _ in 0..20 {
            g.step(&p);
        }
        for i in 0..(w * h) as usize {
            if g.pmass(i) > 1e-4 {
                let c = g.pcolor(i);
                for k in 0..3 {
                    assert!(
                        (c[k] - col[k]).abs() < 2e-3,
                        "cell {i} colour {c:?} != picked {col:?}"
                    );
                }
            }
        }
    }

    // ───────────────── ADR-0081 — pigment staining + lift ─────────────────

    /// Dry a freshly-splatted pigment down into the deposited layer (deposition ON), then return
    /// the grid + the deposited mass — the setup for the lift tests.
    fn deposit_then(staining: f32) -> (DiffusionGrid, f64) {
        let mut g = DiffusionGrid::new(48, 48, 1.0);
        g.splat(24.0, 24.0, 8.0, 0.5, [0.35, 0.2, 0.55], 0.8, staining);
        let dep = DiffusionParams {
            deposition: 0.2,
            deposition_dry: 0.3,
            ..Default::default()
        };
        for _ in 0..80 {
            g.step(&dep); // dries (evaporation) → most pigment frozen into `deposited`
        }
        let dep_mass = g.total_deposited();
        (g, dep_mass)
    }

    /// **THE F1 lift gate (ADR-0081):** re-wetting dried NON-staining pigment re-mobilizes it back
    /// into the flowing layer (deposited↓, flowing↑), and the total mass is conserved.
    #[test]
    fn lift_re_mobilizes_non_staining_deposited_pigment() {
        let (mut g, dep_before) = deposit_then(0.0); // staining 0 = liftable
        let flow_before = g.total_pigment();
        assert!(
            dep_before > flow_before,
            "most pigment dried into deposited"
        );
        // Re-wet the whole field + step WITH lift on (no deposition/evaporation: isolate lift).
        for w in g.water.iter_mut() {
            *w = 0.9;
        }
        let lift = DiffusionParams {
            evaporation: 0.0,
            lift: 0.6,
            ..Default::default()
        };
        for _ in 0..20 {
            g.step(&lift);
        }
        let dep_after = g.total_deposited();
        let flow_after = g.total_pigment();
        assert!(
            dep_after < dep_before * 0.5,
            "lift re-mobilized the deposited pigment: {dep_before} -> {dep_after}"
        );
        assert!(
            flow_after > flow_before,
            "the lifted pigment is back in the flowing layer"
        );
        let (tot_b, tot_a) = (dep_before + flow_before, dep_after + flow_after);
        assert!(
            (tot_b - tot_a).abs() < 1e-3 * (tot_b.abs() + 1.0),
            "lift conserves total mass: {tot_b} -> {tot_a}"
        );
    }

    /// **THE F1 staining gate (ADR-0081):** a STAINING pigment (stain=1) resists lifting — far
    /// more of it stays deposited than a liftable one, under the SAME lift.
    #[test]
    fn staining_resists_lift() {
        let remaining = |staining: f32| -> f64 {
            let (mut g, dep_before) = deposit_then(staining);
            for w in g.water.iter_mut() {
                *w = 0.9;
            }
            let lift = DiffusionParams {
                evaporation: 0.0,
                lift: 0.6,
                ..Default::default()
            };
            for _ in 0..20 {
                g.step(&lift);
            }
            g.total_deposited() / dep_before.max(1e-9) // fraction still deposited
        };
        let liftable = remaining(0.0);
        let staining = remaining(1.0);
        eprintln!("lift remaining: liftable={liftable:.3} staining={staining:.3}");
        assert!(
            staining > liftable + 0.4,
            "staining must resist lift (remaining {staining:.3} ≫ liftable {liftable:.3})"
        );
    }

    /// **Non-destructive (ADR-0081):** `lift = 0` ⇒ the deposited layer stays FROZEN on re-wet,
    /// bit-identical to the pre-ADR-0081 path (the lift pass is dormant).
    #[test]
    fn lift_off_keeps_deposited_frozen() {
        let (mut g, _) = deposit_then(0.0);
        let snapshot = g.deposited().to_vec();
        for w in g.water.iter_mut() {
            *w = 0.9;
        }
        let no_lift = DiffusionParams {
            evaporation: 0.0,
            lift: 0.0,
            ..Default::default()
        };
        for _ in 0..20 {
            g.step(&no_lift);
        }
        let mut maxd = 0.0f32;
        for (a, b) in snapshot.iter().zip(g.deposited().iter()) {
            for k in 0..PIG_CH {
                maxd = maxd.max((a[k] - b[k]).abs());
            }
        }
        assert!(
            maxd < 1e-6,
            "lift=0 must keep deposited frozen (max Δ {maxd})"
        );
    }

    // ───────────────── ADR-0084 — backdrop lift (wet brush re-mobilizes dry paint) ─────────────────

    /// A solid-colour canvas-res backdrop (RGBA8, opaque) — the "dry paint" a wet brush lifts.
    fn solid_backdrop(cw: u32, ch: u32, rgb: [u8; 3]) -> Vec<u8> {
        let mut b = vec![0u8; (cw * ch * 4) as usize];
        for px in b.chunks_exact_mut(4) {
            px[0] = rgb[0];
            px[1] = rgb[1];
            px[2] = rgb[2];
            px[3] = 255;
        }
        b
    }

    /// **THE ADR-0084 gate:** seeding the donor from a backdrop + a WET field + `lift > 0`
    /// re-mobilizes the dry paint off the donor (`lift_source`↓), raises `lifted_frac` (→ the
    /// compositor lightens the backdrop), and bleeds only `LIFT_BLEED_KEEP` of the lifted mass into
    /// the flowing wash (the rest is removed — carried off the brush — so the spot lightens, not darkens).
    #[test]
    fn backdrop_lift_remobilizes_dry_paint() {
        let (gw, gh) = (32u32, 32u32);
        let mut g = DiffusionGrid::new(gw, gh, 1.0);
        // Paper = fully transparent (a blank sprite) → the solid paint is 100% "painted".
        let blank = vec![0u8; 64 * 64 * 4];
        g.seed_lift_source_from_backdrop(&solid_backdrop(64, 64, [40, 50, 200]), &blank, 64, 64);
        let src_before: f64 = g.lift_source().iter().map(|c| f64::from(c[PIG_MASS])).sum();
        let flow_before = g.total_pigment();
        assert!(src_before > 1.0, "donor seeded from the backdrop");
        assert!(flow_before < 1e-6, "nothing flowing yet");
        // Wet the whole field (a wash laid over the dry paint) + lift on, no deposition/evaporation.
        for w in g.water.iter_mut() {
            *w = 0.9;
        }
        let lift = DiffusionParams {
            evaporation: 0.0,
            lift: 0.7,
            ..Default::default()
        };
        for _ in 0..20 {
            g.step(&lift);
        }
        let src_after: f64 = g.lift_source().iter().map(|c| f64::from(c[PIG_MASS])).sum();
        let flow_after = g.total_pigment();
        let max_frac = g.lifted_frac().iter().cloned().fold(0.0f32, f32::max);
        eprintln!(
            "backdrop lift: donor {src_before:.2}->{src_after:.2}, flowing {flow_before:.2}->{flow_after:.2}, max lifted_frac={max_frac:.3}"
        );
        assert!(
            src_after < src_before * 0.5,
            "the donor depleted into the wash"
        );
        assert!(
            flow_after > flow_before + 1.0,
            "some lifted paint bleeds into the wash"
        );
        assert!(
            max_frac > 0.5,
            "lifted_frac records the re-mobilization (compositor lightens the backdrop)"
        );
        // Removal model (ADR-0084): the donor depletes fully, but only LIFT_BLEED_KEEP of the lifted
        // mass lands in the wash (the rest is removed — carried off the brush) so the spot lightens
        // instead of concentrating into a dark merged-mass glaze. flowing gain ≈ 0.25·donor loss.
        let loss = src_before - src_after;
        let gain = flow_after - flow_before;
        assert!(
            (gain - 0.25 * loss).abs() < 0.05 * (loss.abs() + 1.0),
            "backdrop lift bleeds ~0.25 of the lifted mass into the wash: donor lost {loss:.3}, flowing gained {gain:.3} (expected ~{:.3})",
            0.25 * loss
        );
    }

    /// **Non-destructive (ADR-0084):** with `lift = 0`, seeding the donor changes NOTHING — the
    /// donor is never consumed, `lifted_frac ≡ 0`, and the flowing/deposited fields are untouched
    /// (the compositor stays byte-identical). The opt-in guarantee Enio demanded.
    #[test]
    fn backdrop_lift_off_is_inert() {
        let (gw, gh) = (32u32, 32u32);
        let mut g = DiffusionGrid::new(gw, gh, 1.0);
        let blank = vec![0u8; 64 * 64 * 4];
        g.seed_lift_source_from_backdrop(&solid_backdrop(64, 64, [200, 30, 30]), &blank, 64, 64);
        for w in g.water.iter_mut() {
            *w = 0.9;
        }
        let no_lift = DiffusionParams {
            evaporation: 0.0,
            lift: 0.0,
            ..Default::default()
        };
        for _ in 0..20 {
            g.step(&no_lift);
        }
        assert!(
            g.total_pigment() < 1e-6,
            "lift=0 must NOT re-mobilize the donor (flowing stays empty)"
        );
        let max_frac = g.lifted_frac().iter().cloned().fold(0.0f32, f32::max);
        assert!(
            max_frac < 1e-9,
            "lift=0 must leave lifted_frac at 0 (compositor unchanged)"
        );
    }

    /// **Paper-reveal model (ADR-0084 amendment / Enio smoke 2026-06-09):** an UNPAINTED canvas
    /// (backdrop == the session's original paper — e.g. the opaque-beige demo base) seeds an
    /// EMPTY donor: a wet lift brush over bare paper lifts NOTHING (no beige-mud bleeding into the
    /// wash, no `lifted_frac` punching alpha holes that revealed the dark editor background). Only
    /// the painted deviation lifts: with a red square painted onto the beige paper, the donor has
    /// mass ONLY under the square.
    #[test]
    fn backdrop_lift_only_lifts_paint_not_paper() {
        let (gw, gh) = (32u32, 32u32);
        // Opaque beige "paper" canvas (the demo case that produced the dark-mud smoke).
        let paper = solid_backdrop(64, 64, [228, 214, 184]);
        // Case 1 — untouched canvas: backdrop == paper ⇒ donor empty ⇒ lift fully inert.
        let mut g = DiffusionGrid::new(gw, gh, 1.0);
        g.seed_lift_source_from_backdrop(&paper, &paper, 64, 64);
        let donor_mass: f64 = g.lift_source().iter().map(|c| f64::from(c[PIG_MASS])).sum();
        assert!(
            donor_mass < 1e-9,
            "bare paper must seed an EMPTY donor (got mass {donor_mass})"
        );
        for w in g.water.iter_mut() {
            *w = 0.9;
        }
        let lift = DiffusionParams {
            evaporation: 0.0,
            lift: 1.0,
            ..Default::default()
        };
        for _ in 0..10 {
            g.step(&lift);
        }
        assert!(
            g.total_pigment() < 1e-6 && g.lifted_frac().iter().all(|&f| f < 1e-9),
            "lifting bare paper must be a no-op (nothing bleeds, nothing lightens)"
        );
        // Case 2 — a red square painted on the beige paper: ONLY the square is liftable.
        let mut painted = paper.clone();
        for py in 16..48u32 {
            for px in 16..48u32 {
                let o = ((py * 64 + px) * 4) as usize;
                painted[o] = 200;
                painted[o + 1] = 30;
                painted[o + 2] = 30;
            }
        }
        let mut g2 = DiffusionGrid::new(gw, gh, 1.0);
        g2.seed_lift_source_from_backdrop(&painted, &paper, 64, 64);
        let src = g2.lift_source();
        let inside = src[(16 * gw + 16) as usize][PIG_MASS]; // grid cell under the square
        let outside = src[(2 * gw + 2) as usize][PIG_MASS]; // bare-beige corner
        assert!(
            inside > 0.1,
            "the painted square must be liftable (donor mass {inside})"
        );
        assert!(
            outside < 1e-6,
            "bare paper around the square must NOT be liftable (donor mass {outside})"
        );
    }

    /// The staining channel transports + conserves like `ks`/`mass` (survives mixing): under pure
    /// diffusion/advection (no deposition) the flowing `stain_acc` is conserved, and a single
    /// pigment's cells carry its staining ratio.
    #[test]
    fn staining_transports_mass_weighted() {
        let mut g = DiffusionGrid::new(48, 48, 1.0);
        for w in g.water.iter_mut() {
            *w = 1.0;
        }
        g.splat(24.0, 24.0, 8.0, 0.0, [0.3, 0.3, 0.6], 1.0, 0.9); // staining 0.9
        let before = g.total_channels()[PIG_STAIN];
        let p = DiffusionParams {
            evaporation: 0.0,
            ..Default::default()
        };
        for _ in 0..30 {
            g.step(&p);
        }
        let after = g.total_channels()[PIG_STAIN];
        assert!(
            (before - after).abs() < 1e-3 * (before.abs() + 1.0),
            "stain_acc conserved under transport: {before} -> {after}"
        );
        // A pigmented cell carries the pigment's staining ratio (mass-weighted, single pigment).
        let i = (24 * 48 + 24) as usize;
        let cell = &g.pigment()[i];
        assert!(cell[PIG_MASS] > 1e-4, "centre still has pigment");
        let ratio = cell[PIG_STAIN] / cell[PIG_MASS];
        assert!(
            (ratio - 0.9).abs() < 1e-3,
            "staining ratio preserved: {ratio}"
        );
    }
}
