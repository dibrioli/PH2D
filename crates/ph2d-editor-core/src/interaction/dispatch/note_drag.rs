//! ⭐⭐ **O ARRASTO DE UMA NOTA pela pega dela** (ordem do dono, 2026-09-30: *«Coloque os 10
//! pontinhos de arrastar também nas notas»*) — e a pergunta que o criar e o largar partilham:
//! *em que secção está este `y`?*
//!
//! ⚠️ **A geometria vem do HIT-INDEX**, como no arrasto de secção: o painel regista o cabeçalho de
//! cada secção com o id dela e o fundo de cada nota com o id da ranhura, logo *«que secções e que
//! notas estão à vista, e onde»* é exactamente o que o quadro anterior pintou.
//!
//! ⛔ **Uma lei, três leitores:** o botão direito ([`seccao_sob`], para a nota nascer na secção
//! onde se clicou), o Up do arrasto ([`lugar_da_queda`]) e a marca que o painel desenha enquanto
//! a mão anda (a mesma [`lugar_da_queda`], sobre o hit-index do quadro que ele pinta). Duas contas
//! punham a marca num sítio e a nota noutro.

use crate::interaction::{HitIndex, NoteDrag, WidgetStore};
use crate::zones::Rect;
use ph2d_a11y::NodeId;

/// Down primário sobre `id`: se é a pega de uma nota, semeia o arrasto dela.
pub(super) fn seed(store: &mut WidgetStore, id: NodeId, x: f32, y: f32) {
    if let Some(index) = crate::ids::note_of_grip(id)
        && let Some(panel) = store.panel_at(x, y)
        && index < store.notes_for_panel(panel).len()
    {
        store.begin_note_drag(panel, index, x, y);
    }
}

/// O rect dentro de `painel` — pelo centro, que é a pergunta de um cabeçalho que se recorta.
fn dentro(painel: Rect, r: Rect) -> bool {
    painel.contains(r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// ⭐ **Os cabeçalhos de secção à vista num painel, por ordem de `y`** — `(secção, topo)`.
/// Vale para o Inspector (`LIVE_SECTION_IDS`) e para a Galeria (`SECTION_IDS`).
#[must_use]
pub fn seccoes_do_painel(hit_index: &HitIndex, painel: Rect) -> Vec<(NodeId, f32)> {
    let mut out: Vec<(NodeId, f32)> = hit_index
        .iter_registrations()
        .filter(|(id, r)| {
            (crate::ids::LIVE_SECTION_IDS.contains(id) || crate::ids::SECTION_IDS.contains(id))
                && dentro(painel, *r)
        })
        .map(|(id, r)| (id, r.y))
        .collect();
    out.sort_by(|a, b| a.1.total_cmp(&b.1));
    out.dedup_by_key(|(id, _)| *id);
    out
}

/// ⭐⭐ **A secção que contém `y`** — a última cujo cabeçalho começa acima dele; acima de todas,
/// a primeira. `None` num painel sem secções (a nota vai para o fim do painel).
#[must_use]
pub fn seccao_sob(seccoes: &[(NodeId, f32)], y: f32) -> Option<NodeId> {
    seccoes
        .iter()
        .rev()
        .find(|(_, top)| *top <= y)
        .or_else(|| seccoes.first())
        .map(|(id, _)| *id)
}

/// ⭐⭐ **Onde uma nota largada em `y` cai** — `(secção, posição entre as notas dela)`, com a nota
/// arrastada fora da contagem. A posição é quantas notas daquela secção têm o meio acima de `y`.
#[must_use]
pub fn lugar_da_queda(
    store: &WidgetStore,
    hit_index: &HitIndex,
    drag: &NoteDrag,
    y: f32,
) -> Option<(Option<NodeId>, usize)> {
    let painel = store.panel_rect(drag.panel)?;
    let seccao = seccao_sob(&seccoes_do_painel(hit_index, painel), y);
    let notas = store.notes_for_panel(drag.panel);
    let rank = hit_index
        .iter_registrations()
        .filter(|(id, r)| {
            dentro(painel, *r)
                && crate::ids::NOTE_SLOT_IDS
                    .iter()
                    .position(|s| s == id)
                    .is_some_and(|i| {
                        i != drag.index && notas.get(i).is_some_and(|n| n.section == seccao)
                    })
        })
        .filter(|(_, r)| r.y + r.h * 0.5 < y)
        .count();
    Some((seccao, rank))
}

/// Up: um arrasto ACTIVO move a nota. Um Down+Up parado na pega não faz nada.
pub(super) fn drop(store: &mut WidgetStore, hit_index: &HitIndex, y: f32) {
    let Some(drag) = store.end_note_drag() else {
        return;
    };
    if !drag.active {
        return;
    }
    if let Some((seccao, rank)) = lugar_da_queda(store, hit_index, &drag, y) {
        store.note_move(drag.panel, drag.index, seccao, rank);
    }
}

#[cfg(test)]
#[path = "note_drag_tests.rs"]
mod tests;
