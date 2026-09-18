//! **Os instantâneos das secções da FILA DO TOP-20** — irmão do [`super::state`] por `#[path]`.
//!
//! ⚠️ **O corte foi imposto pelo tecto de LOC dos painéis (600) e está certo por RESPONSABILIDADE:**
//! o ficheiro-mãe guarda o estado VIVO do painel (a rolagem, o que está aberto, o que o rato toca);
//! este guarda as **fotografias que a shell publica** das secções que a fila do TOP-20 trouxe —
//! timer, tabela de acções, áudio, câmera, fábrica, mover de vista de cima, projéctil.
//!
//! ⚠️ **Os `thread_local!` continuam no ficheiro-mãe**, e de propósito: eles são `pub(crate)`, logo
//! visíveis daqui, e movê-los partiria a ordem de declaração de um bloco que já tem dez entradas.
//! O que sai são as PORTAS, que é o que cresce uma por wave.
//!
//! ⛔ **Nunca subir o número do cap: ele só desce.**

use super::state::*;
use ph2d_editor_core::particles_edits::InspectorParticlesInfo;
use ph2d_editor_core::projectile_edits::InspectorProjectileInfo;
use ph2d_editor_core::screens::hero::{
    InspectorActionInfo, InspectorAudioInfo, InspectorCameraInfo, InspectorFactoryInfo,
    InspectorTimerInfo,
};
use ph2d_editor_core::script_edits::InspectorScriptInfo;
use ph2d_editor_core::statemachine_edits::InspectorStateMachineInfo;
use ph2d_editor_core::topdown_edits::InspectorTopDownInfo;

