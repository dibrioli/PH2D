//! `ph2d-panel-audio-mixer` — the Audio Mixer panel (Phase 2.3c).
//!
//! Vertical channel strips (the mixer convention the UI research settled on).
//! This scaffold renders the **Master** strip: name + vertical fader (visual) +
//! live [`ph2d_editor_core::widget::LevelMeter`] + mute toggle. The shell's
//! mixer bridge publishes the live snapshot ([`set_snapshot`]) each frame and
//! reads [`master_muted`] back to drive the engine's master gain.
//!
//! The interactive vertical fader (drag → master gain) needs orientation-aware
//! slider dispatch and lands in a follow-up commit; here the fader is a visual
//! readout of the published master gain.

#![forbid(unsafe_code)]

mod event;
mod fader;
mod paint;
mod populate;
pub mod state;

pub use fader::{FADER_UNITY_POS, fader_db, fader_gain};
pub use state::AudioMixerState;

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};
use ph2d_tool_registry::hash_node_id;

/// Outer panel rect id. Single source in editor-core so the hero z-order
/// paint walk (`ids::AUDIO_MIXER_PANEL`) resolves this panel.
pub const AMIX_PANEL: NodeId = ph2d_editor_core::ids::AUDIO_MIXER_PANEL;
/// Header close (X) button — hides the dock (also toggled by the TopBar pill).
pub const AMIX_CLOSE: NodeId = hash_node_id("audio_mixer_close");
/// Master vertical fader (a `Slider` — drag → master gain).
pub const AMIX_FADER: NodeId = hash_node_id("audio_mixer_fader");
/// Master low-pass cutoff (a horizontal `Slider` — drag → filter cutoff).
pub const AMIX_CUTOFF: NodeId = hash_node_id("audio_mixer_cutoff");
/// Master high-pass / low-cut cutoff (a horizontal `Slider` — drag → low-cut Hz).
pub const AMIX_LOWCUT: NodeId = hash_node_id("audio_mixer_lowcut");
/// Master-strip mute toggle.
pub const AMIX_MASTER_MUTE: NodeId = hash_node_id("audio_mixer_master_mute");
/// Master stereo balance (a horizontal `Slider` — drag → master pan).
pub const AMIX_PAN: NodeId = hash_node_id("audio_mixer_pan");
/// Footer "Play Test" toggle — the shell plays a built-in test signal (a pluck
/// loop on Music + a steady tone on SFX) so the mixer is testable without files.
pub const AMIX_PLAY: NodeId = hash_node_id("audio_mixer_play_test");
/// Master meter — clicking it clears the latched clip indicator.
pub const AMIX_MASTER_METER: NodeId = hash_node_id("audio_mixer_master_meter");
/// Master soft-clip limiter toggle (master output stage, below Cutoff).
pub const AMIX_LIMITER: NodeId = hash_node_id("audio_mixer_limiter");
/// Master reverb enable toggle.
pub const AMIX_REVERB: NodeId = hash_node_id("audio_mixer_reverb");
/// Master reverb room-size slider (decay length).
pub const AMIX_REVERB_SIZE: NodeId = hash_node_id("audio_mixer_reverb_size");
/// Master reverb return-level slider (how much of the wet return is folded back;
/// labeled "Return"). The reverb is a send/return fed by the per-bus sends.
pub const AMIX_REVERB_MIX: NodeId = hash_node_id("audio_mixer_reverb_mix");
/// Ducking enable toggle — Music/SFX duck under the Voice bus (dialogue priority).
pub const AMIX_DUCK: NodeId = hash_node_id("audio_mixer_duck");
/// Ducking depth slider (how much the ducked buses drop).
pub const AMIX_DUCK_DEPTH: NodeId = hash_node_id("audio_mixer_duck_depth");

