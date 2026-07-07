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
mod paint_widgets;
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
/// Master 3-band EQ gain sliders (low shelf / mid peak / high shelf; 0.5 = flat).
pub const AMIX_EQ_LOW: NodeId = hash_node_id("audio_mixer_eq_low");
pub const AMIX_EQ_MID: NodeId = hash_node_id("audio_mixer_eq_mid");
pub const AMIX_EQ_HIGH: NodeId = hash_node_id("audio_mixer_eq_high");
/// Ducking enable toggle — Music/SFX duck under the Voice bus (dialogue priority).
pub const AMIX_DUCK: NodeId = hash_node_id("audio_mixer_duck");
/// Ducking depth slider (how much the ducked buses drop).
pub const AMIX_DUCK_DEPTH: NodeId = hash_node_id("audio_mixer_duck_depth");
/// Sidechain key-bus selector button — cycles which sub-bus everything ducks under.
pub const AMIX_DUCK_KEY: NodeId = hash_node_id("audio_mixer_duck_key");

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

mod snapshot;

/// Panel → shell: the master low-pass cutoff (Hz) the Cutoff slider drives.
pub use snapshot::cutoff as master_cutoff_target;
/// Panel → shell: the master 3-band EQ slider positions (0..1; 0.5 = flat).
pub use snapshot::eq as master_eq_target;
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
/// Shell → panel: publish this frame's master momentary loudness (LUFS).
pub use snapshot::set_loudness;
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
pub use snapshot::{duck_depth, ducking, ducking_key};
/// Panel → shell: master reverb enable / room size / wet-dry mix.
pub use snapshot::{reverb_mix, reverb_on, reverb_size};
