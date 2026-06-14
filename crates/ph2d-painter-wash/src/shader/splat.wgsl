// cs_splat — brush input. Adds water + pigment (absorbance) from a dab list onto the
// resident fields. Full-grid dispatch (the minimal core keeps the input path trivial);
// each cell accumulates every dab whose disk covers it, with a smooth falloff.
//
// WATER HALO (wet-on-dry / wet-in-wet blending): the water wets a slightly WIDER, softer disk than
// the pigment deposit, so every dab leaves a wet, gate-open margin around its colour. A new deposit
// then feathers INTO that margin — and if it lands next to a dried/frozen rim (the edge-recession
// freezes rims), the halo RE-WETS that rim so its pigment re-mobilises and the new paint blends
// softly instead of butting a hard internal edge. Bounded by the dab radius (no autonomous spread);
// the edge-biased recession in cs_step removes the halo again.
const WATER_HALO: f32 = 1.5;

// Per-cell field cap (Σ concentration / dye mass) — matches cs_step. A pure STABILITY bound now, not
// a hue control: the composite normalises the concentration ratio to K_REF (ADR-0089 §2.2), so the cap
// only governs how OPAQUE heavy overlap can get, never its colour.
const FIELD_CAP: f32 = 8.0;

struct Dab {
    cx: f32,
    cy: f32,
    r: f32,
    water_add: f32,
    pig: vec4<f32>, // 4 base-pigment concentrations (K–M), added × falloff
    dye: vec4<f32>, // premul linear-RGB + mass (Linear), added × falloff (ADR-0089)
    res: vec4<f32>, // premul signed-RGB residual (ADR-0091), added × falloff
}

struct SplatParams {
    width: u32,
    height: u32,
    n_dabs: u32,
    region_ox: u32,
    region_oy: u32,
    region_w: u32,
    region_h: u32,
    _pad: u32,
}

@group(0) @binding(0) var<uniform> S: SplatParams;
@group(0) @binding(1) var<storage, read_write> water: array<f32>;
@group(0) @binding(2) var<storage, read_write> pig: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> dabs: array<Dab>;
@group(0) @binding(4) var<storage, read_write> dye: array<vec4<f32>>; // ADR-0089 faithful-RGB channel
@group(0) @binding(5) var<storage, read_write> res: array<vec4<f32>>; // ADR-0091 residual channel

@compute @workgroup_size(8, 8, 1)
fn cs_splat(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = S.region_ox + gid.x;
    let y = S.region_oy + gid.y;
    if (gid.x >= S.region_w || gid.y >= S.region_h || x >= S.width || y >= S.height) {
        return;
    }
    let i = y * S.width + x;
    let fx = f32(x);
    let fy = f32(y);
    var w = water[i];
    var p = pig[i];
    var dy_acc = dye[i];
    var rs_acc = res[i];
    for (var d: u32 = 0u; d < S.n_dabs; d = d + 1u) {
        let db = dabs[d];
        let dx = fx - db.cx;
        let dyy = fy - db.cy;
        let dd = sqrt(dx * dx + dyy * dyy);
        let rp = max(db.r, 1.0e-6);
        // Water: a wider, softer halo (re-wets a dried rim the dab lands on so it can blend).
        let distw = dd / (rp * WATER_HALO);
        if (distw < 1.0) {
            let fw = 1.0 - distw * distw * (3.0 - 2.0 * distw);
            w = min(w + db.water_add * fw, 1.0);
        }
        // Colour: the tighter deposit disk (1 at centre → 0 at rim). Both channels share the falloff.
        let distp = dd / rp;
        if (distp < 1.0) {
            let fp = 1.0 - distp * distp * (3.0 - 2.0 * distp);
            p = p + db.pig * fp;
            dy_acc = dy_acc + db.dye * fp;
            rs_acc = rs_acc + db.res * fp;
        }
    }
    water[i] = w;
    // ADR-0091: cap ALL colour channels by the SAME factor (mass = dye.w), preserving their ratios so
    // the decoded colour is unchanged (only opacity is bounded) — see cs_step. Pig/dye clamp ≥0; the
    // residual is SIGNED (no clamp).
    p = max(p, vec4<f32>(0.0));
    dy_acc = max(dy_acc, vec4<f32>(0.0));
    let mass = dy_acc.w;
    if (mass > FIELD_CAP) {
        let s = FIELD_CAP / mass;
        p = p * s;
        dy_acc = dy_acc * s;
        rs_acc = rs_acc * s;
    }
    pig[i] = p;
    dye[i] = dy_acc;
    res[i] = rs_acc;
}
