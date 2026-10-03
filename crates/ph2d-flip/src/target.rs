//! ⭐⭐ **O ALVO da autoria** — o desenho em edição e a camada activa nele (spec/06 F3 do Flip:
//! *Flip ▸ Object · Draw · Edit*, `docs/UI_New_and_Simple/spec/06_tipos_e_modos_de_objeto.md`).
//!
//! Até à F3 cada gesto, painel e pré-visualização resolvia à mão *«o 1.º objecto e a camada activa
//! (ou a de topo)»* — catorze cópias da mesma regra, e com dois desenhos o traço caía sempre no
//! primeiro. Agora o desenho é o do MODO (quem o escreve é a família do modo, em
//! `ph2d_app_flip::flip_mode`) e a regra da camada mora aqui, uma vez.
//!
//! ⛔ **Não é documento**: não se grava nem entra no diff do desfazer (o `FlipDoc` é `PartialEq`
//! para o undo, e um alvo lá dentro faria de cada entrada num modo um passo de histórico).

use crate::doc::FlipDoc;
use crate::ids::{FlipObjectId, LayerId};
use crate::object::FlipObject;

/// Onde a autoria do Flip cai.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FlipTarget {
    /// O desenho que o modo Draw/Edit tem em mãos. `None` = modo Object: nenhum gesto cai.
    pub object: Option<FlipObjectId>,
    /// A camada activa nele (a linha escolhida no painel). `None`, ou uma que já não existe no
    /// desenho, = a de topo.
    pub layer: Option<LayerId>,
}

impl FlipTarget {
    /// O alvo em `object`, com a camada `layer`.
    #[must_use]
    pub const fn on(object: FlipObjectId, layer: Option<LayerId>) -> Self {
        Self {
            object: Some(object),
            layer,
        }
    }

    /// O desenho em edição, se ainda existe.
    #[must_use]
    pub fn drawing(self, doc: &FlipDoc) -> Option<&FlipObject> {
        doc.object(self.object?)
    }

    /// ⭐ **A camada que a autoria usa em `obj`**: a activa se é dele, senão a de topo.
    #[must_use]
    pub fn layer_in(self, obj: &FlipObject) -> Option<LayerId> {
        self.layer
            .filter(|id| obj.layer(*id).is_some())
            .or_else(|| obj.layers().last().map(|l| l.id))
    }

    /// O desenho e a camada, resolvidos — `None` sem desenho em edição ou sem camada.
    #[must_use]
    pub fn resolve(self, doc: &FlipDoc) -> Option<(FlipObjectId, LayerId)> {
        let obj = self.drawing(doc)?;
        Some((obj.id, self.layer_in(obj)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_drawings() -> (FlipDoc, FlipObjectId, FlipObjectId) {
        let mut doc = FlipDoc::new();
        let a = doc.push_object("A");
        doc.object_mut(a).expect("A").add_layer("a1");
        let b = doc.push_object("B");
        let ob = doc.object_mut(b).expect("B");
        ob.add_layer("b1");
        ob.add_layer("b2");
        (doc, a, b)
    }

    /// ⭐ **O alvo é o desenho em EDIÇÃO, não o primeiro** — o defeito que a F3 cura.
    ///
    /// (Mutação: `drawing` devolver `doc.objects().first()` ⇒ RED.)
    #[test]
    fn the_target_is_the_drawing_in_edit_not_the_first() {
        let (doc, _a, b) = two_drawings();
        let (oid, lid) = FlipTarget::on(b, None)
            .resolve(&doc)
            .expect("B tem camadas");
        assert_eq!(oid, b);
        assert_eq!(
            Some(lid),
            doc.object(b).and_then(|o| o.layers().last()).map(|l| l.id),
            "sem camada escolhida, a de topo de B"
        );
    }

    /// **Sem desenho em edição, nada resolve** (Object: nenhum gesto cai).
    #[test]
    fn object_mode_resolves_nothing() {
        let (doc, _, _) = two_drawings();
        assert_eq!(FlipTarget::default().resolve(&doc), None);
    }

    /// **Uma camada que o desenho não tem não serve** — cai na de topo dele; a que ele tem, fica.
    #[test]
    fn a_layer_the_drawing_lacks_falls_back_to_the_top() {
        let (doc, _a, b) = two_drawings();
        let b_obj = doc.object(b).expect("B");
        let (b1, b2) = (b_obj.layers()[0].id, b_obj.layers()[1].id);
        let gone = LayerId(9_999);
        assert!(
            b_obj.layer(gone).is_none(),
            "o controlo: B não tem esta camada"
        );
        assert_eq!(FlipTarget::on(b, Some(gone)).layer_in(b_obj), Some(b2));
        assert_eq!(FlipTarget::on(b, Some(b1)).layer_in(b_obj), Some(b1));
    }
}
