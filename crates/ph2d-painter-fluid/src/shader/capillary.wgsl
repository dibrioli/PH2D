// GPU capillary fringe (ADR-0078 S5 / Curtis 1997 capillary layer) — the GPU mirror of the
// CPU reference `ph2d_painter_brush::diffusion::DiffusionGrid::capillary_flow`. Wicks WATER
// outward into the drier paper *beyond* the painted area AND carries a thread of pigment with
// it, so a soft, feathery, *coloured* fringe grows past the painted edge — the watercolor
// signature the harder wet-gate boundary can't make.
//
// Two coupled transports, both keyed on the same per-face capillary water flux:
//   • water — conservative permeability-weighted diffusion (divergence form).
//   • pigment — dissolved-pigment co-advection: the fraction of a cell's pigment that follows
//     the water to a drier neighbour equals the fraction of its water that leaves (flux/water);
//     GATHERED (each cell sums losses to drier neighbours + gains from wetter ones), so it's
//     mass-conserving and bit-parity-safe with the CPU.
//
// The resident `water`/`pig_a` buffers are read-only inside any one pass, so this is two
// region-scoped passes around the `water_b`/`pig_b` ping-pong scratch:
//   cs_capillary   — water (a)+pig (a) → water_b (b)+pig_b (b).
//   cs_copy_fields — water_b (b)+pig_b (b) → water (a)+pig_a (a): land both back in the
//                    canonical buffers the gate/diffuse/advect/move_water passes read next frame.
//
// Bit-parity with the CPU: every face is gathered in the SAME order (left, right, up, down).
// Border neighbour = self via the Neumann clamp ⇒ Δwater = 0 ⇒ both the water flux and the
// pigment flux for that face are 0 (and `acc + 0.0 == acc`), so the clamped 4-face GPU sum is
// bit-identical to the CPU's guarded 3/4-face sum.
//
// Shares the solver `Params` UBO; `capillary` + `capillary_branching` (ADR-0082, offset 25) sit
// in the bytes the diffuse/advect/transfer/combine shaders treat as padding, so they keep a valid
// view. Only this shader reads `capillary_branching` (the fiber-channeled fringe modulation).
//
// Pigment = PV (=8) vec4 per cell = 32 channels (+ stain, ADR-0080/0081); each face's water flux
// + the pigment co-advect fraction are per-cell scalars, applied to every channel in a loop.

const PV: u32 = 8u;

struct Params {
    width: u32,
    height: u32,
    diffusivity: f32,
    evaporation: f32,
    downhill: f32,
    flow_outward: f32,
    w_lo: f32,
    w_hi: f32,
    perm_valley: f32,
    perm_crest: f32,
    region_ox: u32,
    region_oy: u32,
    region_w: u32,
    region_h: u32,
    deposition: f32,
    deposition_dry: f32,
    granulation: f32,
    velocity: f32,
    viscosity: f32,
    drag: f32,
    pressure: f32,
    // ── ADR-0078 S5 capillary layer ──
    capillary: f32,
    capillary_mobility: f32, // pigment co-advects at this fraction of the water wick (filtering)
    sharpness: f32,          // (offset 23) read by cs_advect_correct; declared here to reach 25
    _lift: f32,              // (offset 24) read by cs_lift; declared here to reach offset 25
    // ── ADR-0082 branched capillary fringe ── fiber-channeled suppression of the face
    // conductance (offset 25 in the shared solver UBO; only this shader reads it).
    capillary_branching: f32,
    surface_tension: f32,    // (offset 26, ADR-0079-amendment-1) raises the δ_s floor → tighter fringe
}

@group(0) @binding(0) var<uniform> P: Params;
@group(0) @binding(1) var<storage, read> water_in: array<f32>;
@group(0) @binding(2) var<storage, read> paper: array<f32>;
@group(0) @binding(3) var<storage, read_write> water_out: array<f32>;
@group(0) @binding(4) var<storage, read> pig_in: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> pig_out: array<vec4<f32>>;

fn idx(x: u32, y: u32) -> u32 {
    return y * P.width + x;
}

fn pidx(cell: u32, v: u32) -> u32 { return v * (P.width * P.height) + cell; }

