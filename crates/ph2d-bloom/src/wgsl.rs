//! ⭐⭐⭐ **A CADEIA DO BRILHO, em WGSL** — a mesma lei do [`crate::halo`], para quem pinta no
//! dispositivo.
//!
//! # ⛔⛔⛔ Porque ela existe: a medição, não o gosto
//!
//! A cadeia em CPU custa **`24,1 ms` a `445×305`** e **`347,2 ms` a `1898×916`** (a sonda
//! `lei_tests::quanto_custa_a_cadeia`, `--release`, melhor de 5). O primeiro é o quadro de
//! **movimento** do modelador e já não cabe num quadro de `16,7 ms`; o segundo é o **assente** e
//! custa um terço de segundo. ⇒ trazer a imagem do dispositivo para a CPU só para o brilho está
//! fora de questão, e este ficheiro não é uma optimização: *é a única forma de este efeito existir
//! no caminho de omissão do modelador*, que é o do dispositivo.
//!
//! # ⚠️ O CONTRATO: quem usa declara o acesso aos dados
//!
//! Esta crate é uma **folha de zero dependências** e não conhece `wgpu`, buffers nem bind groups —
//! exactamente como a `ph2d_view_transform::wgsl` não conhece. O que ela exporta são **funções
//! puras** que lêem o nível de ORIGEM por uma porta que **o chamador declara**:
//!
//! ```wgsl
//! fn bl_le(i: u32) -> vec3<f32>   // o texel `i` do nível de origem deste passe
//! ```
//!
//! É a mesma forma do `ceu_radiance` que o pintor do campo já exige de quem o chama.
//!
//! ⚠️⚠️ **E ela entra NO MEIO da lei — por isso a porta é a [`fonte`] e não uma const.** O WGSL
//! exige **declaração antes do uso**: o [`CORTE`] não toca em dados (logo vem primeiro, e é por
//! isso que o `bl_le` do chamador o pode chamar — que é como o tecto do *firefly* entra no primeiro
//! degrau **sem um passe e sem um buffer a mais**), e a [`CADEIA`] chama o `bl_le` (logo vem depois
//! dele). *Uma const única obrigaria o chamador a escolher entre não poder cortar e pagar uma cópia
//! do quadro inteiro.*
//!
//! # ⚠️ E a aritmética é a da CPU, linha a linha
//!
//! Os pesos, a ordem das somas e o `fma` são os do [`crate::halo`] — o que muda é só de onde vêm os
//! texels. ⭐ E a amostragem é **bilinear escrita à mão**, nunca um `textureSample`: a referência de
//! CPU faz a sua própria bilinear em UV, e um *sampler* de hardware traz arredondamento próprio que
//! a paridade não conseguiria fechar. *Duas leis iguais amostradas por portas diferentes deixam de
//! ser a mesma lei.*

/// ⭐⭐⭐ **A LEI INTEIRA, com a porta do chamador no sítio certo** — ver a nota do módulo.
///
/// `bl_le` é o texto que declara `fn bl_le(i: u32) -> vec3<f32>` sobre os buffers de quem chama.
#[must_use]
pub fn fonte(bl_le: &str) -> String {
    format!("{CORTE}\n{bl_le}\n{CADEIA}")
}

/// **O que NÃO toca em dados** — o corte e a cor. Vem ANTES da porta do chamador.
///
/// ⚠️ **Todas as funções levam o prefixo `bl_`** — este código partilha o módulo com o pintor, o
/// material, o olhar e a lei do dono, e um nome curto colidiria no dia em que um deles crescesse.
pub const CORTE: &str = r#"
// ⭐⭐⭐ **O CORTE** — o `crate::bright`, incluindo o TECTO por canal antes da luminância.
//
// ⚠️ `tecto` é o `BloomParams::clamp_limit()` já resolvido pelo lado Rust: com o knob desligado ele
// é o maior finito do `Rgba16Float`, logo o `min` não morde nada que um quadro consiga guardar.
fn bl_corte(rgb_in: vec3<f32>, limiar: f32, joelho: f32, tecto: f32) -> vec3<f32> {
    let rgb = min(rgb_in, vec3<f32>(tecto));
    let l = max(rgb.x, max(rgb.y, rgb.z));
    // ⚠️ `l != l` é o teste de `NaN` do WGSL — não há `is_nan`. A mesma cerca do `vt_sanitize`.
    if (l != l || l <= 0.0) { return vec3<f32>(0.0); }
    let k = max(joelho, 0.0);
    let duro = l - limiar;
    var acima = duro;
    if (k > 0.0) {
        let s = clamp(duro + k, 0.0, 2.0 * k);
        acima = max(s * s / (4.0 * k), duro);
    }
    if (acima != acima || acima <= 0.0) { return vec3<f32>(0.0); }
    return rgb * (acima / l);
}

// ⭐ **A COR do halo** — o passo (5) do `crate::halo`: dessatura para a luminância, tinge, escala.
fn bl_cor(rgb: vec3<f32>, saturacao: f32, tinta: vec3<f32>, intensidade: f32) -> vec3<f32> {
    let luz = fma(0.2126, rgb.x, fma(0.7152, rgb.y, 0.0722 * rgb.z));
    let d = fma(vec3<f32>(luz), vec3<f32>(1.0 - saturacao), rgb * saturacao);
    return d * tinta * intensidade;
}
"#;

