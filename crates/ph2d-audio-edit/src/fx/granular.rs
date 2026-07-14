//! **Granular** — the clip chopped into overlapping grains and put back down out of order.
//!
//! Each grain is a windowed slice of the source, laid back into the output at a position it is
//! allowed to *wander* from (`scatter`) and at a playback rate it is allowed to *detune* from
//! (`pitch`). Overlap-add the lot and a sound stops being an event and becomes a **texture**:
//! a footstep smears into gravel, a voice into a crowd, a note into a pad. It is the standard
//! way to build ambience out of a one-shot, which is exactly what a game needs it for.
//!
//! # This is not the WSOLA engine wearing a hat
//!
//! [`super::wsola`] also cuts the signal into windowed chunks — but it *synchronises* them: it
//! searches for the lag where the next chunk best correlates with what has already been laid
//! down, precisely so that the result sounds like the input at a different pitch, with no
//! artefacts. Granular wants the opposite. The grains are placed **stochastically** and the
//! artefacts *are the effect*. Same overlap-add machinery, opposite scheduler, and it is the
//! scheduler that is the instrument.
//!
//! # Hann at 50 % overlap sums to exactly one — which is the trap
//!
//! With `scatter` and `pitch` both at zero, every grain lands where it was taken from at the
//! rate it was taken at, and a Hann window at half-hop is a partition of unity: the output is
//! the input, sample for sample. That is a *good* property (it says the overlap-add is
//! transparent and the effect only adds what it is asked to) — but it means **`mix` alone is
//! not enough of an arm**. If `scatter` defaulted to 0, turning the effect fully wet would do
//! nothing at all, and `turning_an_arming_knob_wakes_the_effect_up` would be right to fail.
//! So the neutral point is `mix` 0 (dry, byte-identical, like every other effect) and the
//! **default scatter is a real one** — a granular you switch on is a granular you can hear.
//!
//! Control thread only (HR-5 does not apply).

use ph2d_audio::SampleData;

use crate::ops::channels;

/// How far a grain may wander from where it was taken, at `scatter` 1 — in seconds, each way.
/// Long enough to smear a transient into a texture, short enough that the sound stays in the
/// same moment of the clip.
pub(super) const SCATTER_MAX_S: f32 = 0.2;
/// A granular is neutral when it is fully dry.
pub(super) const GRAIN_BYPASS_MIX: f32 = 0.0;

/// Deterministic per-grain randomness: the grain's index is the seed, so the same clip and the
/// same knobs render the same cloud every time (a rendered effect that differs run to run
/// cannot be finger-printed, and an undo that comes back different is not an undo).
fn hash(g: u64, lane: u64) -> f32 {
    let mut z = g
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(lane.wrapping_mul(0xD1B5_4A32_D192_ED03));
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    // [-1, 1)
    ((z >> 40) as f32 / (1u32 << 23) as f32) - 1.0
}

/// Linear-interpolated read at a fractional frame — the grain's own playback rate lives here.
fn sample_at(src: &[f32], frames: usize, ch: usize, c: usize, pos: f32) -> f32 {
    if pos < 0.0 || pos >= (frames - 1) as f32 {
        return 0.0;
    }
    let i = pos as usize;
    let t = pos - i as f32;
    let a = src[i * ch + c];
    let b = src[(i + 1) * ch + c];
    a + t * (b - a)
}

