//! W23 B3 (a) — a letra do mapa mental muda-se pela RAIZ e escala o mapa inteiro (o Miro: «Fonte só
//! pela raiz, e escala o mapa todo», `docs/MiroClone/04_planear_mapa_e_modelos.md` §1.1).

use std::collections::BTreeSet;

use ph2d_board_model::{BoardDoc, BoardOp, Element, History};

use crate::Editor;

impl Editor {
    /// A letra da selecção (e das formas que nascerem). Numa raiz de mapa escala o mapa todo — a letra
    /// e o tamanho de cada nó por `size / letra da raiz` (o tamanho de um nó é linear na letra,
    /// [`crate::mindmap::node_size`]) — e o layout re-arruma, tudo num passo; um nó que não é raiz
    /// não muda (a letra do mapa é a da raiz).
    pub fn set_font_size(&mut self, doc: &mut BoardDoc, history: &mut History, size: f64) {
        self.style.font_size = size;
        let mut put = Vec::new();
        let mut roots = BTreeSet::new();
        for el in self.editable(doc) {
            match el.mind() {
                Some(m) if m.parent.is_some() => {}
                Some(_) => {
                    let k = size / el.style().font_size.max(f64::EPSILON);
                    let [cx, cy] = el.center();
                    let mut root = scaled(el, k);
                    root.x = cx - root.w / 2.0;
                    root.y = cy - root.h / 2.0;
                    put.push(root);
                    put.extend(
                        doc.mind_descendants(el.id)
                            .into_iter()
                            .filter_map(|id| doc.get(id))
                            .map(|d| scaled(d, k)),
                    );
                    roots.insert(el.id);
                }
                None => {
                    let mut el = el.clone();
                    el.style_mut().font_size = size;
                    put.push(el);
                }
            }
        }
        // O que o passo devolve: os nós dos mapas inteiros (o layout mexe-os todos) e o resto tocado.
        let mut ids: BTreeSet<_> = put.iter().map(|e| e.id).collect();
        for r in &roots {
            ids.extend(doc.mind_descendants(*r));
        }
        let inverse: Vec<BoardOp> = ids
            .iter()
            .filter_map(|id| doc.get(*id).cloned().map(BoardOp::Put))
            .collect();
        for el in put {
            BoardOp::Put(el).apply(doc);
        }
        if !roots.is_empty() {
            let _ = ph2d_board_model::mind_settle(doc, &roots);
        }
        history.record(inverse, doc);
    }
}

fn scaled(el: &Element, k: f64) -> Element {
    let mut out = el.clone();
    out.style_mut().font_size *= k;
    out.w *= k;
    out.h *= k;
    out
}
