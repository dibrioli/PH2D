//! **Um objecto largado na linha Body A / Body B do cartão da junta** (`DropAction::SetJointBody`):
//! a MESMA lei do conta-gotas — só um corpo físico que não é a própria junta, escrito por
//! [`crate::joint::set_joint_body`] (no lugar; o desfazer vem do diff).

use ph2d_ecs::{Entity, SimWorld, StableId, World};

/// O objecto `bits` pode ser uma ponta da junta `joint_bits`? A pergunta do conta-gotas e da queda.
#[must_use]
pub fn is_bindable_body(world: &World, joint_bits: u64, bits: u64) -> bool {
    bits != joint_bits
        && world
            .get::<ph2d_physics_ecs::RigidBody>(Entity::from_bits(bits))
            .is_some()
}

/// Religa o lado A (ou B, `slot_b`) da junta ao objecto `stable_id` largado. `false` = recusado
/// (não há objecto, não é corpo, é a própria junta, ou a junta ficaria ligada a si mesma).
#[must_use]
pub fn drop_body(sim: &mut SimWorld, joint_bits: u64, slot_b: bool, stable_id: u64) -> bool {
    let Some(target) = ph2d_ecs::entity_with_stable_id(sim.world(), StableId(stable_id)) else {
        return false;
    };
    is_bindable_body(sim.world(), joint_bits, target.to_bits())
        && crate::joint::set_joint_body(sim, joint_bits, slot_b, target)
}

/// [`drop_body`] e, se religou, desarma o conta-gotas armado NAQUELE lado (`side` = junta, lado B).
pub fn drop_disarming(
    sim: &mut SimWorld,
    pick: &mut Option<(u64, bool)>,
    side: (u64, bool),
    stable_id: u64,
) {
    if drop_body(sim, side.0, side.1, stable_id) && *pick == Some(side) {
        *pick = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_core::Vec2;
    use ph2d_ecs::{Name, Transform};
    use ph2d_physics_ecs::{BodyKind, JointKind, PhysicsJoint, RigidBody};

    fn body(sim: &mut SimWorld, sid: u64, name: &str) -> Entity {
        sim.world_mut()
            .spawn((
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Transform::from_translation(Vec2::new(0.0, 0.0)),
                Name::new(name),
                StableId(sid),
            ))
            .id()
    }

    /// ⭐ **Largar um corpo em Body A religa o lado A pelo `StableId`**; o lado B fica. Recusam: o
    /// corpo da outra ponta (auto-junta), a própria junta, um objecto sem corpo e um id que não existe
    /// — e nenhuma recusa toca a junta.
    ///
    /// **Mutações que devem sangrar:** tirar o filtro de `RigidBody` · aceitar a própria junta · trocar
    /// o lado.
    #[test]
    fn largar_um_corpo_em_body_a_religa_o_lado_a_e_recusa_o_resto() {
        let mut sim = SimWorld::new();
        let hook = body(&mut sim, 11, "Hook");
        let plank = body(&mut sim, 12, "Plank");
        let post = body(&mut sim, 13, "Post");
        let sem_corpo = sim
            .world_mut()
            .spawn((Name::new("Deco"), StableId(14)))
            .id();
        let joint = crate::joint_create::create_joint(
            &mut sim,
            hook.to_bits(),
            plank.to_bits(),
            JointKind::Pin,
        )
        .expect("junta");
        ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
        let jb = joint.to_bits();
        let joint_sid = sim.world().get::<StableId>(joint).expect("sid").0;
        let antes = *sim.world().get::<PhysicsJoint>(joint).expect("junta");
        let _ = sem_corpo;
        for (sid, porque) in [
            (12, "a outra ponta"),
            (joint_sid, "a própria junta"),
            (14, "sem corpo"),
            (999, "não existe"),
        ] {
            assert!(!drop_body(&mut sim, jb, false, sid), "aceitou {porque}");
            assert_eq!(
                *sim.world().get::<PhysicsJoint>(joint).expect("junta"),
                antes,
                "a recusa ({porque}) tocou a junta"
            );
        }
        assert!(drop_body(&mut sim, jb, false, 13), "o Post foi recusado");
        let j = *sim.world().get::<PhysicsJoint>(joint).expect("junta");
        assert_eq!(j.body_a, 13, "o lado A não passou a ser o Post");
        assert_eq!(j.body_b, antes.body_b, "o lado B mexeu");
        let _ = post;
    }
}
