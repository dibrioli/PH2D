//! ⭐⭐ **AS LEIS DE COERÇÃO de uma linha do painel** (W139) — como um valor é encostado a uma
//! cerca, e onde cada aresta mora na lista de uma forma.
//!
//! # Por que um arquivo irmão
//!
//! O [`super::dims_write`] passou as `700` linhas do gate de LOC ao receber as duas formas da W139,
//! e ⛔ *split, nunca allowlist*. O corte é por responsabilidade: ali vive **quem escreve cada
//! linha** (um `match` de 60 formas), aqui vivem as **leis que valem para todas** — encostar a uma
//! parede, encostar a um piso, e achar o filete e o chanfro **perguntando à [`super::dims`]** em vez
//! de a uma segunda enumeração das 21 primitivas com aresta.

use super::dims;
use super::dims_write_edge::ROUND_MARGIN;
use crate::Primitive;

/// `value`, mantido **estritamente abaixo** de `ceiling` — a folga é uma fração do próprio tecto,
/// pela razão do [`ROUND_MARGIN`] (num alvo de `0,01` um épsilon fixo seria o tecto inteiro).
pub(super) fn keep_below(value: f32, ceiling: f32) -> f32 {
    value.min(ceiling * (1.0 - ROUND_MARGIN))
}

/// `value`, mantido **estritamente acima** de `floor` — a irmã do [`keep_below`].
pub(super) fn keep_above(value: f32, floor: f32) -> f32 {
    value.max(floor / (1.0 - ROUND_MARGIN))
}

/// Onde fica o filete na lista desta forma, se ela tiver um.
pub(super) fn round_index(p: &Primitive) -> Option<usize> {
    dims(p).iter().position(|d| d.key == "field.dim.round")
}

/// Onde fica o **chanfro**, se ela tiver um.
///
/// ⭐ **A pergunta é feita à [`dims`], e não a uma lista escrita à mão** — é isso que faz uma forma
/// nova receber o slider sem uma linha aqui, e é a razão de este arquivo não ter uma segunda
/// enumeração das 21 primitivas com aresta.
pub(super) fn chamfer_index(p: &Primitive) -> Option<usize> {
    dims(p).iter().position(|d| d.key == "field.dim.chamfer")
}

/// Onde fica o chanfro das **PONTAS** (W143) — hoje só a estrela o tem.
///
/// ⚠️ **A pergunta é feita à [`dims`] como as irmãs**, e não a uma lista de formas: a segunda forma
/// que precise de uma aresta com tecto próprio recebe a fileira sem uma linha aqui.
pub(super) fn tip_chamfer_index(p: &Primitive) -> Option<usize> {
    dims(p)
        .iter()
        .position(|d| d.key == "field.dim.tip_chamfer")
}
