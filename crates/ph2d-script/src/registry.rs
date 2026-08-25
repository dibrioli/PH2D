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
        //
        // ⚠️⚠️ **E ACONTECEU OUTRA VEZ, na MESMA linha, com a nota acima já escrita** (2026-08-22):
        // o `SliceNine` e o `NamedAnchorList` levaram o ECS de 61 a 63, os espelhos do `ph2d-ecs` e
        // do `ph2d-render` foram atualizados no mesmo commit — e este ficou 2 atrás outra vez.
        // *Uma nota que descreve o mecanismo não o impede*: quem o apanhou foi o
        // `collision-surface.sh` do handoff, ao pôr os três contadores lado a lado e mostrar que
        // este era o único que não guardava a relação `ecs + 1`. **É essa a leitura que fecha a
        // linha** — não a suíte da crate tocada, que nunca chega aqui.
        //
        // O número CONTA-SE: 63 (`ph2d-ecs`) + 1 (`LuauScript`).
        // 2026-08-22 (integracao): +1 `VecClipContent` da `line/Vector` — o ECS esta' em 64,
        // e este e' `ecs + 1` (LuauScript). ⛔ Grandezas DIFERENTES do `ph2d-ecs`; nao copie.
        // +1 `VecBoolOp` (um verbo por forma, 2026-08-22): ECS 65 ⇒ aqui 66.
        // +1 `AnchorMount` (o consumidor de uma ancora, ADR-0072 §2.6, 2026-08-22): ECS 66 ⇒ 67.
        // ⚠️ Desta vez os TRES foram somados no MESMO commit — que e' o que as duas notas
        // acima pediam depois de este contador ficar 4 atras e depois 2 atras na mesma linha.
        // +1 `AnchorVisibility` (quando as ancoras se desenham, 2026-08-23): ECS 67 ⇒ 68.
        // +2 da §11 Animation (`SpriteAnimations` + `SpriteAnimator`): ECS 69 ⇒ aqui 70.
        // +3 do corte da Sprite (`SpriteGrid`/`SpriteRegion`/`SpriteCornerTint`): ECS 74 ⇒ aqui 75.
        assert_eq!(reg.len(), 75);
        assert!(reg.get_by_name("ph2d::script::LuauScript").is_some());
    }
}
