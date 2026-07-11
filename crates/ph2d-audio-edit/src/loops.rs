//! Loop-point support (W6 asset-prep): build a **click-free looping buffer** from
//! a loop region so continuous audition — and a game's runtime loop — doesn't tick
//! at the wrap.
//!
//! Control-thread only (HR-3/HR-5 do not apply): allocates and uses `sin`/`cos`.
//!
//! ## The pre-loop crossfade
//!
//! A raw loop over `[s, e)` jumps `data[e-1] → data[s]` at the wrap; unless those
//! two samples happen to be continuous, that step is an audible click. The fix used
//! here is the classic *pre-loop crossfade*: morph the last `L` frames of the region
//! into the `L` frames that PRECEDE `s` in the source. Because `data[s-1] → data[s]`
//! is inherently continuous (they are adjacent samples), making the loop tail end on
//! `data[s-1]` makes the wrap back to `data[s]` continuous too — the seam becomes the
//! source's own `s-1 → s` transition. The crossfade entry is likewise seamless: it
//! starts on `data[e-L]`, continuing the untouched body `…data[e-L-1]`.
//!
//! `L` is clamped to the material actually available (`min(L, s, e-s)`), so a loop
//! that starts at frame 0 — no lead-in — degrades to a plain copy (no crossfade
//! possible), which [`snap_loop_to_zero_crossing`](crate::EditClip::snap_loop_to_zero_crossing)
//! is there to help with.

use std::ops::Range;

use ph2d_audio::SampleData;

use crate::ops::channels;

/// Build a click-free looping buffer of exactly the loop region `[region.start,
/// region.end)`, cross-fading the tail into the `xfade`-frame lead-in before the
/// region. The returned clip is meant to be played on repeat (its last frame wraps
/// seamlessly to its first). Returns `None` when the region is empty or out of range.
///
/// The crossfade length is clamped to `min(xfade, region.start, region_len)`; a
/// region starting at frame 0 gets no crossfade (a verbatim copy).
pub fn crossfaded_loop(
    data: &SampleData,
    region: Range<usize>,
    xfade: usize,
) -> Option<SampleData> {
    let ch = channels(data);
    let frames = data.frame_count();
    let start = region.start.min(frames);
    let end = region.end.min(frames);
    if start >= end {
        return None;
    }
    let region_len = end - start;
    let src = data.samples();

    // Copy the region verbatim first — the head and body are untouched.
    let mut out = vec![0.0f32; region_len * ch];
    out.copy_from_slice(&src[start * ch..end * ch]);

    // Clamp the crossfade to the lead-in available before `start` and to the region.
    let l = xfade.min(start).min(region_len);
    if l > 0 {
        // Overwrite the last `l` frames: fade the region tail out while the pre-`start`
        // lead-in fades in. Equal-power (sin/cos) keeps the level steady through the
        // blend. At the final frame the output lands on `data[start-1]`, so the wrap to
        // the first frame `data[start]` is the source's own continuous step.
        let tail0 = region_len - l;
        for j in 0..l {
            // `t` sweeps 0→1 across the fade; +0.5 centres the ramp so neither endpoint
            // is fully one-sided.
            let t = (j as f32 + 0.5) / l as f32;
            let (g_in, g_out) = (
                (t * std::f32::consts::FRAC_PI_2).sin(),
                (t * std::f32::consts::FRAC_PI_2).cos(),
            );
            let tail_frame = tail0 + j;
            let src_tail = start + tail_frame; // data[e-l + j]
            let src_pre = start - l + j; // data[s-l + j]
            for c in 0..ch {
                let a = src[src_tail * ch + c];
                let b = src[src_pre * ch + c];
                out[tail_frame * ch + c] = a * g_out + b * g_in;
            }
        }
    }

    Some(SampleData::from_interleaved(out, data.format()))
}

