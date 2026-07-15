//! Variation-container runtime (docs/Audio/, W6 asset-prep). A **descendant submodule**
//! of `audio::editor`, so `impl AudioSystem` here reaches the private editor fields —
//! same trick as `loops`/`markers`/`fx_rack`.
//!
//! A variation container groups several clips the game runtime plays **one** of per
//! trigger (Random / Sequence / Shuffle) with per-play pitch/gain jitter + per-entry
//! weights (the Wwise Random/Sequence Container). The panel owns the UI; this owns the
//! [`ph2d_audio_edit::VariationSet`] + the decoded clips + the picker, and auditions a
//! pick through the preview voice. The set is authored + saved to a manifest file; a
//! future game runtime reads it (the runtime-trigger path is a separate, blocked story).

use super::AudioSystem;
use ph2d_audio::{PlayParams, SampleData};
use ph2d_audio_edit::{Variation, WEIGHT_RANGE};
use std::path::Path;

/// Maximum clips a set holds — mirrors the panel's `MAX_VARIATIONS` (the list paints
/// exactly this many rows, so the shell must not grow past it).
const MAX_VARIATIONS: usize = 12;
/// Full-scale pitch jitter (`± semitones`) at slider = 1.0 — one octave, plenty for
/// footsteps / impacts.
const MAX_PITCH_JITTER_ST: f32 = 12.0;
/// Full-scale gain jitter (`± dB`) at slider = 1.0.
const MAX_GAIN_JITTER_DB: f32 = 12.0;

impl AudioSystem {
    /// Decode `path` and add it as a variation (no-op past the cap or on decode
    /// failure). The decoded clip is cached index-aligned with the entry.
    pub(crate) fn editor_add_variation(&mut self, path: &Path) {
        if self.editor.variation_set.entries.len() >= MAX_VARIATIONS {
            return;
        }
        let Some(data) = decode_file(path) else {
            return;
        };
        let label = path.to_string_lossy().into_owned();
        self.editor
            .variation_set
            .entries
            .push(Variation::new(label));
        self.editor.variation_clips.push(Some(data));
    }

