//! Dev smoke for the **Multiband** (`PH2D_AUDIO_MULTIBAND_SMOKE=1`) — the clip AND the rack,
//! staged so the A/B is two clicks and no file picking.
//!
//! # The material is the test
//!
//! A multiband compressor only differs from a plain one on audio that has *the problem it
//! solves*: a **loud, intermittent low-frequency event** driving the gain, and **quieter,
//! steady content above it** that has no business moving. So the clip is exactly that, and
//! nothing else:
//!
//! - a **kick** at 60 Hz every half second (120 BPM), peaking near full scale — this is what
//!   pulls a single-band compressor's gain down;
//! - a **pad** (220 + 330 Hz) and a **shimmer** (6 + 9 kHz), both **dead steady**.
//!
//! The steadiness is the whole point: the pad and the shimmer never change on their own, so
//! **any movement you hear in them is the compressor ducking them**, not the material. And the
//! shimmer sits ~25 dB under the kick, which is the ordinary spectral tilt that makes an
//! absolute per-band threshold useless (the bass crosses it constantly; the treble never
//! reaches it).
//!
//! # The A/B is already wired
//!
//! The rack is staged with **two** stages at the **same Ratio (hard right, 20:1)**:
//!
//! | | |
//! |---|---|
//! | **Multiband** | enabled |
//! | **Compress** | bypassed |
//!
//! Flip the per-stage enable on each (that is what `FxStage::enabled` is *for*) and the
//! settings are identical by construction — no re-dialling, nothing to eyeball. On Compress
//! the pad and shimmer **pump at 120 BPM**, once per kick. On Multiband they sit still while
//! the kick alone is tamed. That is the entire feature, and it is audible in one bar.

use ph2d_audio::{AudioFormat, SampleData};
use ph2d_panel_audio_editor::{FxStage, set_fx_chain};

use super::super::fx_params::default_norms;
use super::super::fx_params_table::KINDS;
use crate::audio::AudioSystem;
use crate::audio::editor::EditorTransport;

const SR: u32 = 48_000;
/// Four bars at 120 BPM — long enough to hear the pumping settle into a rhythm.
const SECS: usize = 4;
/// 120 BPM: a kick every half second.
const KICK_PERIOD_S: f32 = 0.5;
/// How fast the kick's body dies away. Short enough to be a hit, not a drone.
const KICK_DECAY_S: f32 = 0.11;

/// The kick: a 60 Hz sine with a sharp exponential decay, near full scale. Loud and
/// intermittent — the two properties that make a single-band compressor duck everything else.
fn kick(t: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let since = t % KICK_PERIOD_S;
    let env = (-since / KICK_DECAY_S).exp();
    0.9 * env * (tau * 60.0 * t).sin()
}

/// Everything the kick must NOT duck: a mid pad and a high shimmer, both at a constant level
/// for the whole clip. `detune` splits L from R so the clip is genuinely stereo.
fn steady(t: f32, detune: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let pad = 0.12 * (tau * (220.0 + detune) * t).sin() + 0.09 * (tau * (330.0 + detune) * t).sin();
    let shimmer = 0.05 * (tau * 6_000.0 * t).sin() + 0.035 * (tau * 9_000.0 * t).sin();
    pad + shimmer
}

/// The smoke's material. A function, not a literal, so the gate below can prove that **this
/// exact clip** exposes the effect — a smoke that does not reproduce the problem is a smoke
/// that wastes the one thing only a human can do.
pub(crate) fn smoke_clip() -> SampleData {
    let frames = SR as usize * SECS;
    SampleData::from_fn(frames * 2, AudioFormat::stereo(SR), |i| {
        let t = (i / 2) as f32 / SR as f32;
        let detune = if i % 2 == 0 { 0.0 } else { 0.7 };
        (kick(t) + steady(t, detune)).clamp(-1.0, 1.0)
    })
}

