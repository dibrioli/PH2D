// cs_composite — ADR-0089 dual-field display, written to a canvas-resolution preview texture the
// sprite renderer samples in place of the sprite. BOTH colour models are FAITHFUL to the picked
// colour by construction (the BUG-C fix), reading the SAME pair of live fields so a model flip is a
// pure re-render (the persistent-pigment "live transform"):
//
//   • Linear (0): colour = dye.rgb / dye.mass (un-premultiply ⇒ the picked colour exactly),
//                 coverage = 1−exp(−mass/k). Wet overlap = mass-weighted RGB average = metameric.
//   • K–M   (1):  hue = compose_over(white, (conc/Σconc)·K_REF)  — taken at the FIXED magnitude
//                 K_REF so it is INDEPENDENT of accumulated mass (no exp(−c·a)=Tᶜ hue drift),
//                 coverage = 1−exp(−(Σconc/K_REF)/k). Wet overlap mixes spectrally (blue+yellow→green).
//
// Both take coverage (lightness vs the backdrop, AND edge-darkening — a thicker rim reads as more
// OPAQUE of the same hue) from the field's accumulated amount. Opaque output (alpha=1): the preview
// replaces the whole canvas sprite, so bare pixels are the backdrop.

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
    coverage_k: f32,// reserved (per-brush coverage softness) — the kernel uses the COVER_K constant
    color_model: u32,// 0 = Linear/RGB (dye field), 1 = spectral Kubelka–Munk (concentration field)
    _p1: f32,
}

@group(0) @binding(0) var<uniform> C: Comp;
@group(0) @binding(1) var<storage, read> pig: array<vec4<f32>>;     // 4 concentrations (K–M)
@group(0) @binding(2) var<storage, read> backdrop: array<u32>;      // packed sRGB8 RGBA, canvas-res
@group(0) @binding(3) var preview_tex: texture_storage_2d<rgba8unorm, write>;
// K–M spectral tables (mirrors km.rs): [curves 3·N][absorb PIGMENTS·N][to_rgb 3·N], N=16.
@group(0) @binding(4) var<storage, read> km: array<f32>;
@group(0) @binding(5) var<storage, read> dye: array<vec4<f32>>;     // premul-RGB + mass (Linear, ADR-0089)

const KM_N: u32 = 16u;
const KM_OFF_CURVES: u32 = 0u;    // 3·N
const KM_OFF_ABSORB: u32 = 48u;   // PIGMENTS(4)·N
const KM_OFF_TORGB: u32 = 112u;   // 3·N  (cumulative 160)
fn km_curve(ch: u32, k: u32) -> f32 { return km[KM_OFF_CURVES + ch * KM_N + k]; }
fn km_absorb(p: u32, k: u32) -> f32 { return km[KM_OFF_ABSORB + p * KM_N + k]; }
fn km_torgb(i: u32, k: u32) -> f32 { return km[KM_OFF_TORGB + i * KM_N + k]; }

// ADR-0089 §2.2 colour constants — MUST match km.rs (`K_REF`, `COVER_K`).
const K_REF: f32 = 3.0;
const COVER_K: f32 = 0.6;

// Opacity from the effective mass (`mass` for Linear, `Σconc/K_REF` for K–M ⇒ both modes build the
// same coverage for the same stroke). Also produces edge-darkening: a FlowOutward-concentrated rim
// has more amount ⇒ more coverage ⇒ a denser (not hue-shifted) edge.
fn coverage(eff: f32) -> f32 {
    return 1.0 - exp(-eff / COVER_K);
}

