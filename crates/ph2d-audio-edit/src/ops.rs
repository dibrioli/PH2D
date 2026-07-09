//! Offline, destructive edit operations on [`SampleData`] — each produces a
//! **fresh** buffer (the clip is an immutable `Arc<[f32]>`). Control-thread only,
//! so HR-3/HR-5 do not apply: these allocate and use `sqrt`/`cos`/`log` freely.
//!
//! Ranges are in **frames**, half-open `start..end`, and are clamped to the clip.

use std::ops::Range;

use ph2d_audio::{SampleData, dsp};

/// A fade/envelope shape (rising 0→1 across the range for a fade-IN; the fade-OUT
/// is the mirror). All are smooth and click-free at the endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FadeShape {
    /// Straight line.
    Linear,
    /// Slow start, fast end (`t²`).
    Exponential,
    /// Fast start, slow end (`√t`).
    Logarithmic,
    /// Raised-cosine S-curve (equal-power-ish, gentlest).
    SCurve,
}

/// Fade direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FadeDir {
    /// Silence → full.
    In,
    /// Full → silence.
    Out,
}

/// Rising gain `0→1` for `x∈[0,1]`.
fn curve(shape: FadeShape, x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    match shape {
        FadeShape::Linear => x,
        FadeShape::Exponential => x * x,
        FadeShape::Logarithmic => x.sqrt(),
        FadeShape::SCurve => 0.5 - 0.5 * (std::f32::consts::PI * x).cos(),
    }
}

fn channels(data: &SampleData) -> usize {
    data.format().channel_count().max(1)
}

/// Clamp a frame range to the clip.
fn clamp_range(data: &SampleData, range: Range<usize>) -> Range<usize> {
    let frames = data.frame_count();
    let start = range.start.min(frames);
    let end = range.end.clamp(start, frames);
    start..end
}

/// Apply a **length-preserving** whole-buffer op to just `range`, splicing the
/// result back into the clip. For selection-scoped gain / normalize / reverse /
/// invert / DC: the op runs on the extracted region as a standalone clip, so it
/// sees ONLY the selected samples (peak, LUFS, mean and frame-order are all
/// computed within the selection). Whole-clip range short-circuits to `op(data)`
/// so the no-selection path stays byte-identical to calling the op directly.
///
/// The op MUST return the same frame count it was given (gain/invert/normalize/
/// reverse/remove_dc all do); the splice relies on it.
pub fn in_range(
    data: &SampleData,
    range: Range<usize>,
    op: impl Fn(&SampleData) -> SampleData,
) -> SampleData {
    let r = clamp_range(data, range);
    if r.start == 0 && r.end == data.frame_count() {
        return op(data); // whole clip — no splice, no extra copy
    }
    if r.start >= r.end {
        return data.clone();
    }
    let region = trim(data, r.clone());
    let processed = op(&region);
    let ch = channels(data);
    let src = data.samples();
    let mut out = Vec::with_capacity(src.len());
    out.extend_from_slice(&src[..r.start * ch]);
    out.extend_from_slice(processed.samples());
    out.extend_from_slice(&src[r.end * ch..]);
    SampleData::from_interleaved(out, data.format())
}

