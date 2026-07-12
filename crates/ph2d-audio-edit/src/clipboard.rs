//! **Cut, copy, paste** — the ops W2 asked for and never got (`docs/Audio/02_plano_implementacao_completo.md` §7).
//!
//! The editor had trim, ripple-delete, silence, fades, gain, normalize, DC, reverse, invert and
//! undo — and no way to move a piece of audio from one place to another. The "Cut" button was a
//! ripple delete wearing the wrong name: it threw the selection away rather than taking it with
//! you.
//!
//! ## The one thing that has to be right: PASTE ACROSS FORMATS
//!
//! The clipboard outlives the clip. Copy a phrase from a 44.1 kHz stereo take, `Load` a 48 kHz mono
//! one, paste — and if the samples go in as they are, they go in **8 % sharp and interleaved into
//! the wrong channels**. Nobody hears that as a bug. They hear it as "the paste sounds weird", which
//! is worse, because it is unfalsifiable.
//!
//! So paste **conforms** first: resample to the destination's rate, fold to its channel layout. A
//! clipboard that already matches is passed through byte-for-byte — conforming must never be a
//! resample of audio that did not need resampling.

use ph2d_audio::{AudioFormat, SampleData};

use crate::ops::channels;

/// Make `src` play correctly *in* `target`: its rate, its channel layout.
///
/// Byte-identical passthrough when the formats already agree — the common case (you copied and
/// pasted inside one clip), and the one where a needless resample would quietly soften the audio.
pub fn conform(src: &SampleData, target: AudioFormat) -> SampleData {
    if src.format() == target {
        return src.clone();
    }
    let src_ch = channels(src);
    let dst_ch = target.channel_count().max(1);
    let ratio = f64::from(target.sample_rate.max(1)) / f64::from(src.format().sample_rate.max(1));
    let src_frames = src.frame_count();
    let dst_frames = ((src_frames as f64) * ratio).round() as usize;
    let s = src.samples();

    // One allocation (ADR-0117 D2). Linear interpolation, and the same up-mix rule the mixer uses
    // (`SampleData::frame_stereo`): mono into both channels, more-than-stereo down to the first two.
    SampleData::from_fn(dst_frames * dst_ch, target, |i| {
        let f = i / dst_ch;
        let c = i % dst_ch;
        // Where this output frame sits in the source.
        let pos = f as f64 / ratio;
        let i0 = pos.floor() as usize;
        let frac = (pos - i0 as f64) as f32;

        // Read source channel `c`, folding when the layouts differ.
        let read = |frame: usize| -> f32 {
            if frame >= src_frames {
                return 0.0;
            }
            let base = frame * src_ch;
            match (src_ch, dst_ch) {
                // Mono source, stereo destination: the one channel goes to both.
                (1, _) => s[base],
                // Stereo (or more) source, mono destination: average the first two, which is what a
                // fold-down is — taking only the left would silently throw away half the take.
                (_, 1) => (s[base] + s[base + 1]) * 0.5,
                // Same-ish layouts: channel for channel, and anything the source lacks is silence.
                _ => s.get(base + c).copied().unwrap_or(0.0),
            }
        };

        let a = read(i0);
        let b = read(i0 + 1);
        a + (b - a) * frac
    })
}

/// Insert `other` into `data` at `at` (in frames), pushing everything after it later — a **ripple
/// insert**, the mirror of the ripple delete the Cut button already did.
///
/// `other` must already be in `data`'s format; [`conform`] is how it gets there.
pub fn insert(data: &SampleData, at: usize, other: &SampleData) -> SampleData {
    let ch = channels(data);
    let src = data.samples();
    let ins = other.samples();
    let cut = (at.min(data.frame_count())) * ch;

    // One pass, one allocation (ADR-0117 D2).
    SampleData::from_fn(src.len() + ins.len(), data.format(), |i| {
        if i < cut {
            src[i]
        } else if i < cut + ins.len() {
            ins[i - cut]
        } else {
            src[i - ins.len()]
        }
    })
}

