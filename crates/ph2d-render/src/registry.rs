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
        // O numero se CONTA, nao se escolhe: e o contador de `ph2d-ecs`
        // (`register_ecs_components_populates_registry`, hoje 38 — inclui
        // VecShape/VecConnector/VecBlend/VecLabel/VecEnvelope/VecOffset/VecTextPath/VecPatternPath/VecFilter)
        // + 1 render component (Sprite). SAO DOIS contadores: quem registra
        // um componente novo no ECS tem de somar aqui tambem, e este gate so
        // roda na suite da ph2d-render.
        assert_eq!(reg.len(), 40);
        assert!(reg.get_by_name("ph2d::render::Sprite").is_some());
    }
}
