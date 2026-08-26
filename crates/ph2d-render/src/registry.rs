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
        // (`register_ecs_components_populates_registry`, **hoje 63** — nao repita a lista dele
        // aqui, ela ja' envelheceu tres vezes neste comentario) + 1 render component (Sprite).
        //
        // SAO DOIS contadores, e este e' o que se esquece: quem regista um componente novo no ECS
        // tem de somar aqui tambem, e **este gate so roda na suite da ph2d-render**.
        //
        // ⚠️ Precedente de 2026-08-20, e vale mais que a regra: a linha `Sprite` acrescentou
        // `SpritePixels` + `SpriteSheetRef` + `SpriteSheetFrame`, somou os tres no contador do ECS
        // (57 -> 60), viu-o verde, e deixou ESTE em 58 durante toda a jornada. O laco de trabalho
        // corria `cargo check -p` e a suite das crates tocadas; a `ph2d-render` nao era uma delas,
        // entao o gate nunca abriu a boca. *Um contador que so' fala na suite de outra crate e' um
        // contador que so' fala no fecho.*
        //
        // Na integracao ele SOMA entre linhas — recontar e' obrigatorio, escolher um dos lados e' o
        // erro que deixa o workspace vermelho com dois merges verdes.
        // 2026-08-21: +1 `SliceNine` (a autoria de 9-slice, spec Sprite 03 §3.5) — e este
        // comentario e' a prova de que o precedente acima funciona: o gate do ECS ficou verde
        // primeiro, e foi ESTE que cobrou a segunda metade.
        // 2026-08-21: +1 `NamedAnchorList` (ADR-0072).
        // 2026-08-22 (integracao): +1 `VecClipContent` da `line/Vector` — o ECS esta' em 64,
        // e este e' `ecs + 1`. ⛔ Nao copie para aqui o numero que o `ph2d-ecs` afirma: sao
        // grandezas DIFERENTES, e copiar foi o erro que trouxe este gate ao vermelho na linha.
        // +1 `VecBoolOp` (um verbo por forma, 2026-08-22): ECS 65 ⇒ aqui 66.
        // +1 `AnchorMount` (o consumidor de uma ancora, ADR-0072 §2.6, 2026-08-22): ECS 66 ⇒ 67.
        // +1 `AnchorVisibility` (quando as ancoras se desenham, 2026-08-23): ECS 67 ⇒ 68.
        // +2 da §11 Animation (`SpriteAnimations` + `SpriteAnimator`): ECS 69 ⇒ aqui 70.
        // +3 do corte da Sprite (`SpriteGrid`/`SpriteRegion`/`SpriteCornerTint`): ECS 73 ⇒ aqui 74.
        // + 1 do MESTRE (MasterRoot, ADR-0164 F4.1, 2026-08-25) — ver a nota do `ph2d-ecs`.
        // + 1 do ELO (InstanceOf, ADR-0164 F4.2, 2026-08-26) — idem.
        // + 1 dos OVERRIDES (ObjectInstance, ADR-0164 F4.4, 2026-08-26) — idem.
        assert_eq!(reg.len(), 77);
        assert!(reg.get_by_name("ph2d::render::Sprite").is_some());
        assert!(reg.get_by_name("ph2d::ecs::SpriteEmissive").is_some());
        assert!(reg.get_by_name("ph2d::ecs::SliceNine").is_some());
        assert!(reg.get_by_name("ph2d::ecs::NamedAnchorList").is_some());
        assert!(reg.get_by_name("ph2d::ecs::AnchorMount").is_some());
        assert!(reg.get_by_name("ph2d::ecs::AnchorVisibility").is_some());
    }
}
