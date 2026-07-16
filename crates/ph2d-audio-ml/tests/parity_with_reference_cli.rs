//! **The gate that authorised the whole feature (ADR-0123 §4.2).**
//!
//! The acceptance experiment ran the *official* DeepFilterNet CLI on the author's own 0 dB-SNR
//! fixture and measured +14 dB of SI-SDR over the noisy input. That number is what bought ~130
//! crates and a 7.6 MB model. This gate proves **our wrapper reproduces it** — that
//! [`ph2d_audio_ml::denoise_ml`] wires the model up right (rate, hop, delay compensation, the
//! CLI's parameters) and is not silently broken while still "passing" some looser check.
//!
//! The fixture is the DeepFilterNet author's `clean_freesound_33711.wav` / `noisy_snr0.wav`
//! (real speech at 0 dB SNR, sample-aligned), trimmed to 6 s. On the full clip the CLI scores
//! 20.0 dB; on this trim it scores 20.6 dB — the same feature.
//!
//! ## Why two assertions, not one
//!
//! A pure "denoise scores ≥ 18 dB" gate would also pass if the fixture were already clean — the
//! denoiser would have nothing to do and score high by doing nothing
//! ([[feedback_absence_gate_needs_a_presence_sibling]]). So the input is pinned noisy first (≈ 6
//! dB), and the gain is what is really being measured. Take the noise out (the ≥ 18 dB), and be
//! starting from a genuinely noisy signal (the ≈ 6 dB) — the pair is the contract.

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};
use std::path::Path;

/// Read a canonical PCM16 mono WAV (the exact shape the two fixtures have: RIFF/WAVE, format 1,
/// 1 channel, 48 kHz, 16-bit, `data` chunk at byte 44). Not a general WAV reader — a reader for
/// fixtures this test owns, so it may assume their layout.
fn read_wav(path: &Path) -> SampleData {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    assert_eq!(&bytes[0..4], b"RIFF", "not a RIFF file");
    assert_eq!(&bytes[8..12], b"WAVE", "not a WAVE file");
    // Canonical layout: fmt chunk (16 bytes) then data chunk header, PCM16 payload at byte 44.
    assert_eq!(&bytes[36..40], b"data", "fixture is not canonical 44-byte-header PCM WAV");
    let sr = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
    let pcm = &bytes[44..];
    let samples: Vec<f32> = pcm
        .chunks_exact(2)
        .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0)
        .collect();
    SampleData::from_interleaved(samples, AudioFormat::new(sr, ChannelLayout::Mono))
}

/// The lag (in samples) that best aligns `est` to `clean`, by cross-correlation over the central
/// half of the clip. DeepFilterNet has a fixed STFT+lookahead delay; the wrapper compensates it,
/// but a residual sample or two would tank a phase-sensitive metric, so the measurement aligns
/// first. Searches ±1024 samples — far more than the compensated delay can leave behind.
fn best_lag(est: &[f32], clean: &[f32]) -> isize {
    let n = est.len().min(clean.len());
    let (lo, hi) = (n / 4, 3 * n / 4);
    let cw = &clean[lo..hi];
    let mut best = f64::NEG_INFINITY;
    let mut best_lag = 0isize;
    for lag in -1024isize..=1024 {
        let s = lo as isize + lag;
        let e = hi as isize + lag;
        if s < 0 || e as usize > est.len() {
            continue;
        }
        let mut dot = 0.0f64;
        for (a, b) in cw.iter().zip(&est[s as usize..e as usize]) {
            dot += *a as f64 * *b as f64;
        }
        if dot > best {
            best = dot;
            best_lag = lag;
        }
    }
    best_lag
}

/// Scale-invariant SDR in dB (the standard denoise metric), aligning `est` to `clean` by
/// cross-correlation first. `alpha = <clean,est>/<clean,clean>`, `target = alpha*clean`,
/// `SI-SDR = 10·log10(||target||² / ||est-target||²)`.
fn si_sdr(est: &[f32], clean: &[f32]) -> f64 {
    let lag = best_lag(est, clean);
    // Shift `est` by `lag` (positive = est is late, drop its head).
    let shifted: Vec<f32> = if lag >= 0 {
        est[lag as usize..].to_vec()
    } else {
        let mut v = vec![0.0f32; (-lag) as usize];
        v.extend_from_slice(&est[..est.len() - (-lag) as usize]);
        v
    };
    let n = shifted.len().min(clean.len());
    let est = &shifted[..n];
    let clean = &clean[..n];
    let dot = |a: &[f32], b: &[f32]| -> f64 {
        a.iter().zip(b).map(|(x, y)| *x as f64 * *y as f64).sum()
    };
    let alpha = dot(clean, est) / dot(clean, clean).max(1e-20);
    let mut tt = 0.0f64;
    let mut nn = 0.0f64;
    for i in 0..n {
        let t = alpha * clean[i] as f64;
        tt += t * t;
        let e = est[i] as f64 - t;
        nn += e * e;
    }
    10.0 * (tt / nn.max(1e-20)).log10()
}

fn fixture(name: &str) -> SampleData {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    read_wav(&p)
}

/// **THE gate.** Our `denoise_ml` must reproduce the reference CLI's gain on the author's
/// fixture: ≥ 18 dB of SI-SDR against the clean reference (the CLI scores 20.6 dB here; 18 leaves
/// slack for alignment and any wrapper-vs-CLI tail handling). If this fails, the model is wired up
/// wrong — the feature does not exist.
#[test]
fn denoise_ml_reproduces_the_reference_cli_gain() {
    let noisy = fixture("noisy.wav");
    let clean = fixture("clean.wav");
    let clean_s = clean.samples();

    // Presence sibling: the input really is noisy (~6 dB). A pre-cleaned fixture would make the
    // gain gate below meaningless.
    let before = si_sdr(noisy.samples(), clean_s);
    assert!(
        (4.0..9.0).contains(&before),
        "the fixture is not at ~6 dB SNR ({before:.2} dB) — the gate would be measuring the wrong \
         thing (the acceptance experiment measured 6.05 dB on the full clip, 6.56 on this trim)"
    );

    // Full model output (amount 1.0 = pure DFN, the CLI's atten_lim=100 path).
    let out = ph2d_audio_ml::denoise_ml(&noisy, 1.0);
    let after = si_sdr(out.samples(), clean_s);
    let gain = after - before;
    println!("SI-SDR: noisy {before:.2} dB -> denoised {after:.2} dB  (gain {gain:+.2} dB; CLI reference 20.59 dB / +14.04 dB)");
    assert!(
        after >= 18.0,
        "denoise_ml scored only {after:.2} dB (CLI reference 20.6) — the wrapper is not \
         reproducing the model: check rate, hop, delay compensation, and the RuntimeParams"
    );
    // And it is a real gain over the input, not a high score on an easy fixture.
    assert!(
        gain >= 10.0,
        "denoise_ml bought only {gain:+.2} dB over the noisy input (reference +14) — \
         it is protecting the signal by declining to denoise"
    );
}

/// **Zero is bypass, byte for byte** — over the *real* fixture, not just a synthetic tone. The
/// rack's non-negotiable neutral point: an "off" that resynthesises the clip cannot be A/B'd.
#[test]
fn amount_zero_is_byte_identical_on_the_fixture() {
    let noisy = fixture("noisy.wav");
    let out = ph2d_audio_ml::denoise_ml(&noisy, 0.0);
    assert_eq!(noisy.samples(), out.samples());
}
