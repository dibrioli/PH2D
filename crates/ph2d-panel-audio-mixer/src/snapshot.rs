//! Live snapshot published by the shell's mixer bridge each frame, read by
//! the panel painter. Thread-local (UI + shell run on the main thread), mirrors
//! the other panels' publish channels (e.g. `panel-painter-layers`). Split out
//! of `lib.rs` to keep it under the panel LOC cap.

use crate::SUB_BUS_COUNT;
use std::cell::Cell;

/// Per-frame peak-hold decay factor (UI ballistics). The held marker falls
/// by this each paint; a new peak snaps it back up.
const PEAK_HOLD_DECAY: f32 = 0.92; // LITERAL-PX-OK: meter peak-hold decay per frame (ballistics, not a UI dimension)
/// Peak above this (full scale) latches the clip indicator.
const CLIP_THRESHOLD: f32 = 1.0; // LITERAL-PX-OK: clip = exceeds full scale (audio domain)

thread_local! {
    static LEVELS_PEAK: Cell<[f32; 2]> = const { Cell::new([0.0, 0.0]) };
    static LEVELS_RMS: Cell<[f32; 2]> = const { Cell::new([0.0, 0.0]) };
    static LOUDNESS_LUFS: Cell<f32> = const { Cell::new(-120.0) }; // LITERAL-PX-OK: LUFS silence-floor default (audio domain)
    static PEAK_HOLD: Cell<[f32; 2]> = const { Cell::new([0.0, 0.0]) };
    static CLIP: Cell<[bool; 2]> = const { Cell::new([false, false]) };
    static SUB_CLIP: Cell<[[bool; 2]; SUB_BUS_COUNT]> = const { Cell::new([[false, false]; SUB_BUS_COUNT]) };
    static MASTER_GAIN: Cell<f32> = const { Cell::new(1.0) };
    static CUTOFF_HZ: Cell<f32> = const { Cell::new(20_000.0) }; // LITERAL-PX-OK: default cutoff 20 kHz (audio frequency, not a UI metric)
    static LOWCUT_HZ: Cell<f32> = const { Cell::new(20.0) }; // LITERAL-PX-OK: default low-cut 20 Hz (off — audio floor, not a UI metric)
    static MUTED: Cell<bool> = const { Cell::new(false) };
    static MASTER_PAN: Cell<f32> = const { Cell::new(0.0) };
    static PLAY_TEST: Cell<bool> = const { Cell::new(false) };
    static LIMITER: Cell<bool> = const { Cell::new(false) };
    static REVERB_ON: Cell<bool> = const { Cell::new(false) };
    static REVERB_SIZE: Cell<f32> = const { Cell::new(0.5) };
    static REVERB_MIX: Cell<f32> = const { Cell::new(0.3) }; // LITERAL-PX-OK: default reverb wet/dry mix (audio param)
    static DUCKING: Cell<bool> = const { Cell::new(false) };
    static DUCK_DEPTH: Cell<f32> = const { Cell::new(0.7) }; // LITERAL-PX-OK: default duck depth (audio param)
    // Sidechain key sub-bus index; everything else ducks under it. Default = the
    // last sub-bus (Voice), preserving the original dialogue-priority behavior.
    static DUCK_KEY: Cell<usize> = const { Cell::new(SUB_BUS_COUNT - 1) };
    // Per-sub-bus channels, index-aligned with `BusId::SUB_BUSES`.
    static SUB_LEVELS_PEAK: Cell<[[f32; 2]; SUB_BUS_COUNT]> = const { Cell::new([[0.0, 0.0]; SUB_BUS_COUNT]) };
    static SUB_LEVELS_RMS: Cell<[[f32; 2]; SUB_BUS_COUNT]> = const { Cell::new([[0.0, 0.0]; SUB_BUS_COUNT]) };
    static SUB_PEAK_HOLD: Cell<[[f32; 2]; SUB_BUS_COUNT]> = const { Cell::new([[0.0, 0.0]; SUB_BUS_COUNT]) };
    static SUB_GAIN: Cell<[f32; SUB_BUS_COUNT]> = const { Cell::new([1.0; SUB_BUS_COUNT]) };
    static SUB_MUTED: Cell<[bool; SUB_BUS_COUNT]> = const { Cell::new([false; SUB_BUS_COUNT]) };
    static SUB_SOLOED: Cell<[bool; SUB_BUS_COUNT]> = const { Cell::new([false; SUB_BUS_COUNT]) };
    static SUB_PAN: Cell<[f32; SUB_BUS_COUNT]> = const { Cell::new([0.0; SUB_BUS_COUNT]) };
    static SUB_TONE_HZ: Cell<[f32; SUB_BUS_COUNT]> = const { Cell::new([20_000.0; SUB_BUS_COUNT]) }; // LITERAL-PX-OK: default tone cutoff 20 kHz (open)
    static SUB_LOWCUT_HZ: Cell<[f32; SUB_BUS_COUNT]> = const { Cell::new([20.0; SUB_BUS_COUNT]) }; // LITERAL-PX-OK: default low-cut 20 Hz (off)
    static SUB_SEND_AMT: Cell<[f32; SUB_BUS_COUNT]> = const { Cell::new([0.0; SUB_BUS_COUNT]) };
    static SUB_DELAY_SEND_AMT: Cell<[f32; SUB_BUS_COUNT]> = const { Cell::new([0.0; SUB_BUS_COUNT]) };
    static SUB_COMP_AMT: Cell<[f32; SUB_BUS_COUNT]> = const { Cell::new([0.0; SUB_BUS_COUNT]) };
    // Master delay/echo: enable + time (slider pos = seconds, 0..1 s) + feedback +
    // return mix, all 0..1.
    static DELAY_ON: Cell<bool> = const { Cell::new(false) };
    static DELAY_TIME: Cell<f32> = const { Cell::new(0.25) }; // LITERAL-PX-OK: default echo 250 ms (audio param)
    static DELAY_FEEDBACK: Cell<f32> = const { Cell::new(0.3) }; // LITERAL-PX-OK: default echo feedback (audio param)
    static DELAY_MIX: Cell<f32> = const { Cell::new(0.3) }; // LITERAL-PX-OK: default delay return level (audio param)
    // Master 3-band EQ slider positions (0..1; 0.5 = flat), [low, mid, high].
    static EQ_POS: Cell<[f32; 3]> = const { Cell::new([0.5; 3]) }; // LITERAL-PX-OK: 3 EQ bands, 0.5 = flat
}

