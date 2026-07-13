//! Loop-point support (W6 asset-prep; **ADR-0119**): bake a **click-free seam** into the audio, so
//! a runtime that loops by *jumping* has something clean to jump across.
//!
//! Control-thread only (HR-3/HR-5 do not apply): allocates and uses `sin`/`cos`.
//!
//! ## The pre-loop crossfade
//!
//! A loop over `[s, e)` jumps `data[e-1] → data[s]` at the wrap; unless those two samples happen to
//! be continuous, that step is an audible click. The fix is the classic *pre-loop crossfade*: morph
//! the last `L` frames of the region into the `L` frames that **precede** `s` in the source.
//!
//! Because `data[s-1] → data[s]` is inherently continuous (they are adjacent samples), making the
//! loop's tail end on `data[s-1]` makes the wrap back to `data[s]` continuous too — the seam becomes
//! the source's own `s-1 → s` transition. The entry into the crossfade is likewise seamless: it
//! starts on `data[e-L]`, continuing the untouched body before it.
//!
//! **The intro is the pre-roll.** `L` is clamped to the material actually available
//! (`min(L, s, e-s)`), so a loop that starts at frame 0 has nothing to fade *from* and the bake is a
//! no-op — there, the tool is
//! [`snap_loop_to_zero_crossing`](crate::EditClip::snap_loop_to_zero_crossing).
//!
//! ## Why this is a BAKE and not a preview
//!
//! It used to build a separate, region-only buffer that the editor played on a whole-buffer loop —
//! a click-free loop that **only existed in the editor**, because the mixer had no loop region to
//! play and no second read head to crossfade with. The preview was clean and the exported asset was
//! not (ADR-0119). Now the runtime honours a real region and *jumps*, so the seam has to be in the
//! audio: what the editor plays is what the game plays, because it is the same samples.

use std::ops::Range;

use ph2d_audio::SampleData;

use crate::ops::channels;

/// Write the pre-loop crossfade **into** `data` at the loop seam: the last `xfade` frames of
/// `[start, end)` are blended into the `xfade` frames leading up to `start`.
///
/// Length-preserving — only the seam changes, so the loop points, the markers and the cuts all stay
/// where they are. `None` when there is nothing to do: no region, or no lead-in to fade from.
///
/// The crossfade length is clamped to `min(xfade, start, region_len)`.
pub fn bake_loop_crossfade(
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
    let l = xfade.min(start).min(end - start);
    if l == 0 {
        // A loop that starts at frame 0 has no audio before it to fade from. Saying so (`None`)
        // beats returning an unchanged buffer that would land a do-nothing step on the undo timeline.
        return None;
    }

    let src = data.samples();
    let tail0 = end - l; // first frame of the seam

    // One buffer, written once (ADR-0117 D2): the clip, verbatim, with the seam overwritten.
    Some(SampleData::build(src.len(), data.format(), |out| {
        out.copy_from_slice(src);
        for j in 0..l {
            // `t` sweeps 0→1 across the fade; +0.5 centres the ramp so neither endpoint is fully
            // one-sided. Equal-power (sin/cos) keeps the level steady through the blend — the two
            // sides are different parts of the take, so a linear fade would dip in the middle.
            let t = (j as f32 + 0.5) / l as f32;
            let (g_in, g_out) = (
                (t * std::f32::consts::FRAC_PI_2).sin(),
                (t * std::f32::consts::FRAC_PI_2).cos(),
            );
            let f_tail = tail0 + j; // data[e-l + j]
            let f_pre = start - l + j; // data[s-l + j]
            for c in 0..ch {
                out[f_tail * ch + c] = src[f_tail * ch + c] * g_out + src[f_pre * ch + c] * g_in;
            }
        }
    }))
}

