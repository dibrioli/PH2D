//! **Harmonizer** — two pitched copies of the take, sung along with it.
//!
//! The cheapest big effect in the rack: it owns no DSP of its own. Two runs of the
//! WSOLA shifter in [`super::wsola`] at the intervals the user dials (a third and a
//! fifth by default — a major triad) and a blend back over the dry voice. A chorus
//! thickens one voice; this one adds *different notes*, which is what turns a line
//! into a choir, a shout into a crowd, or a creature into a swarm.
//!
//! It is also the effect that forced the shifter to be **in tune**: the delay-line
//! granular engine that came before drifted flat by tens of cents (see
//! [`super::wsola`]), which passes unnoticed on a monster voice and sounds simply
//! wrong on a chord.
//!
//! The blend is convex — `(1−mix)·dry + mix·(dry + v1 + v2)/3` — so a full-scale take
//! cannot clip its way out of the harmony, whatever the intervals.
//!
//! Control thread only.

use ph2d_audio::SampleData;

use super::wsola::pitch_shift;

/// Blend the dry take with copies pitched by `v1` and `v2` semitones. `mix` is the
/// level of the harmony (0 = dry). Same length out.
///
/// Called only off the neutral point (`mix` above 0); the caller returns the input
/// untouched otherwise.
pub(super) fn harmonize(data: &SampleData, v1: f32, v2: f32, mix: f32) -> SampleData {
    let frames = data.frame_count();
    if frames == 0 {
        return data.clone();
    }
    let mix = mix.clamp(0.0, 1.0);
    // Fully wet copies: the dry share of the blend is applied once, below, rather than
    // once per voice.
    let a = pitch_shift(data, v1, 1.0);
    let b = pitch_shift(data, v2, 1.0);

    let dry = data.samples();
    let out: Vec<f32> = dry
        .iter()
        .zip(a.samples())
        .zip(b.samples())
        .map(|((&d, &x), &y)| {
            // The wet side is the AVERAGE of the three voices, so it is bounded by
            // full scale even when all three peak together — and the crossfade to it
            // is convex, so the sum is too.
            let chord = (d + x + y) / 3.0;
            (1.0 - mix) * d + mix * chord
        })
        .collect();
    SampleData::from_interleaved(out, data.format())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    const SR: u32 = 48_000;
    /// Skip the shifter's first grain, which has no history to align against.
    const SETTLED: usize = 2_048;

    fn tone(hz: f32, n: usize) -> SampleData {
        let tau = std::f32::consts::TAU;
        let x: Vec<f32> = (0..n)
            .map(|i| 0.6 * (tau * hz * i as f32 / SR as f32).sin())
            .collect();
        SampleData::from_interleaved(x, AudioFormat::mono(SR))
    }

    /// Energy at one frequency — a single DFT bin, computed directly (no FFT).
    fn energy_at(x: &[f32], hz: f64) -> f64 {
        let (mut re, mut im) = (0.0f64, 0.0f64);
        for (n, &v) in x.iter().enumerate() {
            let phase = std::f64::consts::TAU * hz * n as f64 / f64::from(SR);
            re += f64::from(v) * phase.cos();
            im -= f64::from(v) * phase.sin();
        }
        (re * re + im * im).sqrt() / x.len() as f64
    }

    /// THE contract: the harmony notes are actually THERE, at the pitches asked for.
    ///
    /// Feed a 300 Hz root, ask for a third and a fifth, and the output must carry real
    /// energy at ~378 Hz and ~450 Hz — frequencies the input has none of — while the
    /// root survives underneath. Red if a voice is built but never mixed in, if the
    /// intervals are wired to the wrong voice, or if the shifter drifts off the note.
    #[test]
    fn the_harmony_voices_land_on_their_intervals() {
        let root = 300.0f64;
        let d = tone(root as f32, 24_000);
        let out = harmonize(&d, 4.0, 7.0, 1.0);
        let (dry, wet) = (&d.samples()[SETTLED..], &out.samples()[SETTLED..]);

        // Each voice appears at its own interval, and each is a real note — comparable
        // in level to the root it is stacked on, not a whisper 30 dB down.
        let root_level = energy_at(wet, root);
        assert!(
            root_level > 0.05,
            "the dry voice was swallowed: {root_level}"
        );
        for (name, st) in [("third", 4.0f64), ("fifth", 7.0)] {
            let hz = root * (st / 12.0).exp2();
            let before = energy_at(dry, hz);
            let after = energy_at(wet, hz);
            assert!(
                after > before * 10.0,
                "the {name} is missing: {before} dry, {after} wet"
            );
            assert!(
                after > root_level * 0.4,
                "the {name} is only a whisper: {after} against a root of {root_level}"
            );
        }
    }

    /// A convex blend of three bounded voices cannot clip — whatever the intervals.
    #[test]
    fn the_chord_stays_bounded() {
        let d = SampleData::from_interleaved(vec![1.0; 8_000], AudioFormat::mono(SR));
        for mix in [0.25f32, 0.5, 1.0] {
            let out = harmonize(&d, -12.0, 12.0, mix);
            assert!(
                out.samples().iter().all(|s| s.abs() <= 1.0 + 1e-4),
                "clipped at mix {mix}"
            );
        }
    }

    #[test]
    fn harmonize_preserves_length() {
        assert_eq!(
            harmonize(&tone(300.0, 9_600), 4.0, 7.0, 0.5).frame_count(),
            9_600
        );
    }

    /// `mix` 0 returns the dry signal exactly — the blend, pinned. (The rack bypasses
    /// this case before it ever calls in, but the math must still be honest.)
    #[test]
    fn mix_zero_is_dry() {
        let d = tone(300.0, 4_800);
        assert_eq!(harmonize(&d, 4.0, 7.0, 0.0).samples(), d.samples());
    }
}