/// Decay the held peak toward silence, snapping up to any new peak.
fn hold_step(prev: [f32; 2], peak: [f32; 2]) -> [f32; 2] {
    [
        (prev[0] * PEAK_HOLD_DECAY).max(peak[0]),
        (prev[1] * PEAK_HOLD_DECAY).max(peak[1]),
    ]
}

/// Latch a clip flag pair against this frame's peak (`true` sticks until cleared).
fn latch_clip(prev: [bool; 2], peak: [f32; 2]) -> [bool; 2] {
    [
        prev[0] || peak[0] > CLIP_THRESHOLD,
        prev[1] || peak[1] > CLIP_THRESHOLD,
    ]
}

/// Shell → panel: current master output peak + RMS levels for the meter.
/// Latches the clip indicator when the peak exceeds full scale.
pub fn set_levels(peak: [f32; 2], rms: [f32; 2]) {
    LEVELS_PEAK.with(|c| c.set(peak));
    LEVELS_RMS.with(|c| c.set(rms));
    CLIP.with(|c| c.set(latch_clip(c.get(), peak)));
}

/// The master latched clip flags (cleared by clicking the meter). Public so
/// the clip seam is observable.
pub fn master_clipped() -> [bool; 2] {
    CLIP.with(Cell::get)
}

pub(crate) fn clear_master_clip() {
    CLIP.with(|c| c.set([false, false]));
}

/// The master RMS fill for the meter bar.
pub(crate) fn master_rms() -> [f32; 2] {
    LEVELS_RMS.with(Cell::get)
}

/// Shell → panel: the master momentary loudness in LUFS (for the readout).
pub fn set_loudness(lufs: f32) {
    LOUDNESS_LUFS.with(|c| c.set(lufs));
}