// Spectral subtractive glaze of a concentration vector over WHITE (mirror of km.rs `compose_over`
// with backdrop=white): upsample white → ×exp(−Σ cᵢ apᵢ) per λ → integrate back to linear RGB. Used
// for the K–M HUE at the fixed K_REF magnitude.
fn km_hue_over_white(conc: vec4<f32>) -> vec3<f32> {
    var rgb = vec3<f32>(0.0);
    for (var k: u32 = 0u; k < KM_N; k = k + 1u) {
        let white = km_curve(0u, k) + km_curve(1u, k) + km_curve(2u, k); // dot(vec3(1), curve)
        let a = conc.x * km_absorb(0u, k) + conc.y * km_absorb(1u, k)
              + conc.z * km_absorb(2u, k) + conc.w * km_absorb(3u, k);
        let spec = white * exp(-a);
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

// EDGE ANTI-ALIAS: the field is 1:1 with the canvas, so a sharp (dried/frozen) stroke edge that drops
// full→0 over one cell staircases when zoomed in. A small Gaussian (radius 2, σ≈1.2) smooths that
// contour SUB-cell. A uniform interior is unchanged (blurring a constant = the constant), so only
// edges + gradients soften — the watercolor look. Applied to whichever field the active model reads.
const BLUR_RADIUS: i32 = 2;
const BLUR_SIGMA: f32 = 1.2;

fn sample_pig(fx: f32, fy: f32) -> vec4<f32> {
    let ix = i32(round(fx));
    let iy = i32(round(fy));
    let inv2s2 = 1.0 / (2.0 * BLUR_SIGMA * BLUR_SIGMA);
    var acc = vec4<f32>(0.0);
    var wsum = 0.0;
    for (var dy = -BLUR_RADIUS; dy <= BLUR_RADIUS; dy = dy + 1) {
        for (var dx = -BLUR_RADIUS; dx <= BLUR_RADIUS; dx = dx + 1) {
            let x = clamp(ix + dx, 0, i32(C.gw) - 1);
            let y = clamp(iy + dy, 0, i32(C.gh) - 1);
            let wgt = exp(-f32(dx * dx + dy * dy) * inv2s2);
            acc = acc + pig[u32(y) * C.gw + u32(x)] * wgt;
            wsum = wsum + wgt;
        }
    }
    return acc / wsum;
}

fn sample_dye(fx: f32, fy: f32) -> vec4<f32> {
    let ix = i32(round(fx));
    let iy = i32(round(fy));
    let inv2s2 = 1.0 / (2.0 * BLUR_SIGMA * BLUR_SIGMA);
    var acc = vec4<f32>(0.0);
    var wsum = 0.0;
    for (var dy = -BLUR_RADIUS; dy <= BLUR_RADIUS; dy = dy + 1) {
        for (var dx = -BLUR_RADIUS; dx <= BLUR_RADIUS; dx = dx + 1) {
            let x = clamp(ix + dx, 0, i32(C.gw) - 1);
            let y = clamp(iy + dy, 0, i32(C.gh) - 1);
            let wgt = exp(-f32(dx * dx + dy * dy) * inv2s2);
            acc = acc + dye[u32(y) * C.gw + u32(x)] * wgt;
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
    let back_lin = srgb_to_lin(back.xyz);
    let fx = clamp((f32(cx) + 0.5) * C.inv - 0.5, 0.0, f32(C.gw) - 1.0);
    let fy = clamp((f32(cy) + 0.5) * C.inv - 0.5, 0.0, f32(C.gh) - 1.0);

    var rgb = back_lin;
    if (C.color_model == 1u) {
        // Spectral K–M: hue from the concentration RATIO at K_REF (mass-independent), coverage from Σ.
        let conc = max(sample_pig(fx, fy), vec4<f32>(0.0));
        let total = conc.x + conc.y + conc.z + conc.w;
        if (total > 1.0e-6) {
            let ratio = conc / total * K_REF;
            let hue = km_hue_over_white(ratio);
            let cover = coverage(total / K_REF);
            rgb = mix(back_lin, hue, cover);
        }
    } else {
        // Linear/RGB: un-premultiply the dye for the exact picked colour, coverage from the mass.
        let dv = max(sample_dye(fx, fy), vec4<f32>(0.0));
        let mass = dv.w;
        if (mass > 1.0e-6) {
            let color = dv.xyz / mass;
            let cover = coverage(mass);
            rgb = mix(back_lin, color, cover);
        }
    }
    textureStore(preview_tex, vec2<i32>(i32(cx), i32(cy)), vec4<f32>(lin_to_srgb(rgb), 1.0));
}
