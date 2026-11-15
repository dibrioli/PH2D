//! **Os conta-gotas do Inspector que escolhem um objecto no palco** — o Target da bala (GEF 06) e o
//! «Ignore up to» do Ray Sensor (GEF 07 C8). A shell compõe UMA porta: um clique, um `Esc`, um cursor.

use ph2d_ecs::World;
use ph2d_editor_core::action_bus::ActionBus;

/// Algum está armado (o cursor de mira).
#[must_use]
pub fn armado(world: &World) -> bool {
    crate::projectile_inspector::armado(world) || crate::ray_topo::armado(world, None)
}

/// Desarma todos (o `Esc`); `true` = algum estava armado.
pub fn desarma(world: &mut World) -> bool {
    crate::projectile_inspector::desarma(world).is_some() | crate::ray_topo::desarma(world).is_some()
}

/// O clique no palco com `debaixo` (de cima para baixo); `true` = um conta-gotas consumiu-o.
pub fn escolher(world: &mut World, bus: &mut ActionBus, debaixo: &[u64]) -> bool {
    crate::projectile_inspector::escolher_alvo(world, bus, debaixo)
        || crate::ray_topo::escolher_topo(world, bus, debaixo)
}
