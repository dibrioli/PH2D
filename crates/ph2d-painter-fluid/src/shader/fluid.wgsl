// Live watercolor wet-on-wet solver — GPU mirror of
// `ph2d_painter_brush::diffusion` (gated diffusion-advection, Curtis 1997).
// ADR-0049-amendment-1. Two compute passes the driver ping-pongs each substep:
//   cs_diffuse — conservative gated Laplacian of pigment (the bloom).
//   cs_advect  — gated upwind transport along flow = −β·∇h − λ·∇w
//                (downhill paper channeling + the FlowOutward wet→dry push).
//
// Both are GATHER kernels (each output cell reads itself + 4 neighbours, writes
// only itself), so they are order-independent + atomics-free. The CPU advect
// SCATTERS (pushes to the downstream neighbour); here cell c instead pulls the
// net flux: it loses |f_c|·p_c into existing downstream neighbours and gains
// f_n·p_n from any neighbour whose flow points at c. Same mass transfer,
// parallel-safe.
//
// **Pigment = PV (=8) `vec4<f32>` per cell = 32 channels (+ stain)** (ADR-0080/0081): 24
// spectral K/S bands + 3 err + 1 mass (vec4[6] = (err.xyz, mass)) + 1 stain (vec4[7].x) + 3
// pad. Every pass loops the PV vec4 doing the SAME per-component arithmetic the CPU `[f32;32]`
// loop does (stain transports linearly with the rest), so the
// multi-pigment mix transports linearly and the parity stays bit-exact lane-for-lane.
// Water/paper are `f32` arrays. Cell index = y*width + x; channels at `c*PV + v`.

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
    // Region-scoped dispatch (4K real-time arch / ADR-0078 S1): the diffuse/advect/
    // evaporate kernels only WRITE cells in [region_ox, region_ox+region_w) ×
    // [region_oy, region_oy+region_h) — the wet envelope padded so it always ⊇ the
    // composite region (so the visible field is fully stepped; cells beyond are never
    // composited). Full-grid = (0, 0, width, height) reproduces the un-scoped pass.
    // Neighbour reads stay ABSOLUTE (the buffer is full-size), so the gather is exact.
    region_ox: u32,
    region_oy: u32,
    region_w: u32,
    region_h: u32,
    // Pigment-deposition layer (ADR-0078 S3) — read by `cs_transfer` (separate
    // module, same UBO layout); the diffuse/advect/evaporate kernels ignore them.
    deposition: f32,
    deposition_dry: f32,
    granulation: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

fn region_cell(gid: vec2<u32>) -> vec2<u32> {
    return vec2<u32>(P.region_ox + gid.x, P.region_oy + gid.y);
}
fn in_region(gid: vec2<u32>, x: u32, y: u32) -> bool {
    return gid.x < P.region_w && gid.y < P.region_h && x < P.width && y < P.height;
}

// `water` is read_write so `cs_evaporate` can dry it; `cs_diffuse`/`cs_advect`
// only read it (the gate + flow are computed from the pre-evaporation water,
// then `cs_evaporate` runs last each step — matching the CPU `step` order).
@group(0) @binding(0) var<uniform> P: Params;
@group(0) @binding(1) var<storage, read_write> water: array<f32>;
@group(0) @binding(2) var<storage, read> paper: array<f32>;
@group(0) @binding(3) var<storage, read> pig_in: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> pig_out: array<vec4<f32>>;

fn idx(x: u32, y: u32) -> u32 {
    return y * P.width + x;
}

// Wet gate g = smoothstep(w_lo, w_hi, water) · permeability(paper). 0 on dry
// paper / crests ⇒ pigment freezes there (the bloom stops, the stroke "dries").
fn gate(x: u32, y: u32) -> f32 {
    let i = idx(x, y);
    let perm = P.perm_valley + (P.perm_crest - P.perm_valley) * paper[i];
    return smoothstep(P.w_lo, P.w_hi, water[i]) * perm;
}

// Flow field f = −β·∇h − λ·∇w (gated, CFL-clamped to ±0.5 cell/step), matching
// the CPU advect. Border gradients use clamped (saturating) neighbours.
fn flow(x: u32, y: u32) -> vec2<f32> {
    let g = gate(x, y);
    if (g <= 1.0e-4) {
        return vec2<f32>(0.0, 0.0);
    }
    let xm = select(x - 1u, 0u, x == 0u);
    let xp = select(x + 1u, P.width - 1u, x + 1u >= P.width);
    let ym = select(y - 1u, 0u, y == 0u);
    let yp = select(y + 1u, P.height - 1u, y + 1u >= P.height);
    let dhx = paper[idx(xp, y)] - paper[idx(xm, y)];
    let dhy = paper[idx(x, yp)] - paper[idx(x, ym)];
    let dwx = water[idx(xp, y)] - water[idx(xm, y)];
    let dwy = water[idx(x, yp)] - water[idx(x, ym)];
    let fx = clamp(g * (-P.downhill * 0.5 * dhx - P.flow_outward * 0.5 * dwx), -0.5, 0.5);
    let fy = clamp(g * (-P.downhill * 0.5 * dhy - P.flow_outward * 0.5 * dwy), -0.5, 0.5);
    return vec2<f32>(fx, fy);
}

