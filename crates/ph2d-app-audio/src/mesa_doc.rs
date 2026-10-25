//! ⭐⭐ **A MESA DE SOM é documento** (F30 §B2) — o recurso [`MesaDoc`] no mundo da simulação guarda
//! o AUTORADO (o que o `.ph2dproj` grava, `project_save.rs`) e o VIVO (o autorado com a curva da Cena
//! por cima, `canais_mesa::aplicar`: o que o painel pinta e o motor ouve).
//!
//! A cada quadro, [`quadro`] toma a edição do painel para o autorado, devolve-lhe os dois retratos e
//! manda o vivo ao motor. Fora do `ProjectState`/undo como o `physics`: a mesa nunca teve undo.

use ph2d_audio::Mesa;
use ph2d_ecs::{Resource, SimWorld};

use crate::AudioSystem;
use crate::canais_mesa::ConducaoMesa;

/// A mesa do documento aberto.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct MesaDoc {
    pub autorado: Mesa,
    pub vivo: Mesa,
    /// O autorado visto no quadro anterior: a mão ganha à curva no quadro em que muda.
    pub conducao: ConducaoMesa,
}

/// O que o documento tem agora (a de fábrica se ninguém a instalou).
#[must_use]
pub fn doc(sim: &SimWorld) -> MesaDoc {
    sim.world()
        .get_resource::<MesaDoc>()
        .cloned()
        .unwrap_or_default()
}

/// O que o save grava: o AUTORADO, nunca a curva.
#[must_use]
pub fn autorado(sim: &SimWorld) -> Mesa {
    doc(sim).autorado
}

/// O load: a mesa do ficheiro entra inteira, e não é a mão.
pub fn instalar(sim: &mut SimWorld, mesa: Mesa) {
    sim.world_mut().insert_resource(MesaDoc {
        autorado: mesa,
        vivo: mesa,
        conducao: ConducaoMesa::default(),
    });
    let _ = ph2d_panel_audio_mixer::tomar_edicao(); // a mão do documento anterior não entra
    ph2d_panel_audio_mixer::publicar_mesa(mesa, mesa);
}

/// ⭐ **O quadro da mesa**: a edição do painel entra no autorado (e no vivo, que a mostra já), o
/// painel recebe os dois retratos, e o motor ouve o vivo.
pub fn quadro(sim: &mut SimWorld, audio: Option<&mut AudioSystem>) {
    let mut d = doc(sim);
    if let Some(a) = ph2d_panel_audio_mixer::tomar_edicao() {
        d.autorado = a;
        d.vivo = ph2d_panel_audio_mixer::mesa_viva();
    }
    ph2d_panel_audio_mixer::publicar_mesa(d.autorado, d.vivo);
    if let Some(audio) = audio {
        audio.set_test_playing(ph2d_panel_audio_mixer::play_test());
        ph2d_panel_audio_mixer::set_levels(audio.levels(), audio.rms());
        ph2d_panel_audio_mixer::set_loudness(audio.momentary_lufs());
        ph2d_panel_audio_mixer::set_sub_levels(audio.bus_levels(), audio.bus_rms());
        aplicar_ao_motor(audio, &d.vivo);
    }
    sim.world_mut().insert_resource(d);
}

/// A mesa `m` no motor: a faixa master (ganho, filtros, pan, limitador, reverb, EQ, delay) e os
/// sub-barramentos (solo, mute, ducking, sends).
pub fn aplicar_ao_motor(audio: &mut AudioSystem, m: &Mesa) {
    /// ±12 dB sobre o slider `0..1` do EQ (`0.5` = plano).
    const EQ_DB_RANGE: f32 = 24.0;
    audio.set_master_gain(if m.mudo { 0.0 } else { m.ganho });
    audio.set_master_cutoff(m.corte_alto_hz);
    audio.set_master_highpass(m.corte_baixo_hz);
    audio.set_master_pan(m.pan);
    audio.set_master_limiter(m.limitador);
    audio.set_reverb(m.reverb, m.reverb_tamanho, m.reverb_retorno);
    let db = |p: f32| (p - 0.5) * EQ_DB_RANGE;
    audio.set_master_eq(db(m.eq[0]), db(m.eq[1]), db(m.eq[2]));
    audio.set_delay(m.delay, m.delay_tempo, m.delay_feedback, m.delay_retorno);
    let chave = usize::from(m.ducking_chave);
    let duck = audio.update_ducking(m.ducking, m.ducking_profundidade, chave);
    // O solo manda no mute: com algum solo, só os solados soam.
    let algum_solo = m.bus.iter().any(|b| b.solo);
    for (i, b) in m.bus.iter().enumerate() {
        let soa = if algum_solo { b.solo } else { !b.mudo };
        let mut g = if soa { b.ganho } else { 0.0 };
        if i != chave {
            g *= duck;
        }
        audio.set_bus_gain(i, g);
        audio.set_bus_pan(i, b.pan);
        audio.set_bus_cutoff(i, b.tom_hz);
        audio.set_bus_highpass(i, b.corte_baixo_hz);
        audio.set_bus_send(i, b.envio_reverb);
        audio.set_bus_delay_send(i, b.envio_delay);
        audio.set_bus_compressor(i, b.comp);
    }
}
