//! ⭐ **Add ▸ Motion** — o objecto gerador do Motion Nodes nasce pelo menu Add, no grupo 2D (doc
//! ObjectsAndComponents 01 §13, F5).
//!
//! Nasce um objecto VAZIO (o mesmo nascimento do *Empty* — nome único e `StableId` — que a shell
//! passa em `spawn_root`) com o marcador [`ph2d_ecs::MotionGenerator`]. Os nós vêm depois, no modo
//! Edit; a selecção passa a ser ele, e o dono em edição do Motion segue-a (`motion_objeto`).

use ph2d_ecs::SimWorld;
use ph2d_editor_core::object_add::{AddEntry, AddGroup};

/// O item do menu.
pub const MOTION: AddEntry = AddEntry::new("object_add.motion", AddGroup::TwoD);
/// As entradas desta família, na ordem do modal.
pub const ENTRIES: &[AddEntry] = &[MOTION];

/// Cria o objecto de `entry`, se é desta família; devolve os bits dele.
pub fn add(
    entry: AddEntry,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<Result<u64, &'static str>> {
    if entry != MOTION {
        return None;
    }
    let bits = spawn_root(sim, entry.key.tr());
    let Some(e) = ph2d_ecs::Entity::try_from_bits(bits) else {
        return Some(Err(ph2d_i18n::tr("object_add.not_born")));
    };
    let Ok(mut ent) = sim.world_mut().get_entity_mut(e) else {
        return Some(Err(ph2d_i18n::tr("object_add.not_born")));
    };
    ent.insert(ph2d_ecs::MotionGenerator);
    Some(Ok(bits))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_menu_motion_is_born_a_motion_object() {
        let mut sim = SimWorld::new();
        let bits = add(MOTION, &mut sim, |s, name| {
            s.world_mut()
                .spawn((ph2d_ecs::Name::new(name), ph2d_ecs::Transform::IDENTITY))
                .id()
                .to_bits()
        })
        .expect("é desta família")
        .expect("nasce");
        let e = ph2d_ecs::Entity::from_bits(bits);
        assert!(sim.world().get::<ph2d_ecs::MotionGenerator>(e).is_some());
        assert_eq!(
            sim.world().get::<ph2d_ecs::Name>(e).map(|n| n.0.as_str()),
            Some("Motion")
        );
    }
}
