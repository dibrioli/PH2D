// Combine pass — `total = flowing + deposited` (ADR-0078 S3c). The compositor reads
// the TOTAL pigment (flowing wash + the frozen deposited layer), so edge-darkening +
// granulation show up. Kept a SEPARATE pass (not folded into cs_transfer, which
// early-returns when the deposition rate is 0) so `total` is ALWAYS written —
// `total == flowing` when nothing is deposited, so the compositor binds `total`
// uniformly and the look is unchanged until deposition is enabled.
//
// Region-scoped (ADR-0078 S1) exactly like the sim passes; the compositor only ever
// reads cells inside the composite region ⊆ the solver region, so `total` is fresh
// wherever it's sampled. Shares the solver `Params` UBO (uses width + the region).

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
@group(0) @binding(1) var<storage, read> flowing: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> deposited: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> total: array<vec4<f32>>;

@compute @workgroup_size(8, 8, 1)
fn cs_combine(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = P.region_ox + gid.x;
    let y = P.region_oy + gid.y;
    if (gid.x >= P.region_w || gid.y >= P.region_h || x >= P.width || y >= P.height) {
        return;
    }
    let i = y * P.width + x;
    total[i] = vec4<f32>(flowing[i].xyz + deposited[i].xyz, flowing[i].w);
}
