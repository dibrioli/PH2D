//! ⭐⭐⭐ **A CAIXA DA PEÇA DOBRADA** — o arco por aritmética de intervalos, e não o cubo do raio.
//!
//! # ⛔⛔⛔ O defeito que isto cura (report do Enio, 2026-09-01: *«muitíssimo lento»*)
//!
//! A lei da dobra em [`crate::bounds::canonical_step`] escrevia a caixa como `[raio, h₁, raio]` — o
//! **cubo** do raio nos dois eixos do plano do arco. Numa caixa `0,35 × 0,35 × 0,30` a `0,12`
//! voltas isso é `0,957` em `X` e `Z`, e a peça dobrada de facto chega a `0,375` e `0,376`:
//! **`2,5×` de folga em dois eixos de uma vez**.
//!
//! E ela não fica ali: a caixa da dobra é a entrada da torção, que é a entrada da inclinação, e é
//! ela que o `bend_reach` lê para dizer quanta parede o divisor cobra. *Uma folga na primeira lei
//! da pilha multiplica-se por todas as seguintes*
//! ([`crates/ph2d-field-render/tests/what_a_stack_of_deformers_costs_the_march.rs`]).
//!
//! # ⭐ A conta, e por que ela é ESTÁVEL
//!
//! O mapa directo (material → mundo), fora da banda, é
//!
//! ```text
//! Rr = ρ − xₘ·s ;  θ = zₘ/ρ
//! X  = (ρ − Rr·cos θ)·s ;  Z = Rr·sin θ ;  Y = y
//! ```
//!
//! ⚠️ **Escrito assim ele CANCELA** para `κ → 0`: `ρ` e `Rr·cos θ` são dois números enormes cuja
//! diferença é minúscula. A forma algébrica equivalente
//!
//! ```text
//! X = ρ·(1 − cos θ)·s + xₘ·cos θ
//! ```
//!
//! não cancela — o primeiro termo **tende a zero** com a curvatura e o segundo é a identidade. *Uma
//! caixa de bordo calculada com cancelamento é uma caixa que corta a peça, e um corte não avisa.*
//!
//! # ⚠️ Fora da banda o mapa é RÍGIDO, e a cauda entra por soma
//!
//! Além das bordas da banda o eixo continua recto: um ponto a `t` de distância da borda fica a `t`
//! do fim do arco, em alguma direcção do plano. ⇒ a caixa do arco é **engordada por `t`** em `X` e
//! `Z` (`t` = o troço material que sobra de cada lado, mais o `falloff` que suaviza a quina).
//! *Conservador de propósito: a direcção exacta da tangente pouparia pouco e é mais uma conta que
//! pode estar errada num sinal.*

use std::f64::consts::{PI, TAU};

/// Quanto a caixa é engordada para absorver o erro de arredondamento das funções trigonométricas.
///
/// ⚠️ **Não é um épsilon de gosto:** o termo `ρ·(1 − cos θ)` avalia `1 − cos` com erro absoluto da
/// ordem de `ε`, logo o produto erra `ε·ρ = ε/|κ|`. Uma parte por milhão da própria extensão cobre
/// isso com muitas ordens de grandeza de folga, e o preço dela é invisível ao lado dos `2,5×` que a
/// lei nova recupera. *A assimetria do [`crate::bounds`] manda: a folga custa resolução, o aperto
/// corta a peça.*
const PAD_REL: f64 = 1e-6;

/// Abaixo desta curvatura vezes extensão a dobra é indistinguível da identidade, e a conta do arco
/// deixa de ser a mais precisa das duas — fica-se com a lei antiga (a corda), que ali é apertada.
const ARC_FLOOR: f64 = 1e-3;

