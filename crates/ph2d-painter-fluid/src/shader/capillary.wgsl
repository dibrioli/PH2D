// GPU capillary fringe (ADR-0078 S5 / Curtis 1997 capillary layer) — the GPU mirror of the
// CPU reference `ph2d_painter_brush::diffusion::DiffusionGrid::capillary_flow`. Wicks WATER
// outward from wet cells into the drier paper *beyond* the painted area, so the wet gate (and
// then the pigment) creeps into a soft, feathery fringe — the watercolor edge the harder
// wet-gate boundary can't make.
//
// A conservative, permeability-weighted diffusion of the water field (divergence form). The
// resident `water` buffer is read-only inside any one pass, so this is two region-scoped
// passes around a ping-pong scratch (`water_b`):
//   cs_capillary  — water (a) → water_b (b): `water_b = water + capillary·Σ_face cond·(Δwater)`.
//   cs_copy_water  — water_b (b) → water (a): land the result back in the canonical buffer the
//                    gate/diffuse/advect/move_water passes read next frame.
//
// Bit-parity with the CPU: the face flux is accumulated in the SAME order (left, right, up,
// down). Border neighbour = self via the Neumann clamp ⇒ `(water_self − water_self) = 0`
// contribution = the CPU's skipped out-of-bounds face (and `acc + 0.0 == acc`), so the
// clamped 4-face GPU sum is bit-identical to the CPU's guarded 3/4-face sum.
//
// Shares the solver `Params` UBO; `capillary` sits in the byte the diffuse/advect/transfer/
// combine shaders treat as padding (offset 84), so they keep a valid view — only this shader
// (and the matching CPU field) read it.

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

// ── Capillary diffusion of the water field (water a → water_b b) ──
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
    let n = nb(x, y);
    // Per-face conductive flux, accumulated left → right → up → down (the CPU order).
    let il = idx(n.x, y);
    let ir = idx(n.y, y);
    let iu = idx(x, n.z);
    let id = idx(x, n.w);
    var acc = 0.0;
    acc += 0.5 * (permc + perm_at(il)) * (water_in[il] - wc);
    acc += 0.5 * (permc + perm_at(ir)) * (water_in[ir] - wc);
    acc += 0.5 * (permc + perm_at(iu)) * (water_in[iu] - wc);
    acc += 0.5 * (permc + perm_at(id)) * (water_in[id] - wc);
    water_out[c] = wc + P.capillary * acc;
}

// ── Land the wicked field back in the canonical water buffer (water_b b → water a) ──
@compute @workgroup_size(8, 8, 1)
fn cs_copy_water(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cell = region_cell(gid.xy);
    let x = cell.x;
    let y = cell.y;
    if (!in_region(gid.xy, x, y)) {
        return;
    }
    let c = idx(x, y);
    water_out[c] = water_in[c];
}
