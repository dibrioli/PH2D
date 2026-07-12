//! **Export Pieces** — one recording, N takes, N files.
//!
//! The clip is already cut into pieces (Split, or Split at Markers); this writes them out. Files
//! named `<stem>_01..NN` are **exactly what the variation importer already reads back as one
//! group** (`variation.rs`, import-by-convention), so the loop closes on itself: record eight
//! footsteps in one pass, drop seven markers, Split at Markers, Export Pieces — and the eight
//! assets come back as a ready-made variation container with round-robin and weights.
//!
//! ## Why this is not called Split
//!
//! It used to be. The button said "Split at Markers" and it encoded audio to disk, picked a
//! folder, and adopted a variation set — an *emitting* verb wearing an *editing* verb's name, in
//! the Markers section, doing none of what it said. Enio's correction (2026-07-12) was that Split
//! should split the clip; so splitting became structure (`ph2d_audio_edit::pieces`), and the file
//! writing kept the behaviour, took the honest name, and moved to **Delivery** — where emitting
//! lives, and where the codec it emits in was priced.
//!
//! The pieces are written in the codec the **Delivery** section is currently showing, because that
//! is the one the panel just priced. Emitting a format the user did not choose would be a
//! surprise, and the whole point of Delivery is that there are none.

use ph2d_audio::SampleData;
use ph2d_audio_encode::Codec;

use crate::audio::AudioSystem;

impl AudioSystem {
    /// Whether the clipboard holds audio — what lights the Paste button.
    pub(crate) fn editor_has_clipboard(&self) -> bool {
        self.editor.clipboard.is_some()
    }

    /// Write one file per piece into `dir`, then adopt them as the variation set.
    ///
    /// Non-destructive: the clip on screen is untouched. Emitting assets is not editing the one you
    /// have — undo has nothing to undo, and the user's take survives its own export.
    pub(crate) fn editor_export_pieces(&mut self, dir: &std::path::Path) {
        let Some(clip) = self.editor.clip.as_ref() else {
            return;
        };
        let pieces = clip.piece_clips();
        if pieces.len() < 2 {
            // One piece is not a set. An uncut clip has nothing to emit that plain Export does not
            // already do, and writing a single file called `_01` would be a confusing way to say
            // "nothing happened".
            return;
        }

        let codec = self.editor_codec();
        let stem = if self.editor.name.is_empty() {
            "split".to_string()
        } else {
            self.editor.name.clone()
        };

        let mut written: Vec<std::path::PathBuf> = Vec::with_capacity(pieces.len());
        for (i, piece) in pieces.iter().enumerate() {
            // `_01..NN`, zero-padded — the convention the importer groups on. Unpadded, `_10` sorts
            // next to `_1` in a plain string sort, which is how a ten-take set comes back shuffled.
            let path = dir.join(format!("{stem}_{:02}.{}", i + 1, codec.extension()));
            if write_clip(piece, &path, codec) {
                written.push(path);
            }
        }

        // The pieces ARE a variation set — that is what a set of takes IS. Adopting them here is
        // what turns "I exported eight files" into "I have a footstep sound".
        for path in &written {
            self.editor_add_variation(path);
        }
    }
}

/// Encode `clip` to `path` in `codec`. `false` if it could not be written — a full disk or a
/// read-only folder is the user's problem to see, not a reason to lose the other seven takes.
fn write_clip(clip: &SampleData, path: &std::path::Path, codec: Codec) -> bool {
    let quality = ph2d_panel_audio_editor::delivery_state::quality();
    let bytes = match codec {
        Codec::Wav16 => ph2d_audio_encode::encode_wav(clip, ph2d_audio_encode::BitDepth::Pcm16),
        Codec::Wav24 => ph2d_audio_encode::encode_wav(clip, ph2d_audio_encode::BitDepth::Pcm24),
        Codec::OggVorbis => ph2d_audio_encode::encode_ogg(clip, quality),
        Codec::Opus => ph2d_audio_encode::encode_opus(clip, quality),
    };
    match bytes {
        Ok(b) => std::fs::write(path, b).is_ok(),
        Err(_) => false,
    }
}
