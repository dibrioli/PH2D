//! ADR-0025 M14.3a — ComponentRegistry contributions from `ph2d-script`.
//!
//! Right now there's exactly one component to register
//! (`LuauScript`). Future script-related components (e.g. an
//! `AttachedCoroutines` debug helper) plug into the same function.

use ph2d_ecs::scene::ComponentRegistry;

use crate::component::LuauScript;

/// Register the components owned by `ph2d-script` against the shared
/// [`ComponentRegistry`]. Shell calls this once at boot alongside
/// `register_ecs_components` and `register_render_components`.
pub fn register_script_components(reg: &mut ComponentRegistry) {
    reg.register::<LuauScript>("ph2d::script::LuauScript");
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::scene::register_ecs_components;

    #[test]
    fn registers_luau_script_alongside_ecs() {
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        register_script_components(&mut reg);
        // 30 ecs components (ver `register_ecs_components_populates_registry` —
        // inclui VecShape/VecConnector/VecBlend/VecLabel)
        // + 1 script component (LuauScript).
        assert_eq!(reg.len(), 32);
        assert!(reg.get_by_name("ph2d::script::LuauScript").is_some());
    }
}
