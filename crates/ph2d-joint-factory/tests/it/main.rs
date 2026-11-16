mod assembly;
mod every_word_this_crate_shows_comes_from_the_string_table;

/// O registo de componentes de um teste: o do ECS e o da física.
fn test_registry() -> ph2d_ecs::scene::ComponentRegistry {
    let mut reg = ph2d_ecs::scene::ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    reg
}
