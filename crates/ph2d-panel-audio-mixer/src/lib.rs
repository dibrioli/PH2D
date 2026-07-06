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
mod paint;
mod populate;
pub mod state;

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
/// Master-strip mute toggle.
pub const AMIX_MASTER_MUTE: NodeId = hash_node_id("audio_mixer_master_mute");
/// Master stereo balance (a horizontal `Slider` — drag → master pan).
pub const AMIX_PAN: NodeId = hash_node_id("audio_mixer_pan");

/// Sub-buses shown as their own strips, **in `ph2d_audio::BusId::SUB_BUSES`
/// order** (Music, SFX). The panel is UI-only (no `ph2d-audio` dep); the shell's
/// bridge maps strip index `i` → `BusId::SUB_BUSES[i]`, so this count and order
/// must match the core. A compile-time assert in the shell guards the count.
pub const SUB_BUS_COUNT: usize = 2;
/// Strip labels, index-aligned with [`SUB_BUS_COUNT`].
pub const SUB_BUS_LABELS: [&str; SUB_BUS_COUNT] = ["Music", "SFX"];
/// Per-sub-bus vertical fader ids (drag → that bus's gain).
pub const SUB_FADER: [NodeId; SUB_BUS_COUNT] = [
    hash_node_id("audio_mixer_music_fader"),
    hash_node_id("audio_mixer_sfx_fader"),
];
/// Per-sub-bus mute toggle ids.
pub const SUB_MUTE: [NodeId; SUB_BUS_COUNT] = [
    hash_node_id("audio_mixer_music_mute"),
    hash_node_id("audio_mixer_sfx_mute"),
];
/// Per-sub-bus stereo-balance slider ids (drag → that bus's pan).
pub const SUB_PAN: [NodeId; SUB_BUS_COUNT] = [
    hash_node_id("audio_mixer_music_pan"),
    hash_node_id("audio_mixer_sfx_pan"),
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

    thread_local! {
        static LEVELS: Cell<[f32; 2]> = const { Cell::new([0.0, 0.0]) };
        static MASTER_GAIN: Cell<f32> = const { Cell::new(1.0) };
        static CUTOFF_HZ: Cell<f32> = const { Cell::new(20_000.0) }; // LITERAL-PX-OK: default cutoff 20 kHz (audio frequency, not a UI metric)
        static MUTED: Cell<bool> = const { Cell::new(false) };
        static MASTER_PAN: Cell<f32> = const { Cell::new(0.0) };
        // Per-sub-bus channels, index-aligned with `BusId::SUB_BUSES`.
        static SUB_LEVELS: Cell<[[f32; 2]; SUB_BUS_COUNT]> = const { Cell::new([[0.0, 0.0]; SUB_BUS_COUNT]) };
        static SUB_GAIN: Cell<[f32; SUB_BUS_COUNT]> = const { Cell::new([1.0; SUB_BUS_COUNT]) };
        static SUB_MUTED: Cell<[bool; SUB_BUS_COUNT]> = const { Cell::new([false; SUB_BUS_COUNT]) };
        static SUB_PAN: Cell<[f32; SUB_BUS_COUNT]> = const { Cell::new([0.0; SUB_BUS_COUNT]) };
    }

    /// Shell → panel: current master output peak levels for the meter.
    pub fn set_levels(levels: [f32; 2]) {
        LEVELS.with(|c| c.set(levels));
    }

    pub(crate) fn levels() -> [f32; 2] {
        LEVELS.with(Cell::get)
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

    /// Shell → panel: current post-fader peak levels per sub-bus.
    pub fn set_sub_levels(levels: [[f32; 2]; SUB_BUS_COUNT]) {
        SUB_LEVELS.with(|c| c.set(levels));
    }

    pub(crate) fn sub_levels() -> [[f32; 2]; SUB_BUS_COUNT] {
        SUB_LEVELS.with(Cell::get)
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
}

/// Shell → panel: publish this frame's master output peak levels for the meter.
pub use snapshot::set_levels;
/// Panel → shell: the master gain the fader drives — read by the bridge to set
/// the engine's master gain.
pub use snapshot::master_gain as master_gain_target;
/// Panel → shell: the master low-pass cutoff (Hz) the Cutoff slider drives.
pub use snapshot::cutoff as master_cutoff_target;
/// Panel → shell: whether the Master mute is engaged (bridge zeroes the gain).
pub use snapshot::muted as master_muted;
/// Panel → shell: the master stereo balance the Master pan slider drives.
pub use snapshot::master_pan as master_pan_target;
/// Shell → panel: publish each sub-bus's post-fader peak levels for its meter.
pub use snapshot::set_sub_levels;
/// Panel → shell: each sub-bus fader gain (index-aligned with `BusId::SUB_BUSES`).
pub use snapshot::sub_gain as sub_gain_target;
/// Panel → shell: each sub-bus mute flag (bridge zeroes that bus's gain).
pub use snapshot::sub_muted;
/// Panel → shell: each sub-bus stereo balance (index-aligned with `BusId::SUB_BUSES`).
pub use snapshot::sub_pan as sub_pan_target;
