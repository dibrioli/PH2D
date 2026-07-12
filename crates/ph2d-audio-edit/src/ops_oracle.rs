//! **A6 of ADR-0117: the splice did not change a single byte.**
//!
//! The one-pass rewrite (`ops::splice` / `ops::in_range_tail`) is on the path every one of the
//! rack's effects takes. The rack's whole invariant — *every effect is a byte-identical no-op at
//! its neutral point* — sits downstream of it. So the refactor owes a proof, not a compile.
//!
//! The proof is stronger than sweeping the 39 effects, and it does not rot when the 40th lands:
//! **the splice is a pure function of `(src, range, processed, skip)`** and knows nothing about
//! which effect produced `processed`. Show that the new splice agrees with the old one across the
//! geometry — every range position, every warm-up, mono and stereo, the edges — and every effect
//! is byte-identical *by construction*, forever.
//!
//! The oracle below is the **verbatim pre-ADR-0117 implementation**: `Vec::with_capacity`, three
//! `extend_from_slice`, `SampleData::from_interleaved`. Same pattern as `lpc.rs::solve` (the
//! Gaussian oracle for Levinson) and `convolve.rs::direct` (the definition, against FFT
//! overlap-add) elsewhere in this line: keep the slow, obviously-correct version as a test-only
//! judge of the fast one.

use std::ops::Range;

use ph2d_audio::SampleData;

use crate::ops::channels;

/// The old `in_range` splice, verbatim: head + processed + tail through a `Vec`, then
/// `from_interleaved` (which reallocates and memcpy's the lot).
fn splice_oracle(
    data: &SampleData,
    r: &Range<usize>,
    processed: &[f32],
    skip_frames: usize,
) -> SampleData {
    let ch = channels(data);
    let src = data.samples();
    let mut out = Vec::with_capacity(src.len());
    out.extend_from_slice(&src[..r.start * ch]);
    out.extend_from_slice(&processed[skip_frames * ch..]);
    out.extend_from_slice(&src[r.end * ch..]);
    SampleData::from_interleaved(out, data.format())
}

/// The old `in_range_tail`, verbatim: zero a `Vec`, memcpy the source over it, memcpy the region
/// over that, add the ring-out, then let `Arc::from(Vec)` copy the whole thing again.
fn tail_oracle(
    data: &SampleData,
    r: &Range<usize>,
    processed: &[f32],
    region_len: usize,
    tail: usize,
) -> SampleData {
    let ch = channels(data);
    let src = data.samples();
    let out_frames = data.frame_count().max(r.end + tail);
    let mut out = vec![0.0f32; out_frames * ch];
    out[..src.len()].copy_from_slice(src);
    out[r.start * ch..r.end * ch].copy_from_slice(&processed[..region_len * ch]);
    for i in 0..tail * ch {
        let dst = r.end * ch + i;
        out[dst] = (out[dst] + processed[region_len * ch + i]).clamp(-1.0, 1.0);
    }
    SampleData::from_interleaved(out, data.format())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    /// Deterministic pseudo-audio — a real signal, not a ramp: a ramp hides an off-by-one that a
    /// signal with structure exposes.
    fn buf(frames: usize, ch: usize, seed: u32) -> SampleData {
        let mut s = seed.wrapping_mul(2_654_435_761).wrapping_add(1);
        let samples: Vec<f32> = (0..frames * ch)
            .map(|_| {
                s ^= s << 13;
                s ^= s >> 17;
                s ^= s << 5;
                (s as f32 / u32::MAX as f32) * 2.0 - 1.0
            })
            .collect();
        let format = if ch == 2 {
            AudioFormat::stereo(48_000)
        } else {
            AudioFormat::mono(48_000)
        };
        SampleData::from_interleaved(samples, format)
    }

    /// The **A6 gate**. Across the whole geometry, the one-pass splice must agree with the old
    /// Vec-and-memcpy one **to the bit** — not "to within an epsilon". A resampling difference
    /// would be a bug; a rounding difference would be a bug; there is no arithmetic here at all,
    /// so only an indexing mistake can show up, and an indexing mistake is exactly what this
    /// catches.
    #[test]
    fn the_one_pass_splice_is_byte_identical_to_the_old_one() {
        let mut cases = 0;
        for &ch in &[1usize, 2] {
            for &frames in &[1usize, 2, 7, 64, 1000] {
                let data = buf(frames, ch, 7);
                for start in 0..=frames {
                    for end in start..=frames {
                        let region_len = end - start;
                        // Every legal warm-up: none, some, and all the audio that precedes.
                        for warm in [0usize, 1, 3, start].into_iter().filter(|w| *w <= start) {
                            let processed = buf(warm + region_len, ch, 13);
                            let r = start..end;
                            let want = splice_oracle(&data, &r, processed.samples(), warm);
                            let got = crate::ops::splice(&data, &r, processed.samples(), warm);
                            assert_eq!(
                                got.samples(),
                                want.samples(),
                                "splice differs: ch={ch} frames={frames} range={start}..{end} \
                                 warm={warm}"
                            );
                            assert_eq!(got.format().channel_count(), ch);
                            cases += 1;
                        }
                    }
                }
            }
        }
        // If the loops ever collapse to nothing, the gate is green for the wrong reason.
        assert!(cases > 1_000, "the sweep degenerated: only {cases} cases");
    }

    /// The tail splice, same standard. This one has real arithmetic in it (the ring-out is *added*
    /// on top of what follows, and clamped), so a byte difference here would be an audible one.
    #[test]
    fn the_one_pass_tail_is_byte_identical_to_the_old_one() {
        let mut cases = 0;
        for &ch in &[1usize, 2] {
            for &frames in &[1usize, 8, 100] {
                let data = buf(frames, ch, 3);
                for start in 0..frames {
                    for end in (start + 1)..=frames {
                        let region_len = end - start;
                        // A tail that stays inside the clip, one that reaches the end, and one
                        // that grows it well past — the case where the clip lengthens.
                        for &tail in &[0usize, 1, 5, 50, 200] {
                            let processed = buf(region_len + tail, ch, 29);
                            let r = start..end;
                            let want =
                                tail_oracle(&data, &r, processed.samples(), region_len, tail);
                            let got = crate::ops::in_range_tail(&data, r.clone(), tail, |_, _| {
                                processed.clone()
                            });
                            assert_eq!(
                                got.samples(),
                                want.samples(),
                                "tail differs: ch={ch} frames={frames} range={start}..{end} \
                                 tail={tail}"
                            );
                            assert_eq!(
                                got.frame_count(),
                                frames.max(end + tail),
                                "the clip must grow exactly as far as the ring-out reaches"
                            );
                            cases += 1;
                        }
                    }
                }
            }
        }
        assert!(cases > 500, "the sweep degenerated: only {cases} cases");
    }
}
