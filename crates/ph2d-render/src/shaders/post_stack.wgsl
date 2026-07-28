// post_stack.wgsl — the app HDR colour grade (ADR-0145).
//
// A LINE-FOR-LINE mirror of `post_stack::grade_pixel` (the CPU source of truth). The
// `#[ignore]` GPU parity test reads the device back and compares the two, so any drift
// between this shader and the Rust reference fails a gate.
//
// The order is scene-referred (a colour corrector before the display transform):
// exposure -> tint -> contrast(pivot 0.18) -> saturation(mix) -> vignette. It runs on the
// linear Rgba16Float game_rt, BEFORE the AgX tonemap.

struct Params {
    // (exposure_mul, contrast, saturation, vignette_amount)
    v0: vec4<f32>,
    // (tint.r, tint.g, tint.b, vignette_radius)
    v1: vec4<f32>,
    // (vignette_softness, aspect, _, _)
    v2: vec4<f32>,
};

@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var<uniform> P: Params;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Fullscreen triangle; uv is [0,1] over the visible screen, top-left origin.
@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VsOut {
    var out: VsOut;
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    out.uv = vec2<f32>(x, y);
    out.pos = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    return out;
}

const LUMA: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);
const PIVOT: f32 = 0.18;

// Identical formula to `post_stack::vig_smoothstep` (never the builtin, whose edges
// might differ) so CPU<->GPU parity holds on the vignette falloff.
fn vig_smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = clamp((x - e0) / (e1 - e0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

fn vignette_factor(uv: vec2<f32>, aspect: f32, amount: f32, radius: f32, softness: f32) -> f32 {
    let dx = (uv.x - 0.5) * aspect;
    let dy = uv.y - 0.5;
    let norm = 0.5 * sqrt(aspect * aspect + 1.0);
    let d = sqrt(dx * dx + dy * dy) / norm;
    let t = vig_smoothstep(radius, radius + max(softness, 1e-4), d);
    return 1.0 - amount * t;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let texel = textureSample(src, samp, in.uv);

    let exposure_mul = P.v0.x;
    let contrast = P.v0.y;
    let saturation = P.v0.z;
    let vig_amount = P.v0.w;
    let tint = P.v1.xyz;
    let vig_radius = P.v1.w;
    let vig_softness = P.v2.x;
    let aspect = P.v2.y;

    // 1. exposure + 2. tint
    var c = texel.rgb * exposure_mul * tint;
    // 3. contrast around the pivot, clamped non-negative
    c = max((c - vec3<f32>(PIVOT)) * contrast + vec3<f32>(PIVOT), vec3<f32>(0.0));
    // 4. saturation via the mix form (exact at sat == 1)
    let luma = dot(c, LUMA);
    c = max(vec3<f32>(luma) * (1.0 - saturation) + c * saturation, vec3<f32>(0.0));
    // 5. vignette
    let v = vignette_factor(in.uv, aspect, vig_amount, vig_radius, vig_softness);
    c = c * v;

    return vec4<f32>(c, texel.a);
}
