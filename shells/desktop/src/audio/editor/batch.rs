//! Batch LUFS normalisation (W6 asset-prep): make a whole folder of audio files hit a
//! target integrated loudness, writing normalised WAV copies — the "consistent SFX
//! loudness" pass a game library needs. **Non-destructive:** originals are untouched;
//! copies land in a `normalized/` subfolder. Reuses the decode + normalize + encode
//! crates, so no new dependency.

use std::path::Path;

use super::AudioSystem;

/// Audio file extensions the batch walks — the SAME list the Load picker uses, so a format the
/// editor can open is a format the batch will find (Opus was missing from both).
use crate::audio::decode_any::AUDIO_IMPORT_EXTS as AUDIO_EXTS;

impl AudioSystem {
    /// Normalise every audio file directly inside `folder` to `target_lufs`. Thin
    /// wrapper over [`batch_lufs_dir`] so the bridge can call it as `audio.…`.
    pub(crate) fn editor_batch_lufs(&self, folder: &Path, target_lufs: f32) -> (usize, usize) {
        batch_lufs_dir(folder, target_lufs)
    }
}

/// Normalise every audio file directly inside `folder` to `target_lufs`, writing
/// `<folder>/normalized/<name>.wav` (PCM16). Skips non-audio / unreadable files and
/// the output subfolder itself. Prints a summary; returns `(ok, failed)`.
pub(super) fn batch_lufs_dir(folder: &Path, target_lufs: f32) -> (usize, usize) {
    let out_dir = folder.join("normalized");
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        eprintln!("audio: batch LUFS cannot create {}: {e}", out_dir.display());
        return (0, 0);
    }
    let entries = match std::fs::read_dir(folder) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("audio: batch LUFS cannot read {}: {e}", folder.display());
            return (0, 0);
        }
    };
    let (mut ok, mut failed) = (0usize, 0usize);
    for entry in entries.flatten() {
        let path = entry.path();
        let is_audio = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| AUDIO_EXTS.contains(&e.to_ascii_lowercase().as_str()));
        if !path.is_file() || !is_audio {
            continue;
        }
        match normalize_one(&path, &out_dir, target_lufs) {
            Ok(()) => ok += 1,
            Err(e) => {
                eprintln!("audio: batch LUFS skipped {}: {e}", path.display());
                failed += 1;
            }
        }
    }
    println!(
        "audio: batch LUFS to {target_lufs:.0} LUFS -> {ok} ok, {failed} failed, in {}",
        out_dir.display()
    );
    (ok, failed)
}

/// Decode one file, normalise it to `target_lufs`, and write a PCM16 WAV copy under
/// `out_dir` (same stem, `.wav` extension).
fn normalize_one(path: &Path, out_dir: &Path, target_lufs: f32) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let data = crate::audio::decode_any::decode(&bytes).map_err(|e| e.to_string())?;
    let normalized = ph2d_audio_edit::normalize_lufs(&data, target_lufs);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("clip");
    let out = out_dir.join(format!("{stem}.wav"));
    ph2d_audio_encode::write_wav(&out, &normalized, ph2d_audio_encode::BitDepth::Pcm16)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::{AudioFormat, SampleData, dsp};

    /// End-to-end: a quiet tone in a folder gets a louder normalised copy under
    /// `normalized/`, and non-audio files are ignored.
    #[test]
    fn batch_normalizes_audio_and_skips_the_rest() {
        // A unique temp dir (one test → the process id is enough).
        let dir = std::env::temp_dir().join(format!("ph2d_batch_lufs_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // A quiet 1 kHz tone (~0.03 amplitude → well below −16 LUFS).
        let step = std::f32::consts::TAU * 1_000.0 / 48_000.0;
        let v: Vec<f32> = (0..48_000)
            .map(|i| (i as f32 * step).sin() * 0.03)
            .collect();
        let quiet = SampleData::from_interleaved(v, AudioFormat::mono(48_000));
        let bytes =
            ph2d_audio_encode::encode_wav(&quiet, ph2d_audio_encode::BitDepth::Float32).unwrap();
        std::fs::write(dir.join("tone.wav"), &bytes).unwrap();
        // A non-audio file that must be ignored.
        std::fs::write(dir.join("notes.txt"), b"ignore me").unwrap();

        let (ok, failed) = batch_lufs_dir(&dir, -16.0);
        assert_eq!(
            (ok, failed),
            (1, 0),
            "one audio file normalised, txt skipped"
        );

        // The copy exists, decodes, and is louder than the quiet input.
        let out_bytes = std::fs::read(dir.join("normalized/tone.wav")).unwrap();
        let out = ph2d_audio_decode::decode(&out_bytes).unwrap();
        let before = dsp::integrated_lufs(quiet.samples(), 1, 48_000);
        let after = dsp::integrated_lufs(out.samples(), 1, 48_000);
        assert!(
            after > before + 10.0,
            "normalised copy must be much louder (before {before:.1}, after {after:.1})"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