fn region_cell(gid: vec2<u32>) -> vec2<u32> {
    return vec2<u32>(P.region_ox + gid.x, P.region_oy + gid.y);
}
fn in_region(gid: vec2<u32>, x: u32, y: u32) -> bool {
    return gid.x < P.region_w && gid.y < P.region_h && x < P.width && y < P.height;
}

// Clamped (Neumann) neighbour indices (xm, xp, ym, yp) — border neighbour = self.
fn nb(x: u32, y: u32) -> vec4<u32> {
    let xm = select(x - 1u, 0u, x == 0u);
    let xp = select(x + 1u, P.width - 1u, x + 1u >= P.width);
    let ym = select(y - 1u, 0u, y == 0u);
    let yp = select(y + 1u, P.height - 1u, y + 1u >= P.height);
    return vec4<u32>(xm, xp, ym, yp);
}

fn perm_at(i: u32) -> f32 {
    return P.perm_valley + (P.perm_crest - P.perm_valley) * paper[i];
}

// One face's per-cell scalars (matching the CPU `face` closure): water flux `dw`, the pigment
// co-advect fraction `frac` (mobility-scaled), and `kind` (0 none, 1 = c donates → −frac·pc,
// 2 = neighbour donates → +frac·pig[ni]). Border neighbour = self ⇒ wn == wc ⇒ kind 0, dw 0.
struct FaceInfo {
    dw: f32,
    frac: f32,
    kind: u32,
    ni: u32,
}
// Curtis δ_s donor gate — MUST equal the CPU canonical
// `ph2d_painter_brush::diffusion::CAPILLARY_MIN_SATURATION` (see its doc): a face whose
// wetter side is at/below this floor carries no flux ("the wick exhausts"), so the fringe
// equilibrium is bounded and the Keep Wet envelope can't run away. Symmetric in (wc, wn)
// ⇒ anti-symmetric flux ⇒ conservation + CPU bit-parity hold.
const CAPILLARY_MIN_SAT: f32 = 0.005;
// Surface-tension pin on the wick (ADR-0079-amendment-1): RAISE the δ_s floor above the default
// `surface_tension` (0.35) so the capillary fringe also pins under Keep Wet — the FlowOutward pin
// alone left the wick spreading the wash far past the painted area. At `surface_tension ≤ 0.35`
// the floor stays at `CAPILLARY_MIN_SAT` (the validated fringe, unchanged); raising it shortens the
// fringe (the wick exhausts at a higher water level), bounding the Keep-Wet envelope.
const CAP_PIN_BASE: f32 = 0.35;
const CAP_PIN_K: f32 = 1.5;

// Where the surface-tension wick taper reaches FULL strength. Above `surface_tension = 0.35` this
// rises, so the wick is progressively suppressed below it ⇒ a SHORTER fringe; the taper is a
// smoothstep (not a hard cutoff), so the fringe stays SOFT (a gradient), which is what keeps the
// Keep-Wet + high-surface-tension rim from hardening into a crisp dark outline.
fn capillary_taper_hi() -> f32 {
    return max(
        CAPILLARY_MIN_SAT + max(0.0, P.surface_tension - CAP_PIN_BASE) * CAP_PIN_K,
        CAPILLARY_MIN_SAT + 0.02,
    );
}

