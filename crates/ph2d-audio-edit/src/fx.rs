//! Offline effect processors for the editor (W3 §6 rack). Two families, split by
//! whether the output outlives the input:
//!
//! - [`Effect`] — **length-preserving** (filters, EQ, dynamics, character). Routes
//!   through [`crate::in_range_warm`] just like the W2 ops, so applying to a
//!   selection processes only the selected samples — with a pre-roll of the audio
//!   before it, so a filter enters the region already settled instead of clicking.
//! - [`TailEffect`] — **tail-extending** (reverb, delay). Cannot use the
//!   length-preserving splice: it routes through [`crate::in_range_tail`], which
//!   rings the tail out over the audio after the range and grows the clip when the
//!   range touches the end.
//!
//! Both are control-thread only, so HR-3/HR-5 do not apply (they allocate and use
//! `tanh`/`exp` freely). Filters, dynamics, reverb and delay reuse the shared
//! `ph2d_audio::dsp` kit; character effects (saturate/bitcrush/width) and the
//! true-peak limiter are local.
//!
//! Implementations live in the sibling modules ([`tone`], [`dynamics`], [`space`]);
//! this file is the two enums and the neutral points they promise. The restoration and
//! voice-transform end of the rack ([`declick`], [`formant`]) shares one linear-
//! prediction core ([`lpc`]): the same AR fit that tells a click from the signal tells
//! the vocal tract from the pitch.

mod autowah;
mod comb;
mod conv;
mod declick;
mod declip;
mod deess;
mod deplosive;
mod dynamics;
mod formant;
mod harmonize;
mod lpc;
mod modulation;
mod space;
mod tail;
mod tone;
mod transient;
mod warmup;
pub(crate) mod wsola;

pub use tail::TailEffect;

use ph2d_audio::SampleData;
use ph2d_audio::dsp::BiquadCoeffs;

use autowah::auto_wah;
use comb::comb;
use declick::declick;
use declip::declip;
use deess::deess;
use deplosive::deplosive;
use dynamics::{compress, gate, leveler, limit};
use formant::formant_shift;
use harmonize::harmonize;
use modulation::{auto_pan, doubler, modulated_delay, phaser, ring_mod, trance_gate, tremolo};
use tone::{HUM_Q, biquad_all, bitcrush, dehum, distortion, exciter, haas, saturate, stereo_width};
use transient::{TRANSIENT_BYPASS, transient_shape};
use wsola::{PITCH_BYPASS_ST, pitch_shift};

/// Chorus base delay (ms): a detuned doubling sits well behind the dry signal.
const CHORUS_BASE_MS: f32 = 18.0;
/// Flanger base delay (ms): a sweeping comb needs a very short tap.
const FLANGER_BASE_MS: f32 = 2.0;
/// Vibrato base delay (ms): small — just enough headroom for the pitch sweep to stay
/// on a positive tap. Vibrato is a fully-wet modulated delay (no dry, no feedback).
const VIBRATO_BASE_MS: f32 = 5.0;

// Every effect must be a **byte-identical no-op** at its neutral parameters, so
// the editor can select it (and audition it) without touching the audio until the
// user actually turns something. "Almost identity" is not enough: a filter at the
// top of its range still phase-shifts, and a 1:1 compressor still rounds. Each
// neutral point below is therefore an explicit bypass, not an emergent one.

/// A low-pass at or above this cutoff passes the whole audible band → bypass.
const LOWPASS_BYPASS_HZ: f32 = 20_000.0;
/// A high-pass at or below this cutoff passes the whole audible band → bypass.
const HIGHPASS_BYPASS_HZ: f32 = 20.0;
/// An EQ band this close to flat is inaudible — and a "0 dB" RBJ section is only
/// *algebraically* an identity: its coefficients still round every sample.
const EQ_BYPASS_GAIN_DB: f32 = 1e-3;
/// A limiter whose ceiling is at (or above) full scale has nothing to catch that a
/// `[-1, 1]` buffer could contain in its samples. Dial it below 0 dBFS to work —
/// the mastering convention is −1 dBTP.
const LIMITER_BYPASS_CEILING_DB: f32 = 0.0;
/// Drive below this is inaudible (and `tanh(k)` underflows the normalizer).
const SATURATE_BYPASS_DRIVE: f32 = 1e-3;
/// Distortion drive (0..1) below this is a clean pass-through → bypass.
const DISTORT_BYPASS_DRIVE: f32 = 1e-4;
/// Bit depth at or above this, with no decimation, is transparent → bypass.
const BITCRUSH_BYPASS_BITS: u32 = 16;
/// A modulation effect fully dry (`mix`/`depth` at 0) passes the signal through.
const MOD_BYPASS: f32 = 0.0;
/// De-Hum at `depth` 0 makes 0 dB cuts (a pass-through), so it is the neutral point.
const HUM_BYPASS_DEPTH: f32 = 0.0;
/// The leveler at `amount` 0 leaves the gain at unity — a pass-through.
const LEVELER_BYPASS_AMOUNT: f32 = 0.0;
/// A de-clicker at `sensitivity` 0 detects nothing, so it repairs nothing. Off is the
/// neutral point rather than "very insensitive": a repair tool must be *asked* for.
const DECLICK_BYPASS_SENS: f32 = 0.0;

