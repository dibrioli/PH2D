//! **The loop points and the markers survive the file** (ADR-0119 A4).
//!
//! ## The hole this closes
//!
//! `ph2d-audio-encode` has written the `smpl` (loop) and `cue`+`LIST/adtl` (markers) chunks since
//! W6, and it has had readers for both — `read_loop_regions` and `read_markers`, each covered by a
//! round-trip unit test in that crate. **Nothing in the application ever called them.** Export a WAV
//! with a loop, Load it back, and the loop was gone: the readers were proven in isolation and
//! connected to nothing.
//!
//! That is not a missing feature, it is a missing *call*, and no unit test in `ph2d-audio-encode`
//! could ever have caught it — the crate is green either way. So both ends of the round trip live
//! here, as **pure functions of bytes and a clip**, and the gate below drives the whole loop:
//! author → encode → decode → adopt. The dialog is the only part of Export/Load this leaves out,
//! and a dialog is not where a loop point goes missing.

use ph2d_audio_edit::EditClip;
use ph2d_audio_encode::WavMeta;

/// The side-car metadata to write beside `clip`'s audio: its loop region and its cue markers.
///
/// `frames` is the length of the buffer actually being **exported**, which is not always the
/// committed clip's: the effects rack exports its live audition, and a reverb tail makes that
/// longer. Anything past the end of what is written would name audio the file does not contain.
pub(crate) fn wav_meta(clip: &EditClip, frames: u32) -> WavMeta {
    let loops = clip
        .loop_region()
        .and_then(|lp| {
            let start = (lp.start as u32).min(frames);
            let end = (lp.end as u32).min(frames);
            (start < end).then_some(ph2d_audio_encode::LoopRegion { start, end })
        })
        .into_iter()
        .collect();
    let markers = clip
        .markers()
        .iter()
        .filter(|m| (m.frame as u32) < frames)
        .map(|m| ph2d_audio_encode::Marker {
            frame: m.frame as u32,
            name: m.name.clone(),
        })
        .collect();
    WavMeta { loops, markers }
}

/// Adopt the loop region and the cue markers carried by `bytes` into a freshly loaded `clip`.
///
/// Only WAV carries them. For Vorbis and Opus the readers find nothing and this does nothing, which
/// is the honest answer — those formats have no `smpl`, and the Delivery panel says so out loud
/// before the export rather than after.
///
/// Called on the **load** path, on a clip whose history has just been reset, so it deliberately does
/// not commit an undo step: what came out of the file is the clip's starting state, not an edit to it.
pub(crate) fn adopt_wav_meta(clip: &mut EditClip, bytes: &[u8]) {
    if let Some(lp) = ph2d_audio_encode::read_loop_regions(bytes).first() {
        clip.set_loop_region(Some(lp.start as usize..lp.end as usize));
    }
    for m in ph2d_audio_encode::read_markers(bytes) {
        clip.add_marker(m.frame as usize, m.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::{AudioFormat, SampleData};

    fn clip(frames: usize) -> EditClip {
        EditClip::new(SampleData::from_fn(
            frames,
            AudioFormat::mono(48_000),
            |i| (i as f32 / 97.0).sin() * 0.4,
        ))
    }

    /// **The whole round trip, through the application's own two ends** (ADR-0119 A4).
    ///
    /// Author a loop and some markers → encode → decode → adopt. This is the gate that would have
    /// been red for the entire life of the feature: the readers existed, the writers existed, and
    /// the call between them did not.
    #[test]
    fn a_loop_and_its_markers_survive_export_and_load() {
        let mut authored = clip(10_000);
        authored.set_loop_region(Some(2_000..7_000));
        authored.add_marker(1_500, "intro");
        authored.add_marker(8_500, "outro");

        // Export.
        let meta = wav_meta(&authored, authored.frame_count() as u32);
        let bytes = ph2d_audio_encode::encode_wav_with_meta(
            authored.data(),
            ph2d_audio_encode::BitDepth::Pcm16,
            &meta,
        )
        .expect("encode");

        // Load: a brand-new clip, decoded from the file, with nothing authored on it.
        let decoded = ph2d_audio_decode::decode(&bytes).expect("decode");
        let mut loaded = EditClip::new(decoded);
        assert_eq!(
            loaded.loop_region(),
            None,
            "a fresh clip starts with no loop"
        );

        adopt_wav_meta(&mut loaded, &bytes);

        assert_eq!(
            loaded.loop_region(),
            Some(2_000..7_000),
            "the loop did not survive the file — the `smpl` chunk was written and never read"
        );
        let got: Vec<(usize, &str)> = loaded
            .markers()
            .iter()
            .map(|m| (m.frame, m.name.as_str()))
            .collect();
        assert_eq!(
            got,
            vec![(1_500, "intro"), (8_500, "outro")],
            "the cue markers did not survive the file"
        );
    }

    /// Metadata is clamped to the audio that is actually written. The rack exports its live
    /// audition, so a **tail effect** makes the exported buffer longer than the committed clip —
    /// and a trim makes it shorter. A marker past the end names audio the file does not contain.
    #[test]
    fn metadata_is_clamped_to_what_is_actually_exported() {
        let mut authored = clip(10_000);
        authored.set_loop_region(Some(2_000..9_000));
        authored.add_marker(1_000, "keep");
        authored.add_marker(9_500, "drop");

        // Exporting only the first 5_000 frames of it.
        let meta = wav_meta(&authored, 5_000);
        assert_eq!(
            meta.loops[0].end, 5_000,
            "the loop must end where the audio does"
        );
        assert_eq!(
            meta.markers.len(),
            1,
            "a marker past the exported audio names nothing and must not be written"
        );
        assert_eq!(meta.markers[0].name, "keep");
    }

    /// A clip with nothing authored writes **no chunks at all** — an empty `smpl` is not the same
    /// as no `smpl`, and a reader that finds one would report a loop the user never set.
    #[test]
    fn a_clip_with_no_loop_writes_no_chunks() {
        let plain = clip(1_000);
        let meta = wav_meta(&plain, 1_000);
        assert!(meta.loops.is_empty() && meta.markers.is_empty());

        let bytes = ph2d_audio_encode::encode_wav_with_meta(
            plain.data(),
            ph2d_audio_encode::BitDepth::Pcm16,
            &meta,
        )
        .expect("encode");
        let mut loaded = EditClip::new(ph2d_audio_decode::decode(&bytes).expect("decode"));
        adopt_wav_meta(&mut loaded, &bytes);
        assert_eq!(loaded.loop_region(), None);
        assert!(loaded.markers().is_empty());
    }
}