/// The worst per-channel discontinuity at a buffer's loop seam — `|first − last|`
/// across channels. A raw loop over discontinuous endpoints reads large here; a
/// crossfaded one reads near zero. Test/diagnostic helper.
#[cfg(test)]
pub(crate) fn seam_step(data: &SampleData) -> f32 {
    let ch = channels(data);
    let frames = data.frame_count();
    if frames == 0 {
        return 0.0;
    }
    let s = data.samples();
    let last = frames - 1;
    (0..ch)
        .map(|c| (s[c] - s[last * ch + c]).abs())
        .fold(0.0, f32::max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    /// A 100 Hz mono sine at 48 k — smooth, so adjacent samples are close but a loop
    /// over an arbitrary window has discontinuous endpoints.
    fn sine(frames: usize) -> Vec<f32> {
        let step = std::f32::consts::TAU * 100.0 / 48_000.0;
        (0..frames).map(|i| (i as f32 * step).sin()).collect()
    }

    #[test]
    fn crossfade_removes_the_wrap_click() {
        let data = SampleData::from_interleaved(sine(4_800), AudioFormat::mono(48_000));
        // A window whose endpoints are NOT continuous → a raw loop clicks.
        let region = 1_000..3_137;
        let raw = SampleData::from_interleaved(
            data.samples()[region.clone()].to_vec(),
            AudioFormat::mono(48_000),
        );
        let raw_step = seam_step(&raw);
        assert!(raw_step > 0.05, "raw loop should click, got {raw_step}");

        let looped = crossfaded_loop(&data, region.clone(), 256).expect("region is non-empty");
        assert_eq!(looped.frame_count(), region.len(), "length is the region");
        let xf_step = seam_step(&looped);
        assert!(
            xf_step < raw_step * 0.1,
            "crossfade must shrink the seam step (raw {raw_step}, xf {xf_step})"
        );
    }

    #[test]
    fn body_before_the_crossfade_is_untouched() {
        let data = SampleData::from_interleaved(sine(4_800), AudioFormat::mono(48_000));
        let region = 1_000..2_000;
        let l = 128;
        let looped = crossfaded_loop(&data, region.clone(), l).unwrap();
        // Everything before the tail crossfade is a verbatim copy of the source region.
        let body = region.len() - l;
        for f in 0..body {
            assert_eq!(
                looped.samples()[f],
                data.samples()[region.start + f],
                "body frame {f} must be verbatim"
            );
        }
    }

    #[test]
    fn stereo_channels_both_crossfade() {
        // L=+ramp, R=−ramp so the two channels are distinct; a stereo loop must fade
        // each independently.
        let frames = 800;
        let mut v = Vec::with_capacity(frames * 2);
        for i in 0..frames {
            let t = i as f32 / frames as f32;
            v.push(t);
            v.push(-t);
        }
        let data = SampleData::from_interleaved(v, AudioFormat::stereo(48_000));
        let looped = crossfaded_loop(&data, 200..600, 64).unwrap();
        assert_eq!(looped.frame_count(), 400);
        assert_eq!(looped.format().channel_count(), 2);
        // The seam is small on both channels (the source is a smooth ramp).
        assert!(seam_step(&looped) < 0.05);
    }

    #[test]
    fn region_at_the_start_has_no_lead_in_so_copies_verbatim() {
        let data = SampleData::from_interleaved(sine(2_000), AudioFormat::mono(48_000));
        // start == 0 → no material before the loop → L clamps to 0 → verbatim copy.
        let looped = crossfaded_loop(&data, 0..1_000, 256).unwrap();
        assert_eq!(looped.samples(), &data.samples()[0..1_000]);
    }

    // The empty / inverted ranges are the whole point of this test — silence clippy's
    // `reversed_empty_ranges` for the deliberate degenerate literals.
    #[test]
    #[allow(clippy::reversed_empty_ranges)]
    fn empty_or_out_of_range_region_is_none() {
        let data = SampleData::from_interleaved(sine(1_000), AudioFormat::mono(48_000));
        assert!(crossfaded_loop(&data, 500..500, 16).is_none(), "empty");
        assert!(crossfaded_loop(&data, 800..600, 16).is_none(), "inverted");
        // Out of range clamps; 2000..3000 → clamped to 1000..1000 → empty → None.
        assert!(
            crossfaded_loop(&data, 2_000..3_000, 16).is_none(),
            "past end"
        );
    }
}
