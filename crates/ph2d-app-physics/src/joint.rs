//! §12 Physics Joint — the shell half of the Inspector section (W3): the
//! snapshot the panel reads, the ECS write an edit turns into, and the gesture
//! that creates a joint in the first place.

use bevy_ecs::world::World;
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor_core::JointFieldEdit;
use ph2d_physics_ecs::PhysicsJoint;

use ph2d_inspector_ordering::queue_set;

/// O gesto que CRIA um joint mora em `inspector_joint_create` (criar é da §11, editar é da §12).
pub(crate) use crate::joint_create::{create_joint, create_joint_at};
/// A troca de corpo de uma junta mora na `ph2d-joint-assembly` (o soltar-num-papel também a usa).
pub use ph2d_joint_assembly::joint_create::set_joint_body;

const JOINT: &str = "ph2d::physics::PhysicsJoint";

/// A lei do snapshot e da edição mora na folha do assunto (`ph2d-joint-assembly::joint_info`).
pub use ph2d_joint_assembly::joint_info::{
    build_joint_info, joint_with_edit, kind_of, limit_out, tag_of,
};

/// Apply one [`JointFieldEdit`].
///
/// Every arm reads the live joint and writes it back changed — a partial write
/// would drop the fields not being edited, and this component has eleven of
/// them. `Remove` is not here: deleting a joint is deleting an OBJECT, and the
/// shell already knows how to do that.
pub fn apply_joint_edit(
    sim: &SimWorld,
    entity_bits: u64,
    edit: JointFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let entity = Entity::from_bits(entity_bits);
    let Some(&current) = sim.world().get::<PhysicsJoint>(entity) else {
        return;
    };
    let Some(next) = joint_with_edit(current, edit) else {
        return;
    };
    if next != current {
        queue_set(queue, registry, entity_bits, JOINT, &next);
    }
}

/// **Prender o lado B ao MUNDO (`true`) ou a um objeto** (W-JointWorld) — a presença do marcador
/// `JointWorldAnchor` É o booleano, então não passa pelo funil de campos. A porta da edição do
/// painel e da curva (canal `PONTA_NO_MUNDO`). Devolve se o lado B ficou como pedido.
///
/// ⛔ **Soltar do mundo uma junta que não nomeia dois corpos é RECUSADO** — ela ficaria dormente
/// (`is_anchored`), e o corpo cairia sem o artista ter apagado nada. O painel arma então o
/// conta-gotas do corpo B; a curva só ecoa o que ficou.
///
/// ⚠️ Trocar o lado B muda o referencial de `local_b` (o ponto do mundo ↔ o corpo B), por isso a
/// junta volta a `anchored: false` e o reconcile re-deriva as âncoras — como na troca de corpo e de
/// tipo. Sem isso a ponta saltava para o centro do corpo B.
pub fn anchor_to_world(world: &mut World, e: Entity, on: bool) -> bool {
    let Some(&pj) = world.get::<PhysicsJoint>(e) else {
        return false;
    };
    if world.get::<ph2d_physics_ecs::JointWorldAnchor>(e).is_some() == on {
        return true;
    }
    if !on && !pj.names_two_bodies() {
        return false;
    }
    let mut ent = world.entity_mut(e);
    if on {
        ent.insert(ph2d_physics_ecs::JointWorldAnchor);
    } else {
        ent.remove::<ph2d_physics_ecs::JointWorldAnchor>();
    }
    if let Some(mut j) = ent.get_mut::<PhysicsJoint>() {
        j.anchored = false;
    }
    true
}

/// **O conta-gotas depois de um clique em Anchor B** (`took` = o que [`anchor_to_world`] devolveu):
/// «Object» recusado arma a escolha do corpo B; «World» com ela armada nesta junta desarma-a (o
/// painel mostrava «Object» enquanto esperava). Qualquer outra escolha armada fica.
#[must_use]
pub fn pick_after_anchor_b(
    took: bool,
    joint_bits: u64,
    armed: Option<(u64, bool)>,
) -> Option<(u64, bool)> {
    match (took, armed) {
        (false, _) => Some((joint_bits, true)),
        (true, Some((j, true))) if j == joint_bits => None,
        (true, outra) => outra,
    }
}

/// **Colar as propriedades de `source` neste joint** (W-JointCopy).
///
/// Irmã de [`apply_joint_edit`] com outra entrada: um paste não é a edição de um
/// campo (não há tag nem número a converter), é a substituição da metade
/// *"o que a restrição faz"* inteira, decidida por
/// [`PhysicsJoint::with_properties_of`] — a porta única que sabe o que é uma
/// propriedade.
///
/// Sai pelo MESMO `clamped()` e pela MESMA fila que toda edição da §12, então um
/// paste não pode autorar um estado que uma edição à mão não poderia.
pub fn paste_joint_properties(
    sim: &SimWorld,
    entity_bits: u64,
    source: &PhysicsJoint,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let entity = Entity::from_bits(entity_bits);
    let Some(&current) = sim.world().get::<PhysicsJoint>(entity) else {
        return; // não é um joint (a seleção pode ter sprites dentro) — o fan-out passa reto
    };
    let next = current.with_properties_of(source).clamped();
    if next != current {
        queue_set(queue, registry, entity_bits, JOINT, &next);
    }
}