/// Like [`in_range`], but first hands the op `warmup` frames of the audio
/// **immediately before** the range, then keeps only the range's output.
///
/// A stateful processor (an IIR filter) that starts on the first sample of a
/// mid-clip selection begins with cleared memory — as if the audio before the
/// selection had been silence. Its output collapses toward zero for the first few
/// dozen samples while the untouched neighbour sits at full level, so the splice
/// lands a full-scale step: an audible **click at the selection's leading edge**.
/// Pre-rolling the real preceding audio lets the filter enter the region already
/// settled, and the transient is discarded with the pre-roll.
///
/// `warmup` is clamped to the audio that actually exists before the range, so a
/// selection at frame 0 (and the whole-clip case) is byte-identical to
/// [`in_range`] — there is nothing to warm up on, and nothing to click against.
pub fn in_range_warm(
    data: &SampleData,
    range: Range<usize>,
    warmup: usize,
    op: impl Fn(&SampleData) -> SampleData,
) -> SampleData {
    let r = clamp_range(data, range);
    let warm = warmup.min(r.start);
    if warm == 0 || r.start >= r.end {
        return in_range(data, r, op);
    }
    let ch = channels(data);
    let region_len = r.end - r.start;
    let extended = trim(data, r.start - warm..r.end);
    let processed = op(&extended);
    // The op must be length-preserving; if it wasn't, fall back rather than slice
    // garbage out of it.
    if processed.frame_count() != warm + region_len {
        return in_range(data, r, op);
    }
    let src = data.samples();
    let mut out = Vec::with_capacity(src.len());
    out.extend_from_slice(&src[..r.start * ch]);
    out.extend_from_slice(&processed.samples()[warm * ch..]);
    out.extend_from_slice(&src[r.end * ch..]);
    SampleData::from_interleaved(out, data.format())
}

/// Apply a **tail-extending** op to `range`, ringing the tail out over whatever
/// follows. The sibling of [`in_range`] for effects (reverb / delay) whose output
/// outlives their input, so the length-preserving splice doesn't fit.
///
/// `op(region, tail_frames)` must return `region_len + tail_frames` frames: the
/// processed region (dry/wet already mixed) followed by the ring-out. Then:
/// - the range is **replaced** by the processed region;
/// - the tail is **added** on top of the frames after the range — the original
///   audio when the range sits mid-clip, or fresh silence past the old end, which
///   is where the **clip grows** (`out_frames = max(orig, range.end + tail)`).
///
/// This is the DAW behaviour: a reverb on a mid-clip selection bleeds into the
/// audio that follows; on a selection touching the end it lengthens the clip.
pub fn in_range_tail(
    data: &SampleData,
    range: Range<usize>,
    tail_frames: usize,
    op: impl Fn(&SampleData, usize) -> SampleData,
) -> SampleData {
    let r = clamp_range(data, range);
    if r.start >= r.end {
        return data.clone();
    }
    let ch = channels(data);
    let region_len = r.end - r.start;
    let region = trim(data, r.clone());
    let processed = op(&region, tail_frames);
    // Trust but verify: never index past what `op` actually produced.
    let tail = processed
        .frame_count()
        .saturating_sub(region_len)
        .min(tail_frames);

    let src = data.samples();
    let out_frames = data.frame_count().max(r.end + tail);
    let mut out = vec![0.0f32; out_frames * ch];
    out[..src.len()].copy_from_slice(src);

    let p = processed.samples();
    out[r.start * ch..r.end * ch].copy_from_slice(&p[..region_len * ch]);
    for i in 0..tail * ch {
        let dst = r.end * ch + i;
        out[dst] = (out[dst] + p[region_len * ch + i]).clamp(-1.0, 1.0);
    }
    SampleData::from_interleaved(out, data.format())
}

/// Scale every sample by `linear` gain.
pub fn gain(data: &SampleData, linear: f32) -> SampleData {
    let out: Vec<f32> = data.samples().iter().map(|s| s * linear).collect();
    SampleData::from_interleaved(out, data.format())
}

/// Negate every sample (polarity invert).
pub fn invert(data: &SampleData) -> SampleData {
    gain(data, -1.0)
}

/// Peak of `|sample|` across the clip (`0.0` for silence/empty).
pub fn peak(data: &SampleData) -> f32 {
    data.samples().iter().fold(0.0f32, |m, s| m.max(s.abs()))
}

/// Scale so the loudest sample hits `target_peak` (linear, e.g. `1.0` = 0 dBFS).
/// No-op on silence.
pub fn normalize_peak(data: &SampleData, target_peak: f32) -> SampleData {
    let p = peak(data);
    if p <= f32::EPSILON {
        return data.clone();
    }
    gain(data, target_peak / p)
}

