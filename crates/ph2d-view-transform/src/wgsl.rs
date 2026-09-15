//! ⭐ **O OLHAR, em WGSL** — a mesma lei do [`crate::to_display`], para quem pinta no dispositivo.
//!
//! ⚠️ **A `view` viaja como `u32`** e não como enum: `0` é [`crate::ViewTransform::Standard`], `1` é
//! a [`crate::ViewTransform::Neutral`]. A conversão vive em [`view_code`], para que acrescentar uma
//! variante seja **erro de compilação** ali em vez de um braço a faltar no shader.

/// O código que o WGSL lê.
#[must_use]
pub fn view_code(v: crate::ViewTransform) -> u32 {
    match v {
        crate::ViewTransform::Standard => 0,
        crate::ViewTransform::Neutral => 1,
    }
}

/// O corpo, sem entrada nenhuma — quem o usa chama [`SOURCE`] e depois `vt_to_display`.
pub const SOURCE: &str = r#"
// ⚠️ **`NaN` e `∞` tratados por COMPARAÇÃO** — o WGSL não tem `is_nan`. `c != c` é `true` só para
// `NaN`, e o teste de finitude é contra o maior `f32`. ⛔ Um backend com semântica relaxada pode
// assumir que eles não existem; é por isso que a paridade do passe inteiro (não só esta função) é
// que responde por isto.
fn vt_sanitize(c: f32) -> f32 {
    if (c != c) { return 0.0; }
    if (c <= 0.0) { return 0.0; }
    return min(c, 3.40282347e38);
}

fn vt_exposure_scale(stops: f32) -> f32 {
    if (stops != stops || abs(stops) > 3.40282347e38) { return 1.0; }
    return exp2(stops);
}

fn vt_khronos_pbr_neutral(c_in: vec3<f32>) -> vec3<f32> {
    let START_COMPRESSION = 0.8 - 0.04;
    let DESATURATION = 0.15;

    let x = min(c_in.x, min(c_in.y, c_in.z));
    var offset = 0.04;
    if (x < 0.08) { offset = x - 6.25 * x * x; }
    let c = c_in - vec3<f32>(offset);

    let peak = max(c.x, max(c.y, c.z));
    if (peak < START_COMPRESSION) { return c; }
    let d = 1.0 - START_COMPRESSION;
    let new_peak = 1.0 - d * d / (peak + d - START_COMPRESSION);
    let scale = new_peak / peak;
    let g = 1.0 - 1.0 / (DESATURATION * (peak - new_peak) + 1.0);
    return (c * scale) * (1.0 - g) + vec3<f32>(new_peak * g);
}

/// `view`: `0` = Standard · `1` = Neutral (ver `ph2d_view_transform::wgsl::view_code`).
fn vt_to_display(scene: vec3<f32>, stops: f32, view: u32) -> vec3<f32> {
    let k = vt_exposure_scale(stops);
    let lit = vec3<f32>(vt_sanitize(scene.x * k), vt_sanitize(scene.y * k), vt_sanitize(scene.z * k));
    if (view == 0u) { return min(lit, vec3<f32>(1.0)); }
    return vt_khronos_pbr_neutral(lit);
}
"#;
