//! Acabar um gesto sem o largar: fechá-lo pendurado, cancelá-lo com `Esc`, e as cópias vivas de um
//! arrasto com `Alt` — mais o lote de desfazer que devolve os originais.

use ph2d_board_model::{BoardDoc, BoardOp, Element, ElementId, History};

use crate::gesture::{Gesture, same_pose};
use crate::{Editor, live};

impl Editor {
    /// `Esc` a meio de um gesto: devolve o documento ao início dele. `false` = não havia gesto.
    /// ⭐ Fecha um gesto PENDURADO — um cujo largar nunca chegou (a shell entregou-o a outro, ou ele
    /// caiu fora da janela): o traço da caneta guarda-se tal como está (um passo); outro gesto volta
    /// ao início dele. Quem chama é a vista, antes de um carregar que o gesto não pode engolir (um
    /// botão da barra). `false` = não havia gesto.
    pub fn finish_gesture(&mut self, doc: &mut BoardDoc, history: &mut History) -> bool {
        self.ink_finish(doc, history) || self.cancel_gesture(doc)
    }

    pub fn cancel_gesture(&mut self, doc: &mut BoardDoc) -> bool {
        if self.ink_cancel(doc) {
            return true;
        }
        let Some(g) = self.gesture.take() else {
            return false;
        };
        match g {
            Gesture::Move {
                originals, copies, ..
            } => {
                if copies.is_empty() {
                    for op in undo_ops(doc, &originals) {
                        let _ = op.apply(doc);
                    }
                } else {
                    for id in copies {
                        let _ = BoardOp::Delete(id).apply(doc);
                    }
                }
            }
            Gesture::Resize { originals, .. }
            | Gesture::Rotate { originals, .. }
            | Gesture::Grid { originals, .. } => {
                for op in undo_ops(doc, &originals) {
                    let _ = op.apply(doc);
                }
            }
            Gesture::EndDrag { original, .. } | Gesture::Bend { original, .. } => {
                let _ = BoardOp::Put(original).apply(doc);
            }
            Gesture::Create { id: Some(id), .. } | Gesture::Connect { id: Some(id), .. } => {
                let _ = BoardOp::Delete(id).apply(doc);
            }
            Gesture::Marquee { base, .. } => self.selection = base,
            _ => {}
        }
        true
    }

    /// Cópias vivas de `els` (ids novos, à frente de tudo, na mesma ordem relativa; as setas presas
    /// às cópias); devolve os ids.
    pub(crate) fn duplicate_live(&mut self, doc: &mut BoardDoc, els: &[Element]) -> Vec<ElementId> {
        let mut copies = els.to_vec();
        self.detach_outside(doc, &mut copies);
        let mut map = std::collections::BTreeMap::new();
        for el in &mut copies {
            let id = doc.mint_id();
            map.insert(el.id, id);
            el.id = id;
            el.version = 0;
            el.set_imported_seed(None);
        }
        crate::wire::remap(&mut copies, &map);
        let roots = crate::command::remap_organize(doc, &mut copies, &map);
        self.unfitted.extend(roots);
        let ids: Vec<ElementId> = copies
            .into_iter()
            .map(|mut el| {
                el.z = doc.z_on_top();
                let id = el.id;
                live(doc, el);
                id
            })
            .collect();
        self.selection = ids
            .iter()
            .copied()
            .filter(|id| doc.get(*id).is_some_and(|e| !e.is_hidden()))
            .collect();
        ids
    }
}

/// O lote que devolve o documento ao início do gesto («pôr os originais») — só dos que mudaram:
/// um arrasto que voltou ao sítio não é passo. NÃO o aplica: é o passo que o `History` guarda.
pub(crate) fn undo_ops(doc: &BoardDoc, originals: &[Element]) -> Vec<BoardOp> {
    originals
        .iter()
        .filter(|o| doc.get(o.id).is_some_and(|now| !same_pose(now, o)))
        .map(|o| BoardOp::Put(o.clone()))
        .collect()
}
