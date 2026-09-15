//! ⭐⭐⭐ **O CÉU DO PRODUTO, em WGSL** — o que preenche o `ENV_SLOT` do [`ph2d_material::wgsl`].
//!
//! ⚠️ **As duas tabelas viajam, não se refazem.** O pré-filtro da caixa é o produto de uma sequência
//! de Hammersley com uma contagem de amostras **medida** (`docs/Render3d/05`), e reconstruí-lo no
//! dispositivo seria uma segunda resposta à mesma pergunta — que diverge no dia em que alguém mexer
//! numa delas. ⇒ `49 × 513 + 513` floats (`101 KB`) sobem uma vez e ficam.
//!
//! ⚠️ **O `lobe_shrink` NÃO vem aqui** — ele é `f64` e é constante por material, logo viaja no
//! material (ver o `ENV_SLOT`). *Uma lei que precisa de `f64` e não varia por pixel é uma constante.*

/// Quantos `f32` o [`constants`] escreve.
pub const CONSTANTS: usize = 12;

/// As constantes do céu, na ordem que o [`SOURCE`] lê.
#[must_use]
pub fn constants() -> [f32; CONSTANTS] {
    let (_, _, share, amp) = crate::studio::Studio::of_the_product()
        .softbox
        .expect("o estúdio do produto tem caixa")
        .tables();
    let b = ph2d_light::ENV_BASE;
    let s = ph2d_light::ENV_SLOPE;
    [
        b[0],
        b[1],
        b[2],
        ph2d_light::AMBIENT,
        s[0],
        s[1],
        s[2],
        share,
        amp,
        0.0,
        0.0,
        0.0,
    ]
}

/// As duas tabelas, concatenadas: `spec` (`ROUGH_N × ANGLE_N`) e depois `diff` (`ANGLE_N`).
#[must_use]
pub fn tables() -> Vec<f32> {
    let (spec, diff, _, _) = crate::studio::Studio::of_the_product()
        .softbox
        .expect("o estúdio do produto tem caixa")
        .tables();
    let mut v = Vec::with_capacity(spec.len() + diff.len());
    v.extend_from_slice(spec);
    v.extend_from_slice(diff);
    v
}

/// O corpo. Ele espera `ceu: Ceu` no binding que o chamador declarar e `tabela: array<f32>`.
pub const SOURCE: &str = r#"
struct Ceu {
    base_ambient: vec4<f32>,   // ENV_BASE.rgb, AMBIENT
    slope_share: vec4<f32>,    // ENV_SLOPE.rgb, share
    amp: vec4<f32>,            // amp, _, _, _
};

const ROUGH_N: u32 = 49u;
const ANGLE_N: u32 = 513u;
// O `k` cru: o `ENV_SLOPE` já vem convolvido com o lóbulo cosseno (`(2/3)·k`).
const RAW: f32 = 1.5;
const SQRT_2: f32 = 1.4142135623730951;

fn angle_axis(cos_psi: f32) -> f32 {
    return sqrt(max(1.0 - clamp(cos_psi, -1.0, 1.0), 0.0));
}

fn tabela_spec(ri: u32, ai: u32) -> f32 { return tabela[ri * ANGLE_N + ai]; }
fn tabela_diff(ai: u32) -> f32 { return tabela[ROUGH_N * ANGLE_N + ai]; }

/// `BoxPrefilter::specular` — bilinear em (rugosidade, ângulo).
fn softbox_specular(alpha: f32, cos_psi: f32) -> f32 {
    let r = sqrt(clamp(alpha, 0.0, 1.0)) * f32(ROUGH_N - 1u);
    let a = angle_axis(cos_psi) / SQRT_2 * f32(ANGLE_N - 1u);
    let r0 = min(u32(r), ROUGH_N - 2u);
    let a0 = min(u32(a), ANGLE_N - 2u);
    let fr = r - f32(r0);
    let fa = a - f32(a0);
    let baixo = tabela_spec(r0, a0) + (tabela_spec(r0, a0 + 1u) - tabela_spec(r0, a0)) * fa;
    let cima = tabela_spec(r0 + 1u, a0) + (tabela_spec(r0 + 1u, a0 + 1u) - tabela_spec(r0 + 1u, a0)) * fa;
    return baixo + (cima - baixo) * fr;
}

/// `BoxPrefilter::diffuse` — linear no ângulo.
fn softbox_diffuse(cos_psi: f32) -> f32 {
    let a = angle_axis(cos_psi) / SQRT_2 * f32(ANGLE_N - 1u);
    let a0 = min(u32(a), ANGLE_N - 2u);
    let fa = a - f32(a0);
    return tabela_diff(a0) + (tabela_diff(a0 + 1u) - tabela_diff(a0)) * fa;
}

/// `Studio::radiance` — ⚠️ o `shrink` chega pronto (é `f64` na CPU e constante por material).
fn env_radiance(dir: vec3<f32>, alpha: f32, shrink: f32) -> vec3<f32> {
    let up = shrink * dir.y;
    // ⚠️ **O eixo da caixa é `+y`**, logo `cos ψ` é a própria componente `y`.
    let base = 1.0 - ceu.slope_share.a + ceu.amp.x * softbox_specular(alpha, dir.y);
    let ambient = ceu.base_ambient.a;
    return ambient * (ceu.base_ambient.rgb * base + RAW * ceu.slope_share.rgb * up);
}

/// `Studio::irradiance`. ⚠️ Vista (`y` para cima) → canvas (`y` para baixo): a rampa da
/// `ph2d_light::env_ambient` recebe `-n.y`, e o `env_ambient` volta a negar. Aqui está inline, e o
/// sinal é o MESMO: `up = +n.y`.
fn env_irradiance(n: vec3<f32>) -> vec3<f32> {
    let ambient = ceu.base_ambient.a;
    let rampa = ambient * (ceu.base_ambient.rgb + ceu.slope_share.rgb * n.y);
    let delta = ceu.amp.x * softbox_diffuse(n.y) - ceu.slope_share.a;
    return rampa + ambient * ceu.base_ambient.rgb * delta;
}
"#;

#[cfg(test)]
#[path = "studio_wgsl_tests.rs"]
mod tests;