/// See [`Effect::Granular`](super::Effect::Granular).
pub(super) fn granular(
    data: &SampleData,
    grain_ms: f32,
    scatter: f32,
    pitch_st: f32,
    mix: f32,
) -> SampleData {
    let sr = data.format().sample_rate as f32;
    let ch = channels(data);
    let frames = data.frame_count();
    let mix = mix.clamp(0.0, 1.0);
    let scatter = scatter.clamp(0.0, 1.0);

    let len = ((grain_ms * 0.001 * sr) as usize).clamp(4, frames.max(4));
    // Half-hop: with the Hann window below, the grains are a partition of unity, so a cloud
    // with nothing asked of it reconstructs the source exactly (see the module docs).
    let hop = (len / 2).max(1);
    let src = data.samples();
    let peak_in = crate::peak(data);

    SampleData::build(src.len(), data.format(), |out| {
        let grains = frames.div_ceil(hop) + 1;
        for g in 0..grains {
            let out_start = g * hop;
            // One draw per GRAIN, shared by every channel: drawing per channel would send the
            // left and right halves of a grain to different places and the image would fall apart.
            let wander = hash(g as u64, 0) * scatter * SCATTER_MAX_S * sr;
            let detune = hash(g as u64, 1) * pitch_st;
            let rate = if detune == 0.0 {
                1.0
            } else {
                (detune / 12.0).exp2()
            };
            let src_start = out_start as f32 + wander;

            for n in 0..len {
                let o = out_start + n;
                if o >= frames {
                    break;
                }
                // Hann. At half-hop, consecutive windows sum to 1.
                let w = {
                    let p = std::f32::consts::TAU * n as f32 / len as f32;
                    0.5 - 0.5 * p.cos()
                };
                let pos = src_start + n as f32 * rate;
                for c in 0..ch {
                    out[o * ch + c] += w * sample_at(src, frames, ch, c, pos);
                }
            }
        }

        // A cloud's level is whatever the grains happened to pile up to; put it back at the
        // source's peak so switching the effect on is not also a volume change (and so a dense
        // cloud cannot clip). Same peak-preserving contract as the compressor.
        let peak_wet = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        if peak_wet > f32::EPSILON && peak_in > f32::EPSILON {
            let g = peak_in / peak_wet;
            for s in out.iter_mut() {
                *s *= g;
            }
        }
        for (o, d) in out.iter_mut().zip(src) {
            *o = ((1.0 - mix) * d + mix * *o).clamp(-1.0, 1.0);
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    const SR: u32 = 48_000;

    fn tone(hz: f32, secs: f32) -> SampleData {
        let tau = std::f32::consts::TAU;
        let n = (SR as f32 * secs) as usize;
        SampleData::from_interleaved(
            (0..n * 2)
                .map(|i| {
                    let t = (i / 2) as f32 / SR as f32;
                    0.6 * (tau * hz * t).sin()
                })
                .collect(),
            AudioFormat::stereo(SR),
        )
    }

    /// A transient in the middle of silence — the thing granular exists to SMEAR.
    fn click_at(secs: f32) -> SampleData {
        let n = SR as usize;
        let at = (SR as f32 * secs) as usize;
        SampleData::from_interleaved(
            (0..n * 2)
                .map(|i| {
                    let f = i / 2;
                    if f.abs_diff(at) < 60 { 0.9 } else { 0.0 }
                })
                .collect(),
            AudioFormat::stereo(SR),
        )
    }

    /// **The overlap-add is transparent.** With nothing asked of it — no scatter, no detune —
    /// the grains land where they came from and a Hann window at half-hop is a partition of
    /// unity, so the cloud reconstructs the source.
    ///
    /// This is what makes `mix` an honest crossfade rather than a mystery, and it is *also*
    /// why the shipped default scatter cannot be zero: this same property would make a fully
    /// wet granular inaudible. Both halves of that are load-bearing, so both are pinned here.
    #[test]
    fn with_nothing_asked_of_it_the_cloud_reconstructs_the_source() {
        let d = tone(440.0, 0.5);
        let out = granular(&d, 60.0, 0.0, 0.0, 1.0);
        // Skip the first and last grain: they are the only ones with no partner to sum with.
        let skip = (0.06 * SR as f32) as usize * 2;
        let a = &d.samples()[skip..d.samples().len() - skip];
        let b = &out.samples()[skip..out.samples().len() - skip];
        let worst = a
            .iter()
            .zip(b)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0f32, f32::max);
        assert!(
            worst < 0.01,
            "a granular with no scatter and no detune did not reconstruct its input (worst \
             sample error {worst:.4}) -- the window is not a partition of unity at this hop"
        );
    }

    /// How wide, in ms, the sound's energy is spread over time — the energy-weighted standard
    /// deviation of its own timeline. This IS "how smeared is it", and every grain contributes
    /// to it.
    ///
    /// The first version of this gate asked whether one particular window 100 ms past the click
    /// had energy in it. That is a **coin toss**, not a measurement: a grain there only catches
    /// the click if its wander happens to draw into a 60 ms slice of a 320 ms range (~19 %), and
    /// with a handful of grains in the window the gate passes or fails on the seed. It failed on
    /// this one, and the effect was working the whole time. A gate whose answer depends on a
    /// lucky draw tests the draw.
    fn spread_ms(d: &SampleData) -> f32 {
        let e: Vec<f32> = d.samples().chunks(2).map(|f| f[0].abs()).collect();
        let total: f32 = e.iter().sum();
        if total <= f32::EPSILON {
            return 0.0;
        }
        let mean: f32 = e.iter().enumerate().map(|(i, w)| i as f32 * w).sum::<f32>() / total;
        let var: f32 = e
            .iter()
            .enumerate()
            .map(|(i, w)| w * (i as f32 - mean).powi(2))
            .sum::<f32>()
            / total;
        var.sqrt() / SR as f32 * 1_000.0
    }

    /// **Scatter smears a transient in TIME.** A click sitting in silence must come back spread
    /// across the neighbourhood — that IS the effect, and it is what turns a footstep into
    /// gravel.
    #[test]
    fn scatter_spreads_a_transient_over_time() {
        let d = click_at(0.5);
        let tight = granular(&d, 60.0, 0.0, 0.0, 1.0);
        let smeared = granular(&d, 60.0, 0.8, 0.0, 1.0);
        let (dry, a, b) = (spread_ms(&d), spread_ms(&tight), spread_ms(&smeared));
        println!("spread: dry {dry:.2} ms, scatter 0 {a:.2} ms, scatter 0.8 {b:.2} ms");
        // Scatter 0 must NOT smear (it is the transparent case) ...
        assert!(
            a < dry * 2.0 + 1.0,
            "a granular with no scatter smeared the click anyway ({a:.2} ms vs {dry:.2} ms dry)"
        );
        // ... and scatter 0.8 must, by a mile. The bar sits between the two measurements.
        assert!(
            b > 20.0,
            "the click did not smear ({b:.2} ms, against {dry:.2} ms dry) -- scatter is not \
             moving the grains"
        );
    }

    /// **Pitch detunes the grains, and the scheduler is DETERMINISTIC.** Same input, same
    /// knobs, same cloud — twice. An effect whose render changes run to run cannot be undone,
    /// cannot be finger-printed, and cannot be exported twice.
    #[test]
    fn the_same_cloud_renders_the_same_way_twice() {
        let d = tone(440.0, 0.3);
        let a = granular(&d, 40.0, 0.6, 7.0, 1.0);
        let b = granular(&d, 40.0, 0.6, 7.0, 1.0);
        assert_eq!(
            a.samples(),
            b.samples(),
            "the same granular rendered differently twice -- the grain RNG is not seeded by the \
             grain index"
        );
        // ...and it is not simply a no-op that trivially matches itself.
        assert_ne!(a.samples(), d.samples(), "the cloud did nothing at all");
    }
}