fn face_info(ni: u32, wc: f32, permc: f32, cap: f32, mob: f32, paper_c: f32) -> FaceInfo {
    let wetter = max(water_in[ni], wc);
    if (wetter <= CAPILLARY_MIN_SAT) {
        return FaceInfo(0.0, 0.0, 0u, ni);
    }
    let permn = perm_at(ni);
    // SOFT surface-tension pin: ramp the wick conductance to 0 as the wetter side thins toward the
    // pinned floor, instead of a hard cutoff (ADR-0079-amendment-1). A hard floor pinned the water
    // in a step ⇒ a crisp dark rim under Keep Wet + high Surface Tension; the smoothstep keeps the
    // bounded fringe soft. At `surface_tension ≤ 0.35` the band is ~[0.005, 0.025] (the validated
    // fringe, ~unchanged); higher widens it ⇒ shorter but still-feathered fringe.
    var cond = 0.5 * (permc + permn) * smoothstep(CAPILLARY_MIN_SAT, capillary_taper_hi(), wetter);
    // Branched (fiber-channeled) capillary (ADR-0082, crest-gate re-tune 2026-06-09): the face
    // conductance is GATED by the paper fibre — crests (paper_face ≥ 0.60) keep FULL conductance
    // (fingers grow at full wick speed), valleys (≤ 0.40) close completely at branching = 1; the
    // tight smoothstep band is what makes the channels sharp (the earlier linear ×2 gain
    // suppressed crests too → the whole fringe just shrank, "muito discreta"). Suppression-only
    // (gate ∈ [0,1]) ⇒ convex-average stability + conservation hold; `capillary_branching = 0 ⇒
    // cond unchanged` (bit-identical isotropic). Mirrors the CPU `capillary_flow` `face` closure
    // (`BRANCH_GATE_LO/HI` in `diffusion.rs`); same Hermite smoothstep as the CPU helper.
    let paper_face = 0.5 * (paper_c + paper[ni]);
    let gt = clamp((paper_face - 0.40) / max(0.60 - 0.40, 1.0e-6), 0.0, 1.0);
    let gate = gt * gt * (3.0 - 2.0 * gt);
    let fiber_factor = 1.0 - P.capillary_branching * (1.0 - gate);
    cond = cond * fiber_factor;
    let wn = water_in[ni];
    var frac = 0.0;
    var kind = 0u;
    if (wc > wn) {
        frac = mob * cap * cond * (wc - wn) / wc;
        kind = 1u;
    } else if (wn > wc) {
        frac = mob * cap * cond * (wn - wc) / wn;
        kind = 2u;
    }
    return FaceInfo(cond * (wn - wc), frac, kind, ni);
}
// Apply one face's pigment contribution to channel `v` (pc_v = c's original pigment there).
fn apply_face(pn: vec4<f32>, f: FaceInfo, pc_v: vec4<f32>, v: u32) -> vec4<f32> {
    if (f.kind == 1u) {
        return pn - f.frac * pc_v;
    } else if (f.kind == 2u) {
        return pn + f.frac * pig_in[pidx(f.ni, v)];
    }
    return pn;
}

// ── Capillary water diffusion + pigment co-advection (a → b) ──
@compute @workgroup_size(8, 8, 1)
fn cs_capillary(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cell = region_cell(gid.xy);
    let x = cell.x;
    let y = cell.y;
    if (!in_region(gid.xy, x, y)) {
        return;
    }
    let c = idx(x, y);
    let wc = water_in[c];
    let permc = perm_at(c);
    let paper_c = paper[c]; // c's own paper height (for the ADR-0082 face-averaged fibre factor)
    let cap = P.capillary;
    let mob = P.capillary_mobility;
    let n = nb(x, y);
    // Faces left → right → up → down (the CPU order), each a per-cell scalar set.
    let fL = face_info(idx(n.x, y), wc, permc, cap, mob, paper_c);
    let fR = face_info(idx(n.y, y), wc, permc, cap, mob, paper_c);
    let fU = face_info(idx(x, n.z), wc, permc, cap, mob, paper_c);
    let fD = face_info(idx(x, n.w), wc, permc, cap, mob, paper_c);
    water_out[c] = wc + cap * (fL.dw + fR.dw + fU.dw + fD.dw);
    for (var v = 0u; v < PV; v = v + 1u) {
        let pc_v = pig_in[pidx(c, v)];
        var pn = pc_v;
        pn = apply_face(pn, fL, pc_v, v);
        pn = apply_face(pn, fR, pc_v, v);
        pn = apply_face(pn, fU, pc_v, v);
        pn = apply_face(pn, fD, pc_v, v);
        pig_out[pidx(c, v)] = pn;
    }
}

// ── Land the wicked fields back in the canonical buffers (b → a) ──
@compute @workgroup_size(8, 8, 1)
fn cs_copy_fields(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cell = region_cell(gid.xy);
    let x = cell.x;
    let y = cell.y;
    if (!in_region(gid.xy, x, y)) {
        return;
    }
    let c = idx(x, y);
    water_out[c] = water_in[c];
    for (var v = 0u; v < PV; v = v + 1u) {
        pig_out[pidx(c, v)] = pig_in[pidx(c, v)];
    }
}
