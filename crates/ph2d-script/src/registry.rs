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
    // ⚠️ `register_default` desde o TOP-20 #16: a paleta do `+` anexa o PONTO NEUTRO do tipo (um
    // script sem ficheiro), e sem esta porta o descritor `Authored` prometeria o que não constrói
    // (gate `every_offered_component_can_be_constructed`, na shell).
    reg.register_default::<LuauScript>(crate::project_scripts::CANONICAL_NAME);
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
        // O número se CONTA: o contador da `ph2d-ecs` + 1 componente desta crate (`LuauScript`). Antes era um literal espelhado,
        // que ficava para trás em toda linha que registava um componente no ECS.
        let mut so_ecs = ComponentRegistry::new();
        register_ecs_components(&mut so_ecs);
        assert_eq!(reg.len(), so_ecs.len() + 1);
        assert!(reg.get_by_name("ph2d::script::LuauScript").is_some());
    }
}
