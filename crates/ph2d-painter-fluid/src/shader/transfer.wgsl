// GPU pigment-deposition pass — `cs_transfer`, the GPU mirror of the CPU reference
// `ph2d_painter_brush::diffusion::DiffusionGrid::transfer_pigment` (ADR-0078 S3 /
// Curtis 1997 §4.2). Freezes a fraction of the FLOWING pigment into the DEPOSITED
// layer each substep:
//   rate = (deposition + deposition_dry·(1 − smoothstep(w_lo,w_hi,water))) ·
//          (1 + granulation·(1 − paper))
// clamped to [0,1]; `deposited += rate·flowing; flowing -= rate·flowing`. Mass is
// conserved (the gram leaves `flowing`, lands in `deposited`); deposited pigment is
// frozen (never diffuses/advects). `deposition_dry` drives EDGE-DARKENING (the rim
// dries first → freezes first); `granulation` drives GRANULATION (more in the tooth
// valleys). A no-op while the deposition params are 0 (the shipped look is untouched).
//
// Own bind group (flowing + deposited both read_write) — distinct from the solver's
// ping-pong layout, so it's a separate module sharing only the `Params` UBO.
// Region-scoped (ADR-0078 S1) exactly like the diffuse/advect/evaporate kernels.
//
// Pigment = PV (=8) vec4 per cell = 32 channels (+ stain, ADR-0080/0081); the same
// scalar `rate` moves every channel flowing→deposited (mass + K/S + err all proportional).

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
    // Offsets 17..23 in the shared solver UBO are velocity/viscosity/drag/pressure/
    // capillary/capillary_mobility/sharpness — this module reads none of them, so they're
    // padding here. `lift` (offset 24, ADR-0081) is read by `cs_lift` below.
    _p17: f32,
    _p18: f32,
    _p19: f32,
    _p20: f32,
    _p21: f32,
    _p22: f32,
    _p23: f32,
    lift: f32,
}

@group(0) @binding(0) var<uniform> P: Params;
@group(0) @binding(1) var<storage, read> water: array<f32>;
@group(0) @binding(2) var<storage, read> paper: array<f32>;
@group(0) @binding(3) var<storage, read_write> flowing: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> deposited: array<vec4<f32>>;
// ADR-0084 backdrop-lift donor + accumulator (read by `cs_lift` only; `cs_transfer` ignores
// them — it shares the layout but never touches bindings 5/6). `lift_source` is the
// downsampled canvas backdrop ("dry paint available to lift", same 32-ch K–M layout as
// `flowing`/`deposited`), depleted into `flowing` in wet cells; `lifted_frac` accumulates the
// fraction removed per cell (the compositor drops the backdrop alpha by it). All-zero while the
// lift is off / unseeded → the backdrop branch is a no-op (pre-ADR-0084 bit-identical).
@group(0) @binding(5) var<storage, read_write> lift_source: array<vec4<f32>>;
@group(0) @binding(6) var<storage, read_write> lifted_frac: array<f32>;

