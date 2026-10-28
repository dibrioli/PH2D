//! W4 do Kanban (ADR-0195 §3) — **pôr um Kanban no quadro**: a referência a um documento do projecto
//! que já existe (quem chama cria-o no `KanbanSet`). Apagar o elemento nunca apaga o documento.

use ph2d_board_model::{
    BoardDoc, BoardKanban, BoardOp, Element, ElementId, History, KanbanId, Rgba, Style,
};

use crate::Editor;

/// O tamanho de nascença, em unidades do mundo: três listas do Trello (272 + 8 de intervalo) com as
/// margens e o cabeçalho, em 16:9 — o que a aba mostra a zoom 1 num ecrã pequeno.
pub const KANBAN_BIRTH: [f64; 2] = [960.0, 540.0];

impl Editor {
    /// Põe o Kanban `kanban` com o meio em `at`, reduzido para caber em `fit` (meia vista, como o vídeo),
    /// seleccionado; UM passo de desfazer (que tira só a referência).
    pub fn insert_kanban(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        kanban: KanbanId,
        at: [f64; 2],
        fit: [f64; 2],
    ) -> ElementId {
        self.commit_text(doc, history);
        let [nw, nh] = KANBAN_BIRTH;
        let k = (fit[0] / nw).min(fit[1] / nh).min(1.0);
        let (w, h) = (nw * k, nh * k);
        let style = Style::new(None, None, Rgba(ph2d_board_model::DEFAULT_INK));
        let el = Element::new_kanban(
            doc.mint_id(),
            doc.z_on_top(),
            BoardKanban::new(kanban, style),
            [at[0] - w / 2.0, at[1] - h / 2.0, w, h],
        );
        let id = el.id;
        let _ = BoardOp::Put(el).apply(doc);
        history.record(vec![BoardOp::Delete(id)], doc);
        self.selection = std::iter::once(id).collect();
        id
    }
}

#[cfg(test)]
mod tests {
    use ph2d_board_model::{BoardSet, History, KanbanId};

    /// Nasce no meio, redondo, seleccionado, reduzido à vista; desfazer tira-o.
    #[test]
    fn a_kanban_is_born_centred_selected_and_undoable() {
        let mut set = BoardSet::default();
        let id = set.create("k".into());
        let doc = &mut set.get_mut(id).unwrap().doc;
        let mut h = History::default();
        let mut ed = crate::images::tests::editor();
        let made = ed.insert_kanban(doc, &mut h, KanbanId(5), [0.0, 0.0], [480.0, 1e9]);
        let el = doc.get(made).unwrap();
        assert_eq!([el.x, el.y, el.w, el.h], [-240.0, -135.0, 480.0, 270.0]);
        assert_eq!(el.kanban().unwrap().kanban, KanbanId(5));
        assert!(el.style().round);
        assert_eq!(ed.selection().iter().copied().collect::<Vec<_>>(), vec![made]);
        h.undo(doc);
        assert!(doc.get(made).is_none());
    }
}
