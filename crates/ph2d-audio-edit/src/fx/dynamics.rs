//! Dynamics processors for the rack: the feed-forward compressor, the downward
//! **expander / gate**, and the look-ahead **true-peak limiter**.
//!
//! Control thread only — allocates and uses `exp`/`powf` freely.

use ph2d_audio::SampleData;
use ph2d_audio::dsp::Compressor;

use crate::ops::channels;
use crate::truepeak::true_peak_envelope;

/// One-shot / per-sample coefficient for a `secs` time-constant at `sr` Hz.
pub(super) fn time_coeff(secs: f32, sr: f32) -> f32 {
    if secs <= 0.0 {
        return 1.0;
    }
    1.0 - (-1.0 / (secs * sr)).exp()
}

/// Leveler gain clamp (±12 dB) — a slow AGC evens a voice out; it must not swing more
/// than this or amplify near-silence into audible noise.
const LEVELER_MAX_GAIN: f32 = 4.0; // +12 dB
const LEVELER_MIN_GAIN: f32 = 0.25; // −12 dB
/// RMS floor so a silent stretch's `target / rms` doesn't blow up before the clamp.
const LEVELER_FLOOR: f32 = 1e-4;

/// **Leveler / AGC**: slow automatic gain toward a target RMS, evening out a voice's
/// loudness over time. A one-pole mean-square envelope (`speed_secs` time constant)
/// drives `gain = target / rms`, clamped to ±12 dB; `amount` (0..1) crossfades that
/// gain toward unity (0 = bypass — the neutral point). The envelope is **primed at the
/// target**, so the gain starts at 1.0 and eases in — no startup jump (the same idea as
/// the compressor's `prime`). Length-preserving; control thread, so `powf`/`exp` are
/// free.
pub(super) fn leveler(
    data: &SampleData,
    target_db: f32,
    amount: f32,
    speed_secs: f32,
) -> SampleData {
    let sr = data.format().sample_rate as f32;
    let ch = channels(data);
    let target = 10f32.powf(target_db / 20.0);
    let coeff = time_coeff(speed_secs, sr); // one-pole step toward the new level
    let amount = amount.clamp(0.0, 1.0);
    let mut ms = target * target; // prime at target → first gain ≈ 1.0
    let frames = data.frame_count();
    SampleData::map_in_place(data, |out| {
        for f in 0..frames {
            // Mean square across channels for a single, phase-coherent gain.
            let mut frame_ms = 0.0;
            for c in 0..ch {
                let x = out[f * ch + c];
                frame_ms += x * x;
            }
            frame_ms /= ch as f32;
            ms += coeff * (frame_ms - ms);
            let rms = ms.sqrt().max(LEVELER_FLOOR);
            let gain = (target / rms).clamp(LEVELER_MIN_GAIN, LEVELER_MAX_GAIN);
            let g = 1.0 + amount * (gain - 1.0);
            for c in 0..ch {
                let i = f * ch + c;
                out[i] = (out[i] * g).clamp(-1.0, 1.0);
            }
        }
    })
}

/// The interleaved peak of frame `f` across every channel. Dynamics are
/// **stereo-linked** — a per-channel gain swings the image on every transient.
pub(super) fn frame_peak(src: &[f32], ch: usize, f: usize) -> f32 {
    (0..ch).fold(0.0f32, |m, c| m.max(src[f * ch + c].abs()))
}

/// Ceiling on the automatic make-up, so a region that is almost entirely above the
/// threshold (or a near-silent one) can't be blown up. Shared with the multiband, which
/// trims its summed bands on the same terms.
pub(super) const COMPRESS_MAX_MAKEUP: f32 = 8.0;