// Same smoothstep as `ph2d_painter_brush` (matches the CPU `dry` factor).
fn smoothstep_gate(lo: f32, hi: f32, x: f32) -> f32 {
    let t = clamp((x - lo) / max(hi - lo, 1.0e-6), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

@compute @workgroup_size(8, 8, 1)
fn cs_transfer(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = P.region_ox + gid.x;
    let y = P.region_oy + gid.y;
    if (gid.x >= P.region_w || gid.y >= P.region_h || x >= P.width || y >= P.height) {
        return;
    }
    let i = y * P.width + x;
    let dry = 1.0 - smoothstep_gate(P.w_lo, P.w_hi, water[i]);
    let gran = 1.0 + P.granulation * (1.0 - paper[i]);
    let rate = clamp((P.deposition + P.deposition_dry * dry) * gran, 0.0, 1.0);
    if (rate <= 0.0) {
        return;
    }
    for (var v = 0u; v < PV; v = v + 1u) {
        let moved = rate * flowing[i * PV + v];
        flowing[i * PV + v] = flowing[i * PV + v] - moved;
        deposited[i * PV + v] = deposited[i * PV + v] + moved;
    }
}

// GPU lift pass — `cs_lift`, the GPU mirror of the CPU reference
// `ph2d_painter_brush::diffusion::DiffusionGrid::lift_pigment` (ADR-0081). The inverse of
// `cs_transfer`: in WET cells, re-mobilizes a fraction of the DEPOSITED (dried) pigment back
// into the FLOWING layer, so re-wetting dried paint reactivates it (it blooms/moves again):
//   stain = clamp(deposited[PIG_STAIN] / deposited[PIG_MASS], 0, 1)   (the deposited pigment's own)
//   rate  = clamp(lift · smoothstep(w_lo,w_hi,water) · (1 − stain), 0, 1)
//   flowing += rate·deposited; deposited -= rate·deposited
// A staining pigment (stain→1) resists; a sedimentary one (stain→0) lifts. Mass-conserving (the
// gram leaves `deposited`, lands in `flowing`). A no-op while `lift == 0` (the dispatch is gated
// on it solver-side) or the cell has no deposited mass. Same bindings as `cs_transfer` (flowing =
// pig_a, deposited both read_write); runs BEFORE diffuse (the CPU `step` order), so the lifted
// pigment participates this substep.
//
// ADR-0084 — this kernel ALSO runs the backdrop-lift branch (`lift_source → flowing`) in the SAME
// dispatch. The two branches are INDEPENDENT non-returning blocks, mirroring the CPU `lift_pigment`
// (deposited loop) then `lift_from_backdrop`: an early `return` on the deposited branch would skip
// the backdrop branch (and vice versa). Identical rate law; the backdrop branch additionally
// accumulates `lifted_frac` (the compositor reads it). Both are no-ops on an unseeded / zero cell.
@compute @workgroup_size(8, 8, 1)
fn cs_lift(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = P.region_ox + gid.x;
    let y = P.region_oy + gid.y;
    if (gid.x >= P.region_w || gid.y >= P.region_h || x >= P.width || y >= P.height) {
        return;
    }
    let i = y * P.width + x;
    let wet = smoothstep_gate(P.w_lo, P.w_hi, water[i]);
    // Branch 1 — deposited lift (ADR-0081): re-mobilize this stroke's frozen pigment.
    let dep_mass = deposited[i * PV + 6u].w;       // PIG_MASS=27 → vec4[6].w
    if (dep_mass > 1.0e-6) {
        let stain = clamp(deposited[i * PV + 7u].x / dep_mass, 0.0, 1.0); // PIG_STAIN=28 → vec4[7].x
        let rate = clamp(P.lift * wet * (1.0 - stain), 0.0, 1.0);
        if (rate > 0.0) {
            for (var v = 0u; v < PV; v = v + 1u) {
                let moved = rate * deposited[i * PV + v];
                deposited[i * PV + v] = deposited[i * PV + v] - moved;
                flowing[i * PV + v] = flowing[i * PV + v] + moved;
            }
        }
    }
    // Branch 2 — backdrop lift (ADR-0084): re-mobilize the dry-paint reservoir (downsampled
    // backdrop) into the flowing layer + accumulate the cumulative lifted fraction. Same rate law;
    // `lift_source`'s own staining ratio resists (0 for a backdrop seed → dry paint lifts freely).
    let src_mass = lift_source[i * PV + 6u].w;     // PIG_MASS=27 → vec4[6].w
    if (src_mass > 1.0e-6) {
        let stain_src = clamp(lift_source[i * PV + 7u].x / src_mass, 0.0, 1.0); // PIG_STAIN=28 → vec4[7].x
        let rate = clamp(P.lift * wet * (1.0 - stain_src), 0.0, 1.0);
        if (rate > 0.0) {
            // Only LIFT_BLEED_KEEP (0.25, ADR-0084 `diffusion.rs`) of the lifted dry paint bleeds
            // into the wash; the rest is REMOVED (carried off the brush) so the spot LIGHTENS rather
            // than concentrating into a dark merged-mass glaze. `lifted_frac` drives the compositor
            // backdrop-alpha drop (the paper pigment lightens by the FULL lifted amount).
            for (var v = 0u; v < PV; v = v + 1u) {
                let moved = rate * lift_source[i * PV + v];
                lift_source[i * PV + v] = lift_source[i * PV + v] - moved;
                flowing[i * PV + v] = flowing[i * PV + v] + moved * 0.25;
            }
            lifted_frac[i] = lifted_frac[i] + rate * (1.0 - lifted_frac[i]);
        }
    }
}