@compute @workgroup_size(8, 8, 1)
fn cs_diffuse(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cell = region_cell(gid.xy);
    let x = cell.x;
    let y = cell.y;
    if (!in_region(gid.xy, x, y)) {
        return;
    }
    let c = idx(x, y);
    let gc = gate(x, y);
    // Per-cell face conductances + neighbour cell indices (shared by all PV channels).
    let has_l = x > 0u;
    let has_r = x + 1u < P.width;
    let has_u = y > 0u;
    let has_d = y + 1u < P.height;
    var nl = 0u; var nr = 0u; var nu = 0u; var nd = 0u;
    var cl = 0.0; var cr = 0.0; var cu = 0.0; var cd = 0.0;
    if (has_l) { nl = idx(x - 1u, y); cl = 0.5 * (gc + gate(x - 1u, y)); }
    if (has_r) { nr = idx(x + 1u, y); cr = 0.5 * (gc + gate(x + 1u, y)); }
    if (has_u) { nu = idx(x, y - 1u); cu = 0.5 * (gc + gate(x, y - 1u)); }
    if (has_d) { nd = idx(x, y + 1u); cd = 0.5 * (gc + gate(x, y + 1u)); }
    for (var v = 0u; v < PV; v = v + 1u) {
        let pc = pig_in[c * PV + v];
        var acc = vec4<f32>(0.0);
        if (has_l) { acc += cl * (pig_in[nl * PV + v] - pc); }
        if (has_r) { acc += cr * (pig_in[nr * PV + v] - pc); }
        if (has_u) { acc += cu * (pig_in[nu * PV + v] - pc); }
        if (has_d) { acc += cd * (pig_in[nd * PV + v] - pc); }
        pig_out[c * PV + v] = pc + P.diffusivity * acc;
    }
}

@compute @workgroup_size(8, 8, 1)
fn cs_advect(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cell = region_cell(gid.xy);
    let x = cell.x;
    let y = cell.y;
    if (!in_region(gid.xy, x, y)) {
        return;
    }
    let c = idx(x, y);
    let fc = flow(x, y);
    // Neighbour flows (gathered inflow), computed once for all PV channels.
    let has_l = x > 0u;
    let has_r = x + 1u < P.width;
    let has_u = y > 0u;
    let has_d = y + 1u < P.height;
    var fl = vec2<f32>(0.0); var fr = vec2<f32>(0.0);
    var fu = vec2<f32>(0.0); var fd = vec2<f32>(0.0);
    if (has_l) { fl = flow(x - 1u, y); }
    if (has_r) { fr = flow(x + 1u, y); }
    if (has_u) { fu = flow(x, y - 1u); }
    if (has_d) { fd = flow(x, y + 1u); }
    for (var v = 0u; v < PV; v = v + 1u) {
        let pc = pig_in[c * PV + v];
        var out = pc;
        // Outflow from c into existing downstream neighbours (the CPU "push").
        if (fc.x > 0.0 && has_r) { out -= fc.x * pc; }
        if (fc.x < 0.0 && has_l) { out -= (-fc.x) * pc; }
        if (fc.y > 0.0 && has_d) { out -= fc.y * pc; }
        if (fc.y < 0.0 && has_u) { out -= (-fc.y) * pc; }
        // Inflow: any neighbour whose flow points AT c contributes its push.
        if (has_l && fl.x > 0.0) { out += fl.x * pig_in[idx(x - 1u, y) * PV + v]; }
        if (has_r && fr.x < 0.0) { out += (-fr.x) * pig_in[idx(x + 1u, y) * PV + v]; }
        if (has_u && fu.y > 0.0) { out += fu.y * pig_in[idx(x, y - 1u) * PV + v]; }
        if (has_d && fd.y < 0.0) { out += (-fd.y) * pig_in[idx(x, y + 1u) * PV + v]; }
        pig_out[c * PV + v] = out;
    }
}

// Drying: water lost per step. As it falls below `w_lo` the gate closes and the
// pigment freezes (the bloom ends). Runs LAST each step (after diffuse+advect
// read the pre-evaporation water), mirroring the CPU `step`.
@compute @workgroup_size(8, 8, 1)
fn cs_evaporate(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cell = region_cell(gid.xy);
    let x = cell.x;
    let y = cell.y;
    if (!in_region(gid.xy, x, y)) {
        return;
    }
    let i = idx(x, y);
    water[i] = max(water[i] - P.evaporation, 0.0);
}

// Additive dab deposit (W15.3 resident path): `pig_a += deposit` over all PV channels.
@compute @workgroup_size(8, 8, 1)
fn cs_deposit(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    if (x >= P.width || y >= P.height) {
        return;
    }
    let c = idx(x, y);
    for (var v = 0u; v < PV; v = v + 1u) {
        pig_out[c * PV + v] = pig_out[c * PV + v] + pig_in[c * PV + v];
    }
}
