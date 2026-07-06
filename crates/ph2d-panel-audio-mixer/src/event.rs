//! Audio Mixer panel event routing.

use crate::fader::fader_gain;
use crate::state::AudioMixerState;
use crate::{
    AMIX_CLOSE, AMIX_CUTOFF, AMIX_FADER, AMIX_MASTER_MUTE, AMIX_PAN, AudioMixerPanel, SUB_FADER,
    SUB_MUTE, SUB_PAN, SUB_SOLO, snapshot,
};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};

/// Remap a 0..1 slider value to a `-1.0`..`1.0` pan (0.5 → center 0.0).
fn slider_to_pan(v: f32) -> f32 {
    v.clamp(0.0, 1.0) * 2.0 - 1.0
}

pub(crate) fn apply_event(
    _state: &mut AudioMixerState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    match ev {
        WidgetEvent::Click(id) => {
            // TopBar pill toggles the panel (mirrors the Widget Gallery pattern).
            if id == ids::TOPBAR_AUDIO_MIXER {
                let next = !host.panel_visible(AudioMixerPanel::ID);
                host.set_panel_visible(AudioMixerPanel::ID, next);
                return EventOutcome::Consumed;
            }
            // Header close (X) hides the dock.
            if id == AMIX_CLOSE {
                host.set_panel_visible(AudioMixerPanel::ID, false);
                return EventOutcome::Consumed;
            }
            if id == AMIX_MASTER_MUTE {
                snapshot::toggle_muted();
                return EventOutcome::Consumed;
            }
            // Per-sub-bus mute toggles.
            if let Some(i) = SUB_MUTE.iter().position(|&m| m == id) {
                snapshot::toggle_sub_muted(i);
                return EventOutcome::Consumed;
            }
            // Per-sub-bus solo toggles.
            if let Some(i) = SUB_SOLO.iter().position(|&s| s == id) {
                snapshot::toggle_sub_soloed(i);
                return EventOutcome::Consumed;
            }
        }
        // Master fader dragged — the shared slider dispatch wrote the new 0..1
        // position into the store; publish its dB-tapered gain for the shell.
        WidgetEvent::ValueChanged(id) if id == AMIX_FADER => {
            let pos = host
                .store()
                .slider(AMIX_FADER)
                .map(|(_, v)| v)
                .unwrap_or(1.0);
            snapshot::set_master_gain(fader_gain(pos));
            return EventOutcome::Consumed;
        }
        // Master cutoff dragged — log-map the 0..1 slider to 20 Hz..20 kHz.
        WidgetEvent::ValueChanged(id) if id == AMIX_CUTOFF => {
            let v = host
                .store()
                .slider(AMIX_CUTOFF)
                .map(|(_, v)| v)
                .unwrap_or(1.0);
            let hz = 20.0 * 1000.0_f32.powf(v.clamp(0.0, 1.0)); // LITERAL-PX-OK: cutoff log map (20 Hz..20 kHz)
            snapshot::set_cutoff(hz);
            return EventOutcome::Consumed;
        }
        // Master pan dragged — remap 0..1 → -1..1.
        WidgetEvent::ValueChanged(id) if id == AMIX_PAN => {
            let v = host.store().slider(AMIX_PAN).map(|(_, v)| v).unwrap_or(0.5);
            snapshot::set_master_pan(slider_to_pan(v));
            return EventOutcome::Consumed;
        }
        // A sub-bus fader or pan dragged — publish that bus's gain / pan.
        WidgetEvent::ValueChanged(id) => {
            if let Some(i) = SUB_FADER.iter().position(|&f| f == id) {
                let pos = host.store().slider(id).map(|(_, v)| v).unwrap_or(1.0);
                snapshot::set_sub_gain(i, fader_gain(pos));
                return EventOutcome::Consumed;
            }
            if let Some(i) = SUB_PAN.iter().position(|&p| p == id) {
                let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
                snapshot::set_sub_pan(i, slider_to_pan(v));
                return EventOutcome::Consumed;
            }
        }
        _ => {}
    }
    EventOutcome::Ignored
}
