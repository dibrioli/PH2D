// GPU shallow-water velocity layer (ADR-0078 S3d / Curtis 1997 §3) — the GPU mirror of
// the CPU reference `ph2d_painter_brush::diffusion::DiffusionGrid::move_water` (+ the
// velocity-mode advect). Evolves a momentum-carrying velocity field `(u,v)` + pressure
// `p` so pigment advects along *real* flow → directional flow + backruns/cauliflower,
// the watercolor signatures gated diffusion can't make.
//
// Entry points (one module, focused bind groups per pass — the ping-pong pattern):
//   cs_add_forces      — accelerate `(u,v)` by the body forces (−β·∇h − λ·∇w) + viscosity
//                        (μ·∇²) − drag, gated to the wet mask, CFL-clamped. vel a → b.
//   cs_divergence      — write `∇·(u,v)` (the Jacobi RHS) from the forced velocity (b).
//   cs_clear_pressure  — seed a pressure buffer to 0 (the CPU re-seeds each move_water).
//   cs_jacobi          — one Jacobi sweep of ∇²p = divergence (order-independent →
//                        GPU-parallel + bit-parity); run RELAX_ITERS times, ping-pong.
//   cs_project         — subtract `pressure·∇p` → the (near-)divergence-free velocity. b → a.
//   cs_advect_velocity — upwind pigment transport along `clamp(velocity·(u,v), ±0.5)`
//                        (the GATHER form of the CPU scatter, atomics-free). pig b → a.
//
// Velocity is `vec2<f32>` (std430 stride 8). All passes are region-scoped (ADR-0078 S1)
// and share the solver `Params` UBO (the 4 trailing S3d fields live in the bytes the
// diffuse/advect/transfer/combine shaders treat as padding, so those stay byte-compatible).

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
    // ── ADR-0078 S3d shallow-water layer ──
    velocity: f32,
    viscosity: f32,
    drag: f32,
    pressure: f32,
    capillary: f32, // ADR-0078 S5 — read by capillary.wgsl, not here (kept for layout parity)
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(0) var<uniform> P: Params;
@group(0) @binding(1) var<storage, read> water: array<f32>;
@group(0) @binding(2) var<storage, read> paper: array<f32>;
@group(0) @binding(3) var<storage, read> vel_in: array<vec2<f32>>;
@group(0) @binding(4) var<storage, read_write> vel_out: array<vec2<f32>>;
@group(0) @binding(5) var<storage, read> pressure_in: array<f32>;
@group(0) @binding(6) var<storage, read_write> pressure_out: array<f32>;
@group(0) @binding(7) var<storage, read_write> divergence: array<f32>;
@group(0) @binding(8) var<storage, read> pig_in: array<vec4<f32>>;
@group(0) @binding(9) var<storage, read_write> pig_out: array<vec4<f32>>;

fn idx(x: u32, y: u32) -> u32 {
    return y * P.width + x;
}

// Map a region-local invocation to an absolute cell (ADR-0078 S1).
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

