//! **Links e etiquetas** (W5, módulo à parte de `organize.rs`, que estava no tecto de LOC): o modo
//! de ligar (`Ctrl+Alt+K`, o «Link to» do Miro), seguir um link, e pôr/tirar uma etiqueta nas notas.

use std::collections::BTreeSet;

use ph2d_board_model::{BoardDoc, BoardOp, Element, ElementId, History};

use crate::{Editor, Notice};

impl Editor {
    /// `Ctrl+Alt+K` (o do Miro): com a selecção já ligada, tira o link; senão, o próximo clique
    /// escolhe o destino (o Miro: *«Choose Link to … click another object on the board»*).
    pub(crate) fn link_command(&mut self, doc: &mut BoardDoc, history: &mut History) -> bool {
        let sel = self.selected(doc);
        if sel.is_empty() {
            return false;
        }
        if sel.iter().all(|el| el.link().is_some()) {
            let ops = sel
                .into_iter()
                .map(|el| {
                    let mut el = el.clone();
                    el.set_link(None);
                    BoardOp::Put(el)
                })
                .collect();
            history.apply(doc, ops);
            return true;
        }
        self.linking = true;
        self.notice = Some(Notice::LinkPick);
        true
    }

    /// O clique do modo de ligar: a selecção passa a levar ao elemento debaixo do ponteiro (UM
    /// passo); no vazio (ou nela própria) o modo fecha sem mudar nada. `true` = havia modo.
    pub(crate) fn link_click(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        target: Option<ElementId>,
    ) -> bool {
        if !std::mem::take(&mut self.linking) {
            return false;
        }
        let Some(t) = target.filter(|t| !self.selection.contains(t)) else {
            return true;
        };
        let ops = self
            .selected(doc)
            .into_iter()
            .map(|el| {
                let mut el = el.clone();
                el.set_link(Some(t));
                BoardOp::Put(el)
            })
            .collect();
        history.apply(doc, ops);
        true
    }

    /// O modo de ligar está aberto (a dica da vista).
    #[must_use]
    pub fn is_linking(&self) -> bool {
        self.linking
    }

    /// Seguir o link de `id`: o destino, ou o aviso de que ele foi apagado.
    pub fn follow_link(&mut self, doc: &BoardDoc, id: ElementId) -> Option<ElementId> {
        let to = doc.get(id)?.link()?;
        if doc.get(to).is_none() {
            self.notice = Some(Notice::LinkGone);
            return None;
        }
        self.selection = BTreeSet::from([to]);
        Some(to)
    }

    /// ⭐ **Uma etiqueta nas notas seleccionadas** (o Miro: até [`ph2d_board_model::MAX_TAGS`] por
    /// nota): se todas já a têm, sai de todas; senão entra nas que não a têm. UM passo.
    pub fn toggle_tag(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        name: &str,
        color: ph2d_board_model::Rgba,
    ) -> bool {
        let name = name.trim();
        let notes: Vec<Element> = self
            .editable(doc)
            .into_iter()
            .filter(|el| el.shape().is_some_and(|s| s.kind.is_note()))
            .cloned()
            .collect();
        if name.is_empty() || notes.is_empty() {
            return false;
        }
        let has = |el: &Element| el.tags().iter().any(|t| t.name == name);
        let remove = notes.iter().all(has);
        let mut ops = Vec::new();
        for mut el in notes {
            if remove {
                el.tags_mut().retain(|t| t.name != name);
            } else if has(&el) {
                continue;
            } else if el.tags().len() >= ph2d_board_model::MAX_TAGS {
                self.notice = Some(Notice::TooManyTags);
                continue;
            } else {
                el.tags_mut().push(ph2d_board_model::Tag {
                    name: name.to_owned(),
                    color,
                });
            }
            ops.push(BoardOp::Put(el));
        }
        let any = !ops.is_empty();
        history.apply(doc, ops);
        any
    }
}
