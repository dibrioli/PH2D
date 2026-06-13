// cs_splat — brush input. Adds water + pigment (absorbance) from a dab list onto the
// resident fields. Full-grid dispatch (the minimal core keeps the input path trivial);
// each cell accumulates every dab whose disk covers it, with a smooth falloff.

struct Dab {
    cx: f32,
    cy: f32,
    r: f32,
    water_add: f32,
    pig: vec4<f32>, // (absorb.rgb, mass) added at full falloff
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
    for (var d: u32 = 0u; d < S.n_dabs; d = d + 1u) {
        let db = dabs[d];
        let dx = fx - db.cx;
        let dy = fy - db.cy;
        let dist = sqrt(dx * dx + dy * dy) / max(db.r, 1.0e-6);
        if (dist >= 1.0) {
            continue;
        }
        let fall = 1.0 - dist * dist * (3.0 - 2.0 * dist); // 1 at centre → 0 at rim
        w = min(w + db.water_add * fall, 1.0);
        p = p + db.pig * fall;
    }
    water[i] = w;
    pig[i] = max(p, vec4<f32>(0.0));
}
