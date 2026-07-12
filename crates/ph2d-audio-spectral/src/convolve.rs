//! Fast convolution — the operation a **convolution reverb** is made of.
//!
//! An impulse response *is* a room. Fire a starter pistol in a cathedral, record what comes
//! back, and that recording contains everything the cathedral does to a sound: the early
//! reflections off the pillars, the diffuse tail, the way the stone eats the high frequencies
//! before it eats the low ones. Convolve a dry voice with it and the voice is **in** the
//! cathedral — not in an approximation of one.
//!
//! That is a different proposition from the algorithmic reverb the rack already has
//! (Freeverb: a bank of comb and all-pass filters that *sounds like* a room). An algorithmic
//! reverb is a plausible room. A convolution reverb is a specific one, and you can capture
//! your own.
//!
//! ## Why this needs the FFT (and why it is here)
//!
//! Convolution in the time domain is `O(n·m)`: every output sample is a dot product with the
//! whole impulse response. A 2-second IR at 48 kHz is 96 000 taps; a 60-second clip is 2.9 M
//! samples. That is **2.8 × 10¹¹** multiply-adds — minutes, not milliseconds.
//!
//! Multiplication in the frequency domain *is* convolution in the time domain. So: transform,
//! multiply, transform back. The cost falls to `O(n log n)` and the same job takes a fraction
//! of a second. This is the second thing the FFT bought (ADR-0115), and it needed no new
//! dependency at all — which is why it lives in this crate, next to the transform.
//!
//! The signal is cut into blocks and the results **overlap-added**, because each block's
//! convolution is `block + ir - 1` samples long: it rings out past its own block, into the
//! next one's territory. Adding the overlaps back is not an approximation — it is exactly what
//! the definition of convolution says, and `fft_convolution_matches_the_direct_sum` proves it
//! against a literal transcription of that definition.

use realfft::num_complex::Complex32;
use realfft::{RealFftPlanner, num_complex::Complex};

/// The smallest FFT worth planning. Below this the transform's own overhead outweighs what it
/// saves, and a short impulse response is better served by the direct sum.
const MIN_FFT: usize = 1_024;

/// Below this many taps, convolve directly: the FFT costs more than it saves, and the direct
/// sum has no rounding to explain.
const DIRECT_TAPS: usize = 64;

/// Convolve `x` with the impulse response `ir`.
///
/// Returns `x.len() + ir.len() - 1` samples — the input plus the **tail**, which is the part
/// of the room that keeps sounding after the sound stops. That tail is the whole point; a
/// caller that truncates it back to `x.len()` has thrown the reverb away and kept the colour.
///
/// An empty input or an empty IR convolves to nothing, and says so by returning empty.
pub fn convolve(x: &[f32], ir: &[f32]) -> Vec<f32> {
    if x.is_empty() || ir.is_empty() {
        return Vec::new();
    }
    let out_len = x.len() + ir.len() - 1;
    if ir.len() <= DIRECT_TAPS {
        return direct(x, ir, out_len);
    }

    // One FFT big enough to hold a block AND the IR's ring-out without wrapping. A circular
    // convolution that wraps folds the tail back onto the head of the block — the classic
    // "time aliasing", which sounds like a ghost of the reverb arriving before the sound.
    let n = (2 * ir.len()).next_power_of_two().max(MIN_FFT);
    let block = n - ir.len() + 1;

    let mut planner = RealFftPlanner::<f32>::new();
    let fwd = planner.plan_fft_forward(n);
    let inv = planner.plan_fft_inverse(n);
    let bins = n / 2 + 1;

    // The IR's spectrum, once — it does not change from block to block.
    let mut pad = vec![0.0f32; n];
    pad[..ir.len()].copy_from_slice(ir);
    let mut ir_spec = vec![Complex32::default(); bins];
    if fwd.process(&mut pad, &mut ir_spec).is_err() {
        return direct(x, ir, out_len);
    }

    let mut out = vec![0.0f32; out_len + n];
    let mut spec = vec![Complex32::default(); bins];
    let mut time = vec![0.0f32; n];
    let scale = 1.0 / n as f32;
    for (b, chunk) in x.chunks(block).enumerate() {
        time[..chunk.len()].copy_from_slice(chunk);
        time[chunk.len()..].fill(0.0);
        if fwd.process(&mut time, &mut spec).is_err() {
            return direct(x, ir, out_len);
        }
        for (s, h) in spec.iter_mut().zip(&ir_spec) {
            *s = Complex::new(s.re * h.re - s.im * h.im, s.re * h.im + s.im * h.re);
        }
        if inv.process(&mut spec, &mut time).is_err() {
            return direct(x, ir, out_len);
        }
        // Overlap-ADD: this block's result rings out past its own block, into the next one's
        // samples. Those overlaps are not a correction — they ARE the convolution.
        let start = b * block;
        for (i, v) in time.iter().enumerate() {
            out[start + i] += v * scale;
        }
    }
    out.truncate(out_len);
    out
}

/// The definition, transcribed. Used for impulse responses too short to be worth an FFT, as
/// the fallback when a transform fails, and — in the tests — as the oracle the fast path is
/// measured against.
fn direct(x: &[f32], ir: &[f32], out_len: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; out_len];
    for (i, &xi) in x.iter().enumerate() {
        if xi == 0.0 {
            continue;
        }
        for (k, &hk) in ir.iter().enumerate() {
            out[i + k] += xi * hk;
        }
    }
    out
}

