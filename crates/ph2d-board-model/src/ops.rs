//! **As operações** — a ÚNICA porta de mudança de um [`BoardDoc`]. O undo guarda o inverso que
//! [`BoardOp::apply`] devolve; a colaboração (Etapa 2) transmitirá as mesmas operações.

use serde::{Deserialize, Serialize};

use crate::{BoardDoc, Element, ElementId};

/// Uma mudança num quadro.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BoardOp {
    /// Põe o elemento (novo, ou a reposição de um estado anterior inteiro).
    Put(Element),
    /// Apaga (lápide) o elemento.
    Delete(ElementId),
}

impl BoardOp {
    /// Aplica a operação e devolve a que a desfaz. `None` = não mudou nada (id desconhecido ou já
    /// apagado) — quem chama não empilha um passo de undo vazio.
    pub fn apply(self, doc: &mut BoardDoc) -> Option<BoardOp> {
        match self {
            BoardOp::Put(mut el) => {
                let prev = doc.elements.get(&el.id).cloned();
                el.version = prev
                    .as_ref()
                    .map_or(el.version, |p| p.version.max(el.version))
                    + 1;
                doc.next_id = doc.next_id.max(el.id.0);
                let id = el.id;
                doc.elements.insert(id, el);
                Some(match prev {
                    Some(p) if !p.deleted => BoardOp::Put(p),
                    _ => BoardOp::Delete(id),
                })
            }
            BoardOp::Delete(id) => {
                let el = doc.elements.get_mut(&id).filter(|e| !e.deleted)?;
                let before = el.clone();
                el.deleted = true;
                el.version += 1;
                Some(BoardOp::Put(before))
            }
        }
    }
}

/// Aplica um lote (um gesto) e devolve o lote inverso, já na ordem de desfazer.
pub fn apply_batch(doc: &mut BoardDoc, ops: Vec<BoardOp>) -> Vec<BoardOp> {
    let mut inverse: Vec<BoardOp> = ops.into_iter().filter_map(|op| op.apply(doc)).collect();
    inverse.reverse();
    inverse
}

#[cfg(test)]
#[path = "ops_tests.rs"]
mod tests;