pub(crate) fn loudness() -> f32 {
    LOUDNESS_LUFS.with(Cell::get)
}

/// Advance + return the master peak-hold marker (call once per paint frame).
pub(crate) fn tick_master_hold() -> [f32; 2] {
    let peak = LEVELS_PEAK.with(Cell::get);
    PEAK_HOLD.with(|c| {
        let held = hold_step(c.get(), peak);
        c.set(held);
        held
    })
}

/// Panel → shell: the master gain the fader drives (0..1).
pub(crate) fn set_master_gain(gain: f32) {
    MASTER_GAIN.with(|c| c.set(gain));
}

pub fn master_gain() -> f32 {
    MASTER_GAIN.with(Cell::get)
}

/// Panel → shell: the master low-pass cutoff in Hz (from the Cutoff slider).
pub(crate) fn set_cutoff(hz: f32) {
    CUTOFF_HZ.with(|c| c.set(hz));
}

pub fn cutoff() -> f32 {
    CUTOFF_HZ.with(Cell::get)
}

/// Panel → shell: the master high-pass (low-cut) cutoff in Hz (from the Low
/// Cut slider; 20 Hz = off).
pub(crate) fn set_lowcut(hz: f32) {
    LOWCUT_HZ.with(|c| c.set(hz));
}

pub fn lowcut() -> f32 {
    LOWCUT_HZ.with(Cell::get)
}

pub fn muted() -> bool {
    MUTED.with(Cell::get)
}

pub(crate) fn toggle_muted() -> bool {
    MUTED.with(|c| {
        let next = !c.get();
        c.set(next);
        next
    })
}

/// Panel → shell: the master stereo balance (`-1.0`..`1.0`, `0.0` = center).
pub(crate) fn set_master_pan(pan: f32) {
    MASTER_PAN.with(|c| c.set(pan));
}

pub fn master_pan() -> f32 {
    MASTER_PAN.with(Cell::get)
}

/// Panel → shell: whether the built-in test signal should be playing.
pub fn play_test() -> bool {
    PLAY_TEST.with(Cell::get)
}

pub(crate) fn toggle_play_test() -> bool {
    PLAY_TEST.with(|c| {
        let next = !c.get();
        c.set(next);
        next
    })
}

/// Panel → shell: whether the master soft-clip limiter is engaged.
pub fn limiter() -> bool {
    LIMITER.with(Cell::get)
}

pub(crate) fn toggle_limiter() -> bool {
    LIMITER.with(|c| {
        let next = !c.get();
        c.set(next);
        next
    })
}

/// Panel → shell: master reverb enable + room size (0..1) + wet/dry mix (0..1).
pub fn reverb_on() -> bool {
    REVERB_ON.with(Cell::get)
}

pub fn reverb_size() -> f32 {
    REVERB_SIZE.with(Cell::get)
}

pub fn reverb_mix() -> f32 {
    REVERB_MIX.with(Cell::get)
}

pub(crate) fn toggle_reverb() -> bool {
    REVERB_ON.with(|c| {
        let next = !c.get();
        c.set(next);
        next
    })
}

pub(crate) fn set_reverb_size(v: f32) {
    REVERB_SIZE.with(|c| c.set(v));
}

pub(crate) fn set_reverb_mix(v: f32) {
    REVERB_MIX.with(|c| c.set(v));
}

/// Panel → shell: master delay enable + time (s) + feedback + return mix.
pub fn delay_on() -> bool {
    DELAY_ON.with(Cell::get)
}

pub fn delay_time() -> f32 {
    DELAY_TIME.with(Cell::get)
}

pub fn delay_feedback() -> f32 {
    DELAY_FEEDBACK.with(Cell::get)
}

pub fn delay_mix() -> f32 {
    DELAY_MIX.with(Cell::get)
}

pub(crate) fn toggle_delay() -> bool {
    DELAY_ON.with(|c| {
        let next = !c.get();
        c.set(next);
        next
    })
}

pub(crate) fn set_delay_time(v: f32) {
    DELAY_TIME.with(|c| c.set(v));
}

pub(crate) fn set_delay_feedback(v: f32) {
    DELAY_FEEDBACK.with(|c| c.set(v));
}