/// **O que LÊ os dados** — a amostragem e os dois filtros. Vem DEPOIS da porta do chamador.
pub const CADEIA: &str = r#"
// ⭐ A bilinear da CPU (`amostra_bilinear`), com a borda presa. ⚠️ O `-0.5` é o meio do texel: sem
// ele a cadeia inteira desliza meio pixel por nível, e o halo sai deslocado da fonte.
fn bl_amostra_uv(sw: u32, sh: u32, u: f32, v: f32) -> vec3<f32> {
    let x = u * f32(sw) - 0.5;
    let y = v * f32(sh) - 0.5;
    let x0 = floor(x);
    let y0 = floor(y);
    let tx = x - x0;
    let ty = y - y0;
    let xa = min(u32(max(x0, 0.0)), sw - 1u);
    let xb = min(u32(max(x0 + 1.0, 0.0)), sw - 1u);
    let ya = min(u32(max(y0, 0.0)), sh - 1u);
    let yb = min(u32(max(y0 + 1.0, 0.0)), sh - 1u);
    let a = mix(bl_le(ya * sw + xa), bl_le(ya * sw + xb), tx);
    let b = mix(bl_le(yb * sw + xa), bl_le(yb * sw + xb), tx);
    return mix(a, b, ty);
}

// ⭐⭐⭐ **O 13-TAP que DESCE** — o `crate::desce13`, com os mesmos pesos e a mesma ordem de soma.
fn bl_desce13(sw: u32, sh: u32, dw: u32, dh: u32, i: u32, j: u32) -> vec3<f32> {
    let tx = 1.0 / f32(sw);
    let ty = 1.0 / f32(sh);
    let u = (f32(i) + 0.5) / f32(dw);
    let v = (f32(j) + 0.5) / f32(dh);
    let a = bl_amostra_uv(sw, sh, u - 2.0 * tx, v + 2.0 * ty);
    let c = bl_amostra_uv(sw, sh, u + 2.0 * tx, v + 2.0 * ty);
    let g = bl_amostra_uv(sw, sh, u - 2.0 * tx, v - 2.0 * ty);
    let i2 = bl_amostra_uv(sw, sh, u + 2.0 * tx, v - 2.0 * ty);
    let b2 = bl_amostra_uv(sw, sh, u, v + 2.0 * ty);
    let d = bl_amostra_uv(sw, sh, u - 2.0 * tx, v);
    let f = bl_amostra_uv(sw, sh, u + 2.0 * tx, v);
    let h2 = bl_amostra_uv(sw, sh, u, v - 2.0 * ty);
    let e = bl_amostra_uv(sw, sh, u, v);
    let j2 = bl_amostra_uv(sw, sh, u - tx, v + ty);
    let k2 = bl_amostra_uv(sw, sh, u + tx, v + ty);
    let l2 = bl_amostra_uv(sw, sh, u - tx, v - ty);
    let m2 = bl_amostra_uv(sw, sh, u + tx, v - ty);
    let cantos = a + c + g + i2;
    let cruz = b2 + d + f + h2;
    let dentro = j2 + k2 + l2 + m2;
    return fma(vec3<f32>(0.125), e, fma(vec3<f32>(0.03125), cantos, fma(vec3<f32>(0.0625), cruz, dentro * 0.125)));
}

// ⭐⭐⭐ **A TENDA de 9 taps que SOBE** — o `crate::sobe_tenda`. `base` é o
// `BloomParams::upsample_basis`, já resolvido pelo lado Rust (o raio, a anamorfose e o ângulo).
fn bl_sobe_tenda(sw: u32, sh: u32, dw: u32, dh: u32, i: u32, j: u32, base: vec4<f32>) -> vec3<f32> {
    let du = base.xy;
    let dv = base.zw;
    let u = (f32(i) + 0.5) / f32(dw);
    let v = (f32(j) + 0.5) / f32(dh);
    let e = bl_amostra_uv(sw, sh, u, v);
    let b2 = bl_amostra_uv(sw, sh, u + dv.x, v + dv.y);
    let d = bl_amostra_uv(sw, sh, u - du.x, v - du.y);
    let f = bl_amostra_uv(sw, sh, u + du.x, v + du.y);
    let h2 = bl_amostra_uv(sw, sh, u - dv.x, v - dv.y);
    let a = bl_amostra_uv(sw, sh, u - du.x + dv.x, v - du.y + dv.y);
    let c = bl_amostra_uv(sw, sh, u + du.x + dv.x, v + du.y + dv.y);
    let g = bl_amostra_uv(sw, sh, u - du.x - dv.x, v - du.y - dv.y);
    let i2 = bl_amostra_uv(sw, sh, u + du.x - dv.x, v + du.y - dv.y);
    let cruz = b2 + d + f + h2;
    let cantos = a + c + g + i2;
    return fma(vec3<f32>(4.0), e, fma(vec3<f32>(2.0), cruz, cantos)) / 16.0;
}
"#;

#[cfg(test)]
#[path = "wgsl_tests.rs"]
mod wgsl_tests;
