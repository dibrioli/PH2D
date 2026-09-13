//! **Fase do quadro: OS PAINÉIS DE ÁUDIO** — o motor de áudio ouve o mixer e o editor de áudio
//! (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! O `poll` do motor, a faixa master e os sub-barramentos do mixer, e o dreno das intenções de
//! transporte, exportação, efeitos, loop, marcadores e variações do editor. O corpo é o que estava no
//! `run_render_frame`, verbatim, com as `feature`s da shell a guardar cada metade — desde a OBRA 3 da
//! `line/render-bodies` o mixer e as três metades do editor são fases-filhas, chamadas no sítio dos blocos.
//!
//! ⚠️ **Assunto da família `audio`, e fica na shell por agora de propósito:** as três `feature`s que
//! o guardam (`panel-audio-mixer`, `panel-audio-editor`, `audio-ml`) são da SHELL, e uma feature não
//! viaja com o código (HOWTO §2); e duas leituras são do `gfx` (a fila de trabalhos e a secção
//! Delivery aberta). Levá-lo para `ph2d-app-audio` é portar as features, não mover texto.

/// O editor de áudio — as três fases-filhas dele, num ficheiro irmão.
#[cfg(feature = "panel-audio-editor")]
#[path = "fase_audio_editor.rs"]
mod editor;
/// O mixer — fase-filha, num ficheiro irmão (por `#[path]`, para o `render_loop/mod.rs` não passar o tecto).
#[cfg(feature = "panel-audio-mixer")]
#[path = "fase_audio_mixer.rs"]
mod mixer;

impl crate::App {
    /// Ver o cabeçalho do módulo. Não troca local nenhum com o resto do quadro (medido).
    pub(super) fn fase_audio_panels(&mut self) {
        // Phase 2.1: drop finished-sample Arcs on the main thread (HR-3).
        // Phase 2.3c: feed the mixer panel live levels + apply its Master mute.
        if let Some(audio) = self.audio.as_mut() {
            audio.poll();
            #[cfg(feature = "panel-audio-mixer")]
            {
                self.fase_audio_mixer();
            }
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
