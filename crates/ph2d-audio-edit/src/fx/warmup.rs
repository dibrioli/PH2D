//! `Effect::warmup_frames` — how much audio each effect needs to have already heard.
//!
//! When an effect is applied to a mid-clip SELECTION, it starts at the region's first
//! sample with all its state empty: a filter's delay line is zeroed, a delay line holds
//! silence, a limiter's gain curve sits at 1.0. The result is an effect that *fades in*
//! across the first few milliseconds of the region — and, at the boundary, a seam you can
//! hear.
//!
//! The fix is to render some of the audio BEFORE the region and throw it away
//! ([`crate::in_range_warm`]): the state arrives at the region already settled, exactly as
//! it would have been if the effect had been running the whole time. This file is the table
//! of how much each effect needs.
//!
//! A separate file from `fx.rs` because that one had reached its 700-LOC cap, and this is
//! the seam that was actually there: everything here answers one question, and nothing else
//! in the rack asks it.

use super::{
    CHORUS_BASE_MS, Effect, FLANGER_BASE_MS, HUM_Q, VIBRATO_BASE_MS, autowah, biquad_warmup,
    declick, declip, deess, formant, tone, wsola,
};

impl Effect {
    /// Frames of *preceding* audio this effect needs to enter a mid-clip region
    /// already settled — see [`crate::in_range_warm`]. Zero for memoryless effects
    /// and for the compressor, which settles its own envelope via `prime()`.
    ///
    /// A 2nd-order section rings for roughly `Q / (π·f0)` seconds per time
    /// constant, so a low cutoff at high `Q` needs a long pre-roll; the cap keeps
    /// the extra render bounded. The limiter needs its whole look-ahead window, or
    /// its gain curve starts flat at 1.0 and lets the region's first peak through.
    pub fn warmup_frames(&self, sample_rate: u32) -> usize {
        /// Time constants of pre-roll: `1 - e^-8` ≈ 0.9997 of the way settled.
        const TAUS: f32 = 8.0;
        if self.is_bypass() {
            return 0;
        }
        let cap = sample_rate as usize; // 1 s
        match *self {
            Effect::LowPass { cutoff, q } | Effect::HighPass { cutoff, q } => {
                biquad_warmup(cutoff, q, sample_rate, TAUS).min(cap)
            }
            Effect::Peak { freq, q, .. }
            | Effect::LowShelf { freq, q, .. }
            | Effect::HighShelf { freq, q, .. } => {
                biquad_warmup(freq, q, sample_rate, TAUS).min(cap)
            }
            // The de-esser's band split is a biquad too. Without a pre-roll its
            // sidechain opens on a filter transient and ducks audio that never had
            // an "S" in it.
            Effect::DeEss { freq, .. } | Effect::DePlosive { freq, .. } => {
                biquad_warmup(freq, deess::SPLIT_Q, sample_rate, TAUS).min(cap)
            }
            // Narrow (high-Q) cuts at a low frequency ring for a while; pre-roll the
            // fundamental's notch (the longest). Harmonics settle faster.
            Effect::DeHum { freq, .. } => biquad_warmup(freq, HUM_Q, sample_rate, TAUS).min(cap),
            Effect::Limiter { release_secs, .. } => {
                ((release_secs.max(0.0) * sample_rate as f32) as usize).min(cap)
            }
            // Chorus / flanger read a delay line that starts empty at the region
            // edge; without a pre-roll their first `base + depth` ms of wet signal is
            // silence, so the effect swells in (an audible edge). Fill it.
            Effect::Chorus { depth_ms, .. } => {
                (((CHORUS_BASE_MS + depth_ms) * 0.001 * sample_rate as f32) as usize).min(cap)
            }
            Effect::Flanger { depth_ms, .. } => {
                (((FLANGER_BASE_MS + depth_ms) * 0.001 * sample_rate as f32) as usize).min(cap)
            }
            // Same as chorus/flanger — the doubler's delay line starts empty.
            Effect::Doubler {
                delay_ms,
                detune_ms,
                ..
            } => (((delay_ms + detune_ms) * 0.001 * sample_rate as f32) as usize).min(cap),
            // The pitch shifter's delay line starts empty; without a pre-roll its taps
            // read silence for the first grain and the region swells in. Fill it.
            // The harmonizer is two of them.
            Effect::PitchShift { .. } | Effect::Harmonizer { .. } => wsola::WINDOW.min(cap),
            // Block-based, so the region's first block would otherwise be modelled
            // against assumed silence: for the formant shifter that means an envelope
            // fitted to a fade-in, and for the de-clicker it means the edge itself
            // looks like damage and gets "repaired". Pre-roll one analysis block.
            Effect::FormantShift { .. } => formant::BLOCK.min(cap),
            Effect::DeClick { .. } => declick::BLOCK.min(cap),
            // De-Clip refits its AR model per block, like De-Click: it needs a block of
            // preceding audio or the model entering the region has heard nothing.
            Effect::DeClip { .. } => declip::BLOCK.min(cap),
            // Distortion's post low-pass would click at a selection edge without a
            // pre-roll; size it on the darkest (longest-ringing) tone.
            Effect::Distortion { .. } => {
                biquad_warmup(tone::DISTORT_TONE_MIN_HZ, 0.707, sample_rate, TAUS).min(cap)
            }
            // The exciter's high-pass is a biquad — same edge-click risk.
            Effect::Exciter { freq, .. } => biquad_warmup(freq, 0.707, sample_rate, TAUS).min(cap),
            // The auto-wah's band-pass rings longest at its base (lowest) cut-off.
            Effect::AutoWah { base_freq, .. } => {
                biquad_warmup(base_freq, autowah::WAH_Q, sample_rate, TAUS).min(cap)
            }
            // The Haas widener reads `delay_ms` frames of the right channel before the
            // region; pre-roll them so a mid-clip selection's right side isn't silence.
            Effect::Haas { delay_ms } => {
                ((delay_ms.max(0.0) * 0.001 * sample_rate as f32) as usize).min(cap)
            }
            // Vibrato is a modulated delay line that starts empty (like chorus/flanger).
            Effect::Vibrato { depth_ms, .. } => {
                (((VIBRATO_BASE_MS + depth_ms) * 0.001 * sample_rate as f32) as usize).min(cap)
            }
            // The compressor and the gate prime their own envelope on the region's
            // first frame; the phaser's all-pass state settles in a handful of
            // samples; tremolo is memoryless. None need a pre-roll.
            _ => 0,
        }
    }
}