/// Sub-buses shown as their own strips, **in `ph2d_audio::BusId::SUB_BUSES`
/// order** (Music, SFX). The panel is UI-only (no `ph2d-audio` dep); the shell's
/// bridge maps strip index `i` → `BusId::SUB_BUSES[i]`, so this count and order
/// must match the core. A compile-time assert in the shell guards the count.
pub const SUB_BUS_COUNT: usize = 4;
/// Strip labels, index-aligned with [`SUB_BUS_COUNT`].
pub const SUB_BUS_LABELS: [&str; SUB_BUS_COUNT] = ["Music", "SFX", "UI", "Voice"];
/// Per-sub-bus vertical fader ids (drag → that bus's gain).
pub const SUB_FADER: [NodeId; SUB_BUS_COUNT] = [
    hash_node_id("audio_mixer_music_fader"),
    hash_node_id("audio_mixer_sfx_fader"),
    hash_node_id("audio_mixer_ui_fader"),
    hash_node_id("audio_mixer_voice_fader"),
];
/// Per-sub-bus mute toggle ids.
pub const SUB_MUTE: [NodeId; SUB_BUS_COUNT] = [
    hash_node_id("audio_mixer_music_mute"),
    hash_node_id("audio_mixer_sfx_mute"),
    hash_node_id("audio_mixer_ui_mute"),
    hash_node_id("audio_mixer_voice_mute"),
];
/// Per-sub-bus solo toggle ids (solo mutes the other sub-buses).
pub const SUB_SOLO: [NodeId; SUB_BUS_COUNT] = [
    hash_node_id("audio_mixer_music_solo"),
    hash_node_id("audio_mixer_sfx_solo"),
    hash_node_id("audio_mixer_ui_solo"),
    hash_node_id("audio_mixer_voice_solo"),
];
/// Per-sub-bus stereo-balance slider ids (drag → that bus's pan).
pub const SUB_PAN: [NodeId; SUB_BUS_COUNT] = [
    hash_node_id("audio_mixer_music_pan"),
    hash_node_id("audio_mixer_sfx_pan"),
    hash_node_id("audio_mixer_ui_pan"),
    hash_node_id("audio_mixer_voice_pan"),
];
/// Per-sub-bus meter ids (click a meter to clear its latched clip indicator).
pub const SUB_METER: [NodeId; SUB_BUS_COUNT] = [
    hash_node_id("audio_mixer_music_meter"),
    hash_node_id("audio_mixer_sfx_meter"),
    hash_node_id("audio_mixer_ui_meter"),
    hash_node_id("audio_mixer_voice_meter"),
];
/// Per-sub-bus tone (low-pass cutoff) slider ids (drag → that bus's filter).
pub const SUB_TONE: [NodeId; SUB_BUS_COUNT] = [
    hash_node_id("audio_mixer_music_tone"),
    hash_node_id("audio_mixer_sfx_tone"),
    hash_node_id("audio_mixer_ui_tone"),
    hash_node_id("audio_mixer_voice_tone"),
];
/// Per-sub-bus low-cut (high-pass cutoff) slider ids (drag → that bus's low-cut).
pub const SUB_LOWCUT: [NodeId; SUB_BUS_COUNT] = [
    hash_node_id("audio_mixer_music_lowcut"),
    hash_node_id("audio_mixer_sfx_lowcut"),
    hash_node_id("audio_mixer_ui_lowcut"),
    hash_node_id("audio_mixer_voice_lowcut"),
];
/// Per-sub-bus reverb aux-send slider ids (drag → that bus's send into the
/// reverb return). Live in the master Reverb footer section, one labeled row
/// per sub-bus.
pub const SUB_SEND: [NodeId; SUB_BUS_COUNT] = [
    hash_node_id("audio_mixer_music_send"),
    hash_node_id("audio_mixer_sfx_send"),
    hash_node_id("audio_mixer_ui_send"),
    hash_node_id("audio_mixer_voice_send"),
];

/// Zero-size marker implementing the typed Audio Mixer panel contract.
pub struct AudioMixerPanel;

impl Panel for AudioMixerPanel {
    type State = AudioMixerState;

    const ID: &'static str = "audio_mixer";
    const NODE_ID: NodeId = AMIX_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut AudioMixerState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut AudioMixerState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}

/// Live snapshot published by the shell's mixer bridge each frame, read by the
/// panel painter. Thread-local (UI + shell run on the main thread) mirrors the
/// other panels' publish channels (e.g. `panel-painter-layers`).
mod snapshot {
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

    /// Panel → shell: ducking enable + depth (Music/SFX duck under Voice).
    pub fn ducking() -> bool {
        DUCKING.with(Cell::get)
    }

    pub fn duck_depth() -> f32 {
        DUCK_DEPTH.with(Cell::get)
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
}

/// Panel → shell: the master low-pass cutoff (Hz) the Cutoff slider drives.
pub use snapshot::cutoff as master_cutoff_target;
/// Panel → shell: whether the master soft-clip limiter is engaged.
pub use snapshot::limiter;
/// Panel → shell: the master high-pass (low-cut) cutoff (Hz) the Low Cut slider
/// drives (20 Hz = off).
pub use snapshot::lowcut as master_lowcut_target;
/// Observable master clip latch (set when the peak exceeds full scale, cleared
/// by clicking the meter).
pub use snapshot::master_clipped;
/// Panel → shell: the master gain the fader drives — read by the bridge to set
/// the engine's master gain.
pub use snapshot::master_gain as master_gain_target;
/// Panel → shell: the master stereo balance the Master pan slider drives.
pub use snapshot::master_pan as master_pan_target;
/// Panel → shell: whether the Master mute is engaged (bridge zeroes the gain).
pub use snapshot::muted as master_muted;
/// Panel → shell: whether the built-in test signal should be playing.
pub use snapshot::play_test;
/// Shell → panel: publish this frame's master output peak levels for the meter.
pub use snapshot::set_levels;
/// Shell → panel: publish each sub-bus's post-fader peak levels for its meter.
pub use snapshot::set_sub_levels;
/// Panel → shell: each sub-bus fader gain (index-aligned with `BusId::SUB_BUSES`).
pub use snapshot::sub_gain as sub_gain_target;
/// Panel → shell: each sub-bus high-pass (low-cut) cutoff in Hz (from its Low Cut
/// slider; 20 Hz = off).
pub use snapshot::sub_lowcut as sub_lowcut_target;
/// Panel → shell: each sub-bus mute flag (bridge zeroes that bus's gain).
pub use snapshot::sub_muted;
/// Panel → shell: each sub-bus stereo balance (index-aligned with `BusId::SUB_BUSES`).
pub use snapshot::sub_pan as sub_pan_target;
/// Panel → shell: each sub-bus reverb aux-send amount (0..1; 0 = dry).
pub use snapshot::sub_send as sub_send_target;
/// Panel → shell: each sub-bus solo flag (bridge mutes non-soloed buses when any set).
pub use snapshot::sub_soloed;
/// Panel → shell: each sub-bus low-pass cutoff in Hz (from its Tone slider).
pub use snapshot::sub_tone as sub_tone_target;
/// Panel → shell: ducking enable + depth (Music/SFX duck under the Voice bus).
pub use snapshot::{duck_depth, ducking};
/// Panel → shell: master reverb enable / room size / wet-dry mix.
pub use snapshot::{reverb_mix, reverb_on, reverb_size};
