//! The rack's parameter **specs** — one `[FxParamSpec; N]` per effect (label, range,
//! log/linear, neutral default, unit). Split out of `fx_params_table.rs` to keep that
//! file under the HR-18 600-LOC shell cap; the row table (name · specs · constructor ·
//! arming knobs) lives there and references these by `&NAME`.

use super::fx_param_keys as p;
use super::fx_params::{FxParamSpec, FxUnit as U, spec};

// Every `default` below is the effect's NEUTRAL point: selecting an effect (or
// arrowing past it) must leave the audio byte-identical until the user turns
// something. `ph2d_audio_edit::{Effect, TailEffect}::is_bypass` mirrors these, and
// `every_effect_is_a_no_op_at_its_defaults` proves the two agree.
pub static LOW_PASS: [FxParamSpec; 2] = [
    // Neutral at the TOP of the range: nothing is filtered out.
    spec(p::CUTOFF, 20.0, 20_000.0, true, 20_000.0, U::Hz, false),
    spec(p::Q, 0.1, 8.0, false, 0.707, U::None, false),
];
pub static HIGH_PASS: [FxParamSpec; 2] = [
    // Neutral at the BOTTOM of the range.
    spec(p::CUTOFF, 20.0, 20_000.0, true, 20.0, U::Hz, false),
    spec(p::Q, 0.1, 8.0, false, 0.707, U::None, false),
];
// The three EQ bands share a neutral: 0 dB of gain is a flat filter.
pub static PEAK_EQ: [FxParamSpec; 3] = [
    spec(p::FREQ, 20.0, 20_000.0, true, 1_000.0, U::Hz, false),
    spec(p::Q, 0.1, 8.0, false, 1.0, U::None, false),
    spec(p::GAIN, -18.0, 18.0, false, 0.0, U::Db, false),
];
pub static LOW_SHELF: [FxParamSpec; 3] = [
    spec(p::FREQ, 20.0, 2_000.0, true, 200.0, U::Hz, false),
    spec(p::Q, 0.1, 2.0, false, 0.707, U::None, false),
    spec(p::GAIN, -18.0, 18.0, false, 0.0, U::Db, false),
];
pub static HIGH_SHELF: [FxParamSpec; 3] = [
    spec(p::FREQ, 1_000.0, 20_000.0, true, 6_000.0, U::Hz, false),
    spec(p::Q, 0.1, 2.0, false, 0.707, U::None, false),
    spec(p::GAIN, -18.0, 18.0, false, 0.0, U::Db, false),
];
// Depth is the arm (0 = 0 dB cut = pass-through). Freq covers both mains standards.
pub static DE_HUM: [FxParamSpec; 3] = [
    spec(p::FREQ, 50.0, 60.0, false, 50.0, U::Hz, false),
    spec(p::DEPTH, 0.0, 1.0, false, 0.0, U::None, false),
    spec(p::HARMONICS, 1.0, 8.0, false, 4.0, U::None, true),
];
pub static COMPRESS: [FxParamSpec; 4] = [
    spec(p::THRESHOLD, 0.01, 1.0, true, 0.3, U::None, false),
    // Neutral at 1:1 — no reduction at all (make-up is peak-preserving, so it
    // collapses to unity too).
    spec(p::RATIO, 1.0, 20.0, false, 1.0, U::X, false),
    spec(p::ATTACK, 0.001, 0.2, true, 0.005, U::S, false),
    spec(p::RELEASE, 0.01, 1.0, true, 0.1, U::S, false),
];
// Compress's knobs, deliberately: the Multiband IS that compressor, run on three bands that
// cannot duck one another. Same neutral (1:1), same arm (Ratio) — the only thing that reads
// differently is Threshold, which is a fraction of **each band's own peak** rather than an
// absolute level (an absolute one would be crossed constantly by the bass and never reached
// by the treble; see `ph2d_audio_edit::fx::multiband`). The crossovers are fixed at 200 Hz
// and 2 kHz — the standard three-way split, and the rack has four sliders, not six.
pub static MULTIBAND: [FxParamSpec; 4] = [
    spec(p::THRESHOLD, 0.01, 1.0, true, 0.3, U::None, false),
    spec(p::RATIO, 1.0, 20.0, false, 1.0, U::X, false),
    spec(p::ATTACK, 0.001, 0.2, true, 0.005, U::S, false),
    spec(p::RELEASE, 0.01, 1.0, true, 0.1, U::S, false),
];
pub static GATE: [FxParamSpec; 4] = [
    spec(p::THRESHOLD, 0.001, 0.5, true, 0.05, U::None, false),
    // Neutral at 1:1 — `(level/threshold)^0` is unity at every level.
    spec(p::RATIO, 1.0, 20.0, false, 1.0, U::X, false),
    // On a gate, Attack OPENS and Release CLOSES (a compressor's are the other way).
    spec(p::ATTACK, 0.0005, 0.05, true, 0.002, U::S, false),
    spec(p::RELEASE, 0.005, 1.0, true, 0.1, U::S, false),
];
pub static DE_ESSER: [FxParamSpec; 4] = [
    spec(p::FREQ, 2_000.0, 12_000.0, true, 6_000.0, U::Hz, false),
    spec(p::THRESHOLD, 0.005, 0.5, true, 0.05, U::None, false),
    // Neutral at 1:1 — a 0 dB shelf.
    spec(p::RATIO, 1.0, 10.0, false, 1.0, U::X, false),
    // Attack is fixed fast inside the effect: sibilance is a transient.
    spec(p::RELEASE, 0.005, 0.3, true, 0.05, U::S, false),
];
// Mirror of the de-esser on the low "pop" band. Neutral at 1:1 (a 0 dB low-shelf).
pub static DE_PLOSIVE: [FxParamSpec; 4] = [
    spec(p::FREQ, 60.0, 250.0, true, 120.0, U::Hz, false),
    spec(p::THRESHOLD, 0.005, 0.5, true, 0.05, U::None, false),
    spec(p::RATIO, 1.0, 10.0, false, 1.0, U::X, false),
    // Attack is fixed fast inside the effect: a plosive is a transient.
    spec(p::RELEASE, 0.01, 0.4, true, 0.08, U::S, false),
];
pub static LIMITER: [FxParamSpec; 2] = [
    // Neutral at the TOP: a ceiling at 0 dBFS has nothing to catch. −1 dBTP is the
    // mastering convention; that is one notch down from neutral.
    spec(p::CEILING, -12.0, 0.0, false, 0.0, U::Db, false),
    // Doubles as the look-ahead: the gain dips this far ahead of every peak.
    spec(p::RELEASE, 0.002, 0.2, true, 0.02, U::S, false),
];
// Amount is the arm (0 = unity gain = pass-through). Speed maps log (a time param).
pub static LEVELER: [FxParamSpec; 3] = [
    spec(p::TARGET, -30.0, -6.0, false, -18.0, U::Db, false),
    spec(p::AMOUNT, 0.0, 1.0, false, 0.0, U::None, false),
    spec(p::SPEED, 0.05, 2.0, true, 0.5, U::S, false),
];
// BOTH knobs arm it — each shapes a separate part of the hit, either alone wakes it.
// Linear (−1..+1) so the 0 neutral lands exactly at slider centre.
pub static TRANSIENT: [FxParamSpec; 2] = [
    spec(p::ATTACK, -1.0, 1.0, false, 0.0, U::None, false),
    spec(p::SUSTAIN, -1.0, 1.0, false, 0.0, U::None, false),
];
// Linear (not log) so the neutral point can sit at exactly 0.
pub static SATURATE: [FxParamSpec; 1] = [spec(p::DRIVE, 0.0, 12.0, false, 0.0, U::X, false)];
// Drive is the arm (0 = clean = pass-through). Tone sweeps the post low-pass.
pub static DISTORTION: [FxParamSpec; 2] = [
    spec(p::DRIVE, 0.0, 1.0, false, 0.0, U::None, false),
    spec(p::TONE, 0.0, 1.0, false, 0.5, U::None, false),
];
pub static BITCRUSH: [FxParamSpec; 2] = [
    // Neutral = full depth, no decimation.
    spec(p::BITS, 1.0, 16.0, false, 16.0, U::None, true),
    spec(p::DOWNSAMPLE, 1.0, 32.0, false, 1.0, U::X, true),
];
// Neutral at width 1.0 (mid/side passthrough), the middle of the range.
pub static WIDEN: [FxParamSpec; 1] = [spec(p::WIDTH, 0.0, 2.0, false, 1.0, U::X, false)];
// Amount is the arm (0 = no harmonics added). Freq is the crossover into the band.
pub static EXCITER: [FxParamSpec; 2] = [
    spec(p::FREQ, 1_000.0, 10_000.0, true, 3_000.0, U::Hz, false),
    spec(p::AMOUNT, 0.0, 1.0, false, 0.0, U::None, false),
];
// Haas widener: a single delay knob (0 ms = no widening = neutral).
pub static HAAS: [FxParamSpec; 1] = [spec(p::DELAY, 0.0, 30.0, false, 0.0, U::Ms, false)];
// Comb resonator: Mix is the arm. Freq tunes the ring, Resonance sets its length.
pub static COMB: [FxParamSpec; 3] = [
    spec(p::FREQ, 40.0, 2_000.0, true, 200.0, U::Hz, false),
    spec(p::RESONANCE, 0.0, 0.95, false, 0.8, U::None, false),
    spec(p::MIX, 0.0, 1.0, false, 0.0, U::None, false),
];
// Auto-wah: Mix is the arm. Base is the resting cut-off, Sens the sweep depth.
pub static AUTO_WAH: [FxParamSpec; 3] = [
    spec(p::BASE, 200.0, 2_000.0, true, 500.0, U::Hz, false),
    spec(p::SENS, 0.0, 1.0, false, 0.5, U::None, false),
    spec(p::MIX, 0.0, 1.0, false, 0.0, U::None, false),
];
pub static REVERB: [FxParamSpec; 4] = [
    spec(p::ROOM, 0.0, 1.0, false, 0.7, U::None, false),
    spec(p::DAMP, 0.0, 1.0, false, 0.5, U::None, false),
    // Neutral: fully dry. A dry tail effect must not even ring out, or it would
    // lengthen the clip with silence — `tail_frames()` returns 0 when Mix is 0.
    spec(p::MIX, 0.0, 1.0, false, 0.0, U::None, false),
    // Freeverb's shortest comb is ~25 ms: a tail below that renders pure silence.
    spec(p::TAIL, 0.1, 6.0, true, 2.5, U::S, false),
];
pub static ECHO: [FxParamSpec; 4] = [
    // The dsp kit's delay line is one second long, so the tap must stay under it.
    spec(p::TIME, 0.01, 0.99, true, 0.25, U::S, false),
    spec(p::FEEDBACK, 0.0, 0.95, false, 0.4, U::None, false),
    spec(p::MIX, 0.0, 1.0, false, 0.0, U::None, false),
    spec(p::TAIL, 0.1, 6.0, true, 2.0, U::S, false),
];
// Same knobs as Echo — the difference is the cross-fed topology, not the controls.
pub static PING_PONG: [FxParamSpec; 4] = [
    spec(p::TIME, 0.01, 0.99, true, 0.25, U::S, false),
    spec(p::FEEDBACK, 0.0, 0.95, false, 0.4, U::None, false),
    spec(p::MIX, 0.0, 1.0, false, 0.0, U::None, false),
    spec(p::TAIL, 0.1, 6.0, true, 2.0, U::S, false),
];
// The four modulation effects. Each is neutral at Mix (or Depth) 0 — fully dry.
pub static CHORUS: [FxParamSpec; 3] = [
    spec(p::RATE, 0.05, 8.0, true, 1.0, U::Hz, false),
    spec(p::DEPTH, 0.5, 15.0, false, 5.0, U::Ms, false),
    spec(p::MIX, 0.0, 1.0, false, 0.0, U::None, false),
];
// Depth is the arm (0 ms = fixed delay = bypass). Rate is the wobble speed.
pub static VIBRATO: [FxParamSpec; 2] = [
    spec(p::RATE, 0.1, 12.0, true, 5.0, U::Hz, false),
    spec(p::DEPTH, 0.0, 12.0, false, 0.0, U::Ms, false),
];
pub static FLANGER: [FxParamSpec; 4] = [
    spec(p::RATE, 0.05, 8.0, true, 0.4, U::Hz, false),
    spec(p::DEPTH, 0.2, 8.0, false, 3.0, U::Ms, false),
    spec(p::FEEDBACK, 0.0, 0.95, false, 0.4, U::None, false),
    spec(p::MIX, 0.0, 1.0, false, 0.0, U::None, false),
];
pub static PHASER: [FxParamSpec; 3] = [
    spec(p::RATE, 0.05, 8.0, true, 0.5, U::Hz, false),
    spec(p::DEPTH, 0.0, 1.0, false, 0.8, U::None, false),
    spec(p::MIX, 0.0, 1.0, false, 0.0, U::None, false),
];
// Depth (not a Mix) is the arm: at 0 the gain stays at 1.0.
pub static TREMOLO: [FxParamSpec; 2] = [
    spec(p::RATE, 0.1, 16.0, true, 5.0, U::Hz, false),
    spec(p::DEPTH, 0.0, 1.0, false, 0.0, U::None, false),
];
// Depth is the arm (0 = both channels at unity). Rate sweeps slower than a tremolo —
// a pan you follow, not a flutter.
pub static AUTO_PAN: [FxParamSpec; 2] = [
    spec(p::RATE, 0.05, 8.0, true, 1.0, U::Hz, false),
    spec(p::DEPTH, 0.0, 1.0, false, 0.0, U::None, false),
];
// Depth is the arm (0 = gain stays open). Smooth is a time (log): short = stutter.
// Default 12 Hz — a fast, immediately-audible stutter (sixteenths near 180 BPM).
pub static TRANCE_GATE: [FxParamSpec; 3] = [
    spec(p::RATE, 0.5, 16.0, true, 12.0, U::Hz, false),
    spec(p::DEPTH, 0.0, 1.0, false, 0.0, U::None, false),
    spec(p::SMOOTH, 0.001, 0.05, true, 0.005, U::S, false),
];
// Mix is the arm. Delay/Detune are in ms (the doubled voice's tap + sweep).
pub static DOUBLER: [FxParamSpec; 3] = [
    spec(p::DELAY, 5.0, 40.0, true, 20.0, U::Ms, false),
    spec(p::DETUNE, 0.5, 15.0, false, 6.0, U::Ms, false),
    spec(p::MIX, 0.0, 1.0, false, 0.0, U::None, false),
];
// Mix is the arm (0 = dry = pass-through). Freq is the carrier, log-swept across the
// low-mid band where the robot/metallic character lives.
pub static RING_MOD: [FxParamSpec; 2] = [
    spec(p::FREQ, 20.0, 4_000.0, true, 500.0, U::Hz, false),
    spec(p::MIX, 0.0, 1.0, false, 0.0, U::None, false),
];
// Semitones is the arm — neutral at 0 (linear so 0 lands exactly at slider centre).
// Mix stays fully wet by default (the pitched voice), pull down to blend a detune.
pub static PITCH_SHIFT: [FxParamSpec; 2] = [
    spec(p::SEMITONES, -12.0, 12.0, false, 0.0, U::St, false),
    spec(p::MIX, 0.0, 1.0, false, 1.0, U::None, false),
];
// Same shape as Pitch Shift, and deliberately so — the two are complements (one moves
// the pitch and drags the formants along, the other moves the formants and leaves the
// pitch). Shift is the arm; neutral at 0, which is where an unwarped envelope driven by
// its own residual reconstructs the input.
pub static FORMANT_SHIFT: [FxParamSpec; 2] = [
    spec(p::SHIFT, -12.0, 12.0, false, 0.0, U::St, false),
    spec(p::MIX, 0.0, 1.0, false, 1.0, U::None, false),
];
// Mix is the arm (0 = dry = pass-through). The two voices default to a third and a
// fifth: turn Mix up and a major triad is what comes out, which is the thing to hear
// first. Linear, so an interval of 0 lands exactly at slider centre.
pub static HARMONIZER: [FxParamSpec; 3] = [
    spec(p::VOICE_1, -12.0, 12.0, false, 4.0, U::St, false),
    spec(p::VOICE_2, -12.0, 12.0, false, 7.0, U::St, false),
    spec(p::MIX, 0.0, 1.0, false, 0.0, U::None, false),
];
// Mix is the arm (0 = dry). Carrier is the robot's note; Breath crossfades the carrier from a
// sawtooth (0 = voiced = ROBOT) to noise (1 = unvoiced = WHISPER) -- which is why neither is a
// separate row. Bands is the bank's resolution: coarse is more synthetic, fine resynthesises the
// input. See `ph2d_audio_edit::fx::vocoder`.
pub static VOCODER: [FxParamSpec; 4] = [
    spec(p::CARRIER, 50.0, 400.0, true, 110.0, U::Hz, false),
    spec(p::BANDS, 4.0, 32.0, false, 16.0, U::None, true),
    spec(p::BREATH, 0.0, 1.0, false, 0.0, U::None, false),
    spec(p::MIX, 0.0, 1.0, false, 0.0, U::None, false),
];
// Mix is the arm. Scatter DEFAULTS TO A REAL VALUE on purpose: at scatter 0 (and no detune) the
// grains land where they came from and a Hann window at half-hop sums to exactly 1, so a fully-wet
// granular would be a byte-perfect no-op and turning Mix up would do nothing at all. See
// `ph2d_audio_edit::fx::granular`.
pub static GRANULAR: [FxParamSpec; 4] = [
    spec(p::GRAIN, 10.0, 200.0, true, 60.0, U::Ms, false),
    spec(p::SCATTER, 0.0, 1.0, false, 0.35, U::None, false),
    spec(p::PITCH, 0.0, 12.0, false, 0.0, U::St, false),
    spec(p::MIX, 0.0, 1.0, false, 0.0, U::None, false),
];
// Sensitivity is the arm: at 0 the detector finds nothing, so the repairer repairs
// nothing. A restoration tool must be ASKED for — "very insensitive" is not the same
// promise as "off", and the rack's neutral point means off.
//
// Width is the longest run it will touch, in seconds (a click is sub-millisecond; past
// a few, a "click" is the signal). Log, like every other time parameter.
pub static DE_CLICK: [FxParamSpec; 2] = [
    spec(p::SENSITIVITY, 0.0, 1.0, false, 0.0, U::None, false),
    spec(p::WIDTH, 0.0001, 0.003, true, 0.001, U::S, false),
];
// Mix is the only knob: the ROOM carries everything else — its size, its colour, how long it
// rings. That is the difference between a convolution reverb and an algorithmic one, and giving
// it a "room size" slider would be pretending otherwise.
pub static CONV_REVERB: [FxParamSpec; 1] = [spec(p::MIX, 0.0, 1.0, false, 0.0, U::None, false)];
// Amount is the arm (0 = off): a restoration tool has nothing to do in most clips, so it
// starts inert like De-Click. Threshold is where a FLAT run starts counting as clipped —
// just under full scale, which is where a converter actually runs out.
pub static DE_CLIP: [FxParamSpec; 2] = [
    spec(p::AMOUNT, 0.0, 1.0, false, 0.0, U::None, false),
    spec(p::THRESHOLD, 0.5, 1.0, false, 0.95, U::None, false),
];