pub fn set_current_inspector_timer(info: Option<InspectorTimerInfo>) {
    CURRENT_INSPECTOR_TIMER.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_timer() -> Option<InspectorTimerInfo> {
    CURRENT_INSPECTOR_TIMER.with(|c| c.borrow().clone())
}

pub fn set_current_inspector_action(info: Option<InspectorActionInfo>) {
    CURRENT_INSPECTOR_ACTION.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_action() -> Option<InspectorActionInfo> {
    CURRENT_INSPECTOR_ACTION.with(|c| c.borrow().clone())
}

pub fn set_current_inspector_audio(info: Option<InspectorAudioInfo>) {
    CURRENT_INSPECTOR_AUDIO.with(|c| *c.borrow_mut() = info);
}

pub fn set_current_inspector_factory(info: Option<InspectorFactoryInfo>) {
    CURRENT_INSPECTOR_FACTORY.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_factory() -> Option<InspectorFactoryInfo> {
    CURRENT_INSPECTOR_FACTORY.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot do MOVER DE VISTA DE CIMA (TOP-20 #13) — a shell escreve-o todo o quadro.
pub fn set_current_inspector_topdown(info: Option<InspectorTopDownInfo>) {
    CURRENT_INSPECTOR_TOPDOWN.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_topdown() -> Option<InspectorTopDownInfo> {
    CURRENT_INSPECTOR_TOPDOWN.with(|c| c.borrow().clone())
}

pub fn set_current_inspector_projectile(info: Option<InspectorProjectileInfo>) {
    CURRENT_INSPECTOR_PROJECTILE.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_projectile() -> Option<InspectorProjectileInfo> {
    CURRENT_INSPECTOR_PROJECTILE.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot do CÉREBRO (TOP-20 #15) — a shell escreve-o todo o quadro.
pub fn set_current_inspector_statemachine(info: Option<InspectorStateMachineInfo>) {
    CURRENT_INSPECTOR_STATEMACHINE.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_statemachine() -> Option<InspectorStateMachineInfo> {
    CURRENT_INSPECTOR_STATEMACHINE.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot do HUD (TOP-20 #20) — a shell escreve-o todo o quadro.
pub fn set_current_inspector_hud(info: Option<ph2d_editor_core::hud_edits::InspectorHudInfo>) {
    crate::state::CURRENT_INSPECTOR_HUD.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_hud() -> Option<ph2d_editor_core::hud_edits::InspectorHudInfo> {
    crate::state::CURRENT_INSPECTOR_HUD.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot da CUTSCENE (TOP-20 #19) — a shell escreve-o todo o quadro.
pub fn set_current_inspector_sequence(
    info: Option<ph2d_editor_core::sequence_edits::InspectorSequenceInfo>,
) {
    crate::state::CURRENT_INSPECTOR_SEQUENCE.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_sequence()
-> Option<ph2d_editor_core::sequence_edits::InspectorSequenceInfo> {
    crate::state::CURRENT_INSPECTOR_SEQUENCE.with(|c| c.borrow().clone())
}

thread_local! {
    /// ⭐⭐⭐ **O snapshot da secção COUNTER WATCH** — a vigia do contador.
    ///
    /// ⚠️ **Ela mora AQUI e não no [`crate::state`], ao contrário das irmãs mais velhas**, e o
    /// corte foi imposto pelo tecto de 600 LOC daquele ficheiro (ele chegou a `601`). ⭐ É o
    /// certo por responsabilidade: aquele módulo é sobre o que o PAINEL lembra entre quadros, e
    /// isto é um instantâneo que a shell escreve — que é exactamente o assunto deste ficheiro,
    /// onde o `set`/`current` dela já viviam.
    static CURRENT_INSPECTOR_COUNTER_WATCH:
        std::cell::RefCell<
            Option<ph2d_editor_core::counter_watch_edits::InspectorCounterWatchInfo>,
        > = const { std::cell::RefCell::new(None) };

    /// ⭐ O snapshot do GATILHO (suplente #24) — a shell escreve-o todo o quadro.
    static CURRENT_INSPECTOR_ACTION_TRIGGER:
        std::cell::RefCell<
            Option<ph2d_editor_core::action_trigger_edits::InspectorActionTriggerInfo>,
        > = const { std::cell::RefCell::new(None) };
}

/// ⭐ O snapshot da VIGIA DO CONTADOR — a shell escreve-o todo o quadro.
pub fn set_current_inspector_counter_watch(
    info: Option<ph2d_editor_core::counter_watch_edits::InspectorCounterWatchInfo>,
) {
    CURRENT_INSPECTOR_COUNTER_WATCH.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_counter_watch()
-> Option<ph2d_editor_core::counter_watch_edits::InspectorCounterWatchInfo> {
    CURRENT_INSPECTOR_COUNTER_WATCH.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot do GATILHO — a shell escreve-o todo o quadro.
pub fn set_current_inspector_action_trigger(
    info: Option<ph2d_editor_core::action_trigger_edits::InspectorActionTriggerInfo>,
) {
    CURRENT_INSPECTOR_ACTION_TRIGGER.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_action_trigger()
-> Option<ph2d_editor_core::action_trigger_edits::InspectorActionTriggerInfo> {
    CURRENT_INSPECTOR_ACTION_TRIGGER.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot do EMISSOR DE PARTÍCULAS (TOP-20 #18) — a shell escreve-o todo o quadro.
pub fn set_current_inspector_particles(info: Option<InspectorParticlesInfo>) {
    CURRENT_INSPECTOR_PARTICLES.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_particles() -> Option<InspectorParticlesInfo> {
    CURRENT_INSPECTOR_PARTICLES.with(|c| c.borrow().clone())
}

/// ⭐ O snapshot do SCRIPT (TOP-20 #16) — a shell escreve-o todo o quadro.
pub fn set_current_inspector_script(info: Option<InspectorScriptInfo>) {
    CURRENT_INSPECTOR_SCRIPT.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_script() -> Option<InspectorScriptInfo> {
    CURRENT_INSPECTOR_SCRIPT.with(|c| c.borrow().clone())
}

pub fn set_current_inspector_camera(info: Option<InspectorCameraInfo>) {
    CURRENT_INSPECTOR_CAMERA.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_camera() -> Option<InspectorCameraInfo> {
    CURRENT_INSPECTOR_CAMERA.with(|c| c.borrow().clone())
}

pub(crate) fn current_inspector_audio() -> Option<InspectorAudioInfo> {
    CURRENT_INSPECTOR_AUDIO.with(|c| c.borrow().clone())
}
