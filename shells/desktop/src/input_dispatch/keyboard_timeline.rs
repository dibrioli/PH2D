//! Os atalhos que a **TIMELINE reivindica** — extraídos do `keyboard.rs`, que a integração
//! de 2026-07-27 (`line/anim` + `line/physics` na mesma janela) empurrou a 604 > 600 LOC.
//!
//! O corte é por RESPONSABILIDADE e não por tamanho: as quatro travas aqui têm a MESMA
//! pergunta na guarda (`timeline_panel_open()`) e o mesmo dono (o painel docado). A **ORDEM
//! é load-bearing** e não mudou — elas continuam correndo depois dos blocos de Vector/Motion
//! (uma ferramenta ativa fica com o acorde) e antes dos Escapes/atalhos do Painter; o
//! chamador só troca quatro `if … return;` por um `if self.timeline_key(…) { return; }`.
//!
//! `true` = **consumi a tecla**. Cair fora de todas devolve `false`, que é o fall-through
//! que deixa o mesmo Ctrl+Z alcançar o undo do painter/imagem em outros lugares.

use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::App;

impl App {
    /// As teclas do painel de timeline docado. Devolve `true` quando consumiu.
    pub(crate) fn timeline_key(
        &mut self,
        physical_key: PhysicalKey,
        state: ElementState,
        repeat: bool,
    ) -> bool {
        // General timeline (W1) undo/redo with Ctrl/Cmd, while the docked
        // timeline panel is open AND it has a step in that direction. Pushed as
        // an intent so the bridge re-applies the restored document to the scene
        // the same frame (drain → apply_from_doc). Falls through when the
        // timeline stack is empty in that direction, so the same chord still
        // reaches the painter / image-edit undo for other contexts.
        if self.timeline_panel_open()
            && state == ElementState::Pressed
            && !repeat
            && (self.modifiers.control_key() || self.modifiers.super_key())
            && let PhysicalKey::Code(code) = physical_key
        {
            let redo = matches!(code, KeyCode::KeyY)
                || (matches!(code, KeyCode::KeyZ) && self.modifiers.shift_key());
            let undo = matches!(code, KeyCode::KeyZ) && !self.modifiers.shift_key();
            if redo && self.timeline.history.can_redo() {
                self.timeline_intents
                    .push(ph2d_timeline::TimelineIntent::Redo);
                return true;
            }
            if undo && self.timeline.history.can_undo() {
                self.timeline_intents
                    .push(ph2d_timeline::TimelineIntent::Undo);
                return true;
            }
        }

        // General timeline (W2.E5b) — Delete/Backspace removes the selected keys
        // while the panel is open, something is selected, and no text field holds
        // focus. One undo step (via `apply_intent`). Falls through otherwise so
        // Delete keeps its meaning in every other context.
        if self.timeline_panel_open()
            && state == ElementState::Pressed
            && !repeat
            && !self.modifiers.control_key()
            && !self.modifiers.super_key()
            && !self.timeline.selection.is_empty()
            && !self.vector_text_field_focused()
            && matches!(
                physical_key,
                PhysicalKey::Code(KeyCode::Delete | KeyCode::Backspace)
            )
        {
            self.timeline_intents
                .push(ph2d_timeline::TimelineIntent::DeleteSelection);
            return true;
        }

        // (W4.T6/B5) O bloco do "sidecar da timeline" — Ctrl+S/Ctrl+O no contexto do painel —
        // foi REMOVIDO daqui. Era **inalcançável**: o Ctrl+S/Ctrl+O GLOBAL de projeto, lá em
        // cima, já retornava antes dele. E o comentário ainda dizia "não há save de projeto
        // ainda", o que deixou de ser verdade quando o `project.rs` landou. Hoje a timeline
        // viaja DENTRO do arquivo de projeto (`ProjectFile.timeline`) — um dado, um formato.

        // General timeline (W4.T3) — `M` drops a marker at the playhead while the
        // panel is open (Blender's marker key). Intercepted here, before the
        // global theme toggle `M` in `input_handlers`, so it only reclaims the key
        // in the timeline context; a focused text field keeps `M` for typing.
        if self.timeline_panel_open()
            && state == ElementState::Pressed
            && !repeat
            && !self.modifiers.control_key()
            && !self.modifiers.super_key()
            && !self.modifiers.alt_key()
            && !self.vector_text_field_focused()
            && matches!(physical_key, PhysicalKey::Code(KeyCode::KeyM))
        {
            let label = format!("M{}", self.timeline.doc.markers().len() + 1);
            self.timeline_intents
                .push(ph2d_timeline::TimelineIntent::AddMarker {
                    t_seconds: self.playhead.time(),
                    label,
                });
            return true;
        }

        // General timeline (W2.E7) — Ctrl/Cmd + C/X/V/D on the dope-sheet keys.
        // Runs AFTER the Vector/Motion blocks, so an active tool keeps the chord;
        // requires the panel open, no focused text field, and something to act on
        // (a selection to copy/cut/duplicate, a clipboard to paste) — otherwise it
        // falls through to the OS/text clipboard.
        if self.timeline_panel_open()
            && state == ElementState::Pressed
            && !repeat
            && (self.modifiers.control_key() || self.modifiers.super_key())
            && !self.vector_text_field_focused()
            && let PhysicalKey::Code(code) = physical_key
        {
            use ph2d_timeline::TimelineIntent as I;
            let has_selection = !self.timeline.selection.is_empty();
            let intent = match code {
                KeyCode::KeyC if has_selection => Some(I::CopySelection),
                KeyCode::KeyX if has_selection => Some(I::CutSelection),
                KeyCode::KeyV if !self.timeline.clipboard.is_empty() => Some(I::Paste),
                // Duplicate: the copies land on the playhead (or two frames right
                // of the source when the playhead is already on it) and become
                // the selection — `apply_intent` owns that policy, it has both
                // the playhead and the selection.
                KeyCode::KeyD if has_selection => Some(I::DuplicateSelection),
                // Time-Reverse Keyframes (AE): mirror the selected keys in time
                // about their own centre. Same selection-verb family as C/X/V/D,
                // so it lives in the same chord block and needs a selection.
                KeyCode::KeyR if has_selection => Some(I::ReverseSelectedKeys),
                // Distribute Evenly: respace the selected keys uniformly in time,
                // per track (Blender *Distribute Keyframes*). Parameterless — same
                // selection-verb family. Plain `E` (no chord) is the Painter
                // eraser, which requires NO modifier, so the two never collide.
                KeyCode::KeyE if has_selection => Some(I::DistributeSelectedKeys),
                _ => None,
            };
            if let Some(intent) = intent {
                self.timeline_intents.push(intent);
                return true;
            }
        }

        false
    }
}
