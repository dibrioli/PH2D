//! ⭐⭐⭐ **OS BYTES QUE O COMPOSITOR LÊ, em WGSL** — a curva sRGB, o arredondamento e o
//! pré-multiplicado **em ECRÃ**, num texto só.
//!
//! A lei tem duas implementações: a CPU ([`ph2d_field_render::premultiplicado`], com a medição feita
//! no compositor) e esta, que o pintor de **matcap** ([`crate::matcap_wgsl`]) usa. ⚠️ Medido nesta
//! crate em 2026-09-19: *a placa contrai `a*b + c` num `fma` de outra maneira quando o TEXTO da
//! função muda* — por isso o texto vive num `const` só. (O pintor de material do Render traçado
//! tinha uma cópia verbatim, atada por um gate de contenção; saiu com ele em 03/10.)

/// A curva, o arredondamento e o pré-multiplicado em ecrã.
pub(crate) const EMPACOTA: &str = r"
// A curva do `ph2d_color::srgb::linear_to_srgb_unit`.
fn srgb_unit(linear: f32) -> f32 {
    let v = clamp(linear, 0.0, 1.0);
    if (v > 0.0031308) { return 1.055 * pow(v, 1.0 / 2.4) - 0.055; }
    return v * 12.92;
}

fn byte_de(unidade: f32) -> u32 {
    return u32(clamp(unidade * 255.0 + 0.5, 0.0, 255.0));
}

// ⭐⭐⭐ **O ALFA É PRÉ-MULTIPLICADO EM ECRÃ, NUNCA EM LINEAR** — o gémeo do
// `ph2d_field_render::premultiplicado`, com a medição feita NO compositor (ver o doc daquele
// módulo). Ele recebe um pixel pré-multiplicado em LINEAR e grava `sRGB(C)·a`.
//
// ⚠️ **A codificação usa o alfa que de facto vai para o byte**, como o lado da CPU: o consumidor
// compõe com o byte, e pré-multiplicar por um `f32` que depois arredonda para outro valor deixa
// produtor e consumidor a usar dois números.
//
// ⚠️ **`a == 0` com `rgb ≠ 0` é LUZ ADITIVA e passa INTACTA** — é a forma que a luz devolvida ao
// chão usa, e escalá-la por `a` apagava-a.
// ⛔⛔⛔ **E A LUZ ADITIVA ENTRA POR UM ARGUMENTO PRÓPRIO** — a luz que a peça devolve ao chão
// SOMA sem tapar, logo não é `C·a` e dividi-la por `a` inventa uma cobertura que ela não tem.
// Enfiada no mesmo `vec4` que a cobertura, o empacotamento dividia-a pelo alfa da SOMBRA.
fn empacota_com_luz(c: vec4<f32>, luz: vec3<f32>) -> u32 {
    let ab = byte_de(clamp(c.w, 0.0, 1.0));
    let a = f32(ab) / 255.0;
    let inv = select(0.0, 1.0 / a, ab > 0u);
    let r = byte_de(srgb_unit(c.x * inv) * a + srgb_unit(luz.x));
    let g = byte_de(srgb_unit(c.y * inv) * a + srgb_unit(luz.y));
    let b = byte_de(srgb_unit(c.z * inv) * a + srgb_unit(luz.z));
    return r | (g << 8u) | (b << 16u) | (ab << 24u);
}

fn empacota(c: vec4<f32>) -> u32 {
    return empacota_com_luz(c, vec3<f32>(0.0));
}
";
