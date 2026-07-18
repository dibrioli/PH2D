// Impasto light pass — the relief made visible, on the GPU.
//
// The port of `ph2d_tool_painter::tool::paint::impasto_shade::Rig::shade` (plus `channel` and
// `light_pixel` beside it). It shades a freshly-composited, UNLIT region and writes the lit result to a
// separate destination — the CPU pass has the same shape, and for the same reason: lighting is not
// idempotent, so a pass that read and wrote one texture would compound the shading a little more every
// frame.
//
// ## What this file is allowed to know
//
// Only the optics. The *plumbing* — which layers carry relief, in which order, folded by which composite
// mode, with the live stroke merged and the glass ceiling applied — runs once on the CPU and arrives here
// as three finished planes (`relief`, `cover`, `mat0`/`mat1`; see
// `ph2d_tool_painter::tool::paint::impasto_gpu`). That split is deliberate: a shader that re-derived the
// fold would be a second answer to "how do layers of paint stack", and the two would drift somewhere
// nobody can read a number — a screenshot.
//
// ## Two properties this file must not break
//
//  1. **Flat paint is byte-identical.** The shading is RELATIVE: the diffuse is divided by what a FLAT
//     surface of the same material returns, and the specular has that flat surface's own highlight
//     subtracted per lamp. So zero slope multiplies by exactly 1 and adds exactly 0. An absolute model
//     would darken the whole painting the moment the light came on.
//  2. **Bare paper gets exactly nothing.** `body` (the paint's own coverage) fades both terms, so the
//     pass is a strict no-op where there is no paint.
//
// ## Rounding
//
// The store is quantised EXPLICITLY (`floor(v * 255 + 0.5)`, then stored as `q / 255`) rather than left
// to the unorm conversion. Rust rounds half AWAY from zero and WGSL's implicit conversion is free to
// round half to EVEN, which is a documented divergence in this project
// ([[feedback_cpu_gpu_rounding_conventions_diverge]]) and would otherwise cost a byte on every texel that
// lands exactly on a .5 boundary. `q / 255` is an exact multiple of 1/255, so the unorm conversion
// reproduces `q` and the rounding convention is the CPU's.

// The height-to-pixel gain. MUST equal `impasto_light::DEPTH_UNIT_PX` — pinned by
// `impasto_light_shader_constants_match_the_cpu_pass`.
const DEPTH_UNIT_PX: f32 = 16.0;
// What a face turned fully away from the light still returns. MUST equal `impasto_light::AMBIENT`.
const AMBIENT: f32 = 0.35;
// Last index of a specular LUT row (`SPEC_LUT - 1`) and of its rows (`ROUGH_LEVELS - 1`).
const SPEC_LUT_LAST: f32 = 255.0;
const ROUGH_LAST: i32 = 64;
// The divisor floor from `channel`: a channel the rig gives NO light to would divide by zero, its
// diffuse is zero too, and the answer that keeps the contract is 1 (the channel is untouched).
const FLAT_FLOOR: f32 = 1.0e-4;

struct Lamp {
    // xyz = unit direction (z > 0 points out of the canvas); w unused.
    dir: vec4<f32>,
    // xyz = half-vector against the orthographic view (0, 0, 1); w unused.
    hlf: vec4<f32>,
    // xyz = intensity * colour, already multiplied; w unused.
    tint: vec4<f32>,
};

struct Globals {
    lamps: array<Lamp, 4>,
    // How many of `lamps` are lit. Zero never reaches here — the CPU seam returns no planes at all when
    // every lamp is off, which is the same bail the CPU pass makes (an unlit canvas, to the byte, rather
    // than a division by a zero flat response).
    n: u32,
    // The lit region, in canvas coords. Texels outside it keep whatever the destination already held —
    // the same persistent-output contract the layer compositor's region dispatch runs on.
    ox: u32,
    oy: u32,
    rw: u32,
    rh: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};

@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var dst: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var relief: texture_2d<f32>;
@group(0) @binding(3) var cover: texture_2d<f32>;
@group(0) @binding(4) var mat0: texture_2d<f32>;
@group(0) @binding(5) var mat1: texture_2d<f32>;
@group(0) @binding(6) var spec_lut: texture_2d<f32>;
@group(0) @binding(7) var<uniform> u: Globals;

