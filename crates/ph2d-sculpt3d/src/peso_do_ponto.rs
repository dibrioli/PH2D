//! ⭐⭐⭐ **A CURVA DE UM PONTO, e a ORDEM do produto** — as duas portas que o
//! dab por-VÉRTICE e o dab por-AMOSTRA partilham.
//!
//! ⛔⛔ **Elas existem porque a tinta fina precisa do MESMO peso avaliado noutro
//! sítio.** A alternativa era escrever a lei uma segunda vez ao lado da
//! primeira — e *uma lei escrita em dois sítios ainda não é uma lei, só uma
//! PORTA é* (a frase que esta casa já pagou no traço uniforme do vector, na
//! média do anel do pente e na ponte da curva do contorno).
//!
//! ⚠️ **A extracção é VERBATIM:** as duas funções abaixo têm exactamente as
//! contas que estavam em linha no [`crate::stroke_dab_core`], na mesma ordem.
//! O caminho por-vértice fica **byte-idêntico**, e quem o afirma são os corpora
//! de oráculo que já correm (o tecido, a pose, o contorno, o plano, o afiado).

use crate::{Brush, Footprint};

/// ⭐ **A CURVA DE QUEDA de um ponto** — dada a distância dele ao centro do dab.
///
/// ⚠️ **A coordenada vem da SILHUETA** (`footprint.at`), e a dureza e as curvas
/// seguem lendo UM número — é isso que faz uma forma nova não pedir um segundo
/// falloff. O `Disc` devolve `dist · inv_r` e portão `1`, ou seja o mundo que
/// já shipa, ao bit.
///
/// ⚠️ **Cada CANAL traz a dureza dele, e a curva é UMA**: a geometria segue a
/// curva que o artista escolheu no pincel; um canal segue a do `s-mode`, que é
/// a lei portada.
#[must_use]
pub(crate) fn curva_do_ponto(
    brush: &Brush,
    footprint: &Footprint,
    from: [f32; 3],
    dist: f32,
    inv_r: f32,
) -> f32 {
    let (raw, gate) = footprint.at(from, dist, inv_r);
    let t = brush.shaped_distance(raw);
    let c = if brush.verb.paints_mask() {
        brush.mask_weight(t)
    } else if brush.verb.paints_color() {
        brush.paint_weight(t)
    } else {
        brush.falloff.weight(t)
    };
    c * gate
}

/// ⭐⭐ **A ORDEM DO PRODUTO, e ela é load-bearing ao BIT.**
///
/// ⛔ A forma "natural" seria derivar um factor do outro, e ela **re-associa**
/// `(falloff × intensity) × keep` para `(falloff × keep) × intensity`: medido
/// no `stroke_dab_core`, **30,4 % dos triplos divergem**, até ~1 ulp. É por
/// isso que esta função existe em vez de cada chamador multiplicar à maneira
/// dele — *dois sítios que multiplicam os mesmos cinco números em ordens
/// diferentes dão dois pincéis.*
///
/// Devolve `(w, shape)`: o segundo é **a metade SEM intensidade**, porque no
/// original o expoente do vinco cai sobre `curva × máscara × alpha` e a
/// intensidade entra depois, linear nos dois termos.
#[must_use]
pub(crate) fn peso_e_forma(
    curve: f32,
    alpha: f32,
    facing: f32,
    intensity: f32,
    keep: f32,
) -> (f32, f32) {
    let fall = curve * alpha * facing;
    (fall * intensity * keep, fall * keep)
}