/// Split at `frames` — the clip's own frame indices, in any order, duplicates and all.
///
/// Returns the pieces **between** the cuts, clip start and clip end included as implicit ones. A
/// recording of eight takes with seven markers between them comes back as eight clips.
///
/// This is what `split` can mean in an editor whose document is a single buffer: there is no
/// multi-clip track to split *into*, but there is an asset pipeline to split *out to* — the pieces
/// become files, and files named `<stem>_01..NN` are exactly what the variation importer already
/// reads back as one group.
pub fn split_at(data: &SampleData, frames: &[usize]) -> Vec<SampleData> {
    let total = data.frame_count();
    if total == 0 {
        return Vec::new();
    }
    let mut cuts: Vec<usize> = frames
        .iter()
        .copied()
        .filter(|f| *f > 0 && *f < total)
        .collect();
    cuts.sort_unstable();
    cuts.dedup();

    let mut out = Vec::with_capacity(cuts.len() + 1);
    let mut start = 0usize;
    for &c in &cuts {
        out.push(crate::ops::trim(data, start..c));
        start = c;
    }
    out.push(crate::ops::trim(data, start..total));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EditClip;
    use ph2d_audio::ChannelLayout;

    fn clip(frames: usize, rate: u32, ch: ChannelLayout) -> SampleData {
        let n = if ch == ChannelLayout::Stereo { 2 } else { 1 };
        SampleData::from_fn(frames * n, AudioFormat::new(rate, ch), |i| {
            (i % 97) as f32 / 97.0 - 0.5
        })
    }

    /// **A copy-and-paste inside one clip must not touch a single bit.** Same format in, same format
    /// out — conforming has to be a no-op when there is nothing to conform, or every ordinary paste
    /// quietly resamples the audio it was only supposed to move.
    #[test]
    fn conforming_a_matching_format_is_byte_identical() {
        let c = clip(500, 48_000, ChannelLayout::Stereo);
        let out = conform(&c, c.format());
        assert_eq!(out.samples(), c.samples());
        assert_eq!(out.format(), c.format());
    }

    /// **The bug this exists to prevent.** The clipboard outlives the clip: copy from a 44.1 kHz
    /// take, load a 48 kHz one, paste. Raw samples would go in 8.8 % sharp — audible as "this sounds
    /// a bit off", never as a bug.
    #[test]
    fn pasting_across_sample_rates_keeps_the_duration() {
        let src = clip(44_100, 44_100, ChannelLayout::Stereo); // exactly one second
        let out = conform(&src, AudioFormat::new(48_000, ChannelLayout::Stereo));
        assert_eq!(out.format().sample_rate, 48_000);
        // One second is one second, whatever the rate. A ±1-frame rounding is the resampler; an
        // 8 % error is the bug.
        let secs = out.frame_count() as f64 / 48_000.0;
        assert!(
            (secs - 1.0).abs() < 0.001,
            "one second of 44.1 kHz became {secs:.4} s at 48 kHz — the paste is pitch-shifted"
        );
    }

    /// Mono into stereo goes to **both** channels (the mixer's own up-mix rule), and stereo into
    /// mono is the **average** — taking only the left would silently throw away half the take.
    #[test]
    fn conforming_folds_channels_the_way_the_mixer_does() {
        let mono = SampleData::from_fn(4, AudioFormat::new(48_000, ChannelLayout::Mono), |i| {
            (i as f32 + 1.0) * 0.1
        });
        let up = conform(&mono, AudioFormat::new(48_000, ChannelLayout::Stereo));
        assert_eq!(up.frame_count(), 4);
        for f in 0..4 {
            let [l, r] = up.frame_stereo(f);
            assert_eq!(l, r, "mono did not land in both channels");
        }

        let st = SampleData::from_fn(
            2 * 2,
            AudioFormat::new(48_000, ChannelLayout::Stereo),
            |i| {
                if i % 2 == 0 { 1.0 } else { 0.0 }
            },
        );
        let down = conform(&st, AudioFormat::new(48_000, ChannelLayout::Mono));
        assert_eq!(down.format().channel_count(), 1);
        for s in down.samples() {
            assert!((s - 0.5).abs() < 1e-6, "L=1,R=0 must fold to 0.5, got {s}");
        }
    }

    /// **Cut takes the audio with you.** The button that said "Cut" used to delete the selection and
    /// keep nothing — the one thing the word rules out.
    #[test]
    fn cut_returns_what_it_removed_and_ripples_the_rest_closed() {
        let mut c = EditClip::new(clip(100, 48_000, ChannelLayout::Mono));
        let before = c.data().samples().to_vec();
        c.set_selection(Some(20..30));

        let taken = c.apply_cut().expect("cut returns the audio");
        assert_eq!(taken.frame_count(), 10, "cut did not take the selection");
        assert_eq!(taken.samples(), &before[20..30]);
        assert_eq!(c.frame_count(), 90, "the rest did not ripple closed");
        // The seam is exact: frame 20 is now what frame 30 was.
        assert_eq!(c.data().samples()[20], before[30]);
    }

    /// **Round trip.** Cut a piece out, paste it back where it was, and the clip is exactly what it
    /// was — bit for bit. Two ops that do not compose to the identity are two ops that are lying
    /// about what they do.
    #[test]
    fn cut_then_paste_back_restores_the_clip_exactly() {
        let mut c = EditClip::new(clip(200, 48_000, ChannelLayout::Stereo));
        let before = c.data().samples().to_vec();

        c.set_selection(Some(50..80));
        let taken = c.apply_cut().expect("cut");
        assert_eq!(c.frame_count(), 170);

        // The playhead is where a paste lands when nothing is selected. It cannot be expressed as
        // an empty selection: `install` stores an empty range as `None`, so a caret has nowhere to
        // live in the selection — which is exactly why `apply_paste` takes the position.
        c.apply_paste(&taken, 50);

        assert_eq!(c.frame_count(), 200, "paste did not restore the length");
        assert_eq!(
            c.data().samples(),
            before.as_slice(),
            "cut+paste is not the identity — the seam moved or the audio changed"
        );
    }

    /// A paste over a **selection replaces it**, as it does in every DAW — it does not shove it
    /// aside. And it leaves the pasted audio selected, so a second paste overwrites rather than
    /// stacking.
    #[test]
    fn pasting_over_a_selection_replaces_it() {
        let mut c = EditClip::new(clip(100, 48_000, ChannelLayout::Mono));
        let piece = clip(4, 48_000, ChannelLayout::Mono);

        c.set_selection(Some(10..30)); // 20 frames out, 4 frames in
        c.apply_paste(&piece, 0); // the selection wins over the playhead

        assert_eq!(c.frame_count(), 100 - 20 + 4);
        assert_eq!(
            c.selection(),
            Some(10..14),
            "the pasted audio must end up selected"
        );
        assert_eq!(&c.data().samples()[10..14], piece.samples());
    }

    /// Undo covers a paste in **one** step, and restores the clip exactly.
    #[test]
    fn a_paste_is_one_undo_step() {
        let mut c = EditClip::new(clip(60, 48_000, ChannelLayout::Mono));
        let before = c.data().samples().to_vec();
        c.apply_paste(&clip(10, 48_000, ChannelLayout::Mono), 0);
        assert_eq!(c.frame_count(), 70);

        assert!(c.undo(), "paste did not record an undo step");
        assert_eq!(c.data().samples(), before.as_slice());
        assert!(!c.can_undo(), "a paste cost more than one step");
    }

    /// **Split at markers**: N markers make N+1 pieces, and the pieces put back together are the
    /// clip. A split that loses a sample at a seam is a split that clicks.
    #[test]
    fn splitting_at_markers_partitions_the_clip_exactly() {
        let c = clip(100, 48_000, ChannelLayout::Stereo);
        let pieces = split_at(&c, &[25, 60, 90]);
        assert_eq!(pieces.len(), 4, "3 markers must make 4 pieces");

        let lens: Vec<usize> = pieces.iter().map(|p| p.frame_count()).collect();
        assert_eq!(lens, vec![25, 35, 30, 10]);

        // Concatenated, the pieces ARE the clip — no sample lost, none duplicated.
        let rejoined: Vec<f32> = pieces.iter().flat_map(|p| p.samples().to_vec()).collect();
        assert_eq!(rejoined, c.samples(), "the split lost or duplicated audio");
    }

    /// Markers out of order, duplicated, or off the ends must not produce empty or negative pieces.
    #[test]
    fn split_survives_nonsense_markers() {
        let c = clip(50, 48_000, ChannelLayout::Mono);
        let pieces = split_at(&c, &[40, 10, 10, 0, 50, 999]);
        // 0, 50 and 999 are not interior cuts; 10 is duplicated. Interior cuts: 10, 40.
        assert_eq!(pieces.len(), 3);
        assert_eq!(
            pieces.iter().map(|p| p.frame_count()).sum::<usize>(),
            50,
            "the pieces must still add up to the clip"
        );
    }
}
