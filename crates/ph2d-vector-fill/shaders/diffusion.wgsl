// === ph2d-vector-fill diffusion.wgsl (ADR-0060 §2.5) ===
// Walk-on-Spheres Monte-Carlo solver for diffusion-curve mesh gradients.
// One thread per pixel averages `spp` random walks; each walk jumps on spheres
// sized by the distance to the nearest curve until it is absorbed (< epsilon),
// then returns that curve's side colour. The harmonic field is the expectation.
//
// PARITY: this mirrors `diffusion_gpu::walk_on_spheres_field` line-for-line, and
// shares its RNG — `ph2d_noise1` (prepend `ph2d_expr::wgsl_prelude()` before use;
// the renderer / `cache::compile_fill` path does this). The OKLab→linear matrix
// is bit-identical to `ph2d_color::OklabColor::to_linear` (Ottosson 2020).
//
// DETERMINISM (ADR-0060 §2.6, opt-in): the per-walk key is a pure function of
// (pixel, sample, step, seed), the workgroup size is fixed, and the accumulation
// is an ordered sum — so a deterministic-mode run is reproducible cross-OS.

struct GpuSegment {
    endpoints: vec4<f32>,   // ax, ay, bx, by  (normalized [0,1]^2 fill space)
    left: vec4<f32>,        // OKLab L,a,b,alpha on the +normal side
    right: vec4<f32>,       // OKLab L,a,b,alpha on the -normal side
};

struct DiffusionParams {
    width: u32,
    height: u32,
    segment_count: u32,
    spp: u32,
    max_steps: u32,
    seed: u32,
    epsilon: f32,
    _pad: f32,
};

@group(0) @binding(0) var<storage, read> segments: array<GpuSegment>;
@group(0) @binding(1) var<uniform> params: DiffusionParams;
@group(0) @binding(2) var<storage, read_write> field: array<vec4<f32>>;

// OKLab -> linear sRGB (D65). Bit-identical to ph2d_color::OklabColor::to_linear.
fn oklab_to_linear(lab: vec3<f32>) -> vec3<f32> {
    let l_ = lab.x + 0.39633778 * lab.y + 0.21580376 * lab.z;
    let m_ = lab.x - 0.105561346 * lab.y - 0.06385417 * lab.z;
    let s_ = lab.x - 0.08948418 * lab.y - 1.2914855 * lab.z;
    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;
    return vec3<f32>(
        4.0767417 * l3 - 3.3077116 * m3 + 0.23096994 * s3,
        -1.268438 * l3 + 2.6097574 * m3 - 0.34131938 * s3,
        -0.0041960863 * l3 - 0.7034186 * m3 + 1.7076147 * s3,
    );
}

struct Hit {
    dist: f32,
    lab: vec3<f32>,
};

// Nearest segment to `p`: distance + the OKLab colour of the side `p` is on.
fn nearest(p: vec2<f32>) -> Hit {
    var best_d2: f32 = 3.4e38;
    var best: vec3<f32> = vec3<f32>(0.0);
    for (var i: u32 = 0u; i < params.segment_count; i = i + 1u) {
        let seg = segments[i];
        let a = seg.endpoints.xy;
        let b = seg.endpoints.zw;
        let ab = b - a;
        let len2 = max(dot(ab, ab), 1e-20);
        let t = clamp(dot(p - a, ab) / len2, 0.0, 1.0);
        let cp = a + ab * t;
        let d2 = dot(p - cp, p - cp);
        if (d2 < best_d2) {
            best_d2 = d2;
            let normal = vec2<f32>(-ab.y, ab.x);
            let side = dot(p - cp, normal);
            if (side >= 0.0) {
                best = seg.left.xyz;
            } else {
                best = seg.right.xyz;
            }
        }
    }
    return Hit(sqrt(best_d2), best);
}

// One uniform [0,1) sample from the shared integer hash, keyed per walk step.
fn rand01(px: u32, py: u32, s: u32, step: u32) -> f32 {
    let key = f32(px) * 12.9898
        + f32(py) * 78.233
        + f32(s) * 37.719
        + f32(step) * 0.618034
        + f32(params.seed) * 0.314159;
    return ph2d_noise1(key);
}

@compute @workgroup_size(8, 8, 1)
fn diffusion_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= params.width || gid.y >= params.height) {
        return;
    }
    let px = gid.x;
    let py = gid.y;
    let denom_x = max(params.width - 1u, 1u);
    let denom_y = max(params.height - 1u, 1u);
    let p = vec2<f32>(f32(px) / f32(denom_x), f32(py) / f32(denom_y));

    let samples = max(params.spp, 1u);
    var acc: vec3<f32> = vec3<f32>(0.0);
    for (var s: u32 = 0u; s < samples; s = s + 1u) {
        var x = p;
        var color = nearest(x).lab; // fallback if the walk never absorbs
        for (var step: u32 = 0u; step < params.max_steps; step = step + 1u) {
            let hit = nearest(x);
            if (hit.dist <= params.epsilon) {
                color = hit.lab;
                break;
            }
            let theta = rand01(px, py, s, step) * 6.2831855;
            x = x + vec2<f32>(cos(theta), sin(theta)) * hit.dist;
            color = hit.lab; // most-recent nearest, used if max_steps is hit
        }
        acc = acc + color;
    }
    let lab = acc / f32(samples);
    field[py * params.width + px] = vec4<f32>(oklab_to_linear(lab), 1.0);
}
