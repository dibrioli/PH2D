//! W18 (ADR-0185) — **um vídeo entra no quadro**: no meio de `at`, no tamanho de nascença do oráculo
//! (560×315; um `/shorts/` 315×560), reduzido para caber em `fit` (metade da vista) como uma imagem, com o
//! desfazer e a selecção.

use ph2d_board_model::{BoardDoc, BoardOp, Element, ElementId, Embed, History, Rgba, Style, Video};

use crate::Editor;

impl Editor {
    /// Põe o vídeo (`what` = o link como chegou e o que ele é) com o meio em `at`; `natural` = o tamanho de
    /// nascença.
    pub fn insert_embed(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        (link, video): (&str, Video),
        natural: [f64; 2],
        at: [f64; 2],
        fit: [f64; 2],
    ) -> ElementId {
        self.commit_text(doc, history);
        let k = (fit[0] / natural[0]).min(fit[1] / natural[1]).min(1.0);
        let (w, h) = (natural[0] * k, natural[1] * k);
        let mut style = Style::new(None, None, Rgba(ph2d_board_model::DEFAULT_INK));
        style.round = true;
        let e = Embed {
            link: link.trim().to_owned(),
            video,
            style,
        };
        let el = Element::new_embed(
            doc.mint_id(),
            doc.z_on_top(),
            e,
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
    use ph2d_board_model::{BoardSet, History, Video};

    /// O vídeo nasce no meio do ponto, no tamanho do oráculo (ou reduzido à vista), redondo e
    /// seleccionado; desfazer tira-o.
    #[test]
    fn a_video_is_born_centred_selected_and_undoable() {
        let mut set = BoardSet::default();
        let id = set.create("v".into());
        let doc = &mut set.get_mut(id).unwrap().doc;
        let mut h = History::default();
        let mut ed = crate::images::tests::editor();
        let v = Video::YouTube {
            id: "aqz-KE-bpKQ".into(),
            start: 0,
            vertical: false,
        };
        let made = ed.insert_embed(
            doc,
            &mut h,
            (" https://youtu.be/aqz-KE-bpKQ ", v.clone()),
            [560.0, 315.0],
            [100.0, 50.0],
            [1e9; 2],
        );
        let el = doc.get(made).unwrap();
        assert_eq!([el.x, el.y, el.w, el.h], [-180.0, -107.5, 560.0, 315.0]);
        assert_eq!(el.embed().unwrap().link, "https://youtu.be/aqz-KE-bpKQ");
        assert!(el.embed().unwrap().style.round);
        assert_eq!(
            ed.selection().iter().copied().collect::<Vec<_>>(),
            vec![made]
        );
        let small = ed.insert_embed(
            doc,
            &mut h,
            ("x", v),
            [560.0, 315.0],
            [0.0, 0.0],
            [280.0, 280.0],
        );
        assert_eq!(doc.get(small).unwrap().w, 280.0, "cabe em metade da vista");
        let _ = h.undo(doc);
        assert!(doc.get(small).is_none_or(|e| e.deleted));
    }
}