/// Stereo-linked compression, then **peak-preserving** make-up.
///
/// The gain computer only ever attenuates, so the compressed peak is ≤ the input
/// peak; scaling by their quotient puts the peak exactly back where it was. Raising
/// `ratio` therefore lifts the quiet parts (RMS up, dynamic range down) and leaves
/// the waveform's amplitude alone.
///
/// The obvious alternative — the textbook `(1/threshold)^(1 - 1/ratio)` — is the
/// gain that restores a **full-scale** signal, and silently amplifies anything
/// quieter: at `threshold` 0.3 a peak-0.5 signal came out at 0.86 with `ratio` 4
/// and 0.97 with `ratio` 20. Raising the ratio made it LOUDER, which is backwards.
pub(super) fn compress(
    data: &SampleData,
    threshold: f32,
    ratio: f32,
    attack_secs: f32,
    release_secs: f32,
) -> SampleData {
    let sr = data.format().sample_rate as f32;
    let ch = channels(data);
    let frames = data.frame_count();
    let mut comp = Compressor::default();
    comp.set_params(
        true,
        threshold,
        ratio,
        time_coeff(attack_secs, sr),
        time_coeff(release_secs, sr),
    );
    // The make-up reads the PRISTINE peak of `data`, which `map_in_place` leaves
    // untouched (it rewrites its own copy) — so it stays the input's peak, as before.
    let peak_in = crate::peak(data);
    SampleData::map_in_place(data, |out| {
        // Offline: settle the envelope on the first frame. Otherwise the very first
        // sample escapes uncompressed, which both clicks at a selection edge and makes
        // `peak_out == peak_in` (so the make-up below would have nothing to give back)
        // for any region that starts at its loudest.
        if frames > 0 {
            let (l, r) = if ch >= 2 {
                (out[0], out[1])
            } else {
                (out[0], out[0])
            };
            comp.prime(l, r);
        }
        for f in 0..frames {
            let base = f * ch;
            if ch >= 2 {
                let (l, r) = comp.process(out[base], out[base + 1]);
                out[base] = l;
                out[base + 1] = r;
            } else {
                let (l, _) = comp.process(out[base], out[base]);
                out[base] = l;
            }
        }

        let peak_out = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        if peak_out > f32::EPSILON && peak_in > f32::EPSILON {
            let makeup = (peak_in / peak_out).clamp(1.0, COMPRESS_MAX_MAKEUP);
            for s in out.iter_mut() {
                *s = (*s * makeup).clamp(-1.0, 1.0);
            }
        }
    })
}

/// The quietest a gated passage can get: −80 dB. A true zero would make a gated
/// room tone snap to digital silence, which reads as a dropout, not as quiet.
const GATE_FLOOR: f32 = 1e-4;

/// A downward **expander**, which at a high ratio becomes a noise **gate**.
///
/// Above `threshold` the signal passes untouched. Below it, the gain follows
/// `(level / threshold)^(ratio − 1)`: at `ratio` 1 that is `1.0` for every level (a
/// no-op, hence the neutral point), at 2 every dB below the threshold becomes two,
/// and by 20 the floor has effectively been cut away. One knob spans "gentle
/// expander" to "hard gate" without a mode switch.
///
/// `attack_secs` is how fast the gate **opens** (gain rising) and `release_secs` how
/// slowly it **closes** — the opposite assignment to a compressor, where attack is
/// the clamp-down. A gate that closes as fast as it opens chatters on every
/// zero-crossing of a quiet passage.
///
/// Stereo-linked, and primed on the first frame so a region that starts loud does
/// not fade in from the gate's initial state (the same startup transient the
/// compressor's `prime()` exists to kill).
pub(super) fn gate(
    data: &SampleData,
    threshold: f32,
    ratio: f32,
    attack_secs: f32,
    release_secs: f32,
) -> SampleData {
    let frames = data.frame_count();
    if frames == 0 {
        return data.clone();
    }
    let sr = data.format().sample_rate as f32;
    let ch = channels(data);
    let open = time_coeff(attack_secs, sr);
    let close = time_coeff(release_secs, sr);
    let threshold = threshold.max(1e-6);
    let exponent = ratio - 1.0;

    let target_for = |level: f32| {
        if level >= threshold {
            1.0
        } else {
            (level / threshold).powf(exponent).max(GATE_FLOOR)
        }
    };

    // The detector reads frame `f` before the gain is applied to frame `f`, and never
    // looks at any other frame — so reading it out of the buffer being written reads
    // exactly the pristine input, as the `src` copy did.
    SampleData::map_in_place(data, |out| {
        let mut gain = target_for(frame_peak(out, ch, 0));
        for f in 0..frames {
            let target = target_for(frame_peak(out, ch, f));
            let coeff = if target > gain { open } else { close };
            gain += (target - gain) * coeff;
            for c in 0..ch {
                out[f * ch + c] *= gain;
            }
        }
    })
}

