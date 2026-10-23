//! ⭐ **A lei dos números da FONTE DE SOM** (doc CodeEditor 08 §3.5): o cartão do Inspector (o
//! `apply_audio_edit` da shell, `render_loop/inspector_audio.rs`) e o script
//! (`ph2d.set(e, "audio_source_2d.volume", …)`) passam os dois por aqui — o script não põe o que o
//! painel recusa.

use ph2d_ecs::{AUDIO_MAX_DISTANCE_M, AUDIO_MAX_POLYPHONY, AudioBus, AudioSource2D};

/// `v` dentro de `[lo, hi]`; um valor não finito volta ao de fábrica.
fn faixa(v: f32, (lo, hi): (f32, f32), livre: f32) -> f32 {
    if v.is_finite() {
        v.clamp(lo, hi) // CLAMP-OK: a faixa que o cartão aceita
    } else {
        livre
    }
}

/// O saneamento de uma `AudioSource2D` — idempotente.
pub fn sanitize_audio_source(s: &mut AudioSource2D) {
    let livre = AudioSource2D::default();
    if s.sound.trim().len() != s.sound.len() {
        s.sound = s.sound.trim().to_owned();
    }
    s.volume_db = faixa(s.volume_db, (-80.0, 24.0), livre.volume_db);
    s.pitch = faixa(s.pitch, (0.05, 8.0), livre.pitch);
    // A saturação das distâncias é do MOTOR.
    s.max_distance = faixa(
        s.max_distance,
        (0.0, AUDIO_MAX_DISTANCE_M),
        livre.max_distance,
    );
    s.attenuation = faixa(s.attenuation, (0.0, 8.0), livre.attenuation);
    s.non_spatialized_radius = faixa(
        s.non_spatialized_radius,
        (0.0, AUDIO_MAX_DISTANCE_M),
        livre.non_spatialized_radius,
    );
    s.panning_strength = faixa(s.panning_strength, (0.0, 1.0), livre.panning_strength);
    // O teto das vozes é o do pool do mixer (o cartão recusa-o com voz antes de chegar aqui).
    s.max_polyphony = s.max_polyphony.clamp(1, AUDIO_MAX_POLYPHONY); // CLAMP-OK: o teto é o do motor
    s.bus_tag = AudioBus::from_tag(s.bus_tag).tag();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lei_do_som_satura_volta_ao_de_fabrica_e_e_idempotente() {
        let mut s = AudioSource2D {
            sound: "  passo.wav ".into(),
            volume_db: 99.0,
            pitch: f32::NAN,
            max_polyphony: 0,
            bus_tag: 200,
            ..AudioSource2D::default()
        };
        sanitize_audio_source(&mut s);
        assert_eq!(s.sound, "passo.wav");
        assert_eq!(s.volume_db, 24.0);
        assert_eq!(s.pitch, AudioSource2D::default().pitch);
        assert_eq!(s.max_polyphony, 1);
        assert_eq!(s.bus_tag, AudioBus::default().tag());
        let uma = s.clone();
        sanitize_audio_source(&mut s);
        assert_eq!(s, uma);
    }
}
