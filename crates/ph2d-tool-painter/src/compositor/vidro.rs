//! **O VIDRO — a aguada é um filtro** (doc 48, BUGS #46; o estado da arte: o vidrado de Kubelka–Munk,
//! Curtis et al., SIGGRAPH 1997).
//!
//! A óptica da aguada calcula uma transparência POR CANAL (um vermelho deixa passar o vermelho e
//! absorve o verde e o azul), e um píxel RGBA guarda UM alfa. O píxel da camada continua a ser o que
//! sempre foi — a aparência sobre o BRANCO, com o alfa do canal mais absorvido —, e ao lado dele o
//! plano do vidro guarda o alfa de CADA canal e um selo (o próprio píxel que a aguada escreveu). Sobre
//! o branco nada muda, ao byte; sobre um papel de cor o papel atravessa a pilha canal a canal:
//!
//! `mostrado = W − t·(1 − papel)` (tons de ecrã, ADR-0177)
//!
//! com `W` a pilha sobre o branco e `t` o produto, camada a camada, de `1 − alfa` de cada canal. Sem
//! vidro `t = 1 − alfa` e isto é «cobrir com transparência» (`sobre_o_papel`), a lei de sempre.
//!
//! ⭐ **O selo é o que deixa o plano valer sem que nenhum outro pincel o conheça:** com o selo intacto
//! os alfas são exactos; quando outra ferramenta reescreveu o píxel ([`alfas`]), o vidro CONTINUA o
//! que ela fez — o que subiu o alfa cobriu (cada canal atenua-se pela fracção coberta, a lei do
//! `over`), o que o desceu apagou (os alfas escalam). ⛔ Voltar à lei de um alfa NÃO é neutro sobre o
//! papel de cor: é a aguada acesa — foi o contorno claro do Wet Paint, do Digital e do Impasto por
//! cima de uma aguada (smoke do dono 2026-10-07, BUGS #46).

use super::{LayerId, LayerPixelSource, Region};
use crate::layers::LayerStack;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Um texel do vidro: o píxel RGBA que a aguada escreveu (o selo) e o alfa de cada canal.
pub type Vidro = [u8; 7];

/// Os planos do vidro de um documento, por camada (preguiçosos: só as camadas onde a aguada pintou).
pub type Vidros = BTreeMap<LayerId, Arc<Vec<Vidro>>>;

/// Sela o texel: o píxel escrito e os alfas por canal.
#[inline]
#[must_use]
pub fn sela(px: [u8; 4], alfas: [u8; 3]) -> Vidro {
    [px[0], px[1], px[2], px[3], alfas[0], alfas[1], alfas[2]]
}

/// **Os alfas por canal do píxel `px` (RGBA)** pelo vidro `v`. Selo intacto: os do vidro. Outro pincel
/// reescreveu o píxel: o alfa que SUBIU de `a` para `a'` é uma cobertura `f = (a' − a)/(1 − a)` por
/// cima, e cada canal passa a `1 − (1 − a_c)·(1 − f)` (com `a = 1` a cobertura lê-se pela maior mudança
/// de cor, o seu mínimo); o que DESCEU é uma borracha, e os alfas escalam por `a'/a`. Um vidro nunca
/// selado (zeros) dá o alfa único do píxel, exacto.
#[inline]
#[must_use]
pub fn alfas(px: &[u8], v: &Vidro) -> [u8; 3] {
    let ac = [v[4], v[5], v[6]];
    if px[..4] == v[..4] {
        return ac;
    }
    let (a0, a1) = (f32::from(v[3]) / 255.0, f32::from(px[3]) / 255.0);
    if a1 < a0 {
        let k = a1 / a0;
        return ac.map(|c| (f32::from(c) * k).round() as u8);
    }
    let f = if a0 < 1.0 {
        (a1 - a0) / (1.0 - a0)
    } else {
        (0..3)
            .map(|c| px[c].abs_diff(v[c]))
            .max()
            .map_or(0.0, |d| f32::from(d) / 255.0)
    };
    cobre(ac, f)
}

/// **Uma cobertura `f` por cima** (o `over`: `antes·(1 − f) + cor·f`): cada canal deixa passar
/// `1 − f` do que deixava — `1 − (1 − a_c)·(1 − f)`.
#[inline]
#[must_use]
pub fn cobre(ac: [u8; 3], f: f32) -> [u8; 3] {
    let f = f.clamp(0.0, 1.0);
    ac.map(|c| {
        let a = 1.0 - (1.0 - f32::from(c) / 255.0) * (1.0 - f);
        (a.clamp(0.0, 1.0) * 255.0).round() as u8
    })
}

/// Atualiza a transparência por canal `t` de um texel com a camada por cima: `k` é o factor da
/// máscara, do recorte e da opacidade; `px` o píxel e `v` o vidro dele. Um texel sem vidro (ou cujo selo
/// não bate) tira `alfa·k` dos três canais; com vidro, cada canal tira o SEU alfa — também num texel de
/// alfa 0, a orla de pigmento que sobre o branco se anula (o que devolve = o que tira) e sobre o papel
/// de cor se vê.
#[inline]
pub(crate) fn atravessa(t: &mut [f32; 3], k: f32, px: &[u8], v: Option<&Vidro>) {
    match v.map(|v| alfas(px, v)) {
        Some(ac) => {
            for c in 0..3 {
                t[c] *= 1.0 - (f32::from(ac[c]) / 255.0 * k).min(1.0);
            }
        }
        None => {
            let alfa = f32::from(px[3]) / 255.0 * k;
            for tc in t.iter_mut() {
                *tc *= 1.0 - alfa;
            }
        }
    }
}

/// A pilha sobre o PAPEL `p`: `acc` é o acumulador (cor recta codificada + alfa) e `t` a transparência
/// por canal — `W − t·(1 − p)`, opaco.
#[inline]
#[must_use]
pub(crate) fn sobre_o_papel(acc: [f32; 4], t: [f32; 3], p: [u8; 3]) -> [u8; 4] {
    let mut out = [0u8, 0, 0, 255];
    for c in 0..3 {
        let w = acc[c] * acc[3] + (1.0 - acc[3]);
        out[c] = super::encode_byte(w - t[c] * (1.0 - f32::from(p[c]) / 255.0));
    }
    out
}

/// **A região `region` da pilha sobre o PAPEL `papel`, com o VIDRO** ([`super::vidro`]) — opaca. A
/// porta das saídas de um documento com papel de cor onde a aguada pintou: a pilha sobre o branco e a
/// transparência POR CANAL do papel saem da MESMA passada (`compose::composite_into`), então a máscara, o
/// recorte, a opacidade e os grupos são os da imagem, por construção.
#[must_use]
pub(crate) fn composite_region_sobre_o_papel(
    stack: &LayerStack,
    src: &(impl LayerPixelSource + Sync),
    vidros: &super::vidro::Vidros,
    (width, height): (u32, u32),
    region: Region,
    papel: [u8; 3],
) -> Vec<u8> {
    let (acc, t) =
        super::compose::composite_region_linear(stack, src, width, height, region, Some(vidros));
    let t = t.expect("a passada com vidro devolve a transparência");
    use rayon::prelude::*;
    let mut out = vec![0u8; acc.len() * 4];
    out.par_chunks_mut(4 * 4096)
        .zip(acc.par_chunks(4096))
        .zip(t.par_chunks(4096))
        .for_each(|((o, a), tc)| {
            for ((px, a), tc) in o.as_chunks_mut::<4>().0.iter_mut().zip(a).zip(tc) {
                *px = sobre_o_papel(*a, *tc, papel);
            }
        });
    out
}
