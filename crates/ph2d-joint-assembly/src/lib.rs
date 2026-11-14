//! **A montagem de juntas** (ADR-0201) — o objeto que agrupa as juntas de um gabarito, e os
//! gestos que a montam. A `ph2d-app-physics` re-exporta cada módulo pelo mesmo nome.

pub mod joint_assembly;
pub mod joint_assembly_card;
pub mod joint_create;
pub mod joint_drop;
pub mod joint_info;
pub mod joint_world;

#[cfg(test)]
pub(crate) fn test_registry() -> ph2d_ecs::scene::ComponentRegistry {
    let mut reg = ph2d_ecs::scene::ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    reg
}