// Wrap lighting (`Wax`): Valve's half-Lambert. At w = 0 this is `max(N.L, 0)` exactly, which is what
// makes the neutral material the pass as it shaded before materials existed.
fn wrapped_ndl(ndl: f32, wax: f32) -> f32 {
    let w = clamp(wax, 0.0, 1.0);
    return max((ndl + w) / (1.0 + w), 0.0);
}

// pow(ndh, exponent(roughness)), from the table the CPU baked and uploaded. The index truncates, exactly
// as the CPU's `as usize` does.
fn spec_at(level: i32, ndh: f32) -> f32 {
    let i = i32(clamp(ndh, 0.0, 1.0) * SPEC_LUT_LAST);
    return textureLoad(spec_lut, vec2<i32>(i, min(level, ROUGH_LAST)), 0).r;
}

// The composed height, clamped to the CANVAS — so the central difference reads across the lit region's
// edge exactly as a full recompose would, and a partial update draws no seam.
fn height_at(c: vec2<i32>, dims: vec2<i32>) -> f32 {
    let p = clamp(c, vec2<i32>(0, 0), dims - vec2<i32>(1, 1));
    return textureLoad(relief, p, 0).r;
}

// One channel's (multiply, glint) from its summed diffuse and specular — the shared core of the CPU's
// `channel`.
fn channel(diffuse: f32, spec: f32, flat_in: f32, body: f32, gloss: f32, shine: f32) -> vec2<f32> {
    let flat_c = select(flat_in, 1.0, flat_in <= FLAT_FLOOR);
    let ratio = clamp(diffuse / flat_c, 0.0, 2.0);
    let m = AMBIENT + (1.0 - AMBIENT) * ratio;
    return vec2<f32>(1.0 + (m - 1.0) * body, shine * spec * gloss);
}

// The diffuse MODULATES; the highlight is light added against the headroom that is left (a screen), never
// a flat `+ add`. The screen is the only form that keeps the paint's hue: for two channels of one pixel
// `screen(R) - screen(G) = (R - G) * (1 - add)`, so the chroma merely scales.
fn light_pixel(v: f32, mul: f32, add: f32) -> f32 {
    let lit = clamp(v * mul, 0.0, 1.0);
    return clamp(lit + add * (1.0 - lit), 0.0, 1.0);
}

