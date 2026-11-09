//! «Collides with» da FORMA (GEF02 B9, dono 08/11): a exceção dela à matriz, `CollisionOverride`.

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{Entity, World};
use ph2d_editor_core::PhysicsFieldEdit;
use ph2d_inspector_ordering::{queue_remove, queue_set};
use ph2d_physics_ecs::excecao::LiveLayerRows;
use ph2d_physics_ecs::{Collider, CollisionOverride};

const OVERRIDE: &str = "ph2d::physics::CollisionOverride";

/// Aplica `CollidesWith`/`UseLayerDefault` à forma `forma`, contra a linha VIVA da camada dela. `false` = não é
/// edição de exceção.
pub(crate) fn apply_collides_edit(
    world: &World,
    forma: Entity,
    edit: PhysicsFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> bool {
    let alvo = match edit {
        PhysicsFieldEdit::CollidesWith { layer, on } => Some((layer, on)),
        PhysicsFieldEdit::UseLayerDefault => None,
        _ => return false,
    };
    let Some(col) = world.get::<Collider>(forma) else {
        return true;
    };
    let cur = world.get::<CollisionOverride>(forma).copied();
    let row = LiveLayerRows::row(world, col.layer);
    let next = alvo.and_then(|(layer, on)| CollisionOverride::with_bit(row, cur, layer, on));
    let bits = forma.to_bits();
    match next {
        Some(o) if Some(o) != cur => queue_set(queue, registry, bits, OVERRIDE, &o),
        None if cur.is_some() => queue_remove(queue, registry, bits, OVERRIDE),
        _ => {}
    }
    true
}
