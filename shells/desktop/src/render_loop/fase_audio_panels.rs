//! **Fase do quadro: OS PAINÉIS DE ÁUDIO** — o motor de áudio ouve o mixer e o editor de áudio
//! (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! O `poll` do motor, a mesa de som (documento desde a F30 §B2: `ph2d_app_audio::mesa_doc::quadro`,
//! com ou sem o painel) e o dreno das intenções do editor de áudio, que fica na shell porque as
//! `feature`s dele (`panel-audio-editor`, `audio-ml`) são da SHELL e não viajam com o código (HOWTO §2).

/// O editor de áudio — as três fases-filhas dele, num ficheiro irmão.
#[cfg(feature = "panel-audio-editor")]
#[path = "fase_audio_editor.rs"]
mod editor;

impl crate::App {
    /// Ver o cabeçalho do módulo. Não troca local nenhum com o resto do quadro (medido).
    pub(super) fn fase_audio_panels(&mut self) {
        // Phase 2.1: drop finished-sample Arcs on the main thread (HR-3).
        // ⭐ A mesa é documento (F30 §B2): o motor ouve o vivo, com ou sem o painel.
        if let Some(gfx) = self.gfx.as_mut() {
            ph2d_app_audio::mesa_doc::quadro(&mut gfx.sim, self.audio.as_mut());
        }
        if let Some(audio) = self.audio.as_mut() {
            audio.poll();
            // Audio Editor bridge (docs/Audio/, W1): drain the panel's one-shot
            // transport intents → drive the preview engine, then publish the live
            // position/duration/name back for the readout (+ overlay playhead).
            #[cfg(feature = "panel-audio-editor")]
            {
                self.fase_audio_editor_io();
                self.fase_audio_editor_rack();
                self.fase_audio_editor_prep();
            }
        }
    }
}
