//! ⭐⭐⭐⭐ **AS CAIXAS DO PEDIDO** — o recorte do raio primário e a grade que os cones do céu podem
//! marchar. Saiu do [`super`] por tecto de LOC, pela fronteira *qual caixa* / *o que o quadro pede*.

use super::Sonda;

/// ⭐⭐⭐⭐ **O RECORTE DO RAIO PELA CAIXA DA PEÇA** — a porta que o produto e os gates perguntam.
///
/// O raio entra na caixa e sai dela, em vez de percorrer o vazio até ao `t_max`: é o recorte que a
/// marcha da CPU já faz, e é por isso que ele existe aqui — sem ele os dois motores começavam o raio
/// em `t` diferentes, e numa quina viva o ponto de paragem salta.
///
/// ⛔⛔ **A caixa é a [`ph2d_field_eval::bounds_clip::march_clip`], NUNCA o `aabb` cru**, e a razão
/// está medida (`diag_a_grade_de_longe_contra_a_referencia`): com a caixa crua o Δt máximo contra
/// a CPU ia a `4,0e-1` e a silhueta discordava em `12` pixels na cena `=29`; com a da marcha, `0`
/// pixels e o `p99` de `4,8e-7` **em todas as cenas** — melhor do que SEM recorte nenhum (`8`
/// pixels na mesma cena), porque sem ele os dois motores partem de sítios diferentes.
///
/// ⚠️ `res = 0` é **só o recorte**, e é o valor de omissão; `res > 0` acrescenta a grade de longe,
/// que está RECUSADA por medição (ver o doc do [`crate::preview::LONGE_RES`]).
#[must_use]
pub fn a_caixa_da_marcha(
    bola: ph2d_field_eval::bounds::Ball,
    longe: Option<u32>,
) -> Option<ph2d_field_gpu::longe::Longe> {
    let (lo, hi) = ph2d_field_eval::bounds_clip::march_clip(bola);
    longe.map(|res| ph2d_field_gpu::longe::Longe {
        lo,
        hi,
        res,
        perto: crate::preview::LONGE_PERTO,
        so_ceu: false,
        grade_caixa: None,
    })
}

/// ⭐⭐⭐⭐ **A caixa que o pedido leva**, pela [`Sonda`]: o recorte de sempre, ou — com
/// [`Sonda::ceu_na_grade`] — o recorte mais uma grade que SÓ os cones do céu leem, assada na caixa
/// da ESFERA da peça (a cerca dos cones), nunca na justa (`docs/Render3d/03` §W9, «a oclusão na
/// grade»).
#[must_use]
pub fn a_caixa_do_pedido(
    bola: ph2d_field_eval::bounds::Ball,
    sonda: Sonda,
) -> Option<ph2d_field_gpu::longe::Longe> {
    match sonda.ceu_na_grade {
        Some(res) => a_caixa_da_marcha(bola, Some(res)).map(|l| ph2d_field_gpu::longe::Longe {
            so_ceu: true,
            grade_caixa: Some((
                bola.center.map(|c| c - bola.radius),
                bola.center.map(|c| c + bola.radius),
            )),
            ..l
        }),
        None => a_caixa_da_marcha(bola, sonda.longe),
    }
}
