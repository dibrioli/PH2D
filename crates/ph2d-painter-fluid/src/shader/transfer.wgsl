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
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(0) var<uniform> P: Params;
@group(0) @binding(1) var<storage, read> water: array<f32>;
@group(0) @binding(2) var<storage, read> paper: array<f32>;
@group(0) @binding(3) var<storage, read_write> flowing: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> deposited: array<vec4<f32>>;

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
    let moved = rate * flowing[i].xyz;
    flowing[i] = vec4<f32>(flowing[i].xyz - moved, flowing[i].w);
    deposited[i] = vec4<f32>(deposited[i].xyz + moved, deposited[i].w);
}