/// Scale an impulse response so convolving with it **preserves loudness**.
///
/// A raw IR is a recording of a room, at whatever level the microphone happened to see. Convolve
/// with it unnormalised and the result is arbitrarily louder or quieter than the input — the
/// user would be reaching for the gain before they could hear the room. Dividing by the IR's
/// energy (its RMS × √len, i.e. its L2 norm) makes the convolution unity-gain for a broadband
/// signal, so the Mix knob crossfades between two things at the *same* level, which is the only
/// way a Mix knob means anything.
pub fn normalize_ir(ir: &[f32]) -> Vec<f32> {
    let energy: f32 = ir.iter().map(|v| v * v).sum();
    if energy <= 0.0 {
        return ir.to_vec();
    }
    let g = 1.0 / energy.sqrt();
    ir.iter().map(|v| v * g).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn noise(n: usize, seed: u64) -> Vec<f32> {
        let mut s = seed | 1;
        (0..n)
            .map(|_| {
                s = s.wrapping_add(0x9E37_79B9_7F4A_7C15);
                let mut z = s;
                z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                z ^= z >> 31;
                (z >> 40) as f32 / 8_388_608.0 - 1.0
            })
            .collect()
    }

    /// **The fast path gives the same answer as the definition.**
    ///
    /// The FFT convolution is fast *because* it exploits an identity (multiplication in
    /// frequency is convolution in time). The identity is true; the implementation of it is a
    /// claim — block sizes, zero-padding, the overlap-add, the inverse's scaling, every one of
    /// them a place to be quietly wrong. So it is measured against a literal transcription of
    /// `y[i+k] += x[i]·h[k]`, which assumes nothing and is obviously correct.
    ///
    /// Same discipline as the Levinson solver next door: a fast algorithm is only as
    /// trustworthy as the slow one you checked it against.
    #[test]
    fn fft_convolution_matches_the_direct_sum() {
        for (nx, nh) in [
            (1_000usize, 100usize),
            (5_000, 777),
            (300, 4_096),
            (48_000, 2_048),
        ] {
            let x = noise(nx, 0xA11CE);
            let h = noise(nh, 0xB0B);
            let fast = convolve(&x, &h);
            let slow = direct(&x, &h, nx + nh - 1);
            assert_eq!(fast.len(), slow.len(), "({nx}, {nh}): length");
            // Relative to the signal's own scale: an FFT accumulates rounding across n log n
            // operations, and asking for exactness would be asking the wrong question.
            let peak = slow.iter().fold(0.0f32, |m, v| m.max(v.abs())).max(1e-9);
            let worst = fast
                .iter()
                .zip(&slow)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f32, f32::max);
            assert!(
                worst / peak < 1e-4,
                "({nx}, {nh}): the FFT convolution disagrees with the direct sum by {:.2e} \
                 (relative to a peak of {peak:.3})",
                worst / peak
            );
        }
    }

    /// Convolving with a unit impulse returns the input, unchanged and undelayed. (The
    /// identity element — if this is wrong, everything is.)
    #[test]
    fn an_impulse_is_the_identity() {
        let x = noise(4_000, 0xC0FFEE);
        let mut h = vec![0.0f32; 2_000];
        h[0] = 1.0;
        let y = convolve(&x, &h);
        for (i, (a, b)) in x.iter().zip(&y).enumerate() {
            assert!((a - b).abs() < 1e-5, "sample {i}: {a} became {b}");
        }
    }

    /// An impulse at `d` delays by exactly `d`. This is the test that catches an overlap-add
    /// that adds in the wrong place — the block offset being off by one would smear the whole
    /// reverb by a block, and it would still *sound* like a reverb.
    #[test]
    fn a_delayed_impulse_delays_by_exactly_that() {
        let x = noise(3_000, 0xD00D);
        let d = 777usize;
        let mut h = vec![0.0f32; 2_048];
        h[d] = 1.0;
        let y = convolve(&x, &h);
        assert!(
            y[..d].iter().all(|v| v.abs() < 1e-5),
            "signal arrived early"
        );
        for i in 0..x.len() {
            assert!(
                (y[d + i] - x[i]).abs() < 1e-5,
                "sample {i} did not land at {}",
                d + i
            );
        }
    }

    /// The output carries the **tail**: the room keeps sounding after the input stops. A
    /// convolution truncated back to the input's length has thrown the reverb away and kept
    /// only the colouration.
    #[test]
    fn the_tail_is_there() {
        let x = noise(1_000, 1);
        let h = noise(4_000, 2);
        let y = convolve(&x, &h);
        assert_eq!(y.len(), 1_000 + 4_000 - 1);
        let tail_energy: f32 = y[1_000..].iter().map(|v| v * v).sum();
        assert!(
            tail_energy > 0.0,
            "the room went silent the instant the sound did"
        );
    }

    /// A normalised IR is unity-gain: the wet signal comes back at the level the dry one went
    /// in at, which is what lets a Mix knob crossfade between them meaningfully.
    #[test]
    fn a_normalised_ir_preserves_loudness() {
        let x = noise(48_000, 7);
        // A decaying-noise IR — the shape of a real room.
        let h: Vec<f32> = noise(8_000, 9)
            .iter()
            .enumerate()
            .map(|(i, v)| v * (-(i as f32) / 2_000.0).exp())
            .collect();
        let y = convolve(&x, &normalize_ir(&h));
        let rms = |v: &[f32]| (v.iter().map(|s| s * s).sum::<f32>() / v.len() as f32).sqrt();
        let ratio = rms(&y[8_000..48_000]) / rms(&x[8_000..48_000]);
        assert!(
            (0.5..2.0).contains(&ratio),
            "a normalised IR changed the level by {ratio:.2}x — the Mix knob would be a \
             volume knob in disguise"
        );
    }

    /// Nothing in, nothing out — and no panic.
    #[test]
    fn empty_inputs_are_empty_outputs() {
        assert!(convolve(&[], &[1.0, 2.0]).is_empty());
        assert!(convolve(&[1.0, 2.0], &[]).is_empty());
    }
}
