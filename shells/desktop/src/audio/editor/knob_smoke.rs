//! **The knob-drag smoke** (ADR-0120) — the one feature in this line that has nothing to hear.
//!
//! # Why this scene has to exist
//!
//! The incremental preview is **byte-identical** to the full render, on purpose: if you can hear a
//! difference, it is a bug. So there is no A/B of *sound* to be had — the thing that changed is
//! the **cost of a frame while your finger is on a knob**, and that only shows up on a clip long
//! enough for the whole-clip copy to hurt. On a 3-second clip both paths are instant and the smoke
//! would prove nothing.
//!
//! So the scene stages the case the ADR was written about: **3 minutes of stereo audio**, a
//! selection, and a rack with **exactly one** audible Plain stage — which is what a real knob drag
//! is, and what the fast path requires.
//!
//! # How to run it (both halves — one alone is a demo, not a test)
//!
//! ```text
//! PH2D_AUDIO_KNOB_SMOKE=1                          cargo run --release -p ph2d-host-desktop
//! PH2D_AUDIO_KNOB_SMOKE=1 PH2D_AUDIO_SLOW_PREVIEW=1 cargo run --release -p ph2d-host-desktop
//! ```
//!
//! Open the **FX** section of the Audio Editor and drag **Ratio**. The scene turns the frame log
//! on for itself, so each drag frame prints what it cost and which path paid it. With
//! `PH2D_AUDIO_SLOW_PREVIEW=1` the fast path refuses and every frame goes down the old whole-clip
//! render — the drag stutters, and the log says `OVER BUDGET`.
//!
//! **The first two frames of a drag are expensive on BOTH paths, and that is by design**: they
//! fill the two scratch slots, one whole-clip copy each, and only from the third frame on is the
//! drag free. The log labels those `warm-up` and does not accuse them — a design that pays a cost
//! on purpose should say so out loud, not hide it. What you are looking for is the **steady
//! state**: keep dragging.
//!
//! # The gate below, and why it is not optional
//!
//! The fast path fires **only** for a lone audible Plain stage. Stage two effects by accident and
//! it silently never fires: the drag is slow, the log says `full render`, and the smoke measures
//! the thing it was built to disprove — while every other gate stays green. So the gate asserts
//! that **this exact staged rack** takes the fast path, and that the clip is long enough for the
//! copy to be the thing that hurts.

use ph2d_audio::{AudioFormat, SampleData};
use ph2d_panel_audio_editor::{FxStage, set_fx_chain};

use super::super::fx_params::default_norms;
use super::super::fx_params_table::KINDS;
use crate::audio::AudioSystem;
use crate::audio::editor::EditorTransport;

const SR: u32 = 48_000;
/// The clip the ADR measured: three minutes, which is 65.9 MB of f32 — the copy the old path paid
/// **every frame**.
const SECS: usize = 180;

/// The selection: two seconds, early enough that you hear it right after pressing play.
///
/// Two seconds out of 180 is the whole point — the DSP touches 1.1 % of the clip, and the old path
/// still copied the other 98.9 % to change it.
const SEL_START_S: f32 = 1.0;
const SEL_END_S: f32 = 3.0;

/// A groove with real dynamics: a kick, a backbeat, a bass line and a pad. A compressor needs
/// something with **peaks over a bed** to do anything audible — on a steady tone, Ratio does
/// nothing you can hear, and you would be dragging a knob that appears to be broken.
fn groove(t: f32, detune: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let bar = t % 2.0; // a 2-second bar, 120 BPM
    let beat = |at: f32| (bar - at).max(0.0);

    // Kick on 1 and 3: a fast pitch drop, the loudest thing in the clip.
    let k = beat(0.0).min(beat(1.0));
    let kick = (-28.0 * k).exp() * (tau * (55.0 + 90.0 * (-40.0 * k).exp()) * k).sin() * 0.9;

    // Snare on 2 and 4: noise-ish, from a cluster of detuned partials (no RNG needed).
    let s = beat(0.5).min(beat(1.5));
    let snare = (-30.0 * s).exp()
        * 0.5
        * ((tau * 1_800.0 * s).sin() + (tau * 2_930.0 * s).sin() + (tau * 4_470.0 * s).sin())
        / 3.0;

    // Bass: two notes per bar, well under the drums.
    let note = if bar < 1.0 { 82.41 } else { 110.0 }; // E2, A2
    let bass = 0.22 * (tau * (note + detune) * t).sin();

    // Pad: the bed the compressor will pump if the Ratio is cranked.
    let pad =
        0.10 * (tau * (329.63 + detune) * t).sin() + 0.08 * (tau * (493.88 + detune) * t).sin();

    kick + snare + bass + pad
}

/// The smoke's material — a function, not a literal, so the gate can prove **this exact clip**
/// makes the copy the expensive part.
pub(crate) fn smoke_clip() -> SampleData {
    let frames = SR as usize * SECS;
    SampleData::from_fn(frames * 2, AudioFormat::stereo(SR), |i| {
        let t = (i / 2) as f32 / SR as f32;
        let detune = if i % 2 == 0 { 0.0 } else { 0.6 };
        groove(t, detune).clamp(-1.0, 1.0)
    })
}

