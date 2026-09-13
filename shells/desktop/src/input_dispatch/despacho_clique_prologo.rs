//! **O clique, o PRÓLOGO** — ramos do `on_mouse_input` ([`super`]): o arrasto da biblioteca, o aperto no canvas
//! que solta o teclado do painel, a navegação das janelas 3D e a alça do gizmo de âncora (antes das locais do
//! evento), e o editor de áudio (logo depois delas). Os corpos MUDARAM-SE verbatim (`line/input-dispatch`,
//! 2026-09-13) e correm no sítio da chamada, pela mesma ordem.
//!
//! ⚠️ A soltura das mãos NÃO está aqui: ela é a primeira coisa do handler, e fica no índice à vista.

use super::*;

impl crate::App {
    /// O editor de áudio: a régua (scrub), o corpo da onda pela ferramenta armada (Select/Move/Scale) e os Up que
    /// largam a peça, o scrub e a selecção.
    #[cfg(feature = "panel-audio-editor")]
    pub(super) fn ramo_editor_audio(&mut self, kind: PointerKind) -> bool {
        // Audio Editor waveform selection (SHELL-only): a primary press INSIDE the
        // overlay waveform starts a selection (cleared to a point); release ends
        // it. Early-return so the press doesn't drive the canvas/gizmo underneath.
        // Presses on the overlay's title-bar / resize handles fall through (they're
        // outside the waveform rect) to the shared BlenderHit dispatch.
        #[cfg(feature = "panel-audio-editor")]
        match kind {
            // Press on the RULER strip → grab the playhead and scrub (seek).
            PointerKind::Down
                if let Some(frame) =
                    self.audio_ruler_frame_at(self.last_pointer.0, self.last_pointer.1) =>
            {
                self.audio_scrub_drag = true;
                if let Some(a) = self.audio.as_mut() {
                    a.editor_scrub_to_frame(frame);
                }
                return true;
            }
            // Press on the WAVE body → what it means depends on the armed tool (the Edit
            // section's toolbar). Select drags a time range, which is what the waveform has
            // always done; Move drags a piece onto another seam; Scale drags a piece's edge.
            PointerKind::Down
                if let Some(hit) =
                    self.audio_wave_frame_at(self.last_pointer.0, self.last_pointer.1) =>
            {
                use ph2d_panel_audio_editor::tool_state::{EditTool, tool};
                let frame = hit.0 as usize;
                match tool() {
                    EditTool::Move => {
                        if let Some(a) = self.audio.as_mut() {
                            a.editor_piece_grab(frame);
                        }
                    }
                    EditTool::Scale => {
                        if let Some(a) = self.audio.as_mut() {
                            a.editor_piece_scale_grab(frame);
                        }
                    }
                    EditTool::Select => {
                        self.audio_sel_drag = Some(hit);
                        if let Some(a) = self.audio.as_mut() {
                            a.editor_clear_selection();
                        }
                    }
                }
                return true;
            }
            // Let go of a piece: THIS is where the reorder / stretch lands, as one undo step.
            PointerKind::Up
                if self
                    .audio
                    .as_ref()
                    .is_some_and(|a| a.editor_piece_drag().is_some()) =>
            {
                if let Some(a) = self.audio.as_mut() {
                    a.editor_piece_release();
                }
                return true;
            }
            PointerKind::Up if self.audio_scrub_drag => {
                self.audio_scrub_drag = false;
                // Hand the playhead back to playback if it's advancing; else the
                // manual position stays where it was dropped.
                if let Some(a) = self.audio.as_mut() {
                    a.editor_end_scrub();
                }
                return true;
            }
            PointerKind::Up if self.audio_sel_drag.take().is_some() => return true,
            _ => {}
        }
        false
    }
}