/// ⭐ **A caixa da peça DOBRADA, no referencial canónico** — `None` quando a lei antiga serve.
///
/// `k` é a curvatura **efectiva** (já saturada por [`crate::stack_bend::bend_curvature`], que é a
/// que a árvore usa), e `lower`/`upper`/`falloff` são a banda no eixo material.
///
/// ⚠️ Devolve `(lo, hi)` por eixo, em coordenadas do mundo local — o chamador é que decide como as
/// converter em meias-extensões à volta do centro da bola dele.
pub(crate) fn bent_extent(
    center: [f32; 3],
    half: [f32; 3],
    k: f64,
    lower: f64,
    upper: f64,
    falloff: f64,
) -> Option<([f64; 3], [f64; 3])> {
    if k == 0.0 || !k.is_finite() || !(lower.is_finite() && upper.is_finite()) {
        return None;
    }
    let c = [
        f64::from(center[0]),
        f64::from(center[1]),
        f64::from(center[2]),
    ];
    let h = [
        f64::from(half[0]).max(0.0),
        f64::from(half[1]).max(0.0),
        f64::from(half[2]).max(0.0),
    ];
    let x = (c[0] - h[0], c[0] + h[0]);
    let z = (c[2] - h[2], c[2] + h[2]);
    let rho = (1.0 / k).abs();
    if k.abs() * z.0.abs().max(z.1.abs()) < ARC_FLOOR {
        return None;
    }
    let s = if k < 0.0 { -1.0 } else { 1.0 };
    // A banda vive no eixo MATERIAL, e o que sobra dela de cada lado sai do arco como troço recto.
    let (banda_lo, banda_hi) = (lower.min(upper), lower.max(upper));
    let arco = (z.0.clamp(banda_lo, banda_hi), z.1.clamp(banda_lo, banda_hi));
    let cauda = (banda_lo - z.0).max(0.0).max((z.1 - banda_hi).max(0.0)) + falloff.abs();
    let theta = (arco.0 / rho, arco.1 / rho);
    let cos_t = cos_range(theta.0, theta.1);
    let sin_t = sin_range(theta.0, theta.1);
    // `X = ρ·(1 − cos θ)·s + xₘ·cos θ` — ver o doc do módulo para por que não é `ρ − Rr·cos θ`.
    let versina = ((1.0 - cos_t.1) * rho, (1.0 - cos_t.0) * rho);
    let versina = if s > 0.0 {
        versina
    } else {
        (-versina.1, -versina.0)
    };
    let xw = mul(x, cos_t);
    // `Z = Rr·sin θ`, com `Rr = ρ − xₘ·s` — e `Rr > 0` é o que a margem de dobra garante.
    let xs = if s > 0.0 { x } else { (-x.1, -x.0) };
    let rr = ((rho - xs.1).max(0.0), (rho - xs.0).max(0.0));
    let zw = mul(rr, sin_t);
    let pad = |v: f64| PAD_REL * (1.0 + v.abs());
    let lo = [
        versina.0 + xw.0 - cauda - pad(versina.0 + xw.0),
        c[1] - h[1],
        zw.0 - cauda - pad(zw.0),
    ];
    let hi = [
        versina.1 + xw.1 + cauda + pad(versina.1 + xw.1),
        c[1] + h[1],
        zw.1 + cauda + pad(zw.1),
    ];
    Some((lo, hi))
}

/// O produto de dois intervalos — os quatro cantos, que é o exacto para uma bilinear.
fn mul(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let p = [a.0 * b.0, a.0 * b.1, a.1 * b.0, a.1 * b.1];
    (
        p.iter().copied().fold(f64::INFINITY, f64::min),
        p.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    )
}

/// Existe `n` com `fase + n·2π` dentro de `[a, b]`? — é assim que um extremo INTERIOR se declara.
///
/// ⚠️ Sem isto o intervalo lê-se só nas pontas, e um arco que passe por cima de `θ = 0` devolveria
/// um `cos` máximo menor do que `1`. *Uma caixa que ignora o extremo interior corta exactamente a
/// dobra mais fechada.*
fn tem_multiplo(a: f64, b: f64, fase: f64) -> bool {
    let n = ((a - fase) / TAU).ceil();
    fase + n * TAU <= b
}

fn cos_range(a: f64, b: f64) -> (f64, f64) {
    let (a, b) = if a <= b { (a, b) } else { (b, a) };
    let mut lo = a.cos().min(b.cos());
    let mut hi = a.cos().max(b.cos());
    if tem_multiplo(a, b, 0.0) {
        hi = 1.0;
    }
    if tem_multiplo(a, b, PI) {
        lo = -1.0;
    }
    (lo, hi)
}

