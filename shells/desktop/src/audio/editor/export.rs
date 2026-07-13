//! Encoding the loaded clip out to disk — WAV, Ogg Vorbis, Opus.
//!
//! Split out of `editor.rs` to keep it under the shell's 600-LOC cap (HR-18). Split, never an
//! allowlist: the cap exists so a file stays readable, and an exception only moves the problem onto
//! whoever opens it next.
//!
//! **They export what is SOUNDING and SHOWN** — the rack's live audition when one is up, else the
//! committed clip. Exporting `self.editor.clip` here once wrote a dry, shorter file than the
//! waveform on screen (2026-07-09 audit).

use crate::audio::AudioSystem;

impl AudioSystem {
    /// Write the loaded clip out to `path` as a PCM WAV at `depth`.
    pub(crate) fn editor_export(&self, path: &std::path::Path, depth: ph2d_audio_encode::BitDepth) {
        // Export what is SOUNDING and SHOWN — the live audition when the rack is
        // previewing, else the committed clip. `editor_clip` / `editor_duration_secs`
        // already report the audition, so exporting `self.editor.clip` here silently
        // wrote a dry, shorter file than the waveform on screen (2026-07-09 audit).
        let Some(clip) = self.editor_sounding() else {
            return;
        };
        // The loop region and the cue markers ride along in the WAV's `smpl` and `cue`+`adtl`
        // chunks, so the loop is sample-exact on re-import and in a game runtime. Both ends of that
        // round trip live in `meta.rs` — and are gated there, because the readers used to exist and
        // never be called.
        //
        // Clamped to the buffer actually being written: the rack exports its live audition, and a
        // reverb tail makes that longer than the committed clip.
        let frames = clip.frame_count() as u32;
        let meta = self
            .editor
            .clip
            .as_ref()
            .map(|c| super::meta::wav_meta(c, frames))
            .unwrap_or_default();
        match ph2d_audio_encode::write_wav_with_meta(path, clip.data(), depth, &meta) {
            Ok(()) => println!(
                "audio: exported {} (WAV {depth:?}{})",
                path.display(),
                if meta.loops.is_empty() {
                    ""
                } else {
                    ", smpl loop"
                }
            ),
            Err(e) => eprintln!("audio: export failed for {}: {e}", path.display()),
        }
    }

    /// Write the loaded clip out to `path` as compressed **Ogg Vorbis** (VBR, ADR-0113).
    /// Like [`Self::editor_export`], it writes what is SOUNDING (audition / mono view /
    /// bypass), so the file matches the waveform. Ogg Vorbis carries no `smpl`/`cue`
    /// side-car, so loop points + markers are a WAV-only feature — the audio is exported;
    /// re-import the loop from the source WAV if you need sample-exact looping.
    /// Write the clip out as **Opus** (ADR-0116) — the best quality per byte here, and the one
    /// format the app has to be able to read back itself, which it now can (`decode_any`).
    pub(crate) fn editor_export_opus(&self, path: &std::path::Path, quality: f32) {
        let Some(clip) = self.editor_sounding() else {
            return;
        };
        let bitrate = ph2d_audio_encode::opus_bitrate(quality);
        match ph2d_audio_encode::encode_opus(clip.data(), quality)
            .map_err(|e| e.to_string())
            .and_then(|bytes| std::fs::write(path, bytes).map_err(|e| e.to_string()))
        {
            Ok(()) => println!(
                "audio: exported {} (Opus, {} kbps)",
                path.display(),
                bitrate / 1_000
            ),
            Err(e) => eprintln!("audio: opus export failed for {}: {e}", path.display()),
        }
    }

    pub(crate) fn editor_export_ogg(&self, path: &std::path::Path, quality: f32) {
        let Some(clip) = self.editor_sounding() else {
            return;
        };
        match ph2d_audio_encode::write_ogg(path, clip.data(), quality) {
            Ok(()) => println!("audio: exported {} (Ogg Vorbis VBR)", path.display()),
            Err(e) => eprintln!("audio: ogg export failed for {}: {e}", path.display()),
        }
    }
}
