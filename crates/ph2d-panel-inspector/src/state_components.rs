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
use ph2d_editor_core::projectile_edits::InspectorProjectileInfo;
use ph2d_editor_core::screens::hero::{
    InspectorActionInfo, InspectorAudioInfo, InspectorCameraInfo, InspectorFactoryInfo,
    InspectorTimerInfo,
};
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

pub fn set_current_inspector_camera(info: Option<InspectorCameraInfo>) {
    CURRENT_INSPECTOR_CAMERA.with(|c| *c.borrow_mut() = info);
}

pub(crate) fn current_inspector_camera() -> Option<InspectorCameraInfo> {
    CURRENT_INSPECTOR_CAMERA.with(|c| c.borrow().clone())
}

pub(crate) fn current_inspector_audio() -> Option<InspectorAudioInfo> {
    CURRENT_INSPECTOR_AUDIO.with(|c| c.borrow().clone())
}