fn sin_range(a: f64, b: f64) -> (f64, f64) {
    let (a, b) = if a <= b { (a, b) } else { (b, a) };
    let mut lo = a.sin().min(b.sin());
    let mut hi = a.sin().max(b.sin());
    if tem_multiplo(a, b, PI * 0.5) {
        hi = 1.0;
    }
    if tem_multiplo(a, b, -PI * 0.5) {
        lo = -1.0;
    }
    (lo, hi)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ Uma dobra que não dobra devolve a peça — o caso de omissão da porta.
    #[test]
    fn a_bend_too_small_to_see_hands_the_question_back() {
        assert!(bent_extent([0.0; 3], [0.35, 0.35, 0.30], 0.0, -2.0, 2.0, 0.0).is_none());
        assert!(bent_extent([0.0; 3], [0.35, 0.35, 0.30], 1e-6, -2.0, 2.0, 0.0).is_none());
    }

    /// ⛔ **O extremo INTERIOR**: um arco de meia volta tem `cos = −1` no meio, e ler só as pontas
    /// devolveria `cos ∈ [cos(±π/2), …] = [0, 0]`.
    #[test]
    fn the_interior_extreme_of_the_arc_is_seen() {
        assert_eq!(cos_range(-PI, PI).0, -1.0);
        assert_eq!(cos_range(-0.5, 0.5).1, 1.0);
        assert_eq!(sin_range(1.0, 2.0).1, 1.0);
        assert_eq!(sin_range(-2.0, -1.0).0, -1.0);
    }

    /// ⭐⭐ **A caixa contém a peça amostrada** — o mapa directo, ponto a ponto, contra o intervalo.
    ///
    /// ⛔ Prova de mutação: trocar `ρ·(1 − cos θ)·s` por `ρ·(1 − cos θ)` (sem o sinal) reprova com
    /// `κ < 0`, e apagar a cauda reprova com a banda mais curta do que a peça.
    #[test]
    fn the_interval_box_contains_every_bent_point() {
        for k in [-3.0f64, -0.75, 0.12, 0.75, 3.0] {
            for (lo_b, hi_b) in [(-2.0f64, 2.0f64), (-0.1, 0.05)] {
                let (c, h) = ([0.1f32, 0.0, -0.05], [0.35f32, 0.35, 0.30]);
                let (lo, hi) = bent_extent(c, h, k, lo_b, hi_b, 0.0).expect("arco");
                let s = if k < 0.0 { -1.0 } else { 1.0 };
                let rho = (1.0 / k).abs();
                for i in 0..=40 {
                    for j in 0..=40 {
                        let xm = f64::from(c[0]) + f64::from(h[0]) * (f64::from(i) / 20.0 - 1.0);
                        let zm = f64::from(c[2]) + f64::from(h[2]) * (f64::from(j) / 20.0 - 1.0);
                        // Fora da banda o eixo segue recto: o ponto fica no fim do arco, mais o
                        // troço que sobra, na direcção da tangente.
                        let za = zm.clamp(lo_b.min(hi_b), lo_b.max(hi_b));
                        let t = zm - za;
                        let th = za / rho;
                        let rr = rho - xm * s;
                        let xw =
                            (rho * (1.0 - th.cos())).mul_add(s, xm * th.cos()) - t * th.sin() * s;
                        let zw = rr * th.sin() + t * th.cos();
                        assert!(
                            xw >= lo[0] - 1e-9 && xw <= hi[0] + 1e-9,
                            "κ={k} banda=({lo_b},{hi_b}) x={xw} fora de [{}, {}]",
                            lo[0],
                            hi[0]
                        );
                        assert!(
                            zw >= lo[2] - 1e-9 && zw <= hi[2] + 1e-9,
                            "κ={k} banda=({lo_b},{hi_b}) z={zw} fora de [{}, {}]",
                            lo[2],
                            hi[2]
                        );
                    }
                }
            }
        }
    }
}
