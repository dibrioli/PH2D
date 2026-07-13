//! The rack's parameter **specs** — one `[FxParamSpec; N]` per effect (label, range,
//! log/linear, neutral default, unit). Split out of `fx_params_table.rs` to keep that
//! file under the HR-18 600-LOC shell cap; the row table (name · specs · constructor ·
//! arming knobs) lives there and references these by `&NAME`.

use super::fx_params::{FxParamSpec, spec};

// Every `default` below is the effect's NEUTRAL point: selecting an effect (or
// arrowing past it) must leave the audio byte-identical until the user turns
// something. `ph2d_audio_edit::{Effect, TailEffect}::is_bypass` mirrors these, and
// `every_effect_is_a_no_op_at_its_defaults` proves the two agree.
pub(super) static LOW_PASS: [FxParamSpec; 2] = [
    // Neutral at the TOP of the range: nothing is filtered out.
    spec("Cutoff", 20.0, 20_000.0, true, 20_000.0, "Hz", false),
    spec("Q", 0.1, 8.0, false, 0.707, "", false),
];
pub(super) static HIGH_PASS: [FxParamSpec; 2] = [
    // Neutral at the BOTTOM of the range.
    spec("Cutoff", 20.0, 20_000.0, true, 20.0, "Hz", false),
    spec("Q", 0.1, 8.0, false, 0.707, "", false),
];
// The three EQ bands share a neutral: 0 dB of gain is a flat filter.
pub(super) static PEAK_EQ: [FxParamSpec; 3] = [
    spec("Freq", 20.0, 20_000.0, true, 1_000.0, "Hz", false),
    spec("Q", 0.1, 8.0, false, 1.0, "", false),
    spec("Gain", -18.0, 18.0, false, 0.0, "dB", false),
];
pub(super) static LOW_SHELF: [FxParamSpec; 3] = [
    spec("Freq", 20.0, 2_000.0, true, 200.0, "Hz", false),
    spec("Q", 0.1, 2.0, false, 0.707, "", false),
    spec("Gain", -18.0, 18.0, false, 0.0, "dB", false),
];
pub(super) static HIGH_SHELF: [FxParamSpec; 3] = [
    spec("Freq", 1_000.0, 20_000.0, true, 6_000.0, "Hz", false),
    spec("Q", 0.1, 2.0, false, 0.707, "", false),
    spec("Gain", -18.0, 18.0, false, 0.0, "dB", false),
];
// Depth is the arm (0 = 0 dB cut = pass-through). Freq covers both mains standards.
pub(super) static DE_HUM: [FxParamSpec; 3] = [
    spec("Freq", 50.0, 60.0, false, 50.0, "Hz", false),
    spec("Depth", 0.0, 1.0, false, 0.0, "", false),
    spec("Harmonics", 1.0, 8.0, false, 4.0, "", true),
];
pub(super) static COMPRESS: [FxParamSpec; 4] = [
    spec("Threshold", 0.01, 1.0, true, 0.3, "", false),
    // Neutral at 1:1 — no reduction at all (make-up is peak-preserving, so it
    // collapses to unity too).
    spec("Ratio", 1.0, 20.0, false, 1.0, "x", false),
    spec("Attack", 0.001, 0.2, true, 0.005, "s", false),
    spec("Release", 0.01, 1.0, true, 0.1, "s", false),
];
// Compress's knobs, deliberately: the Multiband IS that compressor, run on three bands that
// cannot duck one another. Same neutral (1:1), same arm (Ratio) — the only thing that reads
// differently is Threshold, which is a fraction of **each band's own peak** rather than an
// absolute level (an absolute one would be crossed constantly by the bass and never reached
// by the treble; see `ph2d_audio_edit::fx::multiband`). The crossovers are fixed at 200 Hz
// and 2 kHz — the standard three-way split, and the rack has four sliders, not six.
pub(super) static MULTIBAND: [FxParamSpec; 4] = [
    spec("Threshold", 0.01, 1.0, true, 0.3, "", false),
    spec("Ratio", 1.0, 20.0, false, 1.0, "x", false),
    spec("Attack", 0.001, 0.2, true, 0.005, "s", false),
    spec("Release", 0.01, 1.0, true, 0.1, "s", false),
];
pub(super) static GATE: [FxParamSpec; 4] = [
    spec("Threshold", 0.001, 0.5, true, 0.05, "", false),
    // Neutral at 1:1 — `(level/threshold)^0` is unity at every level.
    spec("Ratio", 1.0, 20.0, false, 1.0, "x", false),
    // On a gate, Attack OPENS and Release CLOSES (a compressor's are the other way).
    spec("Attack", 0.0005, 0.05, true, 0.002, "s", false),
    spec("Release", 0.005, 1.0, true, 0.1, "s", false),
];
pub(super) static DE_ESSER: [FxParamSpec; 4] = [
    spec("Freq", 2_000.0, 12_000.0, true, 6_000.0, "Hz", false),
    spec("Threshold", 0.005, 0.5, true, 0.05, "", false),
    // Neutral at 1:1 — a 0 dB shelf.
    spec("Ratio", 1.0, 10.0, false, 1.0, "x", false),
    // Attack is fixed fast inside the effect: sibilance is a transient.
    spec("Release", 0.005, 0.3, true, 0.05, "s", false),
];
// Mirror of the de-esser on the low "pop" band. Neutral at 1:1 (a 0 dB low-shelf).
pub(super) static DE_PLOSIVE: [FxParamSpec; 4] = [
    spec("Freq", 60.0, 250.0, true, 120.0, "Hz", false),
    spec("Threshold", 0.005, 0.5, true, 0.05, "", false),
    spec("Ratio", 1.0, 10.0, false, 1.0, "x", false),
    // Attack is fixed fast inside the effect: a plosive is a transient.
    spec("Release", 0.01, 0.4, true, 0.08, "s", false),
];
pub(super) static LIMITER: [FxParamSpec; 2] = [
    // Neutral at the TOP: a ceiling at 0 dBFS has nothing to catch. −1 dBTP is the
    // mastering convention; that is one notch down from neutral.
    spec("Ceiling", -12.0, 0.0, false, 0.0, "dB", false),
    // Doubles as the look-ahead: the gain dips this far ahead of every peak.
    spec("Release", 0.002, 0.2, true, 0.02, "s", false),
];
// Amount is the arm (0 = unity gain = pass-through). Speed maps log (a time param).
pub(super) static LEVELER: [FxParamSpec; 3] = [
    spec("Target", -30.0, -6.0, false, -18.0, "dB", false),
    spec("Amount", 0.0, 1.0, false, 0.0, "", false),
    spec("Speed", 0.05, 2.0, true, 0.5, "s", false),
];
// BOTH knobs arm it — each shapes a separate part of the hit, either alone wakes it.
// Linear (−1..+1) so the 0 neutral lands exactly at slider centre.
pub(super) static TRANSIENT: [FxParamSpec; 2] = [
    spec("Attack", -1.0, 1.0, false, 0.0, "", false),
    spec("Sustain", -1.0, 1.0, false, 0.0, "", false),
];
// Linear (not log) so the neutral point can sit at exactly 0.
pub(super) static SATURATE: [FxParamSpec; 1] = [spec("Drive", 0.0, 12.0, false, 0.0, "x", false)];
// Drive is the arm (0 = clean = pass-through). Tone sweeps the post low-pass.
pub(super) static DISTORTION: [FxParamSpec; 2] = [
    spec("Drive", 0.0, 1.0, false, 0.0, "", false),
    spec("Tone", 0.0, 1.0, false, 0.5, "", false),
];
pub(super) static BITCRUSH: [FxParamSpec; 2] = [
    // Neutral = full depth, no decimation.
    spec("Bits", 1.0, 16.0, false, 16.0, "", true),
    spec("Downsample", 1.0, 32.0, false, 1.0, "x", true),
];
// Neutral at width 1.0 (mid/side passthrough), the middle of the range.
pub(super) static WIDEN: [FxParamSpec; 1] = [spec("Width", 0.0, 2.0, false, 1.0, "x", false)];
// Amount is the arm (0 = no harmonics added). Freq is the crossover into the band.
pub(super) static EXCITER: [FxParamSpec; 2] = [
    spec("Freq", 1_000.0, 10_000.0, true, 3_000.0, "Hz", false),
    spec("Amount", 0.0, 1.0, false, 0.0, "", false),
];
// Haas widener: a single delay knob (0 ms = no widening = neutral).
pub(super) static HAAS: [FxParamSpec; 1] = [spec("Delay", 0.0, 30.0, false, 0.0, "ms", false)];
// Comb resonator: Mix is the arm. Freq tunes the ring, Resonance sets its length.
pub(super) static COMB: [FxParamSpec; 3] = [
    spec("Freq", 40.0, 2_000.0, true, 200.0, "Hz", false),
    spec("Resonance", 0.0, 0.95, false, 0.8, "", false),
    spec("Mix", 0.0, 1.0, false, 0.0, "", false),
];
// Auto-wah: Mix is the arm. Base is the resting cut-off, Sens the sweep depth.
pub(super) static AUTO_WAH: [FxParamSpec; 3] = [
    spec("Base", 200.0, 2_000.0, true, 500.0, "Hz", false),
    spec("Sens", 0.0, 1.0, false, 0.5, "", false),
    spec("Mix", 0.0, 1.0, false, 0.0, "", false),
];
pub(super) static REVERB: [FxParamSpec; 4] = [
    spec("Room", 0.0, 1.0, false, 0.7, "", false),
    spec("Damp", 0.0, 1.0, false, 0.5, "", false),
    // Neutral: fully dry. A dry tail effect must not even ring out, or it would
    // lengthen the clip with silence — `tail_frames()` returns 0 when Mix is 0.
    spec("Mix", 0.0, 1.0, false, 0.0, "", false),
    // Freeverb's shortest comb is ~25 ms: a tail below that renders pure silence.
    spec("Tail", 0.1, 6.0, true, 2.5, "s", false),
];
pub(super) static ECHO: [FxParamSpec; 4] = [
    // The dsp kit's delay line is one second long, so the tap must stay under it.
    spec("Time", 0.01, 0.99, true, 0.25, "s", false),
    spec("Feedback", 0.0, 0.95, false, 0.4, "", false),
    spec("Mix", 0.0, 1.0, false, 0.0, "", false),
    spec("Tail", 0.1, 6.0, true, 2.0, "s", false),
];
// Same knobs as Echo — the difference is the cross-fed topology, not the controls.
pub(super) static PING_PONG: [FxParamSpec; 4] = [
    spec("Time", 0.01, 0.99, true, 0.25, "s", false),
    spec("Feedback", 0.0, 0.95, false, 0.4, "", false),
    spec("Mix", 0.0, 1.0, false, 0.0, "", false),
    spec("Tail", 0.1, 6.0, true, 2.0, "s", false),
];
// The four modulation effects. Each is neutral at Mix (or Depth) 0 — fully dry.
pub(super) static CHORUS: [FxParamSpec; 3] = [
    spec("Rate", 0.05, 8.0, true, 1.0, "Hz", false),
    spec("Depth", 0.5, 15.0, false, 5.0, "ms", false),
    spec("Mix", 0.0, 1.0, false, 0.0, "", false),
];
// Depth is the arm (0 ms = fixed delay = bypass). Rate is the wobble speed.
pub(super) static VIBRATO: [FxParamSpec; 2] = [
    spec("Rate", 0.1, 12.0, true, 5.0, "Hz", false),
    spec("Depth", 0.0, 12.0, false, 0.0, "ms", false),
];
pub(super) static FLANGER: [FxParamSpec; 4] = [
    spec("Rate", 0.05, 8.0, true, 0.4, "Hz", false),
    spec("Depth", 0.2, 8.0, false, 3.0, "ms", false),
    spec("Feedback", 0.0, 0.95, false, 0.4, "", false),
    spec("Mix", 0.0, 1.0, false, 0.0, "", false),
];
pub(super) static PHASER: [FxParamSpec; 3] = [
    spec("Rate", 0.05, 8.0, true, 0.5, "Hz", false),
    spec("Depth", 0.0, 1.0, false, 0.8, "", false),
    spec("Mix", 0.0, 1.0, false, 0.0, "", false),
];
// Depth (not a Mix) is the arm: at 0 the gain stays at 1.0.
pub(super) static TREMOLO: [FxParamSpec; 2] = [
    spec("Rate", 0.1, 16.0, true, 5.0, "Hz", false),
    spec("Depth", 0.0, 1.0, false, 0.0, "", false),
];
// Depth is the arm (0 = both channels at unity). Rate sweeps slower than a tremolo —
// a pan you follow, not a flutter.
pub(super) static AUTO_PAN: [FxParamSpec; 2] = [
    spec("Rate", 0.05, 8.0, true, 1.0, "Hz", false),
    spec("Depth", 0.0, 1.0, false, 0.0, "", false),
];
// Depth is the arm (0 = gain stays open). Smooth is a time (log): short = stutter.
// Default 12 Hz — a fast, immediately-audible stutter (sixteenths near 180 BPM).
pub(super) static TRANCE_GATE: [FxParamSpec; 3] = [
    spec("Rate", 0.5, 16.0, true, 12.0, "Hz", false),
    spec("Depth", 0.0, 1.0, false, 0.0, "", false),
    spec("Smooth", 0.001, 0.05, true, 0.005, "s", false),
];
// Mix is the arm. Delay/Detune are in ms (the doubled voice's tap + sweep).
pub(super) static DOUBLER: [FxParamSpec; 3] = [
    spec("Delay", 5.0, 40.0, true, 20.0, "ms", false),
    spec("Detune", 0.5, 15.0, false, 6.0, "ms", false),
    spec("Mix", 0.0, 1.0, false, 0.0, "", false),
];
// Mix is the arm (0 = dry = pass-through). Freq is the carrier, log-swept across the
// low-mid band where the robot/metallic character lives.
pub(super) static RING_MOD: [FxParamSpec; 2] = [
    spec("Freq", 20.0, 4_000.0, true, 500.0, "Hz", false),
    spec("Mix", 0.0, 1.0, false, 0.0, "", false),
];
// Semitones is the arm — neutral at 0 (linear so 0 lands exactly at slider centre).
// Mix stays fully wet by default (the pitched voice), pull down to blend a detune.
pub(super) static PITCH_SHIFT: [FxParamSpec; 2] = [
    spec("Semitones", -12.0, 12.0, false, 0.0, "st", false),
    spec("Mix", 0.0, 1.0, false, 1.0, "", false),
];
// Same shape as Pitch Shift, and deliberately so — the two are complements (one moves
// the pitch and drags the formants along, the other moves the formants and leaves the
// pitch). Shift is the arm; neutral at 0, which is where an unwarped envelope driven by
// its own residual reconstructs the input.
pub(super) static FORMANT_SHIFT: [FxParamSpec; 2] = [
    spec("Shift", -12.0, 12.0, false, 0.0, "st", false),
    spec("Mix", 0.0, 1.0, false, 1.0, "", false),
];
// Mix is the arm (0 = dry = pass-through). The two voices default to a third and a
// fifth: turn Mix up and a major triad is what comes out, which is the thing to hear
// first. Linear, so an interval of 0 lands exactly at slider centre.
pub(super) static HARMONIZER: [FxParamSpec; 3] = [
    spec("Voice 1", -12.0, 12.0, false, 4.0, "st", false),
    spec("Voice 2", -12.0, 12.0, false, 7.0, "st", false),
    spec("Mix", 0.0, 1.0, false, 0.0, "", false),
];
// Sensitivity is the arm: at 0 the detector finds nothing, so the repairer repairs
// nothing. A restoration tool must be ASKED for — "very insensitive" is not the same
// promise as "off", and the rack's neutral point means off.
//
// Width is the longest run it will touch, in seconds (a click is sub-millisecond; past
// a few, a "click" is the signal). Log, like every other time parameter.
pub(super) static DE_CLICK: [FxParamSpec; 2] = [
    spec("Sensitivity", 0.0, 1.0, false, 0.0, "", false),
    spec("Width", 0.0001, 0.003, true, 0.001, "s", false),
];
// Mix is the only knob: the ROOM carries everything else — its size, its colour, how long it
// rings. That is the difference between a convolution reverb and an algorithmic one, and giving
// it a "room size" slider would be pretending otherwise.
pub(super) static CONV_REVERB: [FxParamSpec; 1] = [spec("Mix", 0.0, 1.0, false, 0.0, "", false)];
// Amount is the arm (0 = off): a restoration tool has nothing to do in most clips, so it
// starts inert like De-Click. Threshold is where a FLAT run starts counting as clipped —
// just under full scale, which is where a converter actually runs out.
pub(super) static DE_CLIP: [FxParamSpec; 2] = [
    spec("Amount", 0.0, 1.0, false, 0.0, "", false),
    spec("Threshold", 0.5, 1.0, false, 0.95, "", false),
];
