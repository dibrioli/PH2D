//! ⭐⭐ **O GÉMEO EM WGSL** — a mesma leitura do [`crate::Ceu`], conta a conta.
//!
//! Quem chama escreve a porta de leitura do atlas (a textura é dele):
//!
//! ```wgsl
//! fn sky_atlas(x: u32, y: u32) -> vec3<f32>
//! ```
//!
//! e recebe `sky_gira`, `sky_radiance(dir, alpha)` e `sky_irradiance(n)` — no referencial do céu,
//! sem força (o giro e a força são do chamador, como no [`crate::Orientado`]).
//!
//! ⭐ **O sol** ([`crate::Sol`]): quem chama escreve também `fn sky_sol_ler(i: u32) -> f32` (a tabela
//! dele) e recebe `sky_sol_tabela(alpha, cos_psi)` e `sky_sol_cos(d, s)`.

use crate::{LADOS, NIVEIS, X0, Y0};

const CORPO: &str = r"
fn sky_gira(d: vec3<f32>, giro: vec2<f32>) -> vec3<f32> {
    return vec3<f32>(giro.x * d.x - giro.y * d.z, d.y, giro.y * d.x + giro.x * d.z);
}

fn sky_oct(d: vec3<f32>) -> vec2<f32> {
    var s = abs(d.x) + abs(d.y) + abs(d.z);
    if (!(s > 0.0)) {
        s = 1.0;
    }
    let a = d.x / s;
    let b = d.z / s;
    if (d.y < 0.0) {
        let sa = select(-1.0, 1.0, a >= 0.0);
        let sb = select(-1.0, 1.0, b >= 0.0);
        return vec2<f32>((1.0 - abs(b)) * sa, (1.0 - abs(a)) * sb);
    }
    return vec2<f32>(a, b);
}

fn sky_bilinear(k: u32, ab: vec2<f32>) -> vec3<f32> {
    let nf = f32(SKY_LADOS[k]);
    let fx = clamp((ab.x * 0.5 + 0.5) * nf - 0.5, -0.5, nf - 0.5);
    let fy = clamp((ab.y * 0.5 + 0.5) * nf - 0.5, -0.5, nf - 0.5);
    let x0 = floor(fx);
    let yf = floor(fy);
    let tx = fx - x0;
    let ty = fy - yf;
    let ix = u32(i32(x0) + 1) + SKY_X0[k];
    let iy = u32(i32(yf) + 1) + SKY_Y0[k];
    let p00 = sky_atlas(ix, iy);
    let p10 = sky_atlas(ix + 1u, iy);
    let p01 = sky_atlas(ix, iy + 1u);
    let p11 = sky_atlas(ix + 1u, iy + 1u);
    let cima = p00 + (p10 - p00) * tx;
    let baixo = p01 + (p11 - p01) * tx;
    return cima + (baixo - cima) * ty;
}

fn sky_radiance(dir: vec3<f32>, alpha: f32) -> vec3<f32> {
    let r = sqrt(clamp(alpha, 0.0, 1.0)) * f32(SKY_NIVEIS - 1u);
    let k0 = min(u32(r), SKY_NIVEIS - 2u);
    let t = r - f32(k0);
    let ab = sky_oct(dir);
    let a = sky_bilinear(k0, ab);
    let b = sky_bilinear(k0 + 1u, ab);
    return a + (b - a) * t;
}

fn sky_irradiance(n: vec3<f32>) -> vec3<f32> {
    return sky_bilinear(SKY_NIVEIS - 1u, sky_oct(n));
}

// O SOL: a calote de radiancia 1 sob o lobulo, bilinear em (sqrt(alfa), angulo) — `Sol::tabela_em`.
fn sky_sol_tabela(alpha: f32, cos_psi: f32) -> f32 {
    let r = sqrt(clamp(alpha, 0.0, 1.0)) * f32(SKY_SOL_RUG - 1u);
    let a = sqrt(max(1.0 - clamp(cos_psi, -1.0, 1.0), 0.0)) / 1.4142135 * f32(SKY_SOL_ANG - 1u);
    let r0 = min(u32(r), SKY_SOL_RUG - 2u);
    let a0 = min(u32(a), SKY_SOL_ANG - 2u);
    let fr = r - f32(r0);
    let fa = a - f32(a0);
    let i = r0 * SKY_SOL_ANG + a0;
    let baixo = sky_sol_ler(i) + (sky_sol_ler(i + 1u) - sky_sol_ler(i)) * fa;
    let j = i + SKY_SOL_ANG;
    let cima = sky_sol_ler(j) + (sky_sol_ler(j + 1u) - sky_sol_ler(j)) * fa;
    return baixo + (cima - baixo) * fr;
}

// O cosseno ao sol `s` (unitario), com a direccao `d` em qualquer escala — `sol::cosseno`.
fn sky_sol_cos(d: vec3<f32>, s: vec3<f32>) -> f32 {
    return dot(d, s) / max(length(d), 1.0e-20);
}
";

fn lista(v: &[u32]) -> String {
    v.iter()
        .map(|x| format!("{x}u"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// ⭐ A fonte, com as constantes do atlas escritas a partir das da CPU (nunca à mão).
#[must_use]
pub fn fonte() -> String {
    format!(
        "const SKY_NIVEIS: u32 = {NIVEIS}u;\n\
         const SKY_SOL_RUG: u32 = {}u;\n\
         const SKY_SOL_ANG: u32 = {}u;\n\
         const SKY_LADOS: array<u32, {m}> = array<u32, {m}>({});\n\
         const SKY_X0: array<u32, {m}> = array<u32, {m}>({});\n\
         const SKY_Y0: array<u32, {m}> = array<u32, {m}>({});\n{CORPO}",
        crate::sol::RUGOSIDADES,
        crate::sol::ANGULOS,
        lista(&LADOS),
        lista(&X0),
        lista(&Y0),
        m = NIVEIS,
    )
}
