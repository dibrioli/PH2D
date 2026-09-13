//! **A fase-filha do MIXER DE ÁUDIO** — a faixa master e os sub-barramentos do painel `audio_mixer` aplicados ao
//! motor. Chamada pela [`super`] (`fase_audio_panels`) no sítio do bloco, depois do `poll` (OBRA 3 da
//! `line/render-bodies`); re-deriva o `self.audio`, que não troca.

impl crate::App {
    /// A faixa master (nível, ganho, filtros, pan, limitador, reverb, EQ, delay) e os sub-barramentos (solo, mute,
    /// ducking, sends).
    pub(super) fn fase_audio_mixer(&mut self) {
        let Some(audio) = self.audio.as_mut() else {
            return;
        };
        // Built-in test oscillator (panel footer Play Test).
        audio.set_test_playing(ph2d_panel_audio_mixer::play_test());
        // Master strip.
        ph2d_panel_audio_mixer::set_levels(audio.levels(), audio.rms());
        ph2d_panel_audio_mixer::set_loudness(audio.momentary_lufs());
        let muted = ph2d_panel_audio_mixer::master_muted();
        let gain = ph2d_panel_audio_mixer::master_gain_target();
        audio.set_master_gain(if muted { 0.0 } else { gain });
        audio.set_master_cutoff(ph2d_panel_audio_mixer::master_cutoff_target());
        audio.set_master_highpass(ph2d_panel_audio_mixer::master_lowcut_target());
        audio.set_master_pan(ph2d_panel_audio_mixer::master_pan_target());
        audio.set_master_limiter(ph2d_panel_audio_mixer::limiter());
        audio.set_reverb(
            ph2d_panel_audio_mixer::reverb_on(),
            ph2d_panel_audio_mixer::reverb_size(),
            ph2d_panel_audio_mixer::reverb_mix(),
        );
        // Master 3-band EQ: the panel publishes 0..1 slider positions
        // (0.5 = flat); map each to ±12 dB for the engine.
        const EQ_DB_RANGE: f32 = 24.0; // ±12 dB across the 0..1 EQ slider
        let eq = ph2d_panel_audio_mixer::master_eq_target();
        audio.set_master_eq(
            (eq[0] - 0.5) * EQ_DB_RANGE,
            (eq[1] - 0.5) * EQ_DB_RANGE,
            (eq[2] - 0.5) * EQ_DB_RANGE,
        );
        // Master delay/echo: Time slider position is seconds (0..1 s);
        // feedback + return mix are raw 0..1. The engine clamps time.
        audio.set_delay(
            ph2d_panel_audio_mixer::delay_on(),
            ph2d_panel_audio_mixer::delay_time(),
            ph2d_panel_audio_mixer::delay_feedback(),
            ph2d_panel_audio_mixer::delay_mix(),
        );
        // Sub-bus strips — index-aligned with `BusId::SUB_BUSES` (the
        // panel's strip index i maps to sub-bus i; count guarded below).
        ph2d_panel_audio_mixer::set_sub_levels(audio.bus_levels(), audio.bus_rms());
        let sub_muted = ph2d_panel_audio_mixer::sub_muted();
        let sub_soloed = ph2d_panel_audio_mixer::sub_soloed();
        let sub_gain = ph2d_panel_audio_mixer::sub_gain_target();
        let sub_pan = ph2d_panel_audio_mixer::sub_pan_target();
        let sub_tone = ph2d_panel_audio_mixer::sub_tone_target();
        let sub_lowcut = ph2d_panel_audio_mixer::sub_lowcut_target();
        let sub_send = ph2d_panel_audio_mixer::sub_send_target();
        let sub_delay_send = ph2d_panel_audio_mixer::sub_delay_send_target();
        let sub_comp = ph2d_panel_audio_mixer::sub_comp_target();
        // Sidechain ducking: every bus drops under the selected key bus.
        let duck_key = ph2d_panel_audio_mixer::ducking_key();
        let duck = audio.update_ducking(
            ph2d_panel_audio_mixer::ducking(),
            ph2d_panel_audio_mixer::duck_depth(),
            duck_key,
        );
        // Solo overrides mute: when any bus is soloed, only soloed buses
        // sound; otherwise a bus sounds unless it's muted.
        let any_solo = sub_soloed.iter().any(|&s| s);
        for i in 0..ph2d_audio::SUB_BUS_COUNT {
            let sounds = if any_solo {
                sub_soloed[i]
            } else {
                !sub_muted[i]
            };
            // Every bus except the key ducks under it (the key itself
            // stays at full level so it cuts through).
            let ducks = i != duck_key;
            let mut g = if sounds { sub_gain[i] } else { 0.0 };
            if ducks {
                g *= duck;
            }
            audio.set_bus_gain(i, g);
            audio.set_bus_pan(i, sub_pan[i]);
            audio.set_bus_cutoff(i, sub_tone[i]);
            audio.set_bus_highpass(i, sub_lowcut[i]);
            audio.set_bus_send(i, sub_send[i]);
            audio.set_bus_delay_send(i, sub_delay_send[i]);
            audio.set_bus_compressor(i, sub_comp[i]);
        }
    }
}