/// De-Clip is inert until the user asks for a repair. Same shape as De-Click's: a
/// restoration tool starts OFF, because there is nothing to restore in most clips.
const DECLIP_BYPASS_AMOUNT: f32 = 0.0;

/// A single length-preserving offline effect. Each variant has a **neutral point**
/// at which [`Effect::apply`] returns its input untouched (see
/// [`Effect::is_bypass`]) — that is where the editor's sliders start, so selecting
/// an effect never alters the audio until the user turns something.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Effect {
    /// 2nd-order low-pass (muffle / warm) at `cutoff` Hz, `q` resonance.
    LowPass { cutoff: f32, q: f32 },
    /// 2nd-order high-pass (thin / de-rumble) at `cutoff` Hz, `q` resonance.
    HighPass { cutoff: f32, q: f32 },
    /// Peaking bell: `gain_db` boost/cut centred on `freq` Hz, width set by `q`.
    /// Leaves DC and Nyquist alone. Neutral at `gain_db` 0.
    Peak { freq: f32, q: f32, gain_db: f32 },
    /// Low shelf: `gain_db` applied below the `freq` Hz corner. Neutral at 0 dB.
    LowShelf { freq: f32, q: f32, gain_db: f32 },
    /// High shelf: `gain_db` applied above the `freq` Hz corner. Neutral at 0 dB.
    HighShelf { freq: f32, q: f32, gain_db: f32 },
    /// **De-Hum**: removes mains buzz — a cascade of deep narrow cuts at `freq`
    /// (50 EU / 60 US) and its harmonics. `depth` (0..1) sets the attenuation
    /// (0 = bypass); `harmonics` how many overtones to chase. Neutral at `depth` 0.
    DeHum {
        freq: f32,
        depth: f32,
        harmonics: u32,
    },
    /// Feed-forward compressor (glue / level). Make-up gain is **automatic and
    /// peak-preserving**: the compressed region is scaled back so its peak matches
    /// the input's, so raising `ratio` reduces the dynamic range (quiet parts come
    /// up) without ever raising the waveform's amplitude. Neutral at `ratio` 1.
    Compress {
        /// Level (linear 0..1) above which reduction starts.
        threshold: f32,
        /// Ratio (≥1).
        ratio: f32,
        /// Attack time (seconds).
        attack_secs: f32,
        /// Release time (seconds).
        release_secs: f32,
    },
    /// Downward expander — a noise **gate** at a high ratio. Passes what is above
    /// `threshold` and pulls down what is below it by `(level/threshold)^(ratio−1)`.
    /// `attack_secs` opens the gate, `release_secs` closes it. Neutral at `ratio` 1.
    Gate {
        /// Level (linear 0..1) below which expansion starts.
        threshold: f32,
        /// Expansion ratio (≥1). 1 is a no-op; ~16 is a hard gate.
        ratio: f32,
        /// How fast the gate opens (seconds).
        attack_secs: f32,
        /// How slowly it closes (seconds).
        release_secs: f32,
    },
    /// Split-band **de-esser**: compresses only the band above `freq`, so an "S" is
    /// tamed without ducking the vowel under it. Neutral at `ratio` 1.
    DeEss {
        /// Crossover into the sibilance band (Hz).
        freq: f32,
        /// Level (linear) the high band must exceed before ducking starts.
        threshold: f32,
        /// Compression ratio applied to the high band (≥1).
        ratio: f32,
        /// Recovery time (seconds). Attack is fixed fast — sibilance is a transient.
        release_secs: f32,
    },
    /// Split-band **de-plosive**: the mirror of the de-esser — ducks the low "pop"
    /// band below `freq` when a `p`/`b` breath burst spikes it, leaving the voice's
    /// body alone. Neutral at `ratio` 1.
    DePlosive {
        /// Crossover into the plosive band (Hz), ~80–200.
        freq: f32,
        /// Level (linear) the low band must exceed before ducking starts.
        threshold: f32,
        /// Compression ratio applied to the low band (≥1).
        ratio: f32,
        /// Recovery time (seconds). Attack is fixed fast — a plosive is a transient.
        release_secs: f32,
    },
    /// Look-ahead **true-peak** limiter: holds the *reconstructed* waveform under
    /// `ceiling_db`, not just the samples. Neutral at 0 dBFS.
    Limiter {
        /// Ceiling in dBFS (≤ 0). `−1.0` is the mastering convention.
        ceiling_db: f32,
        /// Look-ahead and recovery radius, in seconds.
        release_secs: f32,
    },
    /// **Leveler / AGC**: slow automatic gain toward `target_db` RMS, evening out a
    /// voice's loudness over time (±12 dB range). `amount` (0..1) crossfades the gain
    /// toward unity; `speed_secs` is the envelope time constant. Neutral at `amount` 0.
    Leveler {
        /// Target RMS level (dBFS).
        target_db: f32,
        /// How much of the computed gain to apply (0 = bypass … 1 = full).
        amount: f32,
        /// Envelope time constant (seconds): larger = slower, gentler ride.
        speed_secs: f32,
    },
    /// **Transient shaper**: boosts or softens a hit's attack and sustain
    /// independently, via a fast/slow amplitude-follower pair. Level-independent, so a
    /// quiet hit shapes like a loud one and steady tone is untouched. Neutral at
    /// `attack` 0 and `sustain` 0.
    Transient {
        /// Attack shaping (−1 soften … 0 off … +1 punch).
        attack: f32,
        /// Sustain shaping (−1 tighten … 0 off … +1 fill).
        sustain: f32,
    },
    /// `tanh` soft-clip saturation (warmth / drive). Neutral at `drive` 0.
    Saturate { drive: f32 },
    /// **Distortion**: a cubic hard-clipper (grittier than `Saturate`) with a post
    /// low-pass **Tone**. `drive` (0..1) blends dry→clipped and drives harder; `tone`
    /// tames the fizz. Neutral at `drive` 0.
    Distortion { drive: f32, tone: f32 },
    /// **Exciter**: adds high-frequency sparkle by generating harmonics of the band
    /// above `freq` and mixing them back in by `amount`. Neutral at `amount` 0.
    Exciter { freq: f32, amount: f32 },
    /// **Vibrato**: pitch modulation — a fully-wet swept delay line at `rate`, sweep
    /// `depth_ms`. Neutral at `depth_ms` 0 (a fixed delay is bypassed).
    Vibrato { rate: f32, depth_ms: f32 },
    /// **Comb resonator**: a tuned feedback comb ringing at `freq` (and harmonics),
    /// `feedback` sets the resonance, blended by `mix`. Neutral at `mix` 0.
    Comb { freq: f32, feedback: f32, mix: f32 },
    /// **Auto-wah**: an envelope-following resonant band-pass rooted at `base_freq`,
    /// `sens` the sweep depth, blended by `mix`. Neutral at `mix` 0.
    AutoWah { base_freq: f32, sens: f32, mix: f32 },
    /// **Haas widener**: delays the right channel by `delay_ms` for stereo width.
    /// Neutral at `delay_ms` 0 (and on mono clips).
    Haas { delay_ms: f32 },
    /// Lo-fi bit-depth reduction to `bits` + sample-hold decimation by
    /// `downsample` (≥1 = no decimation).
    Bitcrush { bits: u32, downsample: u32 },
    /// Mid/Side stereo width (`1.0` = unchanged, `>1` wider, `0` mono). No-op on
    /// mono clips.
    StereoWidth { width: f32 },
    /// **Chorus**: a modulated delay detuned against the dry signal, thickening it.
    /// Neutral at `mix` 0 (fully dry). No feedback (that is the flanger's job).
    Chorus {
        /// LFO rate (Hz).
        rate: f32,
        /// Sweep depth (ms of delay modulation).
        depth_ms: f32,
        /// Dry→wet crossfade (0..1).
        mix: f32,
    },
    /// **Flanger**: a short modulated comb with feedback — the sweeping jet whoosh.
    /// Neutral at `mix` 0.
    Flanger {
        /// LFO rate (Hz).
        rate: f32,
        /// Sweep depth (ms of delay modulation).
        depth_ms: f32,
        /// Comb feedback (0..0.95).
        feedback: f32,
        /// Dry→wet crossfade (0..1).
        mix: f32,
    },
    /// **Phaser**: a swept all-pass cascade mixed with the dry signal — moving
    /// notches, a swirl. Neutral at `mix` 0.
    Phaser {
        /// LFO rate (Hz).
        rate: f32,
        /// Sweep span (0..1) of the notch frequencies.
        depth: f32,
        /// Dry→wet crossfade (0..1).
        mix: f32,
    },
    /// **Tremolo**: amplitude modulation. Neutral at `depth` 0 (gain stays at 1.0).
    Tremolo {
        /// LFO rate (Hz).
        rate: f32,
        /// Modulation depth (0..1); 1 dips to silence at each trough.
        depth: f32,
    },
    /// **Auto-pan**: a counter-phase stereo LFO that walks the image left↔right.
    /// Neutral at `depth` 0 (both channels stay at unity).
    AutoPan {
        /// LFO rate (Hz).
        rate: f32,
        /// Sweep depth (0..1); 1 walks fully hard-left to hard-right.
        depth: f32,
    },
    /// **Trance gate**: a rhythmic on/off amplitude gate at `rate`, edges smoothed by
    /// `smooth_secs`. Neutral at `depth` 0 (gain stays at unity).
    TranceGate {
        /// Gate rate (Hz).
        rate: f32,
        /// How far the "off" phase ducks (0..1); 1 chops to silence.
        depth: f32,
        /// Edge smoothing time (seconds): short = hard stutter, long = tremolo.
        smooth_secs: f32,
    },
    /// **Doubler** (ADT): two detuned, slow-drifting delayed copies hard-panned L/R and
    /// mixed with the dry signal — a faked second take. Neutral at `mix` 0.
    Doubler {
        /// Base delay of the doubled voice (ms).
        delay_ms: f32,
        /// Detune sweep depth (ms).
        detune_ms: f32,
        /// Dry→wet crossfade (0..1).
        mix: f32,
    },
    /// **Ring modulator**: multiplies the signal by an audio-rate sine carrier at
    /// `freq` — the robot / metallic voice. `mix` crossfades dry→wet. Neutral at
    /// `mix` 0 (fully dry).
    RingMod {
        /// Carrier frequency (Hz).
        freq: f32,
        /// Dry→wet crossfade (0..1).
        mix: f32,
    },
    /// **Pitch shift**: retunes by `semitones` (±) with a time-domain granular
    /// shifter — the monster / chipmunk / robot voice. Formants move with the pitch.
    /// `mix` crossfades dry→wet. Neutral at `semitones` 0 (or `mix` 0).
    PitchShift {
        /// Shift in semitones (negative = down, positive = up).
        semitones: f32,
        /// Dry→wet crossfade (0..1).
        mix: f32,
    },
    /// **Formant shift**: moves the spectral envelope — the *size of the head* — by
    /// `semitones`, leaving the excitation (and therefore the pitch) alone. Down is a
    /// giant, up is something small, and the melody does not budge. `mix` crossfades
    /// dry→wet. Neutral at `semitones` 0 (or `mix` 0).
    ///
    /// The complement of [`Effect::PitchShift`], where formants ride *with* the pitch:
    /// chain the two to pitch a voice while choosing its size independently.
    FormantShift {
        /// Envelope shift in semitones (negative = bigger, positive = smaller).
        semitones: f32,
        /// Dry→wet crossfade (0..1).
        mix: f32,
    },
    /// **Harmonizer**: two pitched copies of the take (`v1_st`, `v2_st`) sung along
    /// with it — a third and a fifth make a triad, an octave pair makes a swarm.
    /// `mix` is the level of the harmony against the dry voice. Neutral at `mix` 0.
    Harmonizer {
        /// First harmony voice, in semitones from the dry note.
        v1_st: f32,
        /// Second harmony voice, in semitones.
        v2_st: f32,
        /// Harmony level (0 = dry … 1 = an even blend of the three voices).
        mix: f32,
    },
    /// **De-Click**: repairs clicks, ticks and crackle — the rack's one *restoration*
    /// tool. A click is what the audio's own model could not predict, so the detector
    /// hunts outliers in the prediction residual and the repair re-derives those
    /// samples from the model: the gap is filled with what the signal *would* have
    /// done, not muted. Neutral at `sensitivity` 0.
    DeClick {
        /// Detection threshold (0 = off … 1 = catches the faintest tick, and the odd
        /// glottal pulse with it).
        sensitivity: f32,
        /// The longest run it will repair (seconds). Past this a "click" is the
        /// signal, and rewriting it would hallucinate audio.
        width_secs: f32,
    },
    /// **De-Clip**: puts back the peaks a converter cut off. Clipping is *destruction* —
    /// the samples above the ceiling were replaced by the ceiling, so the waveform grows a
    /// flat top and nothing in the file remembers how high it was going. Which makes it the
    /// same problem as a click (a gap the signal never got to write), and it is repaired by
    /// the same LSAR interpolation. Neutral at `amount` 0.
    DeClip {
        /// Level (0..1 of full scale) at or above which a FLAT run counts as clipped. The
        /// flatness is what separates a clipped peak from a merely hot one.
        threshold: f32,
        /// How much of the reconstruction to blend in (0 = off).
        amount: f32,
    },
}