pub(crate) fn set_delay_mix(v: f32) {
    DELAY_MIX.with(|c| c.set(v));
}

/// Panel → shell: ducking enable + depth (every bus ducks under the key bus).
pub fn ducking() -> bool {
    DUCKING.with(Cell::get)
}

pub fn duck_depth() -> f32 {
    DUCK_DEPTH.with(Cell::get)
}

/// Panel → shell: the sidechain key sub-bus index (everything else ducks under
/// it). Defaults to the last sub-bus (Voice).
pub fn ducking_key() -> usize {
    DUCK_KEY.with(Cell::get)
}

pub(crate) fn toggle_ducking() -> bool {
    DUCKING.with(|c| {
        let next = !c.get();
        c.set(next);
        next
    })
}

pub(crate) fn set_duck_depth(v: f32) {
    DUCK_DEPTH.with(|c| c.set(v));
}

/// Advance the sidechain key to the next sub-bus (wraps). Returns the new index.
pub(crate) fn cycle_duck_key() -> usize {
    DUCK_KEY.with(|c| {
        let next = (c.get() + 1) % SUB_BUS_COUNT;
        c.set(next);
        next
    })
}

/// Shell → panel: current post-fader peak + RMS per sub-bus. Latches each
/// sub-bus's clip indicator when its peak exceeds full scale.
pub fn set_sub_levels(peak: [[f32; 2]; SUB_BUS_COUNT], rms: [[f32; 2]; SUB_BUS_COUNT]) {
    SUB_LEVELS_PEAK.with(|c| c.set(peak));
    SUB_LEVELS_RMS.with(|c| c.set(rms));
    SUB_CLIP.with(|c| {
        let mut v = c.get();
        for i in 0..SUB_BUS_COUNT {
            v[i] = latch_clip(v[i], peak[i]);
        }
        c.set(v);
    });
}

/// The per-sub-bus latched clip flags.
pub(crate) fn sub_clipped() -> [[bool; 2]; SUB_BUS_COUNT] {
    SUB_CLIP.with(Cell::get)
}

pub(crate) fn clear_sub_clip(i: usize) {
    SUB_CLIP.with(|c| {
        let mut v = c.get();
        if let Some(slot) = v.get_mut(i) {
            *slot = [false, false];
        }
        c.set(v);
    });
}

/// The per-sub-bus RMS fills for the meter bars.
pub(crate) fn sub_rms() -> [[f32; 2]; SUB_BUS_COUNT] {
    SUB_LEVELS_RMS.with(Cell::get)
}

/// Advance + return sub-bus `i`'s peak-hold marker (call once per paint frame).
pub(crate) fn tick_sub_hold(i: usize) -> [f32; 2] {
    let peak = SUB_LEVELS_PEAK.with(Cell::get);
    SUB_PEAK_HOLD.with(|c| {
        let mut held = c.get();
        if let Some(slot) = held.get_mut(i) {
            *slot = hold_step(*slot, peak[i]);
        }
        c.set(held);
        held.get(i).copied().unwrap_or([0.0, 0.0])
    })
}

/// Panel → shell: each sub-bus fader gain (0..1).
pub fn sub_gain() -> [f32; SUB_BUS_COUNT] {
    SUB_GAIN.with(Cell::get)
}

pub(crate) fn set_sub_gain(i: usize, gain: f32) {
    SUB_GAIN.with(|c| {
        let mut v = c.get();
        if let Some(slot) = v.get_mut(i) {
            *slot = gain;
        }
        c.set(v);
    });
}

/// Panel → shell: each sub-bus mute flag (bridge sends gain 0 when engaged).
pub fn sub_muted() -> [bool; SUB_BUS_COUNT] {
    SUB_MUTED.with(Cell::get)
}

pub(crate) fn toggle_sub_muted(i: usize) {
    SUB_MUTED.with(|c| {
        let mut v = c.get();
        if let Some(slot) = v.get_mut(i) {
            *slot = !*slot;
        }
        c.set(v);
    });
}

