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
        // + 1 da MAQUINA DE ESTADOS do Morph (VecMorphMachine, `line/Vector`,
        //   2026-08-26) — degrau escrito na INTEGRACAO: aquela linha subiu os tres
        //   contadores e nao os registou em escada nenhuma.
        // +3 do corte da Sprite (`SpriteGrid`/`SpriteRegion`/`SpriteCornerTint`): ECS 74 ⇒ aqui 75.
        // + 1 do MESTRE (MasterRoot, ADR-0164 F4.1, 2026-08-25) — ver a nota do `ph2d-ecs`.
        // + 1 do ELO (InstanceOf, ADR-0164 F4.2, 2026-08-26) — idem.
        // + 1 dos OVERRIDES (ObjectInstance, ADR-0164 F4.4, 2026-08-26) — idem.
        // + 1: LinkedArt (Enio 2026-08-27) -- ver a nota dos TRES contadores em `ph2d-ecs`.
        // + 1: VecBucketFill (o preenchimento do balde, plano 40, 2026-09-01) -- ver a nota
        //   dos TRES contadores em `ph2d-ecs`.
        // ⚠️ **-2 em 2026-09-06: o ESQUELETO SAIU do `register_ecs_components`** e virou modulo
        //   proprio (`ph2d-skeleton-ecs`, com a porta `register_skeleton_components`, precedente da
        //   `ph2d-physics-ecs`). ⛔ Um componente que SAI conta tanto como um que entra: ECS 79 ⇒
        //   aqui 80, e o numero foi CONTADO (o gate imprimiu `left: 80`).
        // ⚠️ **-2 em 2026-09-07: o MOTOR DE INSTANCIA DO VETOR saiu** (`VecComponentMain` e
        //   `VecInstance`, F4.6c) -- ver a nota dos TRES contadores em `ph2d-ecs`. ⛔ Um componente
        //   que SAI conta tanto como um que entra: ECS 77 ⇒ aqui 78, e o numero foi CONTADO (o
        //   gate imprimiu `left: 78`).
        // ⚠️⚠️ **+7 em 2026-09-10, e este espelho esteve VERMELHO desde a wave do `Timer`.** Os
        //   TOP-20 #2 (`Timers`, +1), #4 (`AudioSource2D` + `AudioListener2D`, +2) e #7
        //   (`GameCamera` + `CameraFollow` + `CameraLimits`, +3) entraram no `ph2d-ecs` sem que
        //   ninguem contasse aqui — mais o `TimerRuntime` que o registo do ECS ganhou no mesmo
        //   bloco. ECS `77 -> 84` ⇒ aqui `78 -> 85`, e o numero foi CONTADO (o gate imprimiu
        //   `left: 85`).
        // ⛔⛔ **E o motivo de ter demorado tres waves a aparecer e' de PROCESSO, nao de codigo:**
        //   cada fecho correu `-p` so' sobre as crates EDITADAS, e este gate vive numa que nenhuma
        //   das tres tocou. *Um portao que so' corre o que a linha editou e' cego a todo espelho.*
        // ⚠️ **2026-09-10: `85` -> `86`, delta +1** -- o `Sculpt3dPieceRef` que a `line/quadextract`
        //   registou no ECS (ADR-0150, o mesh na Hierarquia). ⛔ Ela NAO tocou neste ficheiro, e o
        //   fecho dela nao o podia ver: e' exactamente a cegueira que a nota acima descreve. O `86`
        //   foi CONTADO -- o gate imprimiu `left: 86`.
        // ⚠️ **2026-09-13: `86` -> `87`, delta +1** -- o `Tags` que a `line/components` registou no
        //   ECS (TOP-20 #9). Quem integrar conta o DELTA, nunca o literal.
        // ⚠️ **2026-09-19: `100` -> `101`, delta +1** -- o `PathFollow` (suplente #23), registado no
        //   ECS. Quem integrar conta o DELTA, nunca o literal.
        // ⚠️ **2026-09-19: `101` -> `103`, delta +2** -- o `CameraShake` e o `ShakeEmitter`
        //   (suplente #25), registados no ECS. ⛔ **DOIS, e o `CameraShakeRuntime` nao conta**: a
        //   cerca dele e' o TIPO. Quem integrar conta o DELTA, nunca o literal.
        // ⚠️ **2026-09-19: `103` -> `104`, delta +1** -- o `WeaponFire` (a ARMA do jogador),
        //   registado no ECS. ⛔ **UM so'**: nem o `WeaponRuntime` (a cerca e' o TIPO) nem a
        //   MUNICAO (ela e' um `Counter`, que ja' ca' esta') contam. Quem integrar conta o DELTA.
        // ⚠️ **2026-09-22: `104` -> `105`, delta +1** -- o `ScrollFactor` (a PARALAXE, plano 24
        //   W1), registado no ECS. ⛔ **UM so'**: nao existe runtime dele, porque a lei e' pura.
        //   Quem integrar conta o DELTA, nunca o literal.
        // ⚠️ **2026-09-22: `105` -> `106`, delta +1** -- o `ScrollRepeat` (a REPETICAO, plano 24
        //   W2), registado no ECS. ⛔ **UM so'**, e ele e' irmao do `ScrollFactor` e nao um campo
        //   dele — ver a nota do registo. Quem integrar conta o DELTA, nunca o literal.
        assert_eq!(reg.len(), 106);
        assert!(reg.get_by_name("ph2d::render::Sprite").is_some());
        assert!(reg.get_by_name("ph2d::ecs::SpriteEmissive").is_some());
        assert!(reg.get_by_name("ph2d::ecs::SliceNine").is_some());
        assert!(reg.get_by_name("ph2d::ecs::NamedAnchorList").is_some());
        assert!(reg.get_by_name("ph2d::ecs::AnchorMount").is_some());
        assert!(reg.get_by_name("ph2d::ecs::AnchorVisibility").is_some());
    }
}
