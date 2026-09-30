//! ⭐⭐ **O ARRASTO DE UMA SECÇÃO pela pega** (ordem do dono, 2026-09-29: *«um ícone de 10 pontos
//! que serve para arrastar e reorganizar as seções»*).
//!
//! O Down primário na pega semeia o arrasto; o Move avança-o; o Up de um arrasto ACTIVO resolve a
//! queda e grava a ordem nova no store — o painel pinta por ela no quadro seguinte.
//!
//! ⚠️ **A geometria vem do HIT-INDEX, e não de uma tabela à parte:** cada secção pintada regista o
//! rect do cabeçalho com o id dela (`begin_section` do Inspector), logo a lista *«que secções estão
//! à vista, e por que ordem»* é exactamente a que o quadro anterior pintou — nenhuma cópia a
//! envelhecer entre o painel e o despacho.

use crate::interaction::{HitIndex, SECCOES_FIXAS, WidgetStore, alvo_da_queda, reordena_seccoes};
use ph2d_a11y::NodeId;

/// Down primário sobre `id`: se é a pega de uma secção que se arrasta, semeia o arrasto.
pub(super) fn seed(store: &mut WidgetStore, id: NodeId, y: f32) {
    if let Some(section) = crate::ids::section_of_grip(id)
        && !SECCOES_FIXAS.contains(&section)
    {
        store.begin_section_drag(section, y);
    }
}

/// ⭐ **Os cabeçalhos à vista, pela ordem pintada** — `(secção, meio do cabeçalho em y)`.
#[must_use]
pub fn cabecalhos_a_vista(hit_index: &HitIndex) -> Vec<(NodeId, f32)> {
    let mut out: Vec<(NodeId, f32)> = hit_index
        .iter_registrations()
        .filter(|(id, _)| crate::ids::LIVE_SECTION_IDS.contains(id) && !SECCOES_FIXAS.contains(id))
        .map(|(id, r)| (id, r.y + r.h * 0.5))
        .collect();
    out.sort_by(|a, b| a.1.total_cmp(&b.1));
    out.dedup_by_key(|(id, _)| *id);
    out
}

/// Up: um arrasto ACTIVO grava a ordem nova. Um Down+Up parado na pega não faz nada.
pub(super) fn drop(store: &mut WidgetStore, hit_index: &HitIndex, y: f32) {
    let Some(drag) = store.end_section_drag() else {
        return;
    };
    if !drag.active {
        return;
    }
    let heads = cabecalhos_a_vista(hit_index);
    if !heads.iter().any(|(id, _)| *id == drag.section) {
        return;
    }
    let before = alvo_da_queda(&heads, drag.section, y);
    let displayed: Vec<NodeId> = heads.iter().map(|(id, _)| *id).collect();
    let nova = reordena_seccoes(&displayed, store.section_order(), drag.section, before);
    store.set_section_order(nova);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids;
    use crate::zones::Rect;

    fn store() -> WidgetStore {
        WidgetStore::with_capacity(4)
    }

    /// ⭐⭐ **O gesto inteiro, do Down na pega ao Up, grava a ordem que se larga.** *Mutação: não
    /// exigir `active` ⇒ o Down+Up parado reordena; ignorar o `drop` ⇒ a ordem fica vazia.*
    #[test]
    fn arrastar_a_pega_do_sheet_para_cima_do_render_grava_a_ordem() {
        let mut hit = HitIndex::default();
        let h = 26.0;
        for (i, id) in [
            ids::INSP_LIVE_RENDER_SECTION,
            ids::INSP_LIVE_COLOR_SECTION,
            ids::INSP_LIVE_SHEET_SECTION,
        ]
        .into_iter()
        .enumerate()
        {
            hit.register(id, Rect::new(0.0, 100.0 + i as f32 * 40.0, 300.0, h));
        }
        let mut s = store();
        // um Down+Up PARADO na pega não reordena
        seed(&mut s, ids::INSP_LIVE_SHEET_GRIP, 190.0);
        drop(&mut s, &hit, 190.0);
        assert!(
            s.section_order().is_empty(),
            "um clique parado na pega reordenou"
        );
        // o gesto: pega do Sheet, larga acima do Render
        seed(&mut s, ids::INSP_LIVE_SHEET_GRIP, 190.0);
        s.update_section_drag(95.0);
        drop(&mut s, &hit, 95.0);
        assert_eq!(
            s.section_order(),
            &[
                ids::INSP_LIVE_SHEET_SECTION,
                ids::INSP_LIVE_RENDER_SECTION,
                ids::INSP_LIVE_COLOR_SECTION
            ]
        );
    }

    /// ⛔ O Nome e a Visibilidade não se arrastam — a pega delas nem semeia.
    #[test]
    fn as_seccoes_fixas_nao_semeiam_arrasto() {
        let mut s = store();
        seed(&mut s, ids::INSP_LIVE_NAME_GRIP, 10.0);
        assert!(s.section_drag().is_none());
        seed(&mut s, ids::INSP_LIVE_TRANSFORM_GRIP, 10.0);
        assert!(s.section_drag().is_some());
    }
}