/// The staged rack: **one** audible Plain stage, Compress, with Ratio somewhere in the middle so
/// there is room to drag it in both directions.
pub(crate) fn smoke_chain() -> Option<Vec<FxStage>> {
    let kind = KINDS.iter().position(|k| k.name == "Compress")?;
    let mut norms = default_norms(kind);
    norms[0] = 0.35; // Threshold, low enough that the kick is well into it
    norms[1] = 0.55; // Ratio -- the knob to drag
    Some(vec![FxStage {
        kind,
        norms,
        enabled: true,
    }])
}

impl AudioSystem {
    /// Stage the 3-minute clip, the selection and the one-stage rack. See the module docs.
    pub(crate) fn editor_knob_smoke(&mut self) {
        let Some(chain) = smoke_chain() else {
            println!("audio: knob smoke: no Compress in the rack");
            return;
        };
        let data = smoke_clip();

        let _ = self.engine.stop_preview();
        self.editor.name = "knob-smoke".to_string();
        let mut clip = ph2d_audio_edit::EditClip::new(data);
        clip.set_selection(Some(
            (SEL_START_S * SR as f32) as usize..(SEL_END_S * SR as f32) as usize,
        ));
        self.editor.clip = Some(clip);
        self.editor.state = EditorTransport::Stopped;
        self.editor.scrub_frame = Some(0);
        set_fx_chain(chain);

        let slow = std::env::var_os("PH2D_AUDIO_SLOW_PREVIEW").is_some();
        let (path, ab) = if slow {
            (
                "SLOW forced -- the pre-ADR-0120 whole-clip render",
                "A/B:  now re-run WITHOUT PH2D_AUDIO_SLOW_PREVIEW to get the fast path back.",
            )
        } else {
            (
                "ADR-0120 -- the region rewrite (this is the shipping path)",
                "A/B:  re-run with PH2D_AUDIO_SLOW_PREVIEW=1 to force the OLD whole-clip render.",
            )
        };
        println!(
            "audio: knob smoke staged (PH2D_AUDIO_KNOB_SMOKE)\n  \
             clip: {SECS}s stereo groove (kick/snare/bass/pad), selection {SEL_START_S}s..{SEL_END_S}s\n  \
             rack: [Compress] -- ONE audible stage, which is what the fast path needs\n  \
             now:  open FX and drag RATIO. Every drag frame prints what it cost.\n  \
             path: {path}\n  \
             {ab}\n  \
             ears: the two paths are byte-identical -- if you HEAR a difference, that is a bug."
        );
        // The log IS the point of this scene, so it turns itself on -- asking the Enio to remember
        // a second env var is asking him to run the smoke wrong.
        super::fx_rack::enable_preview_log();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::editor::fx_preview::lone_plain_stage;

    /// **The gate that makes this smoke a smoke.** The staged rack must take the FAST path — the
    /// one this ADR is about. Stage two audible effects by accident and the fast path silently
    /// refuses (it needs a lone Plain stage), the drag is slow, and the scene would be
    /// demonstrating the *old* behaviour while claiming to show the new one. Every other gate in
    /// the tree would stay green.
    #[test]
    fn the_staged_rack_is_one_the_fast_path_will_actually_take() {
        let chain = smoke_chain().expect("Compress is in the rack");
        assert!(
            lone_plain_stage(&chain, 0).is_some(),
            "the knob smoke staged a rack the ADR-0120 path REFUSES -- it would drag slowly and \
             the log would say `full render`, which is the opposite of what this scene exists to \
             show"
        );
    }

    /// **The clip has to be long enough for the copy to be the thing that hurts.** The whole claim
    /// of ADR-0120 is that the whole-clip copy dwarfs the DSP; on a short clip both paths are
    /// instant and a human would feel nothing. A smoke whose fixture cannot expose the problem is
    /// a smoke that wastes the one thing only a human can do.
    #[test]
    fn the_clip_is_long_enough_that_the_copy_is_what_hurts() {
        let clip = smoke_clip();
        let mb = (clip.samples().len() * 4) as f64 / 1_048_576.0;
        let sel_frac = (SEL_END_S - SEL_START_S) / SECS as f32;
        println!(
            "knob smoke: {mb:.1} MB, selection = {:.1} % of the clip",
            sel_frac * 100.0
        );
        assert!(
            mb > 50.0,
            "the staged clip is only {mb:.1} MB -- the whole-clip copy is what ADR-0120 removes, \
             and at this size it costs nothing to begin with"
        );
        assert!(
            sel_frac < 0.05,
            "the selection is {:.0} % of the clip -- the DSP would dominate and the copy this ADR \
             removes would not be the interesting number",
            sel_frac * 100.0
        );
    }
}
