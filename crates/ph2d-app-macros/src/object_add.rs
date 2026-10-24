//! **Add ▸ Macro** (W18, `docs/Macros/16_W18_plano.md` §2.1, decisão 5 do dono: «deve ser criada como os
//! outros objetos são criados no + da hierarchy e deve ter seu gizmo como um objeto vazio»).
//!
//! Nasce o vazio da shell (`spawn_root`: nome único, `StableId`, `Transform`) com um `Macro` vazio. Sem
//! visual próprio, a porta `is_empty_object` dá-lhe o anel e o gizmo do vazio; o que ele gravar nasce
//! dentro dele.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::object_add::{AddEntry, AddGroup};

/// O item do menu.
pub const MACRO: AddEntry = AddEntry::new("object_add.macro", AddGroup::Empty);
/// As entradas desta família, na ordem do modal.
pub const ENTRIES: &[AddEntry] = &[MACRO];

/// Cria o objecto de `entry`, se é desta família; devolve os bits dele.
pub fn add(
    entry: AddEntry,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<Result<u64, &'static str>> {
    if entry != MACRO {
        return None;
    }
    let bits = spawn_root(sim, entry.key.tr());
    let ent =
        ph2d_ecs::Entity::try_from_bits(bits).and_then(|e| sim.world_mut().get_entity_mut(e).ok());
    let Some(mut ent) = ent else {
        return Some(Err(ph2d_i18n::tr("object_add.not_born")));
    };
    ent.insert(ph2d_ecs::Macro::default());
    Some(Ok(bits))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_menu_macro_faz_nascer_um_vazio_com_macro() {
        let mut sim = SimWorld::new();
        let bits = add(MACRO, &mut sim, |s, name| {
            s.world_mut()
                .spawn((ph2d_ecs::Name::new(name), ph2d_ecs::Transform::IDENTITY))
                .id()
                .to_bits()
        })
        .expect("é desta família")
        .expect("nasce");
        let e = ph2d_ecs::Entity::from_bits(bits);
        assert_eq!(
            sim.world().get::<ph2d_ecs::Macro>(e),
            Some(&ph2d_ecs::Macro::default())
        );
        assert_eq!(
            sim.world().get::<ph2d_ecs::Name>(e).map(|n| n.0.as_str()),
            Some("Macro")
        );
    }
}
