//! **De-Click / de-crackle** — the rack's one *repair* tool.
//!
//! A click is, by definition, the part of the waveform the signal itself cannot
//! explain: a vinyl tick, a digital dropout, a mouth click between words. So the
//! detector does not look for "loud" or "fast" — it looks for what the audio's own
//! model failed to predict, and the repair puts back what the model says *should*
//! have been there.
//!
//! Two halves, both classic:
//!
//! 1. **Detection** — fit an AR model per block ([`super::lpc`]), take the prediction
//!    residual, and flag the samples where it spikes past `k` robust deviations. The
//!    scale estimate is a **median** absolute deviation, not an RMS: a click is
//!    exactly the kind of outlier that inflates an RMS until it hides itself.
//! 2. **Repair** — replace the flagged run with the samples that **minimise the
//!    residual energy** given the audio on both sides of it: the least-squares AR
//!    (LSAR) interpolation of Janssen, Veldhuis & Vries (1986), "Adaptive
//!    interpolation of discrete-time signals that can be modeled as autoregressive
//!    processes" (IEEE ASSP-34). Minimising `Σ e[n]²` over the gap is a small linear
//!    system, solved exactly. A mute or a linear ramp would be audible; this is not,
//!    because it is what the signal would have done.
//!
//! Pipeline and notation follow Godsill & Rayner, *Digital Audio Restoration*
//! (Springer, 1998), ch. 5. Time-domain throughout — **no FFT, no dependency**.
//!
//! Control thread only (allocates freely).

use std::ops::Range;

use ph2d_audio::SampleData;

use super::lpc::{autocorrelation, levinson, residual};
use crate::ops::channels;

/// AR order. High enough to model a voice's formants plus a few harmonics, so what is
/// left in the residual really is excitation and damage rather than un-modelled tone.
const ORDER: usize = 32;

/// Analysis block: one AR model per this many frames (~43 ms at 48 kHz). Long enough
/// for a stable fit, short enough to track a moving spectrum. Doubles as the effect's
/// pre-roll, so the region's first block is modelled against real audio.
pub(super) const BLOCK: usize = 2_048;

/// `1 / Φ⁻¹(3/4)` — turns a median absolute deviation into a standard deviation for
/// Gaussian data. The textbook constant that makes `k` read as "sigmas".
const MAD_TO_SIGMA: f64 = 1.4826;

/// Detection threshold (in sigmas) at each end of the Sensitivity slider. Wide open
/// (`K_HOT`) will also catch the odd glottal pulse — that is what "too sensitive"
/// means, and why the neutral point is off rather than merely high.
const K_COLD: f64 = 24.0;
const K_HOT: f64 = 2.5;

/// Runs longer than the Width slider are **left alone**: past a certain length a
/// "click" is not damage, it is the signal (or the model is wrong). A repairer that
/// kept going would smear music into a plausible-sounding hallucination.
///
/// See [`WIDTH_MIN_S`] / [`WIDTH_MAX_S`] in the shell's spec table for the range the
/// user actually gets; this is the hard ceiling on the linear system's size.
const MAX_GAP: usize = 192;

/// A pivot smaller than this means the normal equations are singular for this gap —
/// the model has nothing to say about it, so the audio is left untouched.
const PIVOT_EPS: f64 = 1e-12;

/// Audio on each side of a burst that the transient guard listens to (~11 ms at 48 kHz):
/// long enough to fit a model and to hear whether a new sound has started, short enough
/// to stay inside one musical event.
const CONTEXT: usize = 512;

/// How much worse the *before* model may explain the *after* audio before the burst is
/// ruled a **signal onset** rather than damage. Measured on real material: a click's
/// audio resumes at 1.8–2.4× its own residual, a percussive attack starts a new sound at
/// 7.5–13.2×. Four is the ditch between them, with margin on both sides.
const ONSET_RATIO: f64 = 4.0;