    /// Import by convention: add every decodable audio clip in `dir` to the set (the
    /// `name_01..NN → group` convention — point at a per-group folder and get the whole
    /// set in one click), **natural-sorted** so an unpadded run still lands in order
    /// (Sequence depends on it). Stops at the cap; non-audio / undecodable files are
    /// skipped. Appends to the current set (does not clear it).
    pub(crate) fn editor_add_variation_folder(&mut self, dir: &Path) {
        let mut paths: Vec<std::path::PathBuf> = match std::fs::read_dir(dir) {
            Ok(rd) => rd
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.is_file() && is_audio_path(p))
                .collect(),
            Err(e) => {
                eprintln!("audio: cannot read folder {}: {e}", dir.display());
                return;
            }
        };
        paths.sort_by(|a, b| {
            ph2d_audio_edit::natural_cmp(
                &a.file_name().unwrap_or_default().to_string_lossy(),
                &b.file_name().unwrap_or_default().to_string_lossy(),
            )
        });
        // `editor_add_variation` no-ops once the set is full, so this naturally stops at
        // the cap without decoding the overflow.
        for p in paths {
            self.editor_add_variation(&p);
        }
    }

    /// Remove the variation at `sel` (both the entry and its cached clip).
    pub(crate) fn editor_remove_variation(&mut self, sel: usize) {
        if sel < self.editor.variation_set.entries.len() {
            self.editor.variation_set.entries.remove(sel);
            self.editor.variation_clips.remove(sel);
        }
    }

    /// Cycle the pick strategy by `steps` (wrapping over Random/Sequence/Shuffle).
    pub(crate) fn editor_cycle_variation_strategy(&mut self, steps: i32) {
        self.editor.variation_set.strategy = self.editor.variation_set.strategy.cycled(steps);
    }

    /// Bump the selected entry's weight by `steps` doublings (± halves/doubles),
    /// clamped to the model's weight range.
    pub(crate) fn editor_bump_variation_weight(&mut self, sel: usize, steps: i32) {
        if let Some(e) = self.editor.variation_set.entries.get_mut(sel) {
            let factor = 2f32.powi(steps);
            e.weight = (e.weight * factor).clamp(WEIGHT_RANGE.0, WEIGHT_RANGE.1);
        }
    }

    /// Publish the container-level jitter from the panel's `0..1` slider positions.
    pub(crate) fn editor_set_variation_jitter(&mut self, pitch_norm: f32, gain_norm: f32) {
        self.editor.variation_set.pitch_jitter_semitones =
            pitch_norm.clamp(0.0, 1.0) * MAX_PITCH_JITTER_ST;
        self.editor.variation_set.gain_jitter_db = gain_norm.clamp(0.0, 1.0) * MAX_GAIN_JITTER_DB;
    }

    /// Audition the next variation: pick one per the strategy, draw the per-play
    /// pitch/gain jitter, and play it through the preview voice (one-shot). No-op if
    /// nothing is enabled or the pick's clip failed to decode.
    pub(crate) fn editor_play_variation(&mut self) {
        let Some(i) = self
            .editor
            .variation_picker
            .pick(&self.editor.variation_set)
        else {
            return;
        };
        let Some(Some(data)) = self.editor.variation_clips.get(i) else {
            return;
        };
        let data = data.clone();
        let jitter = self
            .editor
            .variation_picker
            .jitter(&self.editor.variation_set);
        let params = PlayParams {
            pitch: jitter.pitch,
            gain: jitter.gain,
            ..PlayParams::default()
        };
        // The audition borrows the preview voice; it is a transient one-shot, so the
        // transport state is left alone (Play/Stop still drive the loaded clip).
        let _ = self.engine.play_preview(data, params);
    }

    /// Save the current set to `path` as a manifest.
    ///
    /// The entries are rewritten **relative to the manifest** on the way out (see
    /// `manifest_path`): a set that records where its clips were on *this* machine works
    /// exactly once, on this machine — and PH2D is deliberately multi-machine.
    pub(crate) fn editor_save_variation_set(&self, path: &Path) {
        let base = path.parent().unwrap_or(Path::new("."));
        let mut portable = self.editor.variation_set.clone();
        for e in &mut portable.entries {
            e.path = super::manifest_path::to_manifest(Path::new(&e.path), base);
        }
        let text = ph2d_audio_edit::serialize_variation_set(&portable);
        if let Err(e) = std::fs::write(path, text) {
            eprintln!(
                "audio: variation-set save failed for {}: {e}",
                path.display()
            );
        }
    }

    /// Load a manifest from `path`, decoding every entry's clip. A path that fails to
    /// decode keeps its entry but with no clip (so the row still shows + round-trips),
    /// and simply cannot be auditioned.
    pub(crate) fn editor_load_variation_set(&mut self, path: &Path) {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!(
                    "audio: variation-set load failed for {}: {e}",
                    path.display()
                );
                return;
            }
        };
        let mut set = ph2d_audio_edit::parse_variation_set(&text);
        set.entries.truncate(MAX_VARIATIONS);
        // Resolve every entry against the manifest's own directory, and keep the ABSOLUTE path
        // in memory: the model is what the audition and a later Save read from, and both want a
        // path that opens from wherever the app happens to be running.
        let base = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        for e in &mut set.entries {
            e.path = super::manifest_path::from_manifest(&e.path, &base)
                .to_string_lossy()
                .into_owned();
        }
        self.editor.variation_clips = set
            .entries
            .iter()
            .map(|e| decode_file(Path::new(&e.path)))
            .collect();
        self.editor.variation_set = set;
        self.editor.variation_picker = ph2d_audio_edit::VariationPicker::default();
    }

    /// Toggle the selected entry in or out of the pick. The picker already skips disabled
    /// entries and the manifest already round-trips the flag — this is the UI finally being
    /// able to turn what the model always carried.
    pub(crate) fn editor_toggle_variation_enabled(&mut self) {
        let sel = ph2d_panel_audio_editor::variation_sel();
        if let Some(e) = self.editor.variation_set.entries.get_mut(sel) {
            e.enabled = !e.enabled;
        }
    }

    /// Whether the SELECTED entry is enabled — published so the toggle shows its state.
    pub(crate) fn editor_variation_enabled(&self) -> bool {
        let sel = ph2d_panel_audio_editor::variation_sel();
        self.editor
            .variation_set
            .entries
            .get(sel)
            .is_none_or(|e| e.enabled)
    }

    /// The row labels for the panel: `stem  ×weight` (a `(off)` prefix when disabled).
    pub(crate) fn editor_variation_names(&self) -> Vec<String> {
        self.editor
            .variation_set
            .entries
            .iter()
            .map(|e| {
                let stem = Path::new(&e.path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&e.path);
                let off = if e.enabled { "" } else { "(off) " };
                format!("{off}{stem}  \u{00d7}{:.1}", e.weight)
            })
            .collect()
    }

    /// The current strategy's display name (panel selector readout).
    pub(crate) fn editor_variation_strategy(&self) -> &'static str {
        self.editor.variation_set.strategy.name()
    }

    /// Dev smoke (`PH2D_AUDIO_LOOP_SMOKE=1`, called from `editor_loop_smoke`): write a
    /// four-clip set of short pitched blips to a temp folder, then populate the set by
    /// **importing that folder** — so the smoke dogfoods `editor_add_variation_folder`
    /// (scan + natural-sort) instead of adding each by hand. Open the Audio Editor pill:
    /// the Variations section comes filled. Hit **Play Variation** repeatedly (Shuffle
    /// never repeats back-to-back); raise **Pitch/Gain jitter**; **Weight ×2** biases a
    /// row; the selector switches Random / Sequence / Shuffle. **Add Folder…** re-imports
    /// the same `$TMPDIR/ph2d_variation_smoke/` folder if you want to see it live.
    pub(crate) fn editor_variation_smoke(&mut self) {
        use ph2d_audio::{AudioFormat, SampleData};
        let sr = 48_000u32;
        let frames = sr as usize / 4; // 0.25 s blips
        // A little C-major arpeggio so the four picks are obviously distinct.
        let hz = [262.0f32, 330.0, 392.0, 523.0];
        let dir = std::env::temp_dir().join("ph2d_variation_smoke");
        let _ = std::fs::create_dir_all(&dir);
        for (i, f) in hz.iter().enumerate() {
            let step = std::f32::consts::TAU * f / sr as f32;
            let mut v = Vec::with_capacity(frames);
            for n in 0..frames {
                // A short decay envelope so each blip is a clean "tick".
                let env = 1.0 - (n as f32 / frames as f32);
                v.push((n as f32 * step).sin() * 0.4 * env);
            }
            let data = SampleData::from_interleaved(v, AudioFormat::mono(sr));
            let path = dir.join(format!("blip_{:02}.wav", i + 1));
            let _ = ph2d_audio_encode::write_wav(&path, &data, ph2d_audio_encode::BitDepth::Pcm16);
        }
        // Import the folder we just wrote — exercises the scan + natural sort.
        self.editor_add_variation_folder(&dir);
        // Jitter starts at 0 (the panel's sliders own it and the bridge pushes their
        // position every frame) — the four distinct pitches carry the variation, and
        // raising the Pitch/Gain jitter sliders makes each press vary further.
    }
}

/// Read + decode an audio file to [`SampleData`] (logging failures). Shared by Add and
/// manifest Load.
fn decode_file(path: &Path) -> Option<SampleData> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("audio: cannot read variation {}: {e}", path.display());
            return None;
        }
    };
    match crate::audio::decode_any::decode(&bytes) {
        Ok(d) => Some(d),
        Err(e) => {
            eprintln!("audio: variation decode failed for {}: {e}", path.display());
            None
        }
    }
}

/// Whether `path` has an audio extension the folder import considers (same set as the
/// file picker). Decode still gates the real add — this only skips obvious non-audio.
fn is_audio_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .is_some_and(|e| crate::audio::decode_any::AUDIO_IMPORT_EXTS.contains(&e.as_str()))
}
