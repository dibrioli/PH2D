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
// Pigment is `vec4<f32>` (xyz = linear-RGB mass, w unused) to avoid the std430
// vec3 stride trap. Water/paper are `f32` arrays. Index = y*width + x.

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
    _pad0: f32,
    _pad1: f32,
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
    let x = gid.x;
    let y = gid.y;
    if (x >= P.width || y >= P.height) {
        return;
    }
    let c = idx(x, y);
    let gc = gate(x, y);
    let pc = pig_in[c].xyz;
    var acc = vec3<f32>(0.0, 0.0, 0.0);
    if (x > 0u) {
        let n = idx(x - 1u, y);
        acc += 0.5 * (gc + gate(x - 1u, y)) * (pig_in[n].xyz - pc);
    }
    if (x + 1u < P.width) {
        let n = idx(x + 1u, y);
        acc += 0.5 * (gc + gate(x + 1u, y)) * (pig_in[n].xyz - pc);
    }
    if (y > 0u) {
        let n = idx(x, y - 1u);
        acc += 0.5 * (gc + gate(x, y - 1u)) * (pig_in[n].xyz - pc);
    }
    if (y + 1u < P.height) {
        let n = idx(x, y + 1u);
        acc += 0.5 * (gc + gate(x, y + 1u)) * (pig_in[n].xyz - pc);
    }
    pig_out[c] = vec4<f32>(pc + P.diffusivity * acc, pig_in[c].w);
}

@compute @workgroup_size(8, 8, 1)
fn cs_advect(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    if (x >= P.width || y >= P.height) {
        return;
    }
    let c = idx(x, y);
    let pc = pig_in[c].xyz;
    let fc = flow(x, y);
    var out = pc;
    // Outflow from c into existing downstream neighbours (the CPU "push").
    if (fc.x > 0.0 && x + 1u < P.width) { out -= fc.x * pc; }
    if (fc.x < 0.0 && x > 0u) { out -= (-fc.x) * pc; }
    if (fc.y > 0.0 && y + 1u < P.height) { out -= fc.y * pc; }
    if (fc.y < 0.0 && y > 0u) { out -= (-fc.y) * pc; }
    // Inflow: any neighbour whose flow points AT c contributes its push.
    if (x > 0u) {
        let fl = flow(x - 1u, y);
        if (fl.x > 0.0) { out += fl.x * pig_in[idx(x - 1u, y)].xyz; }
    }
    if (x + 1u < P.width) {
        let fr = flow(x + 1u, y);
        if (fr.x < 0.0) { out += (-fr.x) * pig_in[idx(x + 1u, y)].xyz; }
    }
    if (y > 0u) {
        let fu = flow(x, y - 1u);
        if (fu.y > 0.0) { out += fu.y * pig_in[idx(x, y - 1u)].xyz; }
    }
    if (y + 1u < P.height) {
        let fd = flow(x, y + 1u);
        if (fd.y < 0.0) { out += (-fd.y) * pig_in[idx(x, y + 1u)].xyz; }
    }
    pig_out[c] = vec4<f32>(out, pig_in[c].w);
}

// Drying: water lost per step. As it falls below `w_lo` the gate closes and the
// pigment freezes (the bloom ends). Runs LAST each step (after diffuse+advect
// read the pre-evaporation water), mirroring the CPU `step`.
@compute @workgroup_size(8, 8, 1)
fn cs_evaporate(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    if (x >= P.width || y >= P.height) {
        return;
    }
    let i = idx(x, y);
    water[i] = max(water[i] - P.evaporation, 0.0);
}