/// Detect and repair clicks. `sensitivity` (0..1) sets the detection threshold;
/// `width_secs` is the longest run that will be repaired.
///
/// Called only off the neutral point (`sensitivity` above 0); the caller returns the
/// input untouched otherwise.
pub(super) fn declick(data: &SampleData, sensitivity: f32, width_secs: f32) -> SampleData {
    let ch = channels(data);
    let frames = data.frame_count();
    // Without context on both sides of a gap there is nothing to interpolate from.
    if frames <= ORDER * 2 || ch == 0 {
        return data.clone();
    }
    let sr = data.format().sample_rate as f64;
    let k = K_HOT + (K_COLD - K_HOT) * (1.0 - f64::from(sensitivity).clamp(0.0, 1.0));
    let max_gap = ((f64::from(width_secs) * sr) as usize).clamp(1, MAX_GAP);

    let src = data.samples();
    let mut out = src.to_vec();
    for c in 0..ch {
        let mut x: Vec<f64> = (0..frames).map(|f| f64::from(src[f * ch + c])).collect();
        let mut block = 0;
        while block < frames {
            let end = (block + BLOCK).min(frames);
            repair_block(&mut x, block..end, k, max_gap);
            block = end;
        }
        for (f, &v) in x.iter().enumerate() {
            // A failed repair must never be louder than the click it replaced.
            out[f * ch + c] = (v as f32).clamp(-1.0, 1.0);
        }
    }
    SampleData::from_interleaved(out, data.format())
}

/// Model one block, flag the outliers in its residual, and repair the short runs.
/// `x` is the whole channel — repairs are written in place, so a later block models
/// audio that has already been healed.
fn repair_block(x: &mut [f64], block: Range<usize>, k: f64, max_gap: usize) {
    let Some((a, _)) = levinson(&autocorrelation(&x[block.clone()], ORDER)) else {
        return; // silence: nothing to model, nothing to repair
    };
    // Residual over the block, computed with its real history (the leading `hist`
    // samples exist only so the first sample of the block is predicted from audio
    // rather than from assumed silence).
    let hist = block.start.saturating_sub(ORDER);
    let res = residual(&x[hist..block.end], &a);
    let lead = block.start - hist;
    let e = &res[lead..];

    let Some(sigma) = robust_scale(e) else {
        return; // a perfectly-predicted block has no outliers to find
    };
    let limit = k * sigma;
    for gap in bursts(e, limit, max_gap, block.start) {
        // The interpolator needs `ORDER` real samples on each side to lean on.
        if gap.start >= ORDER && gap.end + ORDER <= x.len() && signal_resumes_after(x, &gap) {
            lsar_interpolate(x, &a, gap);
        }
    }
}

/// **The transient guard.** Whether the audio *resumes* after `gap` — which is the whole
/// difference between damage and a sound that simply started.
///
/// A drum hit, a consonant, a note attack: to an AR model, the onset of any of them is
/// exactly as unpredictable as a click, and the residual **cannot tell them apart**
/// (measured on real material: clicks peak at 115–162 sigma, percussive attacks at
/// 81–152 — the same range, and the residual *after* the event is the same too, so
/// "the model stays wrong for longer" does not separate them either). A detector that
/// stopped at "outlier" would interpolate away every attack in the take. It is *the*
/// classic failure of this tool, and the reason a de-clicker that only reads the
/// residual is a smearer.
///
/// What separates them is not the spike — it is **what follows it**:
///
/// - a **click** is damage *inside* a signal. Take it out and the same signal is still
///   there on both sides, so a model fitted to the audio BEFORE it still explains the
///   audio AFTER it (measured 1.8–2.4×);
/// - an **onset** is where a *new* signal begins. The old model has never heard it, and
///   fails (measured 7.5–13.2×).
///
/// So: fit on the before, score the after, and repair only what the model says came back.
fn signal_resumes_after(x: &[f64], gap: &Range<usize>) -> bool {
    // Without full context on both sides there is nothing to compare. The caller has
    // already guaranteed the interpolator's own margins, so fall through and repair.
    let Some(pre) = gap.start.checked_sub(CONTEXT + ORDER) else {
        return true;
    };
    let post = gap.end + ORDER;
    if post + CONTEXT > x.len() {
        return true;
    }
    let Some((a, _)) = levinson(&autocorrelation(&x[pre..gap.start], ORDER)) else {
        return true; // silence before the burst: no model, nothing to contradict
    };
    // Residual RMS of that model over a window. The `after` window starts `ORDER` past
    // the burst so the damage itself never enters the prediction history.
    let rms = |lo: usize, hi: usize| -> f64 {
        let sum: f64 = (lo..hi)
            .map(|i| {
                let pred: f64 = a.iter().enumerate().map(|(k, &ak)| ak * x[i - k - 1]).sum();
                (x[i] - pred).powi(2)
            })
            .sum();
        (sum / (hi - lo) as f64).sqrt()
    };
    let base = rms(pre + ORDER, gap.start);
    if base <= 0.0 {
        return true;
    }
    rms(post, post + CONTEXT) / base <= ONSET_RATIO
}

