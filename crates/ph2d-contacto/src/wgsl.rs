//! O gémeo WGSL da leitura da [`crate::Grade`]: `ct_borda` (o ponto da grelha e o factor da cauda) e
//! `ct_oclusao` (a convolução com o cosseno). Quem chama amostra os `3` texels de coeficientes.

/// A fonte WGSL (as constantes já postas).
#[must_use]
pub fn fonte() -> String {
    format!(
        r"
const CT_ALCANCE: f32 = {alcance:?};
// O ponto da grelha `u` (em [0, 1]^3 dentro do cubo) -> (o ponto da borda na mesma direccao a partir do
// centro, o factor 1/s^2 da cauda); factor 0 alem do alcance.
fn ct_borda(u: vec3<f32>) -> vec4<f32> {{
    let s = 2.0 * max(max(abs(u.x - 0.5), abs(u.y - 0.5)), abs(u.z - 0.5));
    if (s > CT_ALCANCE) {{
        return vec4<f32>(0.5, 0.5, 0.5, 0.0);
    }}
    if (s <= 1.0) {{
        return vec4<f32>(u, 1.0);
    }}
    return vec4<f32>(vec3<f32>(0.5) + (u - vec3<f32>(0.5)) / s, 1.0 / (s * s));
}}
// A fraccao do ceu, ponderada pelo cosseno, que os coeficientes (c0.xyzw, c1.xyzw, c2.x) tapam a `n`.
fn ct_oclusao(c0: vec4<f32>, c1: vec4<f32>, c2: vec4<f32>, n: vec3<f32>) -> f32 {{
    let x = n.x;
    let y = n.y;
    let z = n.z;
    let l0 = 3.14159265 * 0.282095 * c0.x;
    let l1 = 2.0943951 * 0.488603 * (c0.y * y + c0.z * z + c0.w * x);
    let l2 = 0.78539816 * (1.092548 * (c1.x * x * y + c1.y * y * z + c1.w * x * z)
        + 0.315392 * c1.z * (3.0 * z * z - 1.0) + 0.546274 * c2.x * (x * x - y * y));
    return clamp((l0 + l1 + l2) / 3.14159265, 0.0, 1.0);
}}
",
        alcance = crate::ALCANCE
    )
}
