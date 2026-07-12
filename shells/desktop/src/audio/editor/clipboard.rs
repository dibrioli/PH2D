//! The clipboard commands (W2): **Cut, Copy, Paste**.
//!
//! They live apart from the rest of `editor_apply` because they are the only edits that touch state
//! *outside* the clip — the clipboard itself, which **outlives the document**. Copy from one take,
//! `Load` another, paste: that is why `EditClip::apply_paste` conforms the audio to the
//! destination's rate and channel layout before splicing it in.

use ph2d_panel_audio_editor::AudioEditCmd as Cmd;

use crate::audio::AudioSystem;

impl AudioSystem {
    /// Run `cmd` if it is a clipboard command.
    ///
    /// Returns `true` when the caller should stop — i.e. when the command changed **nothing** and
    /// so must not re-render the preview. Only `Copy` does that: copying is not an edit. Cut and
    /// Paste return `false` and fall through to the caller's hot-swap, like every other edit.
    pub(crate) fn editor_clipboard_cmd(&mut self, cmd: Cmd) -> bool {
        match cmd {
            Cmd::Copy => {
                if let Some(clip) = self.editor.clip.as_ref() {
                    // No undo step, no re-render, nothing sounds different. An editor that lit its
                    // Undo button for a Copy would be telling the user it had changed their audio.
                    self.editor.clipboard = clip.copy_selection();
                }
                true
            }
            Cmd::Cut => {
                if let Some(clip) = self.editor.clip.as_mut()
                    && let Some(taken) = clip.apply_cut()
                {
                    self.editor.clipboard = Some(taken);
                }
                false
            }
            Cmd::Paste => {
                // With nothing selected a paste lands at the **playhead**. It cannot come from the
                // selection: an empty range is stored as `None`, so a caret has nowhere to live
                // there — which is why `apply_paste` takes the position.
                let at = self.editor_playhead_frame() as usize;
                if let Some(pasted) = self.editor.clipboard.clone()
                    && let Some(clip) = self.editor.clip.as_mut()
                {
                    clip.apply_paste(&pasted, at);
                }
                false
            }
            _ => false,
        }
    }
}
