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
        // 24 ecs components (4 foundational + 16 W3 sorting/visibility/
        // sampling/mask + 1 §10 BlendMode + 3 ADR-0110/undo: Locked/
        // GroupedChildren/VecPathRef) + 1 render component (Sprite).
        assert_eq!(reg.len(), 25);
        assert!(reg.get_by_name("ph2d::render::Sprite").is_some());
    }
}
