//! **Os INSTANTÂNEOS das secções da cauda** — irmão do [`super::paint_optional_top20`].
//!
//! ⚠️ **Ficheiro próprio por CAP de LOC**, e o corte é por RESPONSABILIDADE: o irmão bateu `632` de
//! `600` ao ganhar as duas molduras do ABANÃO (suplente #25), e o que ele faz são **duas** coisas —
//! juntar os instantâneos do quadro numa struct, e pintar as molduras pela ordem. ⛔ **Curado por
//! CORTE, nunca por uma entrada no `FILE_OVERAGE_OK`** — aquela lista está VAZIA.
//!
//! ⭐ **E o corte paga-se duas vezes:** a próxima secção custa **uma** linha aqui e uma no irmão, em
//! vez de duas num ficheiro que já não cabia.

/// Os instantâneos das quatro secções da cauda, mais as duas linhas abertas do cérebro.
pub(crate) struct Top20<'a> {
    /// O CÉREBRO (TOP-20 #15).
    pub statemachine: Option<&'a ph2d_editor_core::statemachine_edits::InspectorStateMachineInfo>,
    /// O SCRIPT (TOP-20 #16).
    pub script: Option<&'a ph2d_editor_core::script_edits::InspectorScriptInfo>,
    /// O EMISSOR DE PARTÍCULAS (TOP-20 #18).
    pub particles: Option<&'a ph2d_editor_core::particles_edits::InspectorParticlesInfo>,
    /// O HUD (TOP-20 #20).
    pub hud: Option<&'a ph2d_editor_core::hud_edits::InspectorHudInfo>,
    /// A CUTSCENE (TOP-20 #19).
    pub sequence: Option<&'a ph2d_editor_core::sequence_edits::InspectorSequenceInfo>,
    /// A VIGIA DO CONTADOR.
    pub watch: Option<&'a ph2d_editor_core::counter_watch_edits::InspectorCounterWatchInfo>,
    /// Qual regra da vigia está aberta — estado do painel, como a dos timers.
    pub watch_selected: usize,
    /// O GATILHO (suplente #24).
    pub trigger: Option<&'a ph2d_editor_core::action_trigger_edits::InspectorActionTriggerInfo>,
    /// Qual linha do gatilho está aberta — estado do painel, como a da vigia.
    pub trigger_selected: usize,
    /// O TWEEN (suplente #22).
    pub tween: Option<&'a ph2d_editor_core::tween_edits::InspectorTweenInfo>,
    /// Qual tween está aberto — estado do painel, como a do gatilho e a da vigia.
    pub tween_selected: usize,
    /// O ABANÃO DA CÂMERA (suplente #25) — ⛔ **sem `selected`**: ela não é uma lista.
    pub shake: Option<&'a ph2d_editor_core::shake_edits::InspectorShakeInfo>,
    /// O EMISSOR DE ABANÃO (suplente #25).
    pub emitter: Option<&'a ph2d_editor_core::shake_edits::InspectorEmitterInfo>,
    /// Qual fonte do emissor está aberta — estado do painel, como a do gatilho.
    pub emitter_selected: usize,
    /// As TAGS (TOP-20 #9).
    pub tags: Option<&'a ph2d_editor_core::screens::hero::InspectorTagsInfo>,
    /// ⚠️ **Duas selecções e não uma** — as listas de estados e de setas são independentes.
    pub sm_state_selected: usize,
    /// Idem, a das setas.
    pub sm_trans_selected: usize,
}

impl<'a> Top20<'a> {
    /// ⭐⭐ **A struct constrói-se do INSTANTÂNEO do quadro**, e não campo a campo no chamador.
    ///
    /// ⚠️ **Ela mudou-se para cá por um TECTO DE FUNÇÃO** (2026-09-19): o `paint_optional_sections`
    /// bateu `202` contra `200` ao ganhar o tween, e cada secção nova custava-lhe duas linhas. ⇒ a
    /// construção é do dono da struct, o chamador passa o instantâneo e as selecções, e a próxima
    /// secção custa **uma** linha, aqui. ⛔ Curado por CORTE, nunca por uma entrada no
    /// `FN_OVERAGE_OK` — aquela lista está VAZIA.
    pub(crate) fn de(snaps: &'a crate::paint_frame::LiveSnapshots, sel: Selecoes) -> Self {
        Self {
            statemachine: snaps.statemachine_info.as_ref(),
            script: snaps.script_info.as_ref(),
            particles: snaps.particles_info.as_ref(),
            hud: snaps.hud_info.as_ref(),
            sequence: snaps.sequence_info.as_ref(),
            watch: snaps.watch_info.as_ref(),
            watch_selected: sel.watch,
            trigger: snaps.trigger_info.as_ref(),
            trigger_selected: sel.trigger,
            tween: snaps.tween_info.as_ref(),
            tween_selected: sel.tween,
            shake: snaps.shake_info.as_ref(),
            emitter: snaps.emitter_info.as_ref(),
            emitter_selected: sel.emitter,
            tags: snaps.tags_info.as_ref(),
            sm_state_selected: sel.sm_state,
            sm_trans_selected: sel.sm_trans,
        }
    }
}

/// **As linhas abertas das secções que são LISTAS** — estado do PAINEL, nunca da cena.
#[derive(Clone, Copy)]
pub(crate) struct Selecoes {
    pub watch: usize,
    pub trigger: usize,
    pub emitter: usize,
    pub tween: usize,
    pub sm_state: usize,
    pub sm_trans: usize,
}