/// Robust spread of the residual: `1.4826 × median(|e|)`. `None` when the median is
/// zero (a silent or perfectly-predicted block), where any threshold would flag
/// everything.
fn robust_scale(e: &[f64]) -> Option<f64> {
    let mut mags: Vec<f64> = e.iter().map(|v| v.abs()).collect();
    if mags.is_empty() {
        return None;
    }
    let mid = mags.len() / 2;
    mags.select_nth_unstable_by(mid, |a, b| a.total_cmp(b));
    let median = mags[mid];
    (median > 0.0).then_some(median * MAD_TO_SIGMA)
}

/// Group the samples whose residual exceeds `limit` into runs, in the block's own
/// coordinates, then hand them back offset by `base` (the channel's coordinates).
///
/// Two touches: single-sample holes inside a run are **bridged** (a click's waveform
/// crosses zero, so its residual does too), and each run is **grown by one sample**
/// on each side (the edges of a click sit just under the threshold — leave them in
/// and the repair splices onto damage). Runs longer than `max_gap` are dropped.
fn bursts(e: &[f64], limit: f64, max_gap: usize, base: usize) -> Vec<Range<usize>> {
    let hot = |i: usize| e.get(i).is_some_and(|v| v.abs() > limit);
    let mut out = Vec::new();
    let mut i = 0;
    while i < e.len() {
        if !hot(i) {
            i += 1;
            continue;
        }
        let start = i;
        let mut end = i + 1;
        while end < e.len() && (hot(end) || hot(end + 1)) {
            end += 1;
        }
        i = end;
        let start = start.saturating_sub(1);
        let end = (end + 1).min(e.len());
        if end - start <= max_gap {
            out.push(base + start..base + end);
        }
    }
    out
}

/// Replace `gap` with the samples that minimise the AR residual energy across it
/// (Janssen et al. 1986). The known audio on both sides pins the solution.
///
/// Writing the error filter as `b = [1, −a₀, −a₁, …]`, the residual energy is
/// quadratic in the unknowns, so setting its derivative to zero gives one linear
/// equation per missing sample:
///
/// ```text
///   Σ_{m' ∈ gap} R(m − m')·x[m']  =  − Σ_{j ∉ gap} R(m − j)·x[j]      ∀ m ∈ gap
/// ```
///
/// where `R` is the autocorrelation of `b`. Small (`gap ≤ MAX_GAP`) and solved
/// exactly by Gaussian elimination.
fn lsar_interpolate(x: &mut [f64], a: &[f64], gap: Range<usize>) {
    let p = a.len();
    let m = gap.len();
    if m == 0 {
        return;
    }
    // The error filter, then its own autocorrelation R(0..=p).
    let mut b = vec![0.0f64; p + 1];
    b[0] = 1.0;
    for (k, &ak) in a.iter().enumerate() {
        b[k + 1] = -ak;
    }
    let rr: Vec<f64> = (0..=p)
        .map(|d| (0..=(p - d)).map(|i| b[i] * b[i + d]).sum())
        .collect();
    let r = |d: isize| -> f64 {
        let d = d.unsigned_abs();
        if d <= p { rr[d] } else { 0.0 }
    };

    let mut mat = vec![0.0f64; m * m];
    let mut rhs = vec![0.0f64; m];
    for (i, mi) in gap.clone().enumerate() {
        for (j, mj) in gap.clone().enumerate() {
            mat[i * m + j] = r(mi as isize - mj as isize);
        }
        // Everything within `p` of the gap that is NOT missing pins this equation.
        let lo = mi.saturating_sub(p);
        let hi = (mi + p + 1).min(x.len());
        let known: f64 = (lo..hi)
            .filter(|j| !gap.contains(j))
            .map(|j| r(mi as isize - j as isize) * x[j])
            .sum();
        rhs[i] = -known;
    }
    if let Some(sol) = solve(&mut mat, &mut rhs, m) {
        for (i, mi) in gap.enumerate() {
            x[mi] = sol[i];
        }
    }
}