impl AudioSystem {
    /// Stage the clip **and** the A/B rack. See the module docs.
    pub(crate) fn editor_multiband_smoke(&mut self) {
        let data = smoke_clip();

        let _ = self.engine.stop_preview();
        self.editor.name = "multiband-smoke".to_string();
        self.editor.clip = Some(ph2d_audio_edit::EditClip::new(data));
        self.editor.state = EditorTransport::Stopped;
        self.editor.scrub_frame = Some(0);

        // Both stages at the SAME setting: Ratio hard right. Identical by construction, so the
        // A/B compares the two designs and not two sets of numbers.
        let armed = |name: &str| -> Option<FxStage> {
            let kind = KINDS.iter().position(|k| k.name == name)?;
            let mut norms = default_norms(kind);
            norms[1] = 1.0; // Ratio, fully clockwise (20:1) — index 1 in both specs.
            Some(FxStage {
                kind,
                norms,
                enabled: false,
            })
        };
        let Some(mut multiband) = armed("Multiband") else {
            println!("audio: multiband smoke: no Multiband in the rack");
            return;
        };
        let Some(compress) = armed("Compress") else {
            println!("audio: multiband smoke: no Compress in the rack");
            return;
        };
        multiband.enabled = true; // ...and Compress stays bypassed: flip them to A/B.
        set_fx_chain(vec![multiband, compress]);

        println!(
            "audio: multiband smoke staged (PH2D_AUDIO_MULTIBAND_SMOKE)\n  \
             clip: a 60 Hz kick every 0.5 s over a STEADY pad + shimmer, 25 dB below it\n  \
             rack: [Multiband: on] [Compress: bypassed] -- both at Ratio hard right\n  \
             A/B:  flip the per-stage enable. On Compress the pad and shimmer pump at 120 BPM,\n  \
                   once per kick. On Multiband they sit still and only the kick is tamed."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::dsp::{Biquad, BiquadCoeffs};
    use ph2d_audio_edit::{EditClip, Effect};

    /// How much the STEADY high content moves, peak-to-trough, as a fraction of its own level.
    ///
    /// The pad and the shimmer are constant in the source, so a compressor is the only thing
    /// that can make them breathe: this number IS the ducking. Measured above 3 kHz (well clear
    /// of the kick), in 20 ms windows, skipping the first half-second so the envelope has
    /// settled.
    fn pumping(d: &SampleData) -> f32 {
        let sr = SR as f32;
        // Two cascaded high-passes: steep enough that the 60 Hz kick contributes nothing here.
        let c = BiquadCoeffs::highpass(sr, 3_000.0, 0.707);
        let (mut a, mut b) = (Biquad::new(c), Biquad::new(c));
        let win = (sr * 0.02) as usize;
        let skip = (sr * 0.5) as usize;
        let mut levels = Vec::new();
        let (mut acc, mut n) = (0.0f32, 0usize);
        for (f, x) in d.samples().iter().step_by(2).enumerate() {
            let y = b.process(a.process(*x));
            acc += y * y;
            n += 1;
            if n == win {
                if f >= skip {
                    levels.push((acc / win as f32).sqrt());
                }
                acc = 0.0;
                n = 0;
            }
        }
        let hi = levels.iter().copied().fold(0.0f32, f32::max);
        let lo = levels.iter().copied().fold(f32::MAX, f32::min);
        if hi <= f32::EPSILON {
            0.0
        } else {
            (hi - lo) / hi
        }
    }

    /// **The smoke reproduces the problem it is a smoke for.**
    ///
    /// If this clip did not make the plain compressor duck the highs, Enio would flip the two
    /// stages, hear nothing, and the only evidence that the Multiband works would be my word
    /// for it. So: the SAME material, the SAME Ratio, and the two designs have to disagree.
    #[test]
    fn the_smoke_clip_makes_the_plain_compressor_duck_the_highs() {
        let d = smoke_clip();
        let clip = EditClip::new(d.clone());
        // Ratio hard right (20:1) on both — exactly what the staged rack hands the user.
        let knobs = (0.3f32, 20.0f32, 0.005f32, 0.1f32);
        let single = clip.render_effect(Effect::Compress {
            threshold: knobs.0,
            ratio: knobs.1,
            attack_secs: knobs.2,
            release_secs: knobs.3,
        });
        let multi = clip.render_effect(Effect::Multiband {
            threshold: knobs.0,
            ratio: knobs.1,
            attack_secs: knobs.2,
            release_secs: knobs.3,
        });

        let (dry, one, three) = (pumping(&d), pumping(&single), pumping(&multi));
        println!(
            "\nhigh-band movement (0 = rock steady):\n  \
             dry        {dry:.3}\n  Compress   {one:.3}   <- the ducking\n  \
             Multiband  {three:.3}\n"
        );

        // The source really is steady, or the rest of this measures the material.
        assert!(
            dry < 0.05,
            "the smoke clip's highs are not steady: {dry:.3}"
        );
        // The plain compressor ducks them...
        assert!(
            one > 0.30,
            "the smoke clip does not make Compress duck the highs ({one:.3}) — flipping the two \
             stages would sound the same and the smoke would prove nothing"
        );
        // ...and the multiband, at the SAME ratio, leaves them alone.
        assert!(
            three < one / 3.0,
            "the Multiband ducks the highs nearly as much as Compress ({three:.3} vs {one:.3}) — \
             the bands are not independent"
        );
    }
}