/// Scale so the clip's integrated loudness hits `target_lufs` (BS.1770
/// K-weighting; ungated approximation). No-op on effective silence.
pub fn normalize_lufs(data: &SampleData, target_lufs: f32) -> SampleData {
    let cur = dsp::integrated_lufs(data.samples(), channels(data), data.format().sample_rate);
    if cur <= dsp::SILENCE_LUFS {
        return data.clone();
    }
    let gain_db = target_lufs - cur;
    gain(data, 10.0f32.powf(gain_db / 20.0))
}

/// Reverse the clip in time (frame order), keeping channel interleave intact.
pub fn reverse(data: &SampleData) -> SampleData {
    let ch = channels(data);
    let frames = data.frame_count();
    let src = data.samples();
    let mut out = vec![0.0f32; src.len()];
    for f in 0..frames {
        let s = (frames - 1 - f) * ch;
        let d = f * ch;
        out[d..d + ch].copy_from_slice(&src[s..s + ch]);
    }
    SampleData::from_interleaved(out, data.format())
}

/// Remove any DC offset by subtracting each channel's mean.
pub fn remove_dc_offset(data: &SampleData) -> SampleData {
    let ch = channels(data);
    let frames = data.frame_count();
    if frames == 0 {
        return data.clone();
    }
    let src = data.samples();
    let mut mean = vec![0.0f64; ch];
    for f in 0..frames {
        for (c, m) in mean.iter_mut().enumerate() {
            *m += src[f * ch + c] as f64;
        }
    }
    for m in &mut mean {
        *m /= frames as f64;
    }
    let mut out = vec![0.0f32; src.len()];
    for f in 0..frames {
        for c in 0..ch {
            out[f * ch + c] = src[f * ch + c] - mean[c] as f32;
        }
    }
    SampleData::from_interleaved(out, data.format())
}

/// Keep only `range`, discarding the rest (crop).
pub fn trim(data: &SampleData, range: Range<usize>) -> SampleData {
    let r = clamp_range(data, range);
    let ch = channels(data);
    let out = data.samples()[r.start * ch..r.end * ch].to_vec();
    SampleData::from_interleaved(out, data.format())
}

/// Zero the samples in `range` (replace with silence, same length).
pub fn silence(data: &SampleData, range: Range<usize>) -> SampleData {
    let r = clamp_range(data, range);
    let ch = channels(data);
    let mut out = data.samples().to_vec();
    for s in &mut out[r.start * ch..r.end * ch] {
        *s = 0.0;
    }
    SampleData::from_interleaved(out, data.format())
}

/// Delete `range`, closing the gap (ripple — the clip gets shorter).
pub fn delete(data: &SampleData, range: Range<usize>) -> SampleData {
    let r = clamp_range(data, range);
    let ch = channels(data);
    let src = data.samples();
    let mut out = Vec::with_capacity(src.len() - (r.end - r.start) * ch);
    out.extend_from_slice(&src[..r.start * ch]);
    out.extend_from_slice(&src[r.end * ch..]);
    SampleData::from_interleaved(out, data.format())
}

/// Apply a fade envelope over `range` (all channels). `In` ramps silence→full
/// across the range; `Out` ramps full→silence.
pub fn fade(data: &SampleData, range: Range<usize>, shape: FadeShape, dir: FadeDir) -> SampleData {
    let r = clamp_range(data, range);
    let ch = channels(data);
    let span = r.end - r.start;
    let mut out = data.samples().to_vec();
    if span == 0 {
        return SampleData::from_interleaved(out, data.format());
    }
    for (i, f) in (r.start..r.end).enumerate() {
        let t = i as f32 / span as f32;
        let g = match dir {
            FadeDir::In => curve(shape, t),
            FadeDir::Out => curve(shape, 1.0 - t),
        };
        for c in 0..ch {
            out[f * ch + c] *= g;
        }
    }
    SampleData::from_interleaved(out, data.format())
}