/// Gaussian elimination with partial pivoting. `None` when the system is singular —
/// the caller then leaves the audio alone rather than writing NaNs into it.
fn solve(mat: &mut [f64], rhs: &mut [f64], n: usize) -> Option<Vec<f64>> {
    for col in 0..n {
        let mut piv = col;
        for row in col + 1..n {
            if mat[row * n + col].abs() > mat[piv * n + col].abs() {
                piv = row;
            }
        }
        if mat[piv * n + col].abs() < PIVOT_EPS {
            return None;
        }
        if piv != col {
            for c in 0..n {
                mat.swap(col * n + c, piv * n + c);
            }
            rhs.swap(col, piv);
        }
        let diag = mat[col * n + col];
        for row in col + 1..n {
            let f = mat[row * n + col] / diag;
            if f == 0.0 {
                continue;
            }
            for c in col..n {
                mat[row * n + c] -= f * mat[col * n + c];
            }
            rhs[row] -= f * rhs[col];
        }
    }
    let mut sol = vec![0.0f64; n];
    for i in (0..n).rev() {
        let acc: f64 = rhs[i] - (i + 1..n).map(|c| mat[i * n + c] * sol[c]).sum::<f64>();
        sol[i] = acc / mat[i * n + i];
    }
    sol.iter().all(|v| v.is_finite()).then_some(sol)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    const SR: u32 = 48_000;

    /// A two-tone mono signal — predictable enough for an AR model, busy enough that
    /// a repair has to be right rather than merely quiet.
    fn clean(n: usize) -> Vec<f32> {
        let tau = std::f32::consts::TAU;
        (0..n)
            .map(|i| {
                let t = i as f32 / SR as f32;
                0.5 * (tau * 220.0 * t).sin() + 0.2 * (tau * 660.0 * t).sin()
            })
            .collect()
    }

    /// Stamp `width` samples of full-scale garbage at each of `at` — a click.
    fn with_clicks(mut x: Vec<f32>, at: &[usize], width: usize) -> Vec<f32> {
        for (c, &pos) in at.iter().enumerate() {
            for i in 0..width {
                let sign = if (c + i) % 2 == 0 { 1.0 } else { -1.0 };
                x[pos + i] = sign * 0.9;
            }
        }
        x
    }

    fn mono(x: Vec<f32>) -> SampleData {
        SampleData::from_interleaved(x, AudioFormat::mono(SR))
    }

    /// Peak error against the clean reference, over the neighbourhood of each click.
    fn peak_err(a: &[f32], b: &[f32], at: &[usize], width: usize) -> f32 {
        let mut worst: f32 = 0.0;
        for &pos in at {
            for i in pos.saturating_sub(4)..(pos + width + 4).min(a.len()) {
                worst = worst.max((a[i] - b[i]).abs());
            }
        }
        worst
    }

    /// THE contract: the repair puts back what the signal would have done. Damage a
    /// clean tone, run the de-clicker, and the audio must land back near the original
    /// — not merely be quieter. Red if detection misses the click, if the LSAR system
    /// is mis-assembled, or if the repair writes the wrong samples.
    #[test]
    fn a_click_is_repaired_back_toward_the_original_signal() {
        let at = [5_000usize, 9_003, 14_500];
        let width = 3;
        let reference = clean(24_000);
        let damaged = with_clicks(reference.clone(), &at, width);

        let before = peak_err(&reference, &damaged, &at, width);
        assert!(before > 0.5, "the test's own click is too gentle: {before}");

        let healed = declick(&mono(damaged), 0.8, 0.001);
        let after = peak_err(&reference, healed.samples(), &at, width);
        assert!(
            after < before * 0.2,
            "click still there: peak error {before} before, {after} after"
        );
    }

    /// ...and it must leave the audio it did not damage alone. A de-clicker that
    /// "repairs" the whole waveform is a smearer.
    #[test]
    fn the_undamaged_audio_survives_untouched() {
        let at = [5_000usize];
        let reference = clean(24_000);
        let damaged = with_clicks(reference.clone(), &at, 3);
        let healed = declick(&mono(damaged), 0.8, 0.001);
        // Well clear of the click and of the block edges it lives between.
        let far = &healed.samples()[16_000..20_000];
        let src = &reference[16_000..20_000];
        let worst = far
            .iter()
            .zip(src)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(worst < 1e-3, "untouched audio drifted by {worst}");
    }

    /// A percussive hit riding on the tone: a sharp attack that then decays away. Its
    /// onset is, to an AR model, exactly as unpredictable as a click.
    fn with_hit(mut x: Vec<f32>, at: usize) -> Vec<f32> {
        let tau = std::f32::consts::TAU;
        for i in 0..3_000 {
            let t = i as f32 / SR as f32;
            x[at + i] += 0.45 * (-t * 60.0).exp() * (tau * 1_800.0 * t).sin();
        }
        x
    }

    /// **The transient guard, pinned.** A de-clicker that only hunts residual outliers
    /// eats every attack in the take: the onset of a hit spikes the residual just as hard
    /// as a click does (measured: attacks at 81–152 sigma, clicks at 115–162 — the same
    /// range). This is THE classic failure of the tool, and it is what the Enio smoke
    /// caught (2026-07-12): the de-clicker dented all five percussive hits.
    ///
    /// Red without `signal_resumes_after`: the attack is interpolated away and moves by
    /// **0.26** — a quarter of full scale. The guard asks whether the audio *resumed*
    /// (a click) or whether something *new began* (a hit), and repairs only the former.
    #[test]
    fn a_percussive_attack_is_not_mistaken_for_a_click() {
        let at = 8_000usize;
        let reference = with_hit(clean(24_000), at);
        let out = declick(&mono(reference.clone()), 1.0, 0.001);

        // The attack — the first few ms, where all the energy is — must survive intact.
        let worst = out.samples()[at..at + 400]
            .iter()
            .zip(&reference[at..at + 400])
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(
            worst < 0.02,
            "the de-clicker ate the attack: it moved by {worst}"
        );
    }

    /// ...and the guard must not become an excuse to repair nothing: a click sitting in
    /// the SAME take, a few thousand samples from that hit, is still damage and still
    /// has to go.
    #[test]
    fn a_click_next_to_a_transient_is_still_repaired() {
        let (hit, click) = (8_000usize, 14_000usize);
        let reference = with_hit(clean(24_000), hit);
        let damaged = with_clicks(reference.clone(), &[click], 3);
        let before = peak_err(&reference, &damaged, &[click], 3);

        let healed = declick(&mono(damaged), 0.8, 0.001);
        let after = peak_err(&reference, healed.samples(), &[click], 3);
        assert!(
            after < before * 0.2,
            "the guard swallowed a real click: peak error {before} before, {after} after"
        );
    }

    /// A run longer than the Width slider is signal, not damage — leave it. Without
    /// this ceiling a wide-open de-clicker rewrites whole notes.
    #[test]
    fn a_run_longer_than_the_width_is_left_alone() {
        let at = [5_000usize];
        let width = 100; // ~2 ms at 48 kHz
        let damaged = with_clicks(clean(24_000), &at, width);
        // Width of 0.2 ms = ~10 frames: far too short to cover this run.
        let out = declick(&mono(damaged.clone()), 1.0, 0.0002);
        let inside = &out.samples()[at[0] + 20..at[0] + width - 20];
        let src = &damaged[at[0] + 20..at[0] + width - 20];
        assert_eq!(inside, src, "a long run must be left alone, not repaired");
    }

    #[test]
    fn declick_preserves_length_and_stays_bounded() {
        let d = mono(with_clicks(clean(12_000), &[3_000], 4));
        let out = declick(&d, 1.0, 0.002);
        assert_eq!(out.frame_count(), 12_000);
        assert!(out.samples().iter().all(|s| s.abs() <= 1.0));
    }

    /// Silence has no model — the block must be skipped, not divided by zero.
    #[test]
    fn silence_survives_the_repair() {
        let d = mono(vec![0.0; 8_000]);
        assert!(declick(&d, 1.0, 0.001).samples().iter().all(|&s| s == 0.0));
    }

    /// Stereo is repaired per channel: a click in the right must not dent the left.
    #[test]
    fn the_channels_are_repaired_independently() {
        let n = 12_000;
        let left = clean(n);
        let right = with_clicks(clean(n), &[6_000], 3);
        let inter: Vec<f32> = (0..n).flat_map(|i| [left[i], right[i]]).collect();
        let d = SampleData::from_interleaved(inter, AudioFormat::stereo(SR));
        let out = declick(&d, 0.8, 0.001);
        let out_left: Vec<f32> = out.samples().iter().step_by(2).copied().collect();
        let worst = out_left
            .iter()
            .zip(&left)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(worst < 1e-3, "the clean channel moved by {worst}");
    }
}
