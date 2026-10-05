//! ⭐ **Add ▸ Skeleton** (A14) — o esqueleto nasce com UM osso e pede o Edit (como o Add do vetor:
//! o próximo gesto é desenhar ossos).

use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_editor_core::object_add::{AddEntry, AddGroup};

use crate::state::SkeletonState;

/// A entrada do menu.
pub const SKELETON: AddEntry = AddEntry::new("object_add.skeleton", AddGroup::TwoD);
/// As entradas desta família.
pub const ENTRIES: &[AddEntry] = &[SKELETON];

/// Cria o esqueleto de `entry`, se é desta família: o objecto (marcador, nome que conta) e o 1.º
/// osso, do tamanho de fábrica do componente, a nascer da origem do esqueleto.
pub fn add(
    entry: AddEntry,
    sim: &mut SimWorld,
    state: &mut SkeletonState,
) -> Option<Result<u64, &'static str>> {
    if entry != SKELETON {
        return None;
    }
    let name = ph2d_unique_name::unique_name(sim, entry.key.tr());
    let skel = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new(name),
            ph2d_skeleton_ecs::Skeleton,
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    let comprimento = ph2d_skeleton_ecs::Bone::default().length;
    if crate::bone_gesture::create(sim, Some(skel), [0.0, 0.0], [comprimento, 0.0]).is_none() {
        sim.world_mut().despawn(skel);
        return Some(Err(ph2d_i18n::tr("object_add.not_born")));
    }
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    state.born = Some(skel.to_bits());
    Some(Ok(skel.to_bits()))
}

/// O esqueleto é a raiz da corrente: os ossos-raiz que o Add cria são filhos dele.
#[must_use]
pub fn is_skeleton(sim: &SimWorld, bits: u64) -> bool {
    sim.world()
        .get::<ph2d_skeleton_ecs::Skeleton>(Entity::from_bits(bits))
        .is_some()
}

#[cfg(test)]
#[path = "object_add_tests.rs"]
mod tests;