/// Nearest frame to `frame` where channel 0 crosses zero (sign change), searched
/// within `window` frames each way. Falls back to `frame` (clamped) if none — for
/// click-free trim/loop boundaries.
pub fn snap_to_zero_crossing(data: &SampleData, frame: usize, window: usize) -> usize {
    let ch = channels(data);
    let frames = data.frame_count();
    if frames == 0 {
        return 0;
    }
    let frame = frame.min(frames - 1);
    let sample = |f: usize| data.samples()[f * ch];
    let is_crossing = |f: usize| f > 0 && (sample(f - 1) <= 0.0) != (sample(f) <= 0.0);
    for d in 0..=window {
        if frame >= d && is_crossing(frame - d) {
            return frame - d;
        }
        if frame + d < frames && is_crossing(frame + d) {
            return frame + d;
        }
    }
    frame
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    fn stereo(v: Vec<f32>) -> SampleData {
        SampleData::from_interleaved(v, AudioFormat::stereo(48_000))
    }

    #[test]
    fn gain_and_invert() {
        let d = stereo(vec![0.2, -0.4, 0.6, -0.8]);
        assert_eq!(gain(&d, 0.5).samples(), &[0.1, -0.2, 0.3, -0.4]);
        assert_eq!(invert(&d).samples(), &[-0.2, 0.4, -0.6, 0.8]);
    }

    #[test]
    fn normalize_peak_hits_target() {
        let d = stereo(vec![0.25, -0.5, 0.1, 0.0]);
        let n = normalize_peak(&d, 1.0);
        assert!((peak(&n) - 1.0).abs() < 1e-6);
        // Ratios preserved (0.25/0.5 = 0.5).
        assert!((n.samples()[0] / n.samples()[1] - (-0.5)).abs() < 1e-6);
        // Silence is a no-op.
        let s = stereo(vec![0.0; 4]);
        assert_eq!(normalize_peak(&s, 1.0).samples(), &[0.0; 4]);
    }

    #[test]
    fn normalize_lufs_moves_toward_target() {
        // A quiet 1 kHz tone normalized up should read louder (closer to target).
        let step = std::f32::consts::TAU * 1_000.0 / 48_000.0;
        let v: Vec<f32> = (0..48_000)
            .flat_map(|i| {
                let s = (i as f32 * step).sin() * 0.05;
                [s, s]
            })
            .collect();
        let d = stereo(v);
        let before = dsp::integrated_lufs(d.samples(), 2, 48_000);
        let n = normalize_lufs(&d, -16.0);
        let after = dsp::integrated_lufs(n.samples(), 2, 48_000);
        assert!(after > before, "normalize up must raise LUFS");
        assert!(
            (after - (-16.0)).abs() < 1.0,
            "should land near -16 LUFS, got {after}"
        );
    }

    #[test]
    fn reverse_flips_frame_order() {
        let d = stereo(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]); // frames (1,2)(3,4)(5,6)
        assert_eq!(reverse(&d).samples(), &[5.0, 6.0, 3.0, 4.0, 1.0, 2.0]);
    }

    #[test]
    fn dc_offset_removed() {
        // Each channel offset by +0.3 / -0.1.
        let d = stereo(vec![0.3, -0.1, 0.3, -0.1, 0.3, -0.1]);
        let r = remove_dc_offset(&d);
        assert!(r.samples().iter().all(|s| s.abs() < 1e-6));
    }

    #[test]
    fn trim_silence_delete() {
        let d = stereo(vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0]); // 4 frames
        assert_eq!(trim(&d, 1..3).samples(), &[2.0, 2.0, 3.0, 3.0]);
        assert_eq!(
            silence(&d, 1..3).samples(),
            &[1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 4.0, 4.0]
        );
        assert_eq!(delete(&d, 1..3).samples(), &[1.0, 1.0, 4.0, 4.0]);
    }

    #[test]
    fn fade_in_endpoints() {
        let d = stereo(vec![1.0; 20]); // 10 frames
        let f = fade(&d, 0..10, FadeShape::Linear, FadeDir::In);
        // First frame silent, later frames rising.
        assert!(f.samples()[0].abs() < 1e-6);
        assert!(f.samples()[18] > f.samples()[2]);
        // Fade out mirrors: first frame full, last near silent.
        let o = fade(&d, 0..10, FadeShape::Linear, FadeDir::Out);
        assert!((o.samples()[0] - 1.0).abs() < 1e-6);
        assert!(o.samples()[18] < o.samples()[2]);
    }

    #[test]
    fn in_range_scopes_op_to_selection() {
        // 4 stereo frames; gain ×0 only on the middle two → the ends survive.
        let d = stereo(vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0]);
        let g = in_range(&d, 1..3, |x| gain(x, 0.0));
        assert_eq!(g.samples(), &[1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 4.0, 4.0]);
        // Whole-clip range is byte-identical to calling the op directly.
        let whole = in_range(&d, 0..4, |x| gain(x, 0.5));
        assert_eq!(whole.samples(), gain(&d, 0.5).samples());
        // Reverse in range flips only the selected frames' order.
        let r = in_range(&d, 1..4, reverse);
        assert_eq!(r.samples(), &[1.0, 1.0, 4.0, 4.0, 3.0, 3.0, 2.0, 2.0]);
        // Empty range is a no-op.
        assert_eq!(in_range(&d, 2..2, invert).samples(), d.samples());
    }

    /// A stand-in tail op: passes the region through, then emits `tail` frames of
    /// a constant 0.25 ring-out.
    fn ring(region: &SampleData, tail: usize) -> SampleData {
        let ch = region.format().channel_count().max(1);
        let mut v = region.samples().to_vec();
        v.extend(std::iter::repeat_n(0.25, tail * ch));
        SampleData::from_interleaved(v, region.format())
    }

    #[test]
    fn in_range_tail_grows_the_clip_at_the_end() {
        // Whole clip (4 frames) + 2 frames of tail → clip grows to 6.
        let d = stereo(vec![0.0; 8]);
        let out = in_range_tail(&d, 0..4, 2, ring);
        assert_eq!(out.frame_count(), 6);
        assert_eq!(
            &out.samples()[8..],
            &[0.25; 4],
            "tail appended past the end"
        );
    }

    #[test]
    fn in_range_tail_bleeds_into_following_audio_without_growing() {
        // 4 frames of 0.5; process frame 1 only, 2 frames of tail. The tail lands
        // on frames 2..4 (which exist), so the clip length is unchanged and the
        // tail is ADDED to the audio there (0.5 + 0.25).
        let d = stereo(vec![0.5; 8]);
        let out = in_range_tail(&d, 1..2, 2, ring);
        assert_eq!(
            out.frame_count(),
            4,
            "tail fits inside the clip → no growth"
        );
        assert_eq!(
            &out.samples()[0..2],
            &[0.5, 0.5],
            "before the range: untouched"
        );
        assert_eq!(
            &out.samples()[2..4],
            &[0.5, 0.5],
            "range: replaced by the op"
        );
        assert_eq!(
            &out.samples()[4..8],
            &[0.75; 4],
            "tail added onto what follows"
        );
    }

    #[test]
    fn in_range_tail_clamps_and_handles_degenerate_input() {
        // Tail summing past full scale is clamped, not wrapped.
        let loud = stereo(vec![0.9; 8]);
        let out = in_range_tail(&loud, 0..1, 2, ring);
        assert!(out.samples().iter().all(|x| x.abs() <= 1.0 + 1e-6));
        // Empty range = no-op; an op that under-delivers its tail must not panic.
        assert_eq!(
            in_range_tail(&loud, 2..2, 4, ring).samples(),
            loud.samples()
        );
        let short = in_range_tail(&loud, 0..4, 3, |r, _| r.clone());
        assert_eq!(short.frame_count(), 4, "no tail produced → no growth");
    }

    #[test]
    fn zero_crossing_snaps() {
        // Mono so frame index == sample index. Sign change between frame 2 (+0.5)
        // and frame 3 (-0.5) → the crossing is at frame 3.
        let d = SampleData::from_interleaved(
            vec![0.5, 0.5, 0.5, -0.5, -0.5, -0.5],
            AudioFormat::mono(48_000),
        );
        assert_eq!(snap_to_zero_crossing(&d, 3, 4), 3);
        assert_eq!(snap_to_zero_crossing(&d, 2, 4), 3); // nearest crossing is at 3
    }
}
