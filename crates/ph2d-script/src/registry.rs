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
        // O numero se CONTA, nao se escolhe: e o contador de `ph2d-ecs`
        // (`register_ecs_components_populates_registry`, hoje 41 — inclui
        // VecShape/VecConnector/VecBlend/VecLabel/VecEnvelope/VecOffset/VecTextPath/VecPatternPath/VecFilter)
        // + 1 script component (LuauScript). SAO TRES contadores desta
        // familia (ecs, render, script): registrar um componente novo no ECS
        // tem de somar nos tres, e cada um so roda na suite da sua crate.
        // ⚠️ **ESTE contador ficou 4 atrás, e a nota acima já tinha PREVISTO como.** A
        // `line/Sprite` levou o do `ph2d-ecs` de 57 a 61 em quatro componentes
        // (`SpritePixels` · `SpriteSheetRef` · `SpriteSheetFrame` · `SpriteEmissive`), viu a suite
        // da própria crate verde de cada vez, e nenhuma delas correu a da `ph2d-script`. Em `main`
        // ele estava certo (57 + 1 = 58); aqui esteve errado durante a jornada inteira.
        //
        // ⚠️ **Quem o encontrou foi o gate BATCHED do fecho**, não o laço de trabalho — o
        // `cargo check -p` e a suite das crates tocadas nunca o tocam. *Um contador que só fala na
        // suite de outra crate é um contador que só fala no fecho*, e o irmão da `ph2d-render` tem
        // esta mesma frase escrita ao lado dele por ter sofrido exactamente isto.
        assert_eq!(reg.len(), 62);
        assert!(reg.get_by_name("ph2d::script::LuauScript").is_some());
    }
}