impl Effect {
    /// Render `data` through this effect, returning a fresh, **same-length** clip.
    pub fn apply(&self, data: &SampleData) -> SampleData {
        let sr = data.format().sample_rate as f32;
        match *self {
            // Each arm short-circuits at its neutral point (see the bypass consts):
            // the returned clip is then the SAME `Arc<[f32]>`, not a re-render.
            Effect::LowPass { cutoff, q } if cutoff < LOWPASS_BYPASS_HZ => {
                biquad_all(data, BiquadCoeffs::lowpass(sr, cutoff, q))
            }
            Effect::HighPass { cutoff, q } if cutoff > HIGHPASS_BYPASS_HZ => {
                biquad_all(data, BiquadCoeffs::highpass(sr, cutoff, q))
            }
            Effect::Peak { freq, q, gain_db } if gain_db.abs() > EQ_BYPASS_GAIN_DB => {
                biquad_all(data, BiquadCoeffs::peak(sr, freq, q, gain_db))
            }
            Effect::LowShelf { freq, q, gain_db } if gain_db.abs() > EQ_BYPASS_GAIN_DB => {
                biquad_all(data, BiquadCoeffs::lowshelf(sr, freq, q, gain_db))
            }
            Effect::HighShelf { freq, q, gain_db } if gain_db.abs() > EQ_BYPASS_GAIN_DB => {
                biquad_all(data, BiquadCoeffs::highshelf(sr, freq, q, gain_db))
            }
            // Depth 0 is a 0 dB cut = pass-through (and would still round each sample).
            Effect::DeHum {
                freq,
                depth,
                harmonics,
            } if depth > HUM_BYPASS_DEPTH => dehum(data, freq, depth, harmonics),
            // Ratio 1:1 reduces nothing — and would still round the samples.
            Effect::Compress {
                threshold,
                ratio,
                attack_secs,
                release_secs,
            } if ratio > 1.0 => compress(data, threshold, ratio, attack_secs, release_secs),
            // Ratio 1:1 expands nothing: `(level/threshold)^0` is unity everywhere.
            Effect::Gate {
                threshold,
                ratio,
                attack_secs,
                release_secs,
            } if ratio > 1.0 => gate(data, threshold, ratio, attack_secs, release_secs),
            Effect::DeEss {
                freq,
                threshold,
                ratio,
                release_secs,
            } if ratio > 1.0 => deess(data, freq, threshold, ratio, release_secs),
            Effect::DePlosive {
                freq,
                threshold,
                ratio,
                release_secs,
            } if ratio > 1.0 => deplosive(data, freq, threshold, ratio, release_secs),
            Effect::Limiter {
                ceiling_db,
                release_secs,
            } if ceiling_db < LIMITER_BYPASS_CEILING_DB => limit(data, ceiling_db, release_secs),
            // Amount 0 leaves the gain at unity — a pass-through (and would still round).
            Effect::Leveler {
                target_db,
                amount,
                speed_secs,
            } if amount > LEVELER_BYPASS_AMOUNT => leveler(data, target_db, amount, speed_secs),
            Effect::Transient { attack, sustain }
                if attack.abs() > TRANSIENT_BYPASS || sustain.abs() > TRANSIENT_BYPASS =>
            {
                transient_shape(data, attack, sustain)
            }
            Effect::Saturate { drive } if drive >= SATURATE_BYPASS_DRIVE => saturate(data, drive),
            Effect::Distortion { drive, tone } if drive > DISTORT_BYPASS_DRIVE => {
                distortion(data, drive, tone)
            }
            Effect::Exciter { freq, amount } if amount > MOD_BYPASS => exciter(data, freq, amount),
            // Fully wet, no feedback: the swept delay becomes pitch modulation.
            Effect::Vibrato { rate, depth_ms } if depth_ms > MOD_BYPASS => {
                modulated_delay(data, VIBRATO_BASE_MS, depth_ms, rate, 0.0, 1.0)
            }
            Effect::Comb {
                freq,
                feedback,
                mix,
            } if mix > MOD_BYPASS => comb(data, freq, feedback, mix),
            Effect::AutoWah {
                base_freq,
                sens,
                mix,
            } if mix > MOD_BYPASS => auto_wah(data, base_freq, sens, mix),
            Effect::Haas { delay_ms } if delay_ms > MOD_BYPASS => haas(data, delay_ms),
            Effect::Bitcrush { bits, downsample }
                if bits < BITCRUSH_BYPASS_BITS || downsample > 1 =>
            {
                bitcrush(data, bits, downsample)
            }
            // Width 1.0 is algebraically identity, but `mid ± side` still rounds.
            Effect::StereoWidth { width } if (width - 1.0).abs() > f32::EPSILON => {
                stereo_width(data, width)
            }
            // Modulation effects are dry (`_ => clone`) at mix/depth 0.
            Effect::Chorus {
                rate,
                depth_ms,
                mix,
            } if mix > MOD_BYPASS => {
                modulated_delay(data, CHORUS_BASE_MS, depth_ms, rate, 0.0, mix)
            }
            Effect::Flanger {
                rate,
                depth_ms,
                feedback,
                mix,
            } if mix > MOD_BYPASS => {
                modulated_delay(data, FLANGER_BASE_MS, depth_ms, rate, feedback, mix)
            }
            Effect::Phaser { rate, depth, mix } if mix > MOD_BYPASS => {
                phaser(data, rate, depth, mix)
            }
            Effect::Tremolo { rate, depth } if depth > MOD_BYPASS => tremolo(data, rate, depth),
            Effect::AutoPan { rate, depth } if depth > MOD_BYPASS => auto_pan(data, rate, depth),
            Effect::TranceGate {
                rate,
                depth,
                smooth_secs,
            } if depth > MOD_BYPASS => trance_gate(data, rate, depth, smooth_secs),
            Effect::Doubler {
                delay_ms,
                detune_ms,
                mix,
            } if mix > MOD_BYPASS => doubler(data, delay_ms, detune_ms, mix),
            Effect::RingMod { freq, mix } if mix > MOD_BYPASS => ring_mod(data, freq, mix),
            // Needs a real shift AND some wet: either at neutral is a pass-through.
            Effect::PitchShift { semitones, mix }
                if semitones.abs() > PITCH_BYPASS_ST && mix > MOD_BYPASS =>
            {
                pitch_shift(data, semitones, mix)
            }
            // Same neutral pair as the pitch shifter: an unwarped envelope driven by
            // its own residual reconstructs the input, so a zero shift is a no-op.
            Effect::FormantShift { semitones, mix }
                if semitones.abs() > PITCH_BYPASS_ST && mix > MOD_BYPASS =>
            {
                formant_shift(data, semitones, mix)
            }
            Effect::Harmonizer { v1_st, v2_st, mix } if mix > MOD_BYPASS => {
                harmonize(data, v1_st, v2_st, mix)
            }
            Effect::DeClick {
                sensitivity,
                width_secs,
            } if sensitivity > DECLICK_BYPASS_SENS => declick(data, sensitivity, width_secs),
            Effect::DeClip { threshold, amount } if amount > DECLIP_BYPASS_AMOUNT => {
                declip(data, threshold, amount)
            }
            _ => data.clone(),
        }
    }

