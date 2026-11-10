//! **O Look da forma de colisão no Inspector e no canvas** (GEF02 onda 8): o arrasto das alças escreve pelo centro
//! real da forma, e o `offset` cru volta pelo pivô do [`ph2d_physics_ecs::ShapeLook`] (a porta `ph2d_physics_ecs::shape_look`).

use ph2d_ecs::{Entity, World};
use ph2d_physics_ecs::{Collider, ColliderShape};

/// A forma que as alças deixaram: `medidas` = meias-extensões (bola: `[raio, 0]`; cápsula: `[raio, meia-altura]`)
/// e `centro` = o centro local que elas mostram. Com pivô, o `offset` cru é `centro + pivô ⊙ meia-extensão nova`.
#[must_use]
pub fn forma_arrastada(
    world: &World,
    e: Entity,
    cur: &Collider,
    medidas: [f32; 2],
    centro: [f32; 2],
) -> Collider {
    let shape = match cur.shape {
        ColliderShape::Cuboid { .. } => ColliderShape::Cuboid {
            half_x: medidas[0],
            half_y: medidas[1],
        },
        ColliderShape::Outline { .. } => ColliderShape::Outline {
            half_x: medidas[0],
            half_y: medidas[1],
        },
        ColliderShape::Ball { .. } => ColliderShape::Ball { radius: medidas[0] },
        ColliderShape::Capsule { .. } => ColliderShape::Capsule {
            half_height: medidas[1],
            radius: medidas[0],
        },
    };
    Collider {
        shape,
        offset: ph2d_physics_ecs::shape_look::offset_para_centro(world, e, shape, centro),
        ..*cur
    }
}

#[cfg(test)]
#[path = "physics_apply_look_tests.rs"]
mod tests;
