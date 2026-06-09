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
// Shares the solver `Params` UBO; `capillary` sits in the byte the diffuse/advect/transfer/
// combine shaders treat as padding (offset 84), so they keep a valid view.

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
    _pad1: f32,
    _pad2: f32,
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

// One face's (Δwater, Δpigment) contribution, matching the CPU `face` closure exactly.
// Border neighbour = self ⇒ wn == wc ⇒ both contributions are 0.
fn face(ni: u32, wc: f32, pc: vec3<f32>, permc: f32, cap: f32, dpig: ptr<function, vec3<f32>>) -> f32 {
    let permn = perm_at(ni);
    let cond = 0.5 * (permc + permn);
    let wn = water_in[ni];
    if (wc > wn) {
        // c donates pigment to the drier neighbour at c's concentration.
        let frac = cap * cond * (wc - wn) / wc;
        *dpig = *dpig - frac * pc;
    } else if (wn > wc) {
        // the wetter neighbour donates to c at its concentration.
        let frac = cap * cond * (wn - wc) / wn;
        *dpig = *dpig + frac * pig_in[ni].xyz;
    }
    return cond * (wn - wc);
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
    let pc = pig_in[c].xyz;
    let permc = perm_at(c);
    let cap = P.capillary;
    let n = nb(x, y);
    // Gather left → right → up → down (the CPU order). pn starts at c's pigment.
    var pn = pc;
    var acc_w = 0.0;
    acc_w += face(idx(n.x, y), wc, pc, permc, cap, &pn);
    acc_w += face(idx(n.y, y), wc, pc, permc, cap, &pn);
    acc_w += face(idx(x, n.z), wc, pc, permc, cap, &pn);
    acc_w += face(idx(x, n.w), wc, pc, permc, cap, &pn);
    water_out[c] = wc + cap * acc_w;
    pig_out[c] = vec4<f32>(pn, pig_in[c].w);
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
    pig_out[c] = pig_in[c];
}