// Same smoothstep as `ph2d_painter_brush` (matches the CPU wet mask / gate).
fn smoothstep_gate(lo: f32, hi: f32, x: f32) -> f32 {
    let t = clamp((x - lo) / max(hi - lo, 1.0e-6), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

// Perm-weighted pigment gate (the advect transport gate) = smoothstep(water)·perm(paper).
fn gate(x: u32, y: u32) -> f32 {
    let i = idx(x, y);
    let perm = P.perm_valley + (P.perm_crest - P.perm_valley) * paper[i];
    return smoothstep_gate(P.w_lo, P.w_hi, water[i]) * perm;
}

// ── MoveWater step 1: accelerate the velocity (vel_in a → vel_out b) ──
@compute @workgroup_size(8, 8, 1)
fn cs_add_forces(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cell = region_cell(gid.xy);
    let x = cell.x;
    let y = cell.y;
    if (!in_region(gid.xy, x, y)) {
        return;
    }
    let i = idx(x, y);
    // Pure wetness mask (NOT the perm-weighted gate): velocity exists only where wet.
    let wet = smoothstep_gate(P.w_lo, P.w_hi, water[i]);
    if (wet <= 1.0e-4) {
        vel_out[i] = vec2<f32>(0.0, 0.0); // dry → zero (the out buffer is not pre-cleared)
        return;
    }
    let n = nb(x, y);
    // Permeability gates the body forces — FlowOutward (−λ·∇w) + paper-slope channeling
    // (−β·∇h, `downhill`). A less permeable patch generates less flow, so the Perm controls
    // visibly modulate the velocity (not just the diffusion gate), and the paper-textured
    // velocity is what `viscosity` then smooths (ADR-0079 — see the CPU `add_forces` doc).
    let perm = P.perm_valley + (P.perm_crest - P.perm_valley) * paper[i];
    let dhx = paper[idx(n.y, y)] - paper[idx(n.x, y)];
    let dhy = paper[idx(x, n.w)] - paper[idx(x, n.z)];
    let dwx = water[idx(n.y, y)] - water[idx(n.x, y)];
    let dwy = water[idx(x, n.w)] - water[idx(x, n.z)];
    let vc = vel_in[i];
    // Neumann velocity Laplacian (border neighbour = self via the clamp).
    let lap = vel_in[idx(n.x, y)] + vel_in[idx(n.y, y)] + vel_in[idx(x, n.z)]
        + vel_in[idx(x, n.w)] - 4.0 * vc;
    let force = vec2<f32>(
        P.downhill * 0.5 * dhx + P.flow_outward * 0.5 * dwx,
        P.downhill * 0.5 * dhy + P.flow_outward * 0.5 * dwy,
    );
    var uv = vc - perm * force + P.viscosity * lap;
    uv = uv * (1.0 - P.drag);
    vel_out[i] = clamp(uv * wet, vec2<f32>(-0.5, -0.5), vec2<f32>(0.5, 0.5));
}

// ── MoveWater step 2a: divergence of the forced velocity (vel_in b → divergence) ──
@compute @workgroup_size(8, 8, 1)
fn cs_divergence(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cell = region_cell(gid.xy);
    let x = cell.x;
    let y = cell.y;
    if (!in_region(gid.xy, x, y)) {
        return;
    }
    let n = nb(x, y);
    divergence[idx(x, y)] = 0.5
        * ((vel_in[idx(n.y, y)].x - vel_in[idx(n.x, y)].x)
            + (vel_in[idx(x, n.w)].y - vel_in[idx(x, n.z)].y));
}

// Seed a pressure buffer to 0 (region-scoped) — the CPU re-seeds pressure each move_water.
@compute @workgroup_size(8, 8, 1)
fn cs_clear_pressure(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cell = region_cell(gid.xy);
    let x = cell.x;
    let y = cell.y;
    if (!in_region(gid.xy, x, y)) {
        return;
    }
    pressure_out[idx(x, y)] = 0.0;
}

// ── MoveWater step 2b: one Jacobi sweep of ∇²p = divergence (pressure_in → pressure_out) ──
@compute @workgroup_size(8, 8, 1)
fn cs_jacobi(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cell = region_cell(gid.xy);
    let x = cell.x;
    let y = cell.y;
    if (!in_region(gid.xy, x, y)) {
        return;
    }
    let n = nb(x, y);
    pressure_out[idx(x, y)] = 0.25
        * (pressure_in[idx(n.x, y)] + pressure_in[idx(n.y, y)] + pressure_in[idx(x, n.z)]
            + pressure_in[idx(x, n.w)] - divergence[idx(x, y)]);
}

// ── MoveWater step 2c: subtract pressure·∇p (vel_in b → vel_out a) ──
@compute @workgroup_size(8, 8, 1)
fn cs_project(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cell = region_cell(gid.xy);
    let x = cell.x;
    let y = cell.y;
    if (!in_region(gid.xy, x, y)) {
        return;
    }
    let i = idx(x, y);
    let n = nb(x, y);
    let gradx = pressure_in[idx(n.y, y)] - pressure_in[idx(n.x, y)];
    let grady = pressure_in[idx(x, n.w)] - pressure_in[idx(x, n.z)];
    vel_out[i] = vel_in[i] - P.pressure * 0.5 * vec2<f32>(gradx, grady);
}

// Velocity-mode flow: `clamp(velocity·(u,v), ±0.5)`, gated (dry cell ⇒ no transport),
// mirroring the CPU advect's velocity branch. `vel_in` here is the projected velocity (a).
fn flow_v(x: u32, y: u32) -> vec2<f32> {
    if (gate(x, y) <= 1.0e-4) {
        return vec2<f32>(0.0, 0.0);
    }
    let v = P.velocity * vel_in[idx(x, y)];
    return clamp(v, vec2<f32>(-0.5, -0.5), vec2<f32>(0.5, 0.5));
}

// ── Pigment transport along the velocity field (GATHER form; pig_in b → pig_out a) ──
@compute @workgroup_size(8, 8, 1)
fn cs_advect_velocity(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cell = region_cell(gid.xy);
    let x = cell.x;
    let y = cell.y;
    if (!in_region(gid.xy, x, y)) {
        return;
    }
    let c = idx(x, y);
    let pc = pig_in[c].xyz;
    let fc = flow_v(x, y);
    var out = pc;
    // Outflow from c into existing downstream neighbours (the CPU "push").
    if (fc.x > 0.0 && x + 1u < P.width) { out -= fc.x * pc; }
    if (fc.x < 0.0 && x > 0u) { out -= (-fc.x) * pc; }
    if (fc.y > 0.0 && y + 1u < P.height) { out -= fc.y * pc; }
    if (fc.y < 0.0 && y > 0u) { out -= (-fc.y) * pc; }
    // Inflow: any neighbour whose flow points AT c contributes its push.
    if (x > 0u) {
        let fl = flow_v(x - 1u, y);
        if (fl.x > 0.0) { out += fl.x * pig_in[idx(x - 1u, y)].xyz; }
    }
    if (x + 1u < P.width) {
        let fr = flow_v(x + 1u, y);
        if (fr.x < 0.0) { out += (-fr.x) * pig_in[idx(x + 1u, y)].xyz; }
    }
    if (y > 0u) {
        let fu = flow_v(x, y - 1u);
        if (fu.y > 0.0) { out += fu.y * pig_in[idx(x, y - 1u)].xyz; }
    }
    if (y + 1u < P.height) {
        let fd = flow_v(x, y + 1u);
        if (fd.y < 0.0) { out += (-fd.y) * pig_in[idx(x, y + 1u)].xyz; }
    }
    pig_out[c] = vec4<f32>(out, pig_in[c].w);
}