/// The discontinuity a looping voice would hear at the wrap: `|data[start] − data[end-1]|`, worst
/// channel. A raw loop over an arbitrary window reads large here; a baked one reads near zero.
///
/// This is measured **across the jump the runtime actually makes** — which is the only place the
/// click can be.
pub fn loop_seam_step(data: &SampleData, region: Range<usize>) -> f32 {
    let ch = channels(data);
    let frames = data.frame_count();
    let start = region.start.min(frames);
    let end = region.end.min(frames);
    if start >= end {
        return 0.0;
    }
    let s = data.samples();
    let last = end - 1;
    (0..ch)
        .map(|c| (s[start * ch + c] - s[last * ch + c]).abs())
        .fold(0.0, f32::max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    /// A 100 Hz mono sine at 48 k — smooth, so adjacent samples are close but a loop over an
    /// arbitrary window has discontinuous endpoints.
    fn sine(frames: usize) -> Vec<f32> {
        let step = std::f32::consts::TAU * 100.0 / 48_000.0;
        (0..frames).map(|i| (i as f32 * step).sin()).collect()
    }

    /// The point of the whole thing: the jump the runtime makes stops clicking.
    #[test]
    fn the_bake_removes_the_click_at_the_jump() {
        let data = SampleData::from_interleaved(sine(4_800), AudioFormat::mono(48_000));
        // A window whose endpoints are NOT continuous → a raw loop clicks.
        let region = 1_000..3_137;

        let raw_step = loop_seam_step(&data, region.clone());
        assert!(raw_step > 0.05, "raw loop should click, got {raw_step}");

        let baked = bake_loop_crossfade(&data, region.clone(), 256).expect("bakeable");
        assert_eq!(
            baked.frame_count(),
            data.frame_count(),
            "the bake is length-preserving — the loop points must not move"
        );
        let baked_step = loop_seam_step(&baked, region);
        assert!(
            baked_step < raw_step * 0.1,
            "the bake must shrink the step across the jump (raw {raw_step}, baked {baked_step})"
        );
    }

    /// Everything outside the seam is **byte-identical**. A bake is a surgical edit, not a re-render
    /// of the clip.
    #[test]
    fn only_the_seam_changes() {
        let data = SampleData::from_interleaved(sine(4_800), AudioFormat::mono(48_000));
        let (region, l) = (1_000..2_000, 128);
        let baked = bake_loop_crossfade(&data, region.clone(), l).unwrap();

        let seam = (region.end - l)..region.end;
        for f in 0..data.frame_count() {
            if seam.contains(&f) {
                continue;
            }
            assert_eq!(
                baked.samples()[f].to_bits(),
                data.samples()[f].to_bits(),
                "frame {f} is outside the seam and must be untouched"
            );
        }
    }

    /// Both channels fade; a stereo take does not go mono at its seam.
    #[test]
    fn stereo_channels_both_crossfade() {
        // L=+ramp, R=−ramp so the two channels are distinct.
        let frames = 800;
        let mut v = Vec::with_capacity(frames * 2);
        for i in 0..frames {
            let t = i as f32 / frames as f32;
            v.push(t);
            v.push(-t);
        }
        let data = SampleData::from_interleaved(v, AudioFormat::stereo(48_000));
        let baked = bake_loop_crossfade(&data, 200..600, 64).unwrap();
        assert_eq!(baked.frame_count(), frames);
        assert_eq!(baked.format().channel_count(), 2);
        assert!(loop_seam_step(&baked, 200..600) < 0.05);
    }

    /// **A loop that starts at frame 0 has no pre-roll**, so there is nothing to fade from. Saying
    /// so beats quietly returning the clip unchanged and landing a do-nothing step on the undo
    /// timeline.
    #[test]
    fn a_loop_with_no_intro_cannot_be_baked() {
        let data = SampleData::from_interleaved(sine(2_000), AudioFormat::mono(48_000));
        assert!(
            bake_loop_crossfade(&data, 0..1_000, 256).is_none(),
            "no audio before the loop start = nothing to crossfade with"
        );
    }

    // The empty / inverted ranges are the whole point of this test — silence clippy's
    // `reversed_empty_ranges` for the deliberate degenerate literals.
    #[test]
    #[allow(clippy::reversed_empty_ranges)]
    fn empty_or_out_of_range_region_is_none() {
        let data = SampleData::from_interleaved(sine(1_000), AudioFormat::mono(48_000));
        assert!(bake_loop_crossfade(&data, 500..500, 16).is_none(), "empty");
        assert!(
            bake_loop_crossfade(&data, 800..600, 16).is_none(),
            "inverted"
        );
        assert!(
            bake_loop_crossfade(&data, 2_000..3_000, 16).is_none(),
            "past end"
        );
    }

    /// A zero-length crossfade is not a bake.
    #[test]
    fn a_zero_crossfade_is_not_a_bake() {
        let data = SampleData::from_interleaved(sine(2_000), AudioFormat::mono(48_000));
        assert!(bake_loop_crossfade(&data, 500..1_500, 0).is_none());
    }
}
