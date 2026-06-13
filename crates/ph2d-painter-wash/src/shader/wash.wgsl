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
// Edge-biased recession floor: thin rim water recedes to feather the edge EVEN at Evaporation 0,
// so a static front can't pin a hard pixelated rim (the v2 failure). Scaled by (1−w) ⇒ ~0 in the
// wet interior (Keep Wet stays wet) and inward-only (no spreading).
const EDGE_EVAP_FLOOR: f32 = 0.01;
// Per-cell field cap (Σ concentration / dye mass) — matches splat.wgsl. Now a pure STABILITY bound
// (no NaN / runaway), NOT a hue control: the composite normalises the concentration ratio to K_REF
// for the hue (ADR-0089 §2.2), so the cap only governs how OPAQUE a heavy overlap / FlowOutward rim
// can get — never its colour. Raised from 2.5 so thick paint + edge-darkening rims can reach full
// coverage (kept < 10 for the INV-4 stability gate).
const FIELD_CAP: f32 = 8.0;

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
// ADR-0089 dual field: the faithful-RGB dye channel (premul rgb + mass), transported by the SAME
// water-driven gather as `pig` so the two colour encodings stay spatially identical.
@group(0) @binding(6) var<storage, read> dye_in: array<vec4<f32>>;
@group(0) @binding(7) var<storage, read_write> dye_out: array<vec4<f32>>;

fn idx(x: u32, y: u32) -> u32 { return y * P.width + x; }

// Wet-gate: transport happens only where the cell is wet (water in the [w_lo, w_hi] band).
// NOTE: the paper-permeability modulation (mix(perm_valley, perm_crest, paper)) is intentionally
// REMOVED. The paper field is per-PIXEL noise, so modulating per-cell transport by it etched the
// grain into the pigment as harsh per-pixel mottling (visible once the saturation cap exposes mass
// variation near the cap). The minimal core keeps transport UNIFORM ⇒ a clean, flat stain.
// Granulation returns later as a deliberate v1.1 feature driven by a smooth low-frequency field,
// not this per-pixel tooth. `pap` is kept in the signature (and `paper` bound) for that future use.
fn gate(w: f32, pap: f32) -> f32 {
    let t = clamp((w - P.w_lo) / max(P.w_hi - P.w_lo, 1.0e-6), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
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
    let dc = dye_in[i];
    let gc = gate(wc, paper[i]);

    let xl = max(x, 1u) - 1u;
    let xr = min(x + 1u, P.width - 1u);
    let yt = max(y, 1u) - 1u;
    let yb = min(y + 1u, P.height - 1u);
    let iL = idx(xl, y);
    let iR = idx(xr, y);
    let iU = idx(x, yt);
    let iD = idx(x, yb);

    // Per-neighbour water + gate, computed ONCE and applied to BOTH colour channels — the transport
    // weights (diffusion D·gf, advection v) depend only on water/paper, so pig and dye move identically
    // (only the upwind donor VALUE differs, handled inside `face`). This keeps the two encodings of the
    // same stroke spatially coherent for the live Linear↔K–M toggle.
    let wL = water_in[iL]; let gL = gate(wL, paper[iL]);
    let wR = water_in[iR]; let gR = gate(wR, paper[iR]);
    let wU = water_in[iU]; let gU = gate(wU, paper[iU]);
    let wD = water_in[iD]; let gD = gate(wD, paper[iD]);

    var p_new = pc;
    p_new = p_new + face(gc, wc, pc, gL, wL, pig_in[iL]);
    p_new = p_new + face(gc, wc, pc, gR, wR, pig_in[iR]);
    p_new = p_new + face(gc, wc, pc, gU, wU, pig_in[iU]);
    p_new = p_new + face(gc, wc, pc, gD, wD, pig_in[iD]);

    var d_new = dc;
    d_new = d_new + face(gc, wc, dc, gL, wL, dye_in[iL]);
    d_new = d_new + face(gc, wc, dc, gR, wR, dye_in[iR]);
    d_new = d_new + face(gc, wc, dc, gU, wU, dye_in[iU]);
    d_new = d_new + face(gc, wc, dc, gD, wD, dye_in[iD]);

    var po = max(p_new, vec4<f32>(0.0));
    let pt = po.x + po.y + po.z + po.w;
    if (pt > FIELD_CAP) {
        po = po * (FIELD_CAP / pt);
    }
    pig_out[i] = po;
    // Dye: clamp ≥0 and cap the MASS (.w) preserving the premul ratio (so the un-premultiplied colour
    // is unchanged — only opacity is bounded). The composite guards mass≈0 → backdrop.
    var do_ = max(d_new, vec4<f32>(0.0));
    if (do_.w > FIELD_CAP) {
        do_ = do_ * (FIELD_CAP / do_.w);
    }
    dye_out[i] = do_;
    // Bulk drying (user Evaporation) + edge-biased recession (feathers the thin rim, ~0 in the wet
    // interior). The latter passes rim cells through the wet-gate band before freezing ⇒ soft edge.
    let edge_dry = EDGE_EVAP_FLOOR * (1.0 - clamp(wc, 0.0, 1.0));
    let w = max(wc - P.evaporation - edge_dry, 0.0);
    water_out[i] = select(0.0, w, w >= 1.0e-4);
}