    /// Whether this effect is at its neutral point and [`Effect::apply`] would
    /// return the input untouched.
    pub fn is_bypass(&self) -> bool {
        matches!(
            *self,
            Effect::LowPass { cutoff, .. } if cutoff >= LOWPASS_BYPASS_HZ)
            || matches!(*self, Effect::HighPass { cutoff, .. } if cutoff <= HIGHPASS_BYPASS_HZ)
            || matches!(
                *self,
                Effect::Peak { gain_db, .. }
                    | Effect::LowShelf { gain_db, .. }
                    | Effect::HighShelf { gain_db, .. }
                    if gain_db.abs() <= EQ_BYPASS_GAIN_DB)
            || matches!(
                *self,
                Effect::Compress { ratio, .. }
                    | Effect::Gate { ratio, .. }
                    | Effect::DeEss { ratio, .. }
                    | Effect::DePlosive { ratio, .. }
                    if ratio <= 1.0)
            || matches!(*self, Effect::Limiter { ceiling_db, .. }
                if ceiling_db >= LIMITER_BYPASS_CEILING_DB)
            || matches!(*self, Effect::Saturate { drive } if drive < SATURATE_BYPASS_DRIVE)
            || matches!(*self, Effect::Distortion { drive, .. } if drive <= DISTORT_BYPASS_DRIVE)
            || matches!(*self, Effect::Exciter { amount, .. } if amount <= MOD_BYPASS)
            || matches!(*self, Effect::Vibrato { depth_ms, .. } if depth_ms <= MOD_BYPASS)
            || matches!(
                *self,
                Effect::Comb { mix, .. } | Effect::AutoWah { mix, .. } if mix <= MOD_BYPASS)
            || matches!(*self, Effect::Haas { delay_ms } if delay_ms <= MOD_BYPASS)
            || matches!(*self, Effect::Bitcrush { bits, downsample }
                if bits >= BITCRUSH_BYPASS_BITS && downsample <= 1)
            || matches!(*self, Effect::StereoWidth { width } if (width - 1.0).abs() <= f32::EPSILON)
            || matches!(
                *self,
                Effect::Chorus { mix, .. }
                    | Effect::Flanger { mix, .. }
                    | Effect::Phaser { mix, .. }
                    | Effect::RingMod { mix, .. }
                    if mix <= MOD_BYPASS)
            || matches!(
                *self,
                Effect::Tremolo { depth, .. }
                    | Effect::AutoPan { depth, .. }
                    | Effect::TranceGate { depth, .. }
                    if depth <= MOD_BYPASS)
            || matches!(*self, Effect::Doubler { mix, .. } if mix <= MOD_BYPASS)
            || matches!(
                *self,
                Effect::PitchShift { semitones, mix } | Effect::FormantShift { semitones, mix }
                    if semitones.abs() <= PITCH_BYPASS_ST || mix <= MOD_BYPASS)
            || matches!(*self, Effect::Harmonizer { mix, .. } if mix <= MOD_BYPASS)
            || matches!(*self, Effect::DeClick { sensitivity, .. }
                if sensitivity <= DECLICK_BYPASS_SENS)
            || matches!(*self, Effect::DeClip { amount, .. } if amount <= DECLIP_BYPASS_AMOUNT)
            || matches!(*self, Effect::DeHum { depth, .. } if depth <= HUM_BYPASS_DEPTH)
            || matches!(*self, Effect::Leveler { amount, .. } if amount <= LEVELER_BYPASS_AMOUNT)
            || matches!(*self, Effect::Transient { attack, sustain }
                if attack.abs() <= TRANSIENT_BYPASS && sustain.abs() <= TRANSIENT_BYPASS)
    }
}

/// Frames a 2nd-order section needs to settle: `TAUS` time constants of `Q/(π·f0)`.
fn biquad_warmup(freq: f32, q: f32, sample_rate: u32, taus: f32) -> usize {
    let tau_frames = q.max(0.1) * sample_rate as f32 / (std::f32::consts::PI * freq.max(1.0));
    (taus * tau_frames) as usize
}

#[cfg(test)]
mod tests;