/// A look-ahead **true-peak** brickwall limiter.
///
/// A sample-domain limiter guarantees `max|y[n]| ≤ ceiling` and still lets the
/// *reconstructed* waveform overshoot by up to 3 dB between samples. This one builds
/// its gain curve from [`crate::truepeak`]'s oversampled envelope, so the ceiling
/// holds for the analogue signal a DAC or a lossy decoder will produce.
///
/// ## Why it cannot overshoot
///
/// Let `g[n] = min(1, ceiling / tp[n])` be the gain each frame *needs*. Take a
/// sliding **minimum** over `±R`, then smooth with two box averages whose combined
/// support is also `±R`:
///
/// - after the minimum, `g_min[n+k] ≤ g[n]` for every `|k| ≤ R`, because `n` lies
///   inside the window that produced `g_min[n+k]`;
/// - the smoother is a weighted average over exactly that `|k| ≤ R`, with weights
///   summing to 1.
///
/// So `g_smooth[n] ≤ g[n]` **everywhere**, and `g_smooth[n]·tp[n] ≤ ceiling`. The
/// minimum is what makes the gain dip *before* the peak arrives (the look-ahead);
/// the smoothing is what keeps that dip from being a click. Neither can undo the
/// other — that is the whole point of pairing them at the same radius.
///
/// Gain is stereo-linked: a per-channel gain would swing the stereo image on every
/// transient. It never exceeds 1, so the limiter can only ever turn things down.
pub(super) fn limit(data: &SampleData, ceiling_db: f32, release_secs: f32) -> SampleData {
    let frames = data.frame_count();
    if frames == 0 {
        return data.clone();
    }
    let sr = data.format().sample_rate as f32;
    let ch = channels(data);
    let ceiling = 10f32.powf(ceiling_db / 20.0);

    let tp = true_peak_envelope(data);
    let need: Vec<f32> = tp
        .iter()
        .map(|&p| if p > ceiling { ceiling / p } else { 1.0 })
        .collect();

    // Look-ahead / recovery radius. At least one frame, and never more than the clip
    // (a radius past the ends just flattens the whole gain curve, which is correct).
    let radius = ((release_secs.max(0.0) * sr) as usize).clamp(1, frames);
    let dipped = sliding_min(&need, radius);
    // Two boxes of radius r1 + r2 = radius: support stays inside ±radius, so the
    // proof above holds. (A single box would ramp linearly; two give a smooth knee.)
    let r1 = radius / 2;
    let smoothed = box_average(&box_average(&dipped, r1), radius - r1);

    SampleData::map_in_place(data, |out| {
        for (f, &g) in smoothed.iter().enumerate() {
            for c in 0..ch {
                let i = f * ch + c;
                out[i] = (out[i] * g).clamp(-1.0, 1.0);
            }
        }
    })
}

/// Sliding minimum over `[n-radius, n+radius]`, clamped at the ends. `O(n)` via a
/// monotonic deque — the naive nested loop is `O(n·radius)` and a 50 ms window on a
/// minute of 48 kHz audio is 2400 taps × 2.9 M frames.
fn sliding_min(v: &[f32], radius: usize) -> Vec<f32> {
    let n = v.len();
    let mut out = vec![0.0f32; n];
    // Indices of candidates, values strictly increasing front→back.
    let mut deque: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
    let mut next = 0usize; // first index not yet pushed
    for (i, slot) in out.iter_mut().enumerate() {
        let hi = (i + radius).min(n - 1);
        while next <= hi {
            while deque.back().is_some_and(|&b| v[b] >= v[next]) {
                deque.pop_back();
            }
            deque.push_back(next);
            next += 1;
        }
        let lo = i.saturating_sub(radius);
        while deque.front().is_some_and(|&f| f < lo) {
            deque.pop_front();
        }
        *slot = v[*deque.front().expect("window is never empty")];
    }
    out
}

