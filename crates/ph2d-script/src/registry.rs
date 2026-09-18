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
    reg.register_default::<LuauScript>("ph2d::script::LuauScript");
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
        assert_eq!(reg.len(), 99);
        assert!(reg.get_by_name("ph2d::script::LuauScript").is_some());
    }
}
