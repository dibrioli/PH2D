//! ⭐ **O desenho Flip no menu Add de objectos** (spec/06 F1) — a entrada e o nascimento.
//!
//! ⚠️ **Até aqui não havia porta de produto para um objecto Flip:** a cena de boot é vazia
//! ([`crate::demo::demo_scene`] só semeia sob `PH2D_FLIP_DEMO`), e a caneta sem objecto descarta o
//! traço. O desenho nasce com UMA camada, para o primeiro traço ter onde cair.
//!
//! ⭐ **Nasce em Draw** (escolha do dono, 03/10: vazio, o próximo gesto é desenhar) — marca
//! [`FlipState::born`] e o quadro do modo entra ([`crate::flip_mode`]).

use ph2d_ecs::SimWorld;
use ph2d_editor_core::object_add::{AddEntry, AddGroup};
use ph2d_flip::FlipDoc;

use crate::state::FlipState;

/// A entrada do menu.
pub const FLIP: AddEntry = AddEntry::new("object_add.flip", AddGroup::TwoD);
/// O que esta família põe no menu.
pub const ENTRIES: &[AddEntry] = &[FLIP];

/// ⭐ **Cria o desenho Flip** e devolve os bits da entidade — `None` se a entrada não é desta
/// família.
///
/// ⚠️ **A entidade nasce pela ponte de SEMPRE** ([`ph2d_flip_entities::entities::sync`]), chamada
/// já aqui para o objecto novo poder ser seleccionado neste quadro; o `sync` do quadro é
/// idempotente.
pub fn add(
    entry: AddEntry,
    sim: &mut SimWorld,
    doc: &mut FlipDoc,
    state: &mut FlipState,
) -> Option<Result<u64, &'static str>> {
    if entry != FLIP {
        return None;
    }
    let oid = doc.push_object(entry.key.tr());
    if let Some(obj) = doc.object_mut(oid) {
        obj.add_layer(ph2d_i18n::tr("object_add.flip.first_layer"));
    }
    ph2d_flip_entities::entities::sync(sim, doc, &mut state.entities);
    state.born = Some(oid);
    Some(
        state
            .entities
            .get(&oid)
            .copied()
            .ok_or_else(|| ph2d_i18n::tr("object_add.not_born")),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **O desenho do menu nasce como objecto FLIP, com uma camada** — a caneta precisa dela.
    ///
    /// (Mutação: tirar o `add_layer` ⇒ RED.)
    #[test]
    fn the_menu_flip_is_born_a_flip_object_with_a_layer() {
        let mut sim = SimWorld::new();
        let mut doc = FlipDoc::new();
        let mut state = FlipState::default();
        let bits = add(FLIP, &mut sim, &mut doc, &mut state)
            .expect("é desta família")
            .expect("nasce");
        let e = ph2d_ecs::Entity::from_bits(bits);
        let oref = sim
            .world()
            .get::<ph2d_ecs::FlipObjectRef>(e)
            .expect("a entidade não é um objecto Flip");
        let obj = doc
            .objects()
            .iter()
            .find(|o| o.id.0 == oref.0)
            .expect("a entidade aponta para o objecto novo");
        assert_eq!(
            obj.layers().len(),
            1,
            "sem camada, o primeiro traço cai no chão"
        );
        assert_eq!(
            state.born,
            Some(obj.id),
            "nasce a pedir o Draw (escolha do dono)"
        );
    }

    /// **Uma entrada de outra família não é desta** — e nada nasce.
    #[test]
    fn an_entry_of_another_family_is_not_ours() {
        let mut sim = SimWorld::new();
        let mut doc = FlipDoc::new();
        let mut state = FlipState::default();
        let empty = ph2d_editor_core::object_add::EMPTY;
        assert!(add(empty, &mut sim, &mut doc, &mut state).is_none());
        assert!(doc.objects().is_empty());
    }
}
