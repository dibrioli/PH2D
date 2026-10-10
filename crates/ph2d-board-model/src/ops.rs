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
        let inverse = self.apply_inner(doc);
        if inverse.is_some() {
            doc.rev = crate::element::Rev::fresh();
        }
        inverse
    }

    fn apply_inner(self, doc: &mut BoardDoc) -> Option<BoardOp> {
        match self {
            BoardOp::Put(mut el) => {
                let prev = doc.elements.get(&el.id).cloned();
                el.version = prev
                    .as_ref()
                    .map_or(el.version, |p| p.version.max(el.version))
                    + 1;
                // Os ids de grupo partilham o contador (`mint_group`): um grupo que chega com o elemento
                // (um ficheiro lido, uma colagem) também o sobe. Um id sorteado numa sessão ao vivo não.
                let local = |id: u64| id & crate::LIVE_ID_BIT == 0;
                for id in std::iter::once(el.id.0).chain(el.groups().iter().map(|g| g.0)) {
                    if local(id) {
                        doc.next_id = doc.next_id.max(id);
                    }
                }
                if doc.top_z.as_ref().is_none_or(|t| el.z > *t) {
                    doc.top_z = Some(el.z.clone());
                }
                let id = el.id;
                if el.as_frame().is_some() && !el.deleted {
                    doc.frames.insert(id);
                } else {
                    doc.frames.remove(&id);
                }
                let veil = prev.as_ref().and_then(|p| {
                    p.mind()
                        .filter(|_| !p.deleted)
                        .map(|m| m.hidden || m.collapsed)
                });
                if let Some(p) = &prev {
                    doc.mind_unlink(p);
                }
                doc.elements.insert(id, el);
                doc.mind_link(id, veil);
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
                doc.frames.remove(&id);
                doc.mind_unlink(&before);
                doc.mind_link(id, before.mind().map(|m| m.hidden || m.collapsed));
                Some(BoardOp::Put(before))
            }
        }
    }
}

impl BoardOp {
    /// O id do elemento que a operação muda.
    #[must_use]
    pub fn id(&self) -> ElementId {
        match self {
            BoardOp::Put(el) => el.id,
            BoardOp::Delete(id) => *id,
        }
    }
}

/// Aplica um lote (um gesto) e devolve o lote inverso, já na ordem de desfazer. Os mapas mentais que
/// o lote tocou (antes ou depois) saem arrumados, no MESMO passo ([`crate::mind::settle`], M17).
pub fn apply_batch(doc: &mut BoardDoc, ops: Vec<BoardOp>) -> Vec<BoardOp> {
    let mut roots = std::collections::BTreeSet::new();
    let mut touched = Vec::with_capacity(ops.len());
    let mut inverse: Vec<BoardOp> = Vec::with_capacity(ops.len());
    for op in ops {
        let id = op.id();
        crate::mind::roots_of(doc, id, &mut roots);
        touched.push(id);
        inverse.extend(op.apply(doc));
    }
    for id in touched {
        crate::mind::roots_of(doc, id, &mut roots);
    }
    if !roots.is_empty() {
        inverse.extend(crate::mind::settle(doc, &roots));
    }
    inverse.reverse();
    inverse
}

#[cfg(test)]
#[path = "ops_tests.rs"]
mod tests;
