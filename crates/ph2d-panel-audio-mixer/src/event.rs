//! Audio Mixer panel event routing.

use crate::fader::fader_gain;
use crate::state::AudioMixerState;
use crate::{
    AMIX_CLOSE, AMIX_CUTOFF, AMIX_DUCK, AMIX_DUCK_DEPTH, AMIX_DUCK_KEY, AMIX_EQ_HIGH, AMIX_EQ_LOW,
    AMIX_EQ_MID, AMIX_FADER, AMIX_LIMITER, AMIX_LOWCUT, AMIX_MASTER_METER, AMIX_MASTER_MUTE,
    AMIX_PAN, AMIX_PLAY, AMIX_REVERB, AMIX_REVERB_MIX, AMIX_REVERB_SIZE, AudioMixerPanel,
    SUB_FADER, SUB_LOWCUT, SUB_METER, SUB_MUTE, SUB_PAN, SUB_SEND, SUB_SOLO, SUB_TONE, snapshot,
};

/// Log-map a 0..1 tone slider to a cutoff in Hz (20 Hz..20 kHz).
fn slider_to_hz(v: f32) -> f32 {
    20.0 * 1000.0_f32.powf(v.clamp(0.0, 1.0)) // LITERAL-PX-OK: cutoff log map (20 Hz..20 kHz)
}
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
            // Footer Play Test toggle — the shell starts/stops the test signal.
            if id == AMIX_PLAY {
                snapshot::toggle_play_test();
                return EventOutcome::Consumed;
            }
            // Master output limiter toggle.
            if id == AMIX_LIMITER {
                snapshot::toggle_limiter();
                return EventOutcome::Consumed;
            }
            // Master reverb enable toggle.
            if id == AMIX_REVERB {
                snapshot::toggle_reverb();
                return EventOutcome::Consumed;
            }
            // Ducking enable toggle.
            if id == AMIX_DUCK {
                snapshot::toggle_ducking();
                return EventOutcome::Consumed;
            }
            // Sidechain key-bus selector — cycle to the next sub-bus.
            if id == AMIX_DUCK_KEY {
                snapshot::cycle_duck_key();
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
            // Click a meter to clear its latched clip indicator.
            if id == AMIX_MASTER_METER {
                snapshot::clear_master_clip();
                return EventOutcome::Consumed;
            }
            if let Some(i) = SUB_METER.iter().position(|&m| m == id) {
                snapshot::clear_sub_clip(i);
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
        // Master tone (cutoff) dragged — log-map the 0..1 slider to 20 Hz..20 kHz.
        WidgetEvent::ValueChanged(id) if id == AMIX_CUTOFF => {
            let v = host
                .store()
                .slider(AMIX_CUTOFF)
                .map(|(_, v)| v)
                .unwrap_or(1.0);
            snapshot::set_cutoff(slider_to_hz(v));
            return EventOutcome::Consumed;
        }
        // Master low-cut (high-pass) dragged — same log map; 0.0 → ~20 Hz (off).
        WidgetEvent::ValueChanged(id) if id == AMIX_LOWCUT => {
            let v = host
                .store()
                .slider(AMIX_LOWCUT)
                .map(|(_, v)| v)
                .unwrap_or(0.0);
            snapshot::set_lowcut(slider_to_hz(v));
            return EventOutcome::Consumed;
        }
        // Master pan dragged — remap 0..1 → -1..1.
        WidgetEvent::ValueChanged(id) if id == AMIX_PAN => {
            let v = host.store().slider(AMIX_PAN).map(|(_, v)| v).unwrap_or(0.5);
            snapshot::set_master_pan(slider_to_pan(v));
            return EventOutcome::Consumed;
        }
        // Reverb Size / Mix dragged — publish the raw 0..1 value.
        WidgetEvent::ValueChanged(id) if id == AMIX_REVERB_SIZE => {
            let v = host
                .store()
                .slider(AMIX_REVERB_SIZE)
                .map(|(_, v)| v)
                .unwrap_or(0.5);
            snapshot::set_reverb_size(v.clamp(0.0, 1.0));
            return EventOutcome::Consumed;
        }
        WidgetEvent::ValueChanged(id) if id == AMIX_REVERB_MIX => {
            let v = host
                .store()
                .slider(AMIX_REVERB_MIX)
                .map(|(_, v)| v)
                .unwrap_or(0.3); // LITERAL-PX-OK: default reverb wet/dry mix (audio param)
            snapshot::set_reverb_mix(v.clamp(0.0, 1.0));
            return EventOutcome::Consumed;
        }
        WidgetEvent::ValueChanged(id) if id == AMIX_DUCK_DEPTH => {
            let v = host
                .store()
                .slider(AMIX_DUCK_DEPTH)
                .map(|(_, v)| v)
                .unwrap_or(0.7); // LITERAL-PX-OK: default duck depth (audio param)
            snapshot::set_duck_depth(v.clamp(0.0, 1.0));
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
            if let Some(i) = SUB_TONE.iter().position(|&t| t == id) {
                let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(1.0);
                snapshot::set_sub_tone(i, slider_to_hz(v));
                return EventOutcome::Consumed;
            }
            if let Some(i) = SUB_LOWCUT.iter().position(|&t| t == id) {
                let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
                snapshot::set_sub_lowcut(i, slider_to_hz(v));
                return EventOutcome::Consumed;
            }
            if let Some(i) = SUB_SEND.iter().position(|&s| s == id) {
                let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.0);
                snapshot::set_sub_send(i, v.clamp(0.0, 1.0));
                return EventOutcome::Consumed;
            }
            // Master 3-band EQ — publish the raw 0..1 position; the shell maps it
            // to ±12 dB. Index-aligned [low, mid, high].
            let eq_ids = [AMIX_EQ_LOW, AMIX_EQ_MID, AMIX_EQ_HIGH];
            if let Some(band) = eq_ids.iter().position(|&e| e == id) {
                let v = host.store().slider(id).map(|(_, v)| v).unwrap_or(0.5);
                snapshot::set_eq(band, v.clamp(0.0, 1.0));
                return EventOutcome::Consumed;
            }
        }
        _ => {}
    }
    EventOutcome::Ignored
}
