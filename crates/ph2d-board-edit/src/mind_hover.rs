//! W23 B3 (b) — o mapa mental ao PAIRAR, como o Miro (`docs/MiroClone/04_planear_mapa_e_modelos.md`
//! §1.1): o nó sob o ponteiro (sem nada seleccionado) mostra o «+» do filho, e um nó com filhos à
//! vista mostra o «−» de recolher na borda de onde saem os ramos.

use ph2d_board_model::{BoardDoc, ElementId};

use crate::{Editor, Pointer, Tool};

impl Editor {
    /// O nó de mapa dos «+» e do «−»: o seleccionado, ou — sem selecção — o que está sob o ponteiro.
    pub(crate) fn mind_target(&self, doc: &BoardDoc) -> Option<ElementId> {
        if self.editing.is_some() || self.tool != Tool::Select {
            return None;
        }
        self.mind_focus(doc).or_else(|| {
            self.selection
                .is_empty()
                .then_some(self.hover?)
                .filter(|h| {
                    doc.get(*h)
                        .is_some_and(|el| el.mind().is_some() && !el.is_hidden())
                })
        })
    }

    /// O «−» de recolher: o nó, o meio da borda de fora (mundo) e a direcção para fora. Só num nó que
    /// não é a raiz (o mapa recolhe-se ramo a ramo) e com filhos à vista. Quem desenha afasta-o
    /// [`mind_minus_offset`] px do lado: no meio da borda está a pega de redimensionar da moldura.
    #[must_use]
    pub fn mind_minus(&self, doc: &BoardDoc) -> Option<(ElementId, [f64; 2], [f64; 2])> {
        let id = self.mind_target(doc)?;
        let el = doc.get(id)?;
        let m = el.mind()?;
        if m.parent.is_none() || m.collapsed || doc.mind_children(id).is_empty() {
            return None;
        }
        let (vertical, sign) = ph2d_board_model::outward(doc, id);
        let [cx, cy] = el.center();
        let k = if sign > 0.0 { 1.0 } else { -1.0 };
        Some(if vertical {
            (id, [cx, cy + k * el.h / 2.0], [0.0, k])
        } else {
            (id, [cx + k * el.w / 2.0, cy], [k, 0.0])
        })
    }

    /// O nó do «−» sob `p`, se há (o alcance é o da pega, como o «+»).
    pub(crate) fn mind_minus_at(&self, doc: &BoardDoc, p: Pointer) -> Option<ElementId> {
        let reach = self.metrics.handle * p.px;
        let off = mind_minus_offset(&self.metrics) * p.px;
        self.mind_minus(doc)
            .filter(|(_, at, d)| {
                let c = [at[0] + d[0] * off, at[1] + d[1] * off];
                (c[0] - p.world[0]).hypot(c[1] - p.world[1]) <= reach
            })
            .map(|(id, _, _)| id)
    }
}

/// Quanto o «−» se afasta (px de ecrã) da borda: a meio caminho do «+» (`Metrics::dot`), fora da
/// pega de redimensionar que a moldura pinta no meio da borda.
#[must_use]
pub fn mind_minus_offset(m: &crate::Metrics) -> f64 {
    m.dot / 2.0
}