/// Panel → shell: each sub-bus stereo balance (`-1.0`..`1.0`).
pub fn sub_pan() -> [f32; SUB_BUS_COUNT] {
    SUB_PAN.with(Cell::get)
}

pub(crate) fn set_sub_pan(i: usize, pan: f32) {
    SUB_PAN.with(|c| {
        let mut v = c.get();
        if let Some(slot) = v.get_mut(i) {
            *slot = pan;
        }
        c.set(v);
    });
}

/// Panel → shell: each sub-bus low-pass cutoff in Hz (from its Tone slider).
pub fn sub_tone() -> [f32; SUB_BUS_COUNT] {
    SUB_TONE_HZ.with(Cell::get)
}

pub(crate) fn set_sub_tone(i: usize, hz: f32) {
    SUB_TONE_HZ.with(|c| {
        let mut v = c.get();
        if let Some(slot) = v.get_mut(i) {
            *slot = hz;
        }
        c.set(v);
    });
}

/// Panel → shell: each sub-bus high-pass (low-cut) cutoff in Hz (from its Low
/// Cut slider; 20 Hz = off).
pub fn sub_lowcut() -> [f32; SUB_BUS_COUNT] {
    SUB_LOWCUT_HZ.with(Cell::get)
}

pub(crate) fn set_sub_lowcut(i: usize, hz: f32) {
    SUB_LOWCUT_HZ.with(|c| {
        let mut v = c.get();
        if let Some(slot) = v.get_mut(i) {
            *slot = hz;
        }
        c.set(v);
    });
}

/// Panel → shell: each sub-bus reverb aux-send amount (0..1; 0 = dry) from its
/// Send slider in the master Reverb section.
pub fn sub_send() -> [f32; SUB_BUS_COUNT] {
    SUB_SEND_AMT.with(Cell::get)
}

pub(crate) fn set_sub_send(i: usize, amount: f32) {
    SUB_SEND_AMT.with(|c| {
        let mut v = c.get();
        if let Some(slot) = v.get_mut(i) {
            *slot = amount;
        }
        c.set(v);
    });
}

/// Panel → shell: each sub-bus delay aux-send amount (0..1; 0 = dry) from its
/// Send slider in the master Delay section.
pub fn sub_delay_send() -> [f32; SUB_BUS_COUNT] {
    SUB_DELAY_SEND_AMT.with(Cell::get)
}

pub(crate) fn set_sub_delay_send(i: usize, amount: f32) {
    SUB_DELAY_SEND_AMT.with(|c| {
        let mut v = c.get();
        if let Some(slot) = v.get_mut(i) {
            *slot = amount;
        }
        c.set(v);
    });
}

/// Panel → shell: each sub-bus compressor amount (0..1; 0 = off) from its Comp
/// slider in the master Compressor section.
pub fn sub_comp() -> [f32; SUB_BUS_COUNT] {
    SUB_COMP_AMT.with(Cell::get)
}

pub(crate) fn set_sub_comp(i: usize, amount: f32) {
    SUB_COMP_AMT.with(|c| {
        let mut v = c.get();
        if let Some(slot) = v.get_mut(i) {
            *slot = amount;
        }
        c.set(v);
    });
}

/// Panel → shell: the master 3-band EQ slider positions (0..1; 0.5 = flat),
/// `[low, mid, high]`. The shell maps each to ±12 dB.
pub fn eq() -> [f32; 3] {
    EQ_POS.with(Cell::get)
}

pub(crate) fn set_eq(band: usize, pos: f32) {
    EQ_POS.with(|c| {
        let mut v = c.get();
        if let Some(slot) = v.get_mut(band) {
            *slot = pos;
        }
        c.set(v);
    });
}

/// Panel → shell: each sub-bus solo flag. When *any* is set, the shell mutes
/// every non-soloed sub-bus (so you hear only the soloed ones).
pub fn sub_soloed() -> [bool; SUB_BUS_COUNT] {
    SUB_SOLOED.with(Cell::get)
}

pub(crate) fn toggle_sub_soloed(i: usize) {
    SUB_SOLOED.with(|c| {
        let mut v = c.get();
        if let Some(slot) = v.get_mut(i) {
            *slot = !*slot;
        }
        c.set(v);
    });
}