// Rust's `(x * 255.0 + 0.5) as u8`, made explicit so the unorm store cannot apply its own convention.
fn quantise(v: f32) -> f32 {
    return floor(clamp(v, 0.0, 1.0) * 255.0 + 0.5) / 255.0;
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= u.rw || gid.y >= u.rh) {
        return;
    }
    let dims = vec2<i32>(textureDimensions(dst));
    let px = i32(u.ox + gid.x);
    let py = i32(u.oy + gid.y);
    if (px >= dims.x || py >= dims.y) {
        return;
    }
    let coord = vec2<i32>(px, py);
    let texel = textureLoad(src, coord, 0);
    let albedo = texel.rgb;

    // Central differences: a rising slope tilts the normal AGAINST the rise.
    let dhx = (height_at(coord + vec2<i32>(1, 0), dims) - height_at(coord - vec2<i32>(1, 0), dims)) * 0.5;
    let dhy = (height_at(coord + vec2<i32>(0, 1), dims) - height_at(coord - vec2<i32>(0, 1), dims)) * 0.5;
    // The weight IS the coverage: the pass multiplies a composited pixel, and at a stroke's translucent
    // edge that pixel is mostly paper showing through. Shading it in full means shading the paper.
    let body = textureLoad(cover, coord, 0).r;
    let gloss = body;

    // Flat paint — or bare paper — is untouched, to the byte. Writing the source back is byte-identical:
    // `textureLoad` returns an exact multiple of 1/255 and `quantise` reproduces it.
    if ((dhx == 0.0 && dhy == 0.0) || body <= 0.0) {
        textureStore(dst, coord, texel);
        return;
    }

    let m0 = textureLoad(mat0, coord, 0);
    let m1 = textureLoad(mat1, coord, 0);
    let shine = clamp(m0.r, 0.0, 1.0);
    let roughness = m0.g;
    let metallic = clamp(m0.b, 0.0, 1.0);
    let wax = m0.a;
    let wax_color = m1.rgb;
    let level = i32(roughness * f32(ROUGH_LAST) + 0.5);

    // --- the material, RESOLVED against the rig (the CPU's `Rig::resolve`) -------------------------
    // The flat response the whole pass divides by. It depends on the PAINT, not on the lamps alone:
    // Roughness moves the specular exponent and Wax wraps the diffuse, so the wrap is applied to the
    // FLAT surface too. That is the line that keeps flat paint byte-identical at any Wax.
    var flat_base = vec3<f32>(0.0);
    var flat_wax = vec3<f32>(0.0);
    var flat_spec: array<f32, 4>;
    for (var i = 0u; i < u.n; i = i + 1u) {
        let l = u.lamps[i];
        let lz = max(l.dir.z, 0.0);
        let wrapped = wrapped_ndl(l.dir.z, wax) - lz;
        flat_base = flat_base + l.tint.rgb * lz;
        flat_wax = flat_wax + l.tint.rgb * wrapped;
        flat_spec[i] = spec_at(level, l.hlf.z);
    }

    // --- shade (the CPU's `Rig::shade`, coloured path) ---------------------------------------------
    // Only the coloured path is implemented. The CPU's achromatic fast lane is an OPTIMISATION that
    // computes one channel instead of three, and it is bit-identical to this one: on that lane wax = 0,
    // so `scattered` is identically 0.0 and `direct + 0.0 * wax_tint` is exactly `direct`; and
    // metallic = 0, so `spec_tint` is exactly 1.0. Pinned by
    // `the_achromatic_fast_lane_is_the_coloured_one_to_the_bit`.
    let v = vec3<f32>(-dhx * DEPTH_UNIT_PX, -dhy * DEPTH_UNIT_PX, 1.0);
    let len = max(sqrt(v.x * v.x + v.y * v.y + v.z * v.z), 1.0e-6);
    let nrm = v / len;
    // How much of the highlight takes the PAINT's colour instead of the LAMP's — the one line that
    // separates a conductor from a dielectric in every PBR model.
    let spec_tint = vec3<f32>(1.0) + (albedo - vec3<f32>(1.0)) * metallic;
    // What the light that went THROUGH the paint comes back wearing: the paint's own colour (that is the
    // physics, and it is free) times the artist's FILTER.
    let wax_tint = albedo * wax_color;

    var diffuse = vec3<f32>(0.0);
    var spec = vec3<f32>(0.0);
    for (var i = 0u; i < u.n; i = i + 1u) {
        let l = u.lamps[i];
        let raw = nrm.x * l.dir.x + nrm.y * l.dir.y + nrm.z * l.dir.z;
        let direct = max(raw, 0.0);
        let scattered = wrapped_ndl(raw, wax) - direct;
        let ndh = nrm.x * l.hlf.x + nrm.y * l.hlf.y + nrm.z * l.hlf.z;
        // Each lamp's highlight is relative to ITS OWN flat response, and clamped at zero THERE: summing
        // the raw speculars and subtracting the flat total would let a lamp facing away borrow headroom
        // from one facing the paint, and the pass would glint on flat ground.
        let ds = max(spec_at(level, ndh) - flat_spec[i], 0.0);
        diffuse = diffuse + l.tint.rgb * (vec3<f32>(direct) + scattered * wax_tint);
        spec = spec + l.tint.rgb * ds * spec_tint;
    }

    // The divisor wears the SAME tint the pixel does — filter included. That is what keeps a flat surface
    // at ratio exactly 1 in every channel.
    let flat_rgb = flat_base + wax_tint * flat_wax;
    let cr = channel(diffuse.r, spec.r, flat_rgb.r, body, gloss, shine);
    let cg = channel(diffuse.g, spec.g, flat_rgb.g, body, gloss, shine);
    let cb = channel(diffuse.b, spec.b, flat_rgb.b, body, gloss, shine);

    let out = vec4<f32>(
        quantise(light_pixel(albedo.r, cr.x, cr.y)),
        quantise(light_pixel(albedo.g, cg.x, cg.y)),
        quantise(light_pixel(albedo.b, cb.x, cb.y)),
        texel.a,
    );
    textureStore(dst, coord, out);
}
