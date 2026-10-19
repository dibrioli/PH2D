//! W23 B5(1) — arrastar o segmento do MEIO de uma seta cotovelo ([doc 13]): a pega do trecho em Z
//! move-o só na perpendicular e grava a fracção do vão (`Connector::set_mid`), UM passo de desfazer;
//! o duplo clique larga-o (volta ao de nascença). Num trecho que não sai em Z a bolinha continua a
//! criar um ponto de ajuste (`wire.rs`).
//!
//! [doc 13]: ../../../docs/MiroClone/13_trecho_do_cotovelo.md

use ph2d_board_model::{BoardDoc, BoardOp, Element, ElementId, History};
use ph2d_board_route::ElbowMid;

use crate::gesture::Gesture;
use crate::{Editor, Pointer, live};

impl Editor {
    /// O segmento do meio de uma seta seleccionada sob `p`: a seta, o trecho e o segmento.
    #[must_use]
    pub(crate) fn segment_at(&self, doc: &BoardDoc, p: Pointer) -> Option<(ElementId, usize, ElbowMid)> {
        let reach = self.metrics.handle * p.px;
        self.selection
            .iter()
            .filter(|id| doc.get(**id).and_then(Element::connector).is_some())
            .find_map(|id| {
                let r = self.routes.get(*id)?;
                r.elbows.iter().enumerate().find_map(|(leg, e)| {
                    let e = (*e)?;
                    let d = (e.at[0] - p.world[0]).hypot(e.at[1] - p.world[1]);
                    (d <= reach).then_some((*id, leg, e))
                })
            })
    }

    /// Carregar na pega do segmento do meio do trecho `leg`.
    pub(crate) fn begin_slide(
        &mut self,
        doc: &BoardDoc,
        id: ElementId,
        leg: usize,
        p: Pointer,
    ) -> Option<Gesture> {
        let mid = self.routes.get(id)?.elbows.get(leg).copied().flatten()?;
        Some(Gesture::Slide {
            id,
            leg,
            mid,
            start: p.world,
            moved: false,
            original: doc.get(id)?.clone(),
        })
    }

    /// Arrastar o segmento: o ponteiro anda, o segmento acompanha-o só no eixo dele.
    pub(crate) fn slide_move(&mut self, doc: &mut BoardDoc, g: &mut Gesture, p: Pointer) {
        let Gesture::Slide {
            leg,
            mid,
            start,
            moved,
            original,
            ..
        } = g
        else {
            return;
        };
        let d = [p.world[0] - start[0], p.world[1] - start[1]];
        *moved |= d[0].hypot(d[1]) > self.metrics.drag * p.px;
        if !*moved {
            return;
        }
        let frac = mid.frac([mid.at[0] + d[0], mid.at[1] + d[1]]);
        let mut el = original.clone();
        if let Some(c) = el.connector_mut() {
            c.set_mid(*leg, Some(frac));
            live(doc, el);
        }
    }

    /// Largar o segmento: UM passo de desfazer; um toque sem arrastar não muda nada.
    pub(crate) fn slide_up(&mut self, doc: &mut BoardDoc, history: &mut History, g: Gesture) {
        let Gesture::Slide {
            id,
            moved,
            original,
            ..
        } = g
        else {
            return;
        };
        if !moved {
            let _ = BoardOp::Put(original).apply(doc);
        } else if doc.get(id).is_some_and(|now| now.kind != original.kind) {
            history.record(vec![BoardOp::Put(original)], doc);
        }
    }

    /// Duplo clique no segmento do meio: larga-o (volta ao de nascença), UM passo. `false` = não havia
    /// segmento fixo ali.
    pub(crate) fn let_go_segment_at(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        p: Pointer,
    ) -> bool {
        self.routes.sync(doc);
        let Some((id, leg, _)) = self.segment_at(doc, p) else {
            return false;
        };
        let Some(mut el) = doc.get(id).cloned() else {
            return false;
        };
        let Some(c) = el.connector_mut().filter(|c| c.mid(leg).is_some()) else {
            return false;
        };
        c.set_mid(leg, None);
        history.apply(doc, vec![BoardOp::Put(el)]);
        true
    }
}
