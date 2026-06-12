//! Watercolor **wet-on-wet diffusion** — the algorithm spec + CPU-side field container behind
//! live blooms / bleeds / backruns (Curtis et al., "Computer-Generated Watercolor", SIGGRAPH
//! 1997, with the expensive Navier-Stokes momentum solver replaced by **gated diffusion** — the
//! real-time-feasible simplification used by GPU watercolor systems, Van Laerhoven CAVW 2005;
//! TAMU GPU-watercolor thesis).
//!
//! **ADR-0085:** the live sim is GPU-resident (`ph2d-painter-fluid`). This module no longer runs
//! the loop — it owns the deterministic paper tooth + the per-frame dab buffer the GPU is seeded
//! from ([`DiffusionGrid`]), the shared optical constants/types ([`WetCell`], [`DiffusionParams`],
//! the `PIG_*` layout), and documents the per-cell update the GPU shader implements.
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
//! ## The per-tick update (the GPU step)
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
//! A canvas-res field stepped a few sub-steps per frame, composited (K–M glaze, bicubic) to the
//! canvas — all GPU-resident to hit 4K live. The live per-frame driver, the compute passes, and
//! the composite all live in ADR-0049/ADR-0085 (`ph2d-painter-fluid`, W15); this module is the
//! shared spec + types they build on.

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
    /// (the GPU capillary pass) — the soft, feathery wet-edge that defines the
    /// watercolor look. A conservative, permeability-weighted diffusion of the water field
    /// (`water += capillary·Σ_face ½(perm_c+perm_n)(water_n − water_c)`); because diffusion only
    /// acts where there is a water gradient, the saturated pool interior is untouched and only
    /// the wet→dry boundary creeps out, carrying the wet gate (so the existing pigment then
    /// bleeds into the fringe). CFL-bounded: keep ≤ `0.24` (perm ≤ 1 ⇒ `capillary·Σ_face cond ≤
    /// 1`, so water stays in `[0,1]` and mass is conserved without a clamp). `0` ⇒ the layer is
    /// **dormant**: the GPU capillary pass is skipped and the water field is
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
    /// of the GPU capillary pass. `0` ⇒ `fiber_factor = 1` ⇒ the isotropic capillary is
    /// **bit-identical** to today (opt-in). Only read while the capillary layer is active.
    pub capillary_branching: f32,
    /// **Surface tension — the contact-line pinning threshold (ADR-0079-amendment-1 / ADR-0085 C1).**
    /// The Curtis FlowOutward driving force (`−λ·∇w`) ramps OUT as the film thins below this water
    /// level, so the wet front PINS where it thins past it — the fixed point that stops the wash
    /// spreading. Under Keep Wet / `evaporation = 0` the film only thins by spreading (water is
    /// conserved, not evaporated), so this threshold alone governs how far the pool creeps past the
    /// painted area: HIGHER = the meniscus holds tighter = pins sooner = LESS bleed. Drives
    /// `FLOW_PIN_HI` in `shallow.wgsl` (with `FLOW_PIN_LO` derived proportionally). Default `0.35`
    /// reproduces the pre-amendment hard-coded look bit-for-bit.
    pub surface_tension: f32,
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
            // Contact-line pinning threshold (ADR-0079-amendment-1) — 0.35 reproduces the
            // pre-amendment hard-coded `FLOW_PIN_HI` bit-for-bit.
            surface_tension: 0.35,
        }
    }
}

/// Frequency of the paper-height field sampled into the grid (cycles per grid
/// cell) — coarse enough that valleys span a few cells so pigment can pool.
const HEIGHT_FREQ: f32 = 0.13;
const HEIGHT_SEED: u32 = 0x70a9_e2c5; // shared with cpu_render paper tooth

/// **Water epsilon-clamp (perf block 2a, 2026-06-10).** After evaporation, any cell with
/// `water < WATER_EPS` snaps to 0. Without it, `capillary_flow`/`move_water` push a
/// sub-visible NUMERIC FILM of water outward each step; the film ACCUMULATES across steps,
/// trips the wet-bbox reduction, the bbox (a monotonic union) grows, the solver pad follows,
/// and the envelope runs away until it saturates the canvas — O(envelope) bandwidth cost
/// forever, the §1/§2 FPS collapse of `HANDOFF_painter_fluid_perf_block.md`. Under Keep Wet
/// (`evaporation = 0`) this clamp is the ONLY brake.
///
/// **Calibration (don't raise it to the visible contour):** the clamp kills only what a
/// single step delivers BELOW it — a per-step inflow ≥ EPS survives and accumulates
/// normally, so the real wick dynamics above EPS are intact. At 1e-3 the clamp ate the
/// chromatographic halo (the `capillary_water_wicks_ahead_of_pigment` contract): the halo
/// builds from per-step inflows in [1e-4, 1e-3) that legitimately accumulate past 1e-3.
/// 1e-4 blocks only the sub-1e-4 trickle (the runaway mist), matching the existing flow-gate
/// floor (`g ≤ 1e-4` ⇒ pigment frozen), so clamped cells never carry visible paint. The
/// visible-fringe contour for the wet bbox is [`WET_BBOX_WATER_THRESHOLD`]. This value is the
/// canonical source for the literal in `fluid.wgsl` (`cs_evaporate`) — keep both in sync.
pub const WATER_EPS: f32 = 1.0e-4;

