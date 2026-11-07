//! ADR-0025 M14.3a — ComponentRegistry contributions from `ph2d-render`.
//!
//! Right now there's exactly one component to register (`Sprite`).
//! Future renderable components (`SpriteAnimation`, `LightSource`,
//! `Material`, …) plug into the same function.

use ph2d_ecs::scene::ComponentRegistry;

use crate::sprite::Sprite;

/// Register the components owned by `ph2d-render` against the shared
/// [`ComponentRegistry`]. Shell calls this once at boot alongside
/// `register_ecs_components` and `register_script_components`.
pub fn register_render_components(reg: &mut ComponentRegistry) {
    reg.register::<Sprite>("ph2d::render::Sprite");
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::scene::register_ecs_components;

    #[test]
    fn registers_sprite_alongside_ecs() {
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        register_render_components(&mut reg);
        // O número se CONTA: o contador da `ph2d-ecs` + 1 componente desta crate (`Sprite`). Antes era um literal espelhado,
        // que ficava para trás em toda linha que registava um componente no ECS.
        let mut so_ecs = ComponentRegistry::new();
        register_ecs_components(&mut so_ecs);
        assert_eq!(reg.len(), so_ecs.len() + 1);
        assert!(reg.get_by_name("ph2d::render::Sprite").is_some());
        assert!(reg.get_by_name("ph2d::ecs::SpriteEmissive").is_some());
        assert!(reg.get_by_name("ph2d::ecs::SliceNine").is_some());
        assert!(reg.get_by_name("ph2d::ecs::NamedAnchorList").is_some());
        assert!(reg.get_by_name("ph2d::ecs::AnchorMount").is_some());
        assert!(reg.get_by_name("ph2d::ecs::AnchorVisibility").is_some());
    }
}
