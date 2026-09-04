//! ⭐⭐⭐ **O RECORTE DA MARCHA LEVA UMA MARGEM — e a caixa do BORDO não** (2026-09-01).
//!
//! # ⛔⛔⛔ O defeito: um recorte que ENCOSTA na peça faz o traçador parar em cima dela
//!
//! Desde que o [`crate::bounds::Ball::aabb`] devolve as meias-extensões em vez do cubo, a caixa
//! **toca** a superfície — é essa a definição dela. E um traçador de esferas anda o **valor** do
//! campo: um raio que entra exactamente sobre a peça lê `f ≈ 0`, dá passos de tamanho zero e fica
//! parado, enquanto a marcha honesta (passo fixo minúsculo) continua a andar. As duas deixam de
//! concordar **sem que o campo tenha deixado de ser um minorante**.
//!
//! ⚠️ **Foi bisectado modificador a modificador**, e nenhum bordo está errado: os `1 000` trios e os
//! `100` pares de `‖∇f‖ ≤ 1` ficaram verdes com todos os apertos. O que mudou foi só **o sítio onde
//! o raio começa** — a lei da inclinação apertou a caixa em `7 %`–`11 %` e
//! `the_deformed_rosette_agrees_with_an_honest_march` passou de `6` para `16` pixels divergentes
//! (de `6 844`).
//!
//! # ⭐ A margem VARRIDA (2026-09-01, `160²`, caixa `0,35³`)
//!
//! | margem | imagem | `[]` | `[Bend]` | `[Twist]` | `[Taper]` | `[Bend, Twist]` | trio |
//! |---:|---|---:|---:|---:|---:|---:|---:|
//! | `0` | ⛔ roseta **16**/`6 844` | `1,0` | `8,3` | `11,5` | `12,3` | `23,6` | `89,0` |
//! | `0,005` | ✅ | `5,6` | `10,1` | `13,5` | `15,7` | `23,5` | `89,2` |
//! | **`0,01`** | ✅ | `6,4` | `10,6` | `14,0` | `16,6` | `23,4` | `89,5` |
//! | `0,02` | ✅ | `7,2` | `11,0` | `14,5` | `17,6` | `23,2` | `90,2` |
//! | `0,05` | ✅ | `8,3` | `11,3` | `14,7` | `18,7` | `22,6` | `92,5` |
//!
//! ⭐ **Fica o DOBRO da primeira célula que cura**, e não a primeira: uma fixtura um pouco pior do
//! que a roseta ainda tem de passar, e o preço de duplicar é **`≤ +1,0` passo por raio** em qualquer
//! pilha — *uma escolha na BORDA da grade é uma grade curta demais*.
//!
//! ⚠️ **O que a margem custa é concentrado nas peças BARATAS** — uma caixa sem modificador nenhum
//! paga `1,0 → 6,4` e o trio paga `89,0 → 89,5`. Um raio que não tem nada a fazer entra na caixa e
//! anda até à peça; um que tem muito já andava.
//!
//! ⚠️ **Declarado:** o `[Twist]` fica `+0,4` acima do que shipa hoje (`13,6`), que é `+3 %` no
//! deformador mais barato — é a única célula da tabela que não melhora, e a razão é ele não ganhar
//! nada nesta wave (o alcance dele voltou à esfera, ver [`crate::stack::axis_reach`]).
//!
//! # ⚠️ Por que a margem NÃO vive no bordo
//!
//! O exportador quer a caixa **justa** — é dela que sai a resolução da grade — e ele já soma a
//! margem dele por cima (`PAD_FRACTION`). Uma margem no [`crate::bounds::Ball::aabb`] seria paga
//! duas vezes lá e não seria vista por quem lê o bordo para enquadrar a câmera.

/// Quanta margem o recorte da marcha leva, em fracção da **maior** extensão da peça.
///
/// ⛔ **Ela só ENCOLHE**, e a cerca dos dois lados é o gate
/// `the_march_clip_has_the_margin_it_was_measured_to_need` (que mede a PROPRIEDADE) mais o
/// `a_stack_of_deformers_never_costs_the_march_more_than_it_did` (que apanha um valor grande demais
/// pelo custo). Ver a tabela no topo deste módulo antes de lhe tocar.
pub const MARCH_CLIP_PAD: f32 = 0.01;

/// ⭐ **A caixa que a marcha percorre** — a do bordo mais a [`MARCH_CLIP_PAD`].
///
/// ⚠️ **É a porta**: o produto, as duas sondas de ladrilho e os gates de `‖∇f‖` têm de perguntar
/// aqui. Um deles a ler o `aabb` cru mediria uma região **mais pequena** do que a que o raio de
/// facto visita, e um defeito na casca entre as duas ficaria invisível.
#[must_use]
pub fn march_clip(ball: crate::bounds::Ball) -> ([f32; 3], [f32; 3]) {
    let (lo, hi) = ball.aabb();
    // ⚠️ A margem é **uniforme** e sai da MAIOR extensão: numa peça achatada a extensão do eixo
    // fino é ~zero, e uma margem proporcional a ela seria zero — que é exactamente o caso em que o
    // raio entra rente à superfície.
    let base = (0..3).map(|e| (hi[e] - lo[e]).abs()).fold(0.0f32, f32::max);
    let pad = base * MARCH_CLIP_PAD;
    (
        [lo[0] - pad, lo[1] - pad, lo[2] - pad],
        [hi[0] + pad, hi[1] + pad, hi[2] + pad],
    )
}

#[cfg(test)]
mod tests {
    use crate::bounds::Ball;

    /// ⭐ A margem existe, é simétrica, e uma peça achatada recebe-a nos TRÊS eixos.
    ///
    /// ⛔ Prova de mutação: tirar a margem da MAIOR extensão e pô-la por eixo devolve `0` no eixo
    /// fino, que é o único onde ela era precisa.
    #[test]
    fn a_flat_piece_gets_the_margin_on_every_axis() {
        let b = Ball::of([0.0; 3], 1.0, [0.5, 0.5, 0.001]);
        let (lo, hi) = super::march_clip(b);
        let (a_lo, a_hi) = b.aabb();
        for e in 0..3 {
            let m = a_lo[e] - lo[e];
            assert!(m > 0.0, "eixo {e} sem margem");
            assert!(
                (m - (hi[e] - a_hi[e])).abs() < 1e-6,
                "eixo {e}: a margem não é simétrica"
            );
        }
        assert!(
            (a_lo[2] - lo[2]) > 0.9 * (a_lo[0] - lo[0]),
            "o eixo fino recebeu menos margem do que o grosso"
        );
    }
}
