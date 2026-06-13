// cs_step — the WHOLE physics of the minimal watercolor core (ADR-0086), one gather
// kernel per substep. Ping-pong: (water_in, pig_in) → (water_out, pig_out).
//
//   gated diffusion (bloom) + FlowOutward −λ∇w (edge-darkening) + evaporation
//
// Conservative gather (face quantities symmetric in the cell pair ⇒ pigment mass
// conserved). Deposition is IMPLICIT — a drying cell's wet-gate closes, freezing its
// pigment in place.
//
// POSITIVITY (fixes the keep-wet/evap-0 checkerboard): diffusion and advection share ONE
// CFL budget. A 5-point cell can shed at most `4·(D + |v|)·pc` per substep (all 4 faces
// outgoing); if that exceeds `pc` the cell goes NEGATIVE and the `max(·,0)` clamp snaps it
// to zero (white) while a neighbour keeps the mass (red) — the dithered checkerboard. The
// old code clamped D≤0.25 and |v|≤0.25 *independently* (4·0.25 + 4·0.25 = 2.0 ≫ 1). We cap
// the SUM: `4·(D_MAX + V_MAX) = 4·(0.20 + 0.03) = 0.92 < 1` ⇒ no cell can go negative ⇒ no
// checkerboard, in every regime, by construction.

const D_MAX: f32 = 0.20; // diffusion budget (≤ 0.25 heat CFL; leaves headroom for advection)
const V_MAX: f32 = 0.03; // advective face-speed budget (the rest of the CFL budget)

struct Params {
    width: u32,
    height: u32,
    region_ox: u32,
    region_oy: u32,
    region_w: u32,
    region_h: u32,
    diffusivity: f32,    // D ≤ 0.25 — bloom rate
    flow_outward: f32,   // λ — edge-darkening (pigment drift toward drier cells)
    evaporation: f32,    // per-step water loss
    w_lo: f32,           // wet-gate band (transport only in wet cells)
    w_hi: f32,
    perm_valley: f32,    // paper permeability at a valley (paper=0) …
    perm_crest: f32,     // … and a crest (paper=1)
    granulation: f32,    // reserved (v1.1) — unused in v1
    _pad0: f32,
    _pad1: f32,
}

@group(0) @binding(0) var<uniform> P: Params;
@group(0) @binding(1) var<storage, read> water_in: array<f32>;
@group(0) @binding(2) var<storage, read> paper: array<f32>;
@group(0) @binding(3) var<storage, read> pig_in: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> water_out: array<f32>;
@group(0) @binding(5) var<storage, read_write> pig_out: array<vec4<f32>>;

fn idx(x: u32, y: u32) -> u32 { return y * P.width + x; }

// Wet-gate × paper permeability: transport happens only where the paper is wet.
fn gate(w: f32, pap: f32) -> f32 {
    let perm = mix(P.perm_valley, P.perm_crest, pap);
    let t = clamp((w - P.w_lo) / max(P.w_hi - P.w_lo, 1.0e-6), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t) * perm;
}

// Net Δpigment for the centre cell `c` contributed by one face to neighbour `n`.
//  • diffusion: gather D·gf·(pn−pc)         (bloom; gf = min(gc,gn) ⇒ symmetric ⇒ conserved)
//  • advection: −flux, flux = gf·v·upwind   (v = λ·(wc−wn) wet→dry, clamped; donor-cell upwind)
// Both face quantities are anti-symmetric in (c,n) ⇒ pigment mass is conserved exactly
// (when v is unclamped; the clamp only triggers at extreme λ and is mass-safe via max(0)).
fn face(gc: f32, wc: f32, pc: vec4<f32>, gn: f32, wn: f32, pn: vec4<f32>) -> vec4<f32> {
    let gf = min(gc, gn);
    let d = min(P.diffusivity, D_MAX);
    let diff = d * gf * (pn - pc);
    let v = clamp(P.flow_outward * (wc - wn), -V_MAX, V_MAX);
    let up = select(pn, pc, wc > wn);
    let flux = gf * v * up;
    return diff - flux;
}

@compute @workgroup_size(8, 8, 1)
fn cs_step(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = P.region_ox + gid.x;
    let y = P.region_oy + gid.y;
    if (gid.x >= P.region_w || gid.y >= P.region_h || x >= P.width || y >= P.height) {
        return;
    }
    let i = idx(x, y);
    let wc = water_in[i];
    let pc = pig_in[i];
    let gc = gate(wc, paper[i]);

    let xl = max(x, 1u) - 1u;
    let xr = min(x + 1u, P.width - 1u);
    let yt = max(y, 1u) - 1u;
    let yb = min(y + 1u, P.height - 1u);
    let iL = idx(xl, y);
    let iR = idx(xr, y);
    let iU = idx(x, yt);
    let iD = idx(x, yb);

    var p_new = pc;
    p_new = p_new + face(gc, wc, pc, gate(water_in[iL], paper[iL]), water_in[iL], pig_in[iL]);
    p_new = p_new + face(gc, wc, pc, gate(water_in[iR], paper[iR]), water_in[iR], pig_in[iR]);
    p_new = p_new + face(gc, wc, pc, gate(water_in[iU], paper[iU]), water_in[iU], pig_in[iU]);
    p_new = p_new + face(gc, wc, pc, gate(water_in[iD], paper[iD]), water_in[iD], pig_in[iD]);

    pig_out[i] = max(p_new, vec4<f32>(0.0));
    let w = max(wc - P.evaporation, 0.0);
    water_out[i] = select(0.0, w, w >= 1.0e-4);
}