/// Box average over `[n-radius, n+radius]`, clamped at the ends. `O(n)` via a prefix
/// sum in `f64` — an `f32` running sum drifts over millions of frames.
fn box_average(v: &[f32], radius: usize) -> Vec<f32> {
    let n = v.len();
    if radius == 0 || n == 0 {
        return v.to_vec();
    }
    let mut prefix = Vec::with_capacity(n + 1);
    prefix.push(0.0f64);
    for &x in v {
        prefix.push(prefix[prefix.len() - 1] + x as f64);
    }
    (0..n)
        .map(|i| {
            let lo = i.saturating_sub(radius);
            let hi = (i + radius + 1).min(n);
            ((prefix[hi] - prefix[lo]) / (hi - lo) as f64) as f32
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::truepeak::true_peak;
    use ph2d_audio::AudioFormat;

    fn stereo(v: Vec<f32>) -> SampleData {
        SampleData::from_interleaved(v, AudioFormat::stereo(48_000))
    }

    fn mono(v: Vec<f32>) -> SampleData {
        SampleData::from_interleaved(v, AudioFormat::mono(48_000))
    }

    /// A compressor must SHRINK dynamic range, never grow the waveform. Raising
    /// `ratio` may only raise the quiet parts (RMS up); the peak stays put.
    ///
    /// Regression: the old textbook auto-makeup `(1/threshold)^(1-1/ratio)` assumed
    /// a full-scale input, so on a peak-0.5 signal `ratio` 4 came out at 0.86 and
    /// `ratio` 20 at 0.97 — turning the ratio up made it louder.
    #[test]
    fn higher_ratio_lifts_rms_without_raising_the_peak() {
        // A step: a loud half (0.5, above the 0.3 threshold) then a quiet half
        // (0.1, below it), sign-alternating so |level| is constant within each.
        // This isolates the gain computer from the sine's per-cycle level swing.
        let n = 4_800;
        let samples: Vec<f32> = (0..n)
            .flat_map(|i| {
                let amp = if i < n / 2 { 0.5 } else { 0.1 };
                let s = if i % 2 == 0 { amp } else { -amp };
                [s, s]
            })
            .collect();
        let d = stereo(samples);
        let peak_in = crate::peak(&d);
        let rms = |x: &SampleData| {
            (x.samples().iter().map(|s| s * s).sum::<f32>() / x.samples().len() as f32).sqrt()
        };

        let at = |ratio: f32, attack_secs: f32| compress(&d, 0.3, ratio, attack_secs, 0.005);

        // THE guarantee: whatever the ratio or attack, the waveform never grows.
        for ratio in [2.0, 4.0, 20.0] {
            for attack in [0.0001, 0.005, 0.05] {
                let out = at(ratio, attack);
                assert!(
                    crate::peak(&out) <= peak_in + 1e-4,
                    "ratio {ratio} / attack {attack}s raised the peak: {} > {peak_in}",
                    crate::peak(&out)
                );
            }
        }

        // With an attack fast enough to actually catch the peak, the make-up hands
        // the level back and a heavier ratio lifts the quiet parts (RMS up).
        let fast = 0.0001;
        assert!(
            rms(&at(20.0, fast)) > rms(&at(2.0, fast)),
            "a heavier ratio must lift the RMS: {} !> {}",
            rms(&at(20.0, fast)),
            rms(&at(2.0, fast))
        );
        assert!(rms(&at(2.0, fast)) > rms(&d), "2:1 already lifts the RMS");
    }

    /// A loud half then a quiet half, sign-alternating so `|level|` is constant
    /// within each — the gate's job in one signal.
    fn loud_then_quiet(loud: f32, quiet: f32) -> SampleData {
        let n = 9_600;
        mono(
            (0..n)
                .map(|i| {
                    let amp = if i < n / 2 { loud } else { quiet };
                    if i % 2 == 0 { amp } else { -amp }
                })
                .collect(),
        )
    }

    /// The gate must pass what is above the threshold and pull down what is below —
    /// and the ratio must be the knob that says how hard.
    #[test]
    fn the_gate_passes_above_the_threshold_and_expands_below_it() {
        let d = loud_then_quiet(0.5, 0.02);
        let rms = |x: &SampleData, range: std::ops::Range<usize>| {
            let s = &x.samples()[range];
            (s.iter().map(|v| v * v).sum::<f32>() / s.len() as f32).sqrt()
        };
        // Sample the steady state of each half, clear of the attack/release ramps.
        // The quiet window starts 5 release time-constants (5 × 10 ms = 2 400 frames)
        // after the transition at 4 800 — a 50 ms release would still be closing all
        // the way to the end of the clip.
        let loud = 1_000..4_000;
        let quiet = 7_500..9_500;
        let release = 0.01;

        let out = gate(&d, 0.1, 8.0, 0.002, release);
        assert!(
            (rms(&out, loud.clone()) - rms(&d, loud.clone())).abs() < 0.01,
            "the loud half must pass: {} vs {}",
            rms(&out, loud.clone()),
            rms(&d, loud.clone())
        );
        assert!(
            rms(&out, quiet.clone()) < rms(&d, quiet.clone()) * 0.2,
            "the quiet half was not gated: {} vs {}",
            rms(&out, quiet.clone()),
            rms(&d, quiet.clone())
        );

        // A heavier ratio expands harder — that is what the knob means.
        let gentle = gate(&d, 0.1, 2.0, 0.002, release);
        let hard = gate(&d, 0.1, 16.0, 0.002, release);
        assert!(
            rms(&hard, quiet.clone()) < rms(&gentle, quiet.clone()),
            "ratio 16 must gate harder than ratio 2"
        );
        assert!(
            rms(&gentle, quiet.clone()) < rms(&d, quiet),
            "even 2:1 expands the floor down"
        );
    }

    /// A gate that could raise the gain above 1 would be an upward expander wearing
    /// a gate's name — and would push a loud passage into clipping.
    #[test]
    fn the_gate_never_amplifies() {
        let d = loud_then_quiet(0.9, 0.01);
        let out = gate(&d, 0.1, 8.0, 0.001, 0.02);
        for (a, b) in d.samples().iter().zip(out.samples()) {
            assert!(b.abs() <= a.abs() + 1e-6, "{b} > {a}");
        }
    }

    /// The gate opens on `attack` and closes on `release` — the OPPOSITE assignment
    /// to a compressor. Swap them and a gate chatters on every quiet passage instead
    /// of holding it closed. Proof: a slower release leaves more signal right after
    /// the loud half ends (the gate is still closing).
    #[test]
    fn the_gate_release_governs_how_slowly_it_closes() {
        let d = loud_then_quiet(0.5, 0.02);
        let just_after = 4_800..5_400; // the first 12 ms of the quiet half
        let energy = |x: &SampleData| {
            x.samples()[just_after.clone()]
                .iter()
                .map(|v| v * v)
                .sum::<f32>()
        };
        let quick = gate(&d, 0.1, 8.0, 0.002, 0.002);
        let slow = gate(&d, 0.1, 8.0, 0.002, 0.2);
        assert!(
            energy(&slow) > energy(&quick) * 2.0,
            "a slow release must still be closing: {} vs {}",
            energy(&slow),
            energy(&quick)
        );
    }

    /// Stereo gain must be linked, or a gate opens on the left and leaves the right
    /// behind — the image jumps every time someone breathes.
    #[test]
    fn the_gate_links_the_stereo_gain() {
        // Left is loud, right is under the threshold. Linking means the right rides
        // the left's open gate: neither channel is gated.
        let samples: Vec<f32> = (0..4_000).flat_map(|_| [0.5f32, 0.02f32]).collect();
        let d = stereo(samples);
        let out = gate(&d, 0.1, 16.0, 0.001, 0.05);
        let mid = 2_000 * 2;
        let gain_l = out.samples()[mid] / 0.5;
        let gain_r = out.samples()[mid + 1] / 0.02;
        assert!(
            (gain_l - gain_r).abs() < 1e-4,
            "channels got different gains: {gain_l} vs {gain_r}"
        );
        assert!(gain_l > 0.99, "the loud channel held the gate open");
    }

    #[test]
    fn sliding_min_matches_the_naive_window() {
        let v: Vec<f32> = (0..40).map(|i| ((i * 7) % 13) as f32).collect();
        for radius in [1usize, 3, 9, 60] {
            for (i, &got) in sliding_min(&v, radius).iter().enumerate() {
                let lo = i.saturating_sub(radius);
                let hi = (i + radius + 1).min(v.len());
                let want = v[lo..hi].iter().fold(f32::INFINITY, |m, &x| m.min(x));
                assert_eq!(got, want, "radius {radius}, index {i}");
            }
        }
    }

    /// The box average must not move a constant — a drifting prefix sum would tilt
    /// the whole gain curve on a long clip.
    #[test]
    fn box_average_preserves_a_constant() {
        let v = vec![0.375f32; 5_000];
        for out in box_average(&v, 700) {
            assert!((out - 0.375).abs() < 1e-6);
        }
    }

    /// A full-scale sine at `fs/4`, 45° off phase: every sample sits at ±0.707 while
    /// the waveform swings to ±1.0. A sample-domain limiter sees nothing to do. Ours
    /// must pull the **true** peak under the ceiling.
    #[test]
    fn the_limiter_catches_an_intersample_peak_a_sample_limiter_would_miss() {
        let amp = 0.999f32;
        let x: Vec<f32> = (0..4_800)
            .map(|n| {
                amp * (std::f32::consts::FRAC_PI_2 * n as f32 + std::f32::consts::FRAC_PI_4).cos()
            })
            .collect();
        let d = mono(x);

        let ceiling_db = -1.0f32; // the mastering convention: −1 dBTP
        let ceiling = 10f32.powf(ceiling_db / 20.0);
        assert!(
            crate::peak(&d) < ceiling,
            "the samples are already under the ceiling — a sample limiter would idle"
        );
        assert!(
            true_peak(&d) > ceiling,
            "but the reconstruction overshoots it"
        );

        let out = limit(&d, ceiling_db, 0.02);
        let tp = true_peak(&out);
        assert!(
            tp <= ceiling * 1.02,
            "true peak {tp} still over the {ceiling} ceiling"
        );
        // ...and it did not just squash the signal into nothing.
        assert!(tp > ceiling * 0.8, "over-attenuated to {tp}");
    }

    /// The look-ahead, isolated. A quiet passage that suddenly bursts into a
    /// full-scale transient is where a limiter *without* the sliding minimum fails:
    /// its smoother would average the gain of 1.0 that precedes the burst into the
    /// gain the burst needs, and let the first cycles through. The minimum is what
    /// makes the gain arrive **before** the peak does.
    ///
    /// (Verified by mutation: replacing `sliding_min(&need, radius)` with `need`
    /// makes the true peak land ~1.5× over the ceiling and this test fail.)
    #[test]
    fn the_limiter_ducks_before_a_transient_arrives_not_after() {
        let quarter = std::f32::consts::FRAC_PI_2;
        let phase = std::f32::consts::FRAC_PI_4;
        let x: Vec<f32> = (0..6_000)
            .map(|n| {
                // Silence, then a full-scale burst with inter-sample peaks, then silence.
                let amp = if (3_000..3_600).contains(&n) {
                    0.999
                } else {
                    0.0
                };
                amp * (quarter * n as f32 + phase).cos()
            })
            .collect();
        let d = mono(x);

        let ceiling_db = -1.0f32;
        let ceiling = 10f32.powf(ceiling_db / 20.0);
        assert!(
            true_peak(&d) > ceiling,
            "the burst must overshoot to begin with"
        );

        let out = limit(&d, ceiling_db, 0.02);
        let tp = true_peak(&out);
        assert!(
            tp <= ceiling * 1.02,
            "the transient's leading edge escaped: true peak {tp} > {ceiling}"
        );

        // And the gain really did dip ahead of the burst, not on it: the frame just
        // before the burst is already attenuated.
        let env = true_peak_envelope(&d);
        assert_eq!(env.len(), 6_000);
        let g_before = {
            let need: Vec<f32> = env
                .iter()
                .map(|&p| if p > ceiling { ceiling / p } else { 1.0 })
                .collect();
            let radius = (0.02 * 48_000.0) as usize;
            let dipped = sliding_min(&need, radius);
            let r1 = radius / 2;
            box_average(&box_average(&dipped, r1), radius - r1)[2_999]
        };
        assert!(
            g_before < 0.99,
            "gain at the frame before the burst was {g_before} — no look-ahead"
        );
    }

    /// The limiter may only ever turn things DOWN. A gain that could exceed 1 would
    /// make "limiting" a maximizer, and the ceiling a suggestion.
    #[test]
    fn the_limiter_never_amplifies() {
        let quiet: Vec<f32> = (0..2_000).map(|n| 0.05 * (n as f32 * 0.01).sin()).collect();
        let d = mono(quiet);
        let out = limit(&d, -1.0, 0.02);
        for (a, b) in d.samples().iter().zip(out.samples()) {
            assert!(b.abs() <= a.abs() + 1e-6, "{b} > {a}");
        }
    }

    /// Stereo gain must be LINKED: a per-channel gain swings the image whenever one
    /// side transiently peaks. Feed a loud left and a quiet right; the right must be
    /// pulled down by exactly the same factor as the left.
    #[test]
    fn the_limiter_links_the_stereo_gain() {
        let samples: Vec<f32> = (0..2_000).flat_map(|_| [0.99f32, 0.20f32]).collect();
        let d = stereo(samples);
        let out = limit(&d, -6.0, 0.01);
        let mid = 1_000 * 2;
        let gain_l = out.samples()[mid] / 0.99;
        let gain_r = out.samples()[mid + 1] / 0.20;
        assert!(
            (gain_l - gain_r).abs() < 1e-4,
            "channels got different gains: {gain_l} vs {gain_r}"
        );
        assert!(gain_l < 1.0, "the loud channel was not limited at all");
    }

    #[test]
    fn leveler_lifts_a_quiet_signal_toward_the_target() {
        let sr = 48_000usize;
        let n = sr / 2; // 0.5 s
        // A very quiet 300 Hz tone; the leveler (target −12 dB) should bring it UP.
        let quiet: Vec<f32> = (0..n)
            .map(|i| (i as f32 * std::f32::consts::TAU * 300.0 / sr as f32).sin() * 0.03)
            .collect();
        let d = SampleData::from_interleaved(quiet, AudioFormat::mono(48_000));
        let out = leveler(&d, -12.0, 1.0, 0.1);
        // Skip the ease-in and compare the settled tail's RMS.
        let rms = |s: &[f32]| (s.iter().map(|x| x * x).sum::<f32>() / s.len() as f32).sqrt();
        let in_rms = rms(&d.samples()[n * 3 / 4..]);
        let out_rms = rms(&out.samples()[n * 3 / 4..]);
        assert!(
            out_rms > in_rms * 2.0,
            "leveler did not lift the quiet signal: {in_rms} -> {out_rms}"
        );
    }

    #[test]
    fn leveler_at_zero_amount_is_identity() {
        // amount 0 → gain stays 1.0 → pass-through (the rack's neutral point).
        let d = SampleData::from_interleaved(vec![0.1, -0.2, 0.3, -0.4], AudioFormat::mono(48_000));
        let out = leveler(&d, -12.0, 0.0, 0.5);
        for (a, b) in d.samples().iter().zip(out.samples()) {
            assert!((a - b).abs() < 1e-6, "0-amount leveler altered the signal");
        }
    }
}
