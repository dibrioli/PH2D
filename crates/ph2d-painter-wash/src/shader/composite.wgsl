// cs_composite — subtractive (Beer–Lambert) glaze of the pigment field over the canvas
// backdrop, written to a canvas-resolution preview texture the sprite renderer samples.
//
//   rgb = backdrop · exp(−absorb)        (linear-light multiply; absorb=0 ⇒ backdrop unchanged)
//
// Opaque output (alpha = 1): the preview replaces the whole canvas sprite, so bare pixels are
// the backdrop and painted pixels are the backdrop glazed by the pigment. Premultiplied-by-1 ⇒
// the stored texel is straight = premul (matches the v2 premultiplied-rgba8 contract).
//
// PAPER SATURATION (fixes B1 — overlap → black): the splat accumulates absorbance + mass with no
// ceiling, so painting the same spot repeatedly drove `absorb → ∞` ⇒ `exp(−absorb) → 0` = black.
// Real paper holds a finite amount of pigment. The pigment hue per unit mass is `absorb/mass`
// (= −ln(c) for a colour c); we saturate the EFFECTIVE mass toward `MASS_MAX`, so a heavily-loaded
// cell glazes to ≈`exp(−(absorb/mass)·MASS_MAX) = c` — the pigment's full masstone — never darker.
//
// SMOOTH saturation (fixes the stepped core↔halo edge): a HARD cap `min(mass, MASS_MAX)` leaves a
// flat saturated core and a separate gradient halo with a visible CONTOUR between them (the
// mass=MASS_MAX iso-line, quantised per-pixel ⇒ a staircased edge). Instead the effective mass
// rolls off ASYMPTOTICALLY — `eff = MASS_MAX·(1 − exp(−mass/MASS_MAX))` — which is ≈mass for thin
// glazes (proportional darkening) and → MASS_MAX for thick ones, with NO discontinuity in value OR
// slope. The whole stroke is one smooth proportional ramp from paper to the masstone.
//
// Swapping in Kubelka–Munk / Mixbox later is a change to THIS kernel only — the transport never
// moves.

// Paper pigment-holding capacity, in dab-mass units. A heavily-loaded cell converges to the chosen
// pigment colour `c` over white; lighter deposits glaze proportionally up toward it.
const MASS_MAX: f32 = 1.0;

struct Comp {
    cw: u32,        // canvas (output) dims
    ch: u32,
    gw: u32,        // grid (field) dims
    gh: u32,
    region_ox: u32, // canvas-space dispatch region
    region_oy: u32,
    region_w: u32,
    region_h: u32,
    inv: f32,       // grid-per-canvas (gw/cw) — maps a canvas pixel to a grid coord
    coverage_k: f32,// reserved (mass→alpha) — unused while output is opaque
    _p0: f32,
    _p1: f32,
}

@group(0) @binding(0) var<uniform> C: Comp;
@group(0) @binding(1) var<storage, read> pig: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> backdrop: array<u32>; // packed sRGB8 RGBA, canvas-res
@group(0) @binding(3) var preview_tex: texture_storage_2d<rgba8unorm, write>;

fn unpack(w: u32) -> vec4<f32> {
    return vec4<f32>(
        f32(w & 0xffu),
        f32((w >> 8u) & 0xffu),
        f32((w >> 16u) & 0xffu),
        f32((w >> 24u) & 0xffu),
    ) / 255.0;
}
fn srgb_to_lin(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}
fn lin_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let cc = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    let lo = cc * 12.92;
    let hi = 1.055 * pow(cc, vec3<f32>(1.0 / 2.4)) - vec3<f32>(0.055);
    return select(hi, lo, cc <= vec3<f32>(0.0031308));
}

// Bilinear sample of the pigment field at grid coords (fx,fy).
fn sample_pig(fx: f32, fy: f32) -> vec4<f32> {
    let gw = i32(C.gw);
    let gh = i32(C.gh);
    let x0 = i32(floor(fx));
    let y0 = i32(floor(fy));
    let tx = fx - floor(fx);
    let ty = fy - floor(fy);
    let xa = clamp(x0, 0, gw - 1);
    let xb = clamp(x0 + 1, 0, gw - 1);
    let ya = clamp(y0, 0, gh - 1);
    let yb = clamp(y0 + 1, 0, gh - 1);
    let p00 = pig[u32(ya) * C.gw + u32(xa)];
    let p10 = pig[u32(ya) * C.gw + u32(xb)];
    let p01 = pig[u32(yb) * C.gw + u32(xa)];
    let p11 = pig[u32(yb) * C.gw + u32(xb)];
    let top = mix(p00, p10, tx);
    let bot = mix(p01, p11, tx);
    return mix(top, bot, ty);
}

@compute @workgroup_size(8, 8, 1)
fn cs_composite(@builtin(global_invocation_id) gid: vec3<u32>) {
    let cx = C.region_ox + gid.x;
    let cy = C.region_oy + gid.y;
    if (gid.x >= C.region_w || gid.y >= C.region_h || cx >= C.cw || cy >= C.ch) {
        return;
    }
    let pix = cy * C.cw + cx;
    let back = unpack(backdrop[pix]);
    let fx = clamp((f32(cx) + 0.5) * C.inv - 0.5, 0.0, f32(C.gw) - 1.0);
    let fy = clamp((f32(cy) + 0.5) * C.inv - 0.5, 0.0, f32(C.gh) - 1.0);
    let praw = sample_pig(fx, fy);
    let absorb_raw = max(praw.xyz, vec3<f32>(0.0));
    let mass = max(praw.w, 0.0);
    // Smooth saturation of the effective mass (preserves hue = absorb/mass): proportional for thin
    // glazes, asymptotic toward the masstone for thick ones, with NO stepped core↔halo contour.
    let eff = MASS_MAX * (1.0 - exp(-mass / MASS_MAX));
    let scale = select(0.0, eff / mass, mass > 1.0e-6);
    let absorb = absorb_raw * scale;
    let glaze = srgb_to_lin(back.xyz) * exp(-absorb);
    let rgb = lin_to_srgb(glaze);
    textureStore(preview_tex, vec2<i32>(i32(cx), i32(cy)), vec4<f32>(rgb, 1.0));
}
