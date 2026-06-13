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
    color_model: u32,// 0 = RGB Beer–Lambert (default), 1 = spectral Kubelka–Munk
    _p1: f32,
}

@group(0) @binding(0) var<uniform> C: Comp;
@group(0) @binding(1) var<storage, read> pig: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read> backdrop: array<u32>; // packed sRGB8 RGBA, canvas-res
@group(0) @binding(3) var preview_tex: texture_storage_2d<rgba8unorm, write>;
// K–M spectral tables (mirrors km.rs): [curves 3·N][absorb PIGMENTS·N][to_rgb 3·N], N=16.
@group(0) @binding(4) var<storage, read> km: array<f32>;

const KM_N: u32 = 16u;
const KM_OFF_CURVES: u32 = 0u;   // 3·N
const KM_OFF_ABSORB: u32 = 48u;  // PIGMENTS(4)·N
const KM_OFF_TORGB: u32 = 112u;  // 3·N  (total 160)
fn km_curve(ch: u32, k: u32) -> f32 { return km[KM_OFF_CURVES + ch * KM_N + k]; }
fn km_absorb(p: u32, k: u32) -> f32 { return km[KM_OFF_ABSORB + p * KM_N + k]; }
fn km_torgb(i: u32, k: u32) -> f32 { return km[KM_OFF_TORGB + i * KM_N + k]; }

// Spectral subtractive glaze of 4 pigment concentrations over a LINEAR-sRGB backdrop (mirror of
// km.rs `compose_over`): upsample backdrop → ×exp(−Σ cᵢ apᵢ) per λ → integrate back to linear RGB.
fn km_compose(backdrop_lin: vec3<f32>, conc: vec4<f32>) -> vec3<f32> {
    var rgb = vec3<f32>(0.0);
    for (var k: u32 = 0u; k < KM_N; k = k + 1u) {
        let cv = vec3<f32>(km_curve(0u, k), km_curve(1u, k), km_curve(2u, k));
        let b = max(dot(backdrop_lin, cv), 0.0);
        let a = conc.x * km_absorb(0u, k) + conc.y * km_absorb(1u, k)
              + conc.z * km_absorb(2u, k) + conc.w * km_absorb(3u, k);
        let spec = b * exp(-a);
        rgb.x = rgb.x + km_torgb(0u, k) * spec;
        rgb.y = rgb.y + km_torgb(1u, k) * spec;
        rgb.z = rgb.z + km_torgb(2u, k) * spec;
    }
    return max(rgb, vec3<f32>(0.0));
}

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

// EDGE ANTI-ALIAS: the pigment field is 1:1 with the canvas, so a sharp (dried/frozen) stroke edge
// that drops from full→0 over one cell staircases when zoomed in. A small Gaussian (radius 2,
// σ≈1.2) over the field smooths that contour SUB-cell. A uniform interior is unchanged (blurring a
// constant = the constant), so only edges + gradients soften — exactly the watercolor look. Wet AND
// dry edges both anti-alias, since this is purely a display-side resample of the field.
const BLUR_RADIUS: i32 = 2;
const BLUR_SIGMA: f32 = 1.2;

fn sample_pig(fx: f32, fy: f32) -> vec4<f32> {
    let gw = i32(C.gw);
    let gh = i32(C.gh);
    let ix = i32(round(fx));
    let iy = i32(round(fy));
    let inv2s2 = 1.0 / (2.0 * BLUR_SIGMA * BLUR_SIGMA);
    var acc = vec4<f32>(0.0);
    var wsum = 0.0;
    for (var dy = -BLUR_RADIUS; dy <= BLUR_RADIUS; dy = dy + 1) {
        for (var dx = -BLUR_RADIUS; dx <= BLUR_RADIUS; dx = dx + 1) {
            let x = clamp(ix + dx, 0, gw - 1);
            let y = clamp(iy + dy, 0, gh - 1);
            let wgt = exp(-f32(dx * dx + dy * dy) * inv2s2);
            acc = acc + pig[u32(y) * C.gw + u32(x)] * wgt;
            wsum = wsum + wgt;
        }
    }
    return acc / wsum;
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
    var rgb: vec3<f32>;
    if (C.color_model == 1u) {
        // ── Spectral Kubelka–Munk (optional): praw = 4 pigment concentrations. Saturate the SUM
        //    (smooth, preserving the concentration RATIOS = hue), then glaze spectrally. ──
        let conc_raw = max(praw, vec4<f32>(0.0));
        let total = conc_raw.x + conc_raw.y + conc_raw.z + conc_raw.w;
        let eff = MASS_MAX * (1.0 - exp(-total / MASS_MAX));
        let scale = select(0.0, eff / total, total > 1.0e-6);
        rgb = lin_to_srgb(km_compose(srgb_to_lin(back.xyz), conc_raw * scale));
    } else {
        // ── RGB Beer–Lambert (default): praw = (absorb.rgb, mass). ──
        let absorb_raw = max(praw.xyz, vec3<f32>(0.0));
        let mass = max(praw.w, 0.0);
        // Smooth saturation of the effective mass (preserves hue = absorb/mass): proportional for
        // thin glazes, asymptotic toward the masstone for thick ones, no stepped core↔halo contour.
        let eff = MASS_MAX * (1.0 - exp(-mass / MASS_MAX));
        let scale = select(0.0, eff / mass, mass > 1.0e-6);
        rgb = lin_to_srgb(srgb_to_lin(back.xyz) * exp(-absorb_raw * scale));
    }
    textureStore(preview_tex, vec2<i32>(i32(cx), i32(cy)), vec4<f32>(rgb, 1.0));
}