/// **Wet-bbox reduction threshold (perf block 2a).** Cells wetter than this vote in the
/// `read_field_stats` bbox the driver grows the composite envelope from. Calibrated to the
/// REAL visible fringe contour — the same 1e-3 the fringe tests probe — not to the damp
/// band above [`WATER_EPS`] (water in [1e-4, 1e-3) is invisible: the pigment gate only
/// opens at `w_lo = 0.05 ≫ 1e-3`, and pigment reaches only ~2 cells past the 1e-3 contour
/// — covered by the driver's `CAPILLARY_FRINGE_PAD = 8`; the §2.2 envelope-invariant test
/// proves pigment ⊆ `water_bbox(1e-3)`). The old 1e-4 threshold captured the numeric mist
/// and fed the envelope runaway.
pub const WET_BBOX_WATER_THRESHOLD: f32 = 1.0e-3;

/// **Capillary minimum saturation (Curtis 1997 δ_s — perf block 2a, 2026-06-10).** A face
/// carries capillary flux only while its DONOR cell (the wetter side) holds more water than
/// this floor — in Curtis's capillary layer, water transfers between cells only above a
/// minimum saturation ("the wick exhausts"). Without it our diffusion-form wick never
/// terminates: under Keep Wet (evaporation = 0) the wash keeps creeping outward forever —
/// the §4b envelope runaway was REAL water spreading, not just numeric mist (the
/// [`WATER_EPS`] clamp alone only slowed it). The gate is on the SOURCE; the wick into dry
/// receiver cells stays fully open. Gated symmetrically by `max(wc, wn)` so both
/// cells of a face compute the identical (anti-symmetric) flux: water/pigment conservation
/// holds. Equilibrium becomes BOUNDED: the front stalls where the boundary dilutes below δ_s,
/// so the wet envelope ≲ total_water / δ_s cells. The transparent halo + branching fingers
/// live well above this floor (fringe contour 1e-3, plateau ~δ_s; probes in the
/// `capillary_*` tests stay green). GPU mirror literal in `capillary.wgsl::face_info`.
pub const CAPILLARY_MIN_SATURATION: f32 = 0.005;

/// Fixed Jacobi-iteration count for the shallow-water pressure projection
/// (the GPU shallow-water pressure projection, ADR-0078 S3d). A *fixed* count (not a
/// convergence threshold) keeps the solve deterministic + bounded (HR-5). A handful of
/// iterations is enough to turn the local body forces into directional flow at watercolor's
/// visual budget; the GPU re-seeds the pressure field to 0 each projection (no warm-start).
///
/// **Public** so the GPU solver (`ph2d-painter-fluid`) runs this exact sweep count. **Keep
/// even** (the GPU Jacobi ping-pong lands the result in a fixed buffer when the count is even).
pub const RELAX_ITERS: u32 = 6;

/// **ADR-0084 (paper-reveal model) — downsample the PAINT on a canvas into `gw·gh` K–M donor
/// cells.** "Paint" = how the current `backdrop` deviates from the session's original `paper`
/// canvas (the Curtis-1997 semantics: lift *desorbs deposited pigment*, revealing the substrate —
/// it must never treat the substrate itself as liftable pigment). Per pixel, the paintedness is
/// `max(|Δr|,|Δg|,|Δb|,|Δa|)` (linear light + straight alpha) — 0 on untouched pixels (bare paper
/// contributes NOTHING: no beige-mud lifting, no lift on unpainted areas), rising with how much
/// pigment the pixel visibly carries. Each grid cell paintedness-weights the backdrop colour
/// (so bare pixels in a half-painted cell don't dilute the paint hue) and gets
/// `mass = avg paintedness`. Seeds the GPU-resident `lift_source` donor (ADR-0085: the GPU is the
/// single live path). `backdrop.len() == paper.len() == cw·ch·4`;
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

/// The CPU-side wet-field container (ADR-0085). Holds the per-frame dab water + pigment and the
/// static paper-height map; the live wet-on-wet sim itself is GPU-resident (`ph2d-painter-fluid`),
/// seeded from this grid (`paper`/`pigment`/`water`) and stepped + composited on the GPU. The
/// grid is no longer a CPU solver — it is the deterministic paper tooth + the dab-deposit buffer.
pub struct DiffusionGrid {
    width: u32,
    height: u32,
    /// Wetness ∈ [0,1] per cell.
    water: Vec<f32>,
    /// Pigment per cell — the [`PIG_CH`]-channel mass-weighted K/S accumulation (ADR-0080):
    /// `ks[24]` + `err[3]` + `mass`. Mass-conserving under diffusion/advection (every channel
    /// transports linearly), and the multi-pigment mix emerges from the transport.
    pigment: Vec<WetCell>,
    /// Static paper-tooth height ∈ [0,1] per cell (1 = crest, 0 = valley).
    paper: Vec<f32>,
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
    /// for caching). `paper.len()` must equal `width * height`. ADR-0085: the grid is a slim
    /// CPU-side container (paper tooth + per-frame dab water/pigment) that seeds the
    /// GPU-resident solver; the water/pigment buffers are fresh zeroed allocations
    /// (lazy-zeroed pages — ~free until touched).
    #[must_use]
    pub fn with_paper(width: u32, height: u32, paper: Vec<f32>) -> Self {
        let n = (width as usize) * (height as usize);
        debug_assert_eq!(paper.len(), n, "paper length must match width*height");
        Self {
            width,
            height,
            water: vec![0.0; n],
            pigment: vec![[0.0; PIG_CH]; n],
            paper,
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

    /// Grid dimensions.
    #[must_use]
    pub fn dims(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Pigment field — the raw [`PIG_CH`]-channel mass-weighted K/S accumulation per cell
    /// (ADR-0080). This raw view seeds the GPU-resident pigment (`cs_splat`) each frame.
    #[must_use]
    pub fn pigment(&self) -> &[WetCell] {
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
}
