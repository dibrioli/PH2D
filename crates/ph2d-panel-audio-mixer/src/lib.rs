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
/// Master-strip mute toggle.
pub const AMIX_MASTER_MUTE: NodeId = hash_node_id("audio_mixer_master_mute");

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
    use std::cell::Cell;

    thread_local! {
        static LEVELS: Cell<[f32; 2]> = const { Cell::new([0.0, 0.0]) };
        static MASTER_GAIN: Cell<f32> = const { Cell::new(1.0) };
        static MUTED: Cell<bool> = const { Cell::new(false) };
    }

    /// Shell → panel: current output peak levels for the meter.
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
}

/// Shell → panel: publish this frame's output peak levels for the meter.
pub use snapshot::set_levels;
/// Panel → shell: the master gain the fader drives — read by the bridge to set
/// the engine's master gain.
pub use snapshot::master_gain as master_gain_target;
/// Panel → shell: whether the Master mute is engaged (bridge zeroes the gain).
pub use snapshot::muted as master_muted;
