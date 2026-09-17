//! **OS DOIS MOTORES QUE CORREM NA JANELA DOS CÉREBROS** — os scripts do artista (TOP-20 #16) e os
//! emissores de partículas (TOP-20 #18), tirados do corpo da [`super::fase_signal_outbox`].
//!
//! ⚠️ **São funções LIVRES e não fases-filhas**, e a razão é um empréstimo: o outbox segura o `sim`
//! (e o resto do `gfx`) do princípio ao fim do corpo, então um `self.fase_*(…)` a meio é um segundo
//! empréstimo de `self`. Cada motor recebe as peças de que precisa — e a CHAMADA fica no corpo do
//! outbox, que é o que o texto emendado do quadro colhe: a ORDEM continua medível.
//!
//! ⚠️ **Saíram por TECTO DE LOC** (`fase_signal_outbox` foi a `229` contra `200` ao ganhar o
//! terceiro motor) — cura por CORTE, nunca uma entrada nova de dívida.

use ph2d_app_components::particles_bridge::ParticlesState;
use ph2d_ecs::SimWorld;
use ph2d_ecs::sort_key::SortScratch;
use ph2d_runtime::{SignalOutbox, SignalReader};
use ph2d_script::ScriptHost;

/// **O relógio do quadro**, para os dois motores: se a corrida anda, quantos passos fixos e quanto
/// dura cada um.
pub(super) struct Relogio {
    /// A corrida está a andar (`playhead.is_playing()`)?
    pub(super) playing: bool,
    /// Quantos passos fixos este quadro deve.
    pub(super) ticks: u32,
    /// Quanto dura um passo fixo.
    pub(super) dt: f64,
}

/// ⭐⭐⭐ **OS SCRIPTS DO ARTISTA** (TOP-20 #16) — o que um script emite chega à tabela de acções
/// NESTE quadro.
///
/// ⚠️ O cursor lê em TODO quadro, mesmo parado (a lei do `ui_signal_reader`), e uma falha imprime
/// UMA linha — a mensagem fica no Inspector.
pub(super) fn scripts(
    sim: &mut SimWorld,
    script: &mut Option<ScriptHost>,
    drive: &mut ph2d_preview_drive::PreviewDrive,
    signals: &mut SignalOutbox,
    reader: &mut SignalReader,
    relogio: &Relogio,
) {
    let ouvidos: Vec<String> = signals.read(reader).map(|s| s.name.to_string()).collect();
    let Some(host) = script.as_mut() else {
        return;
    };
    let nomes: Vec<&str> = ouvidos.iter().map(String::as_str).collect();
    let f = ph2d_app_components::script_bridge::frame(
        host,
        sim,
        drive,
        relogio.playing,
        relogio.ticks,
        relogio.dt,
        &nomes,
    );
    for (bits, nome) in f.emitted {
        signals.publish(ph2d_runtime::Signal::from_script(&nome, bits));
    }
    for (bits, msg) in f.failed {
        eprintln!("[script] o objecto {bits} parou: {msg}");
    }
}

/// ⭐⭐⭐ **OS EMISSORES DE PARTÍCULAS** (TOP-20 #18) — o que um emissor GRITA ao acabar chega à
/// tabela de acções NESTE quadro, e o que o liga/desliga/recomeça foi ouvido aqui.
///
/// ⚠️ **A profundidade vem do `sort_scratch`, que é do quadro ANTERIOR** — esta janela corre antes
/// do `fase_sim_extract`, que é quem o preenche. Uma partícula desenha-se à profundidade que o
/// objecto tinha no quadro passado, e um objecto acabado de reordenar na Hierarquia leva um quadro
/// a levar o penacho consigo. ⛔ A alternativa (carimbar no presente) obrigaria cada instância a
/// lembrar-se de quem a emitiu.
pub(super) fn particulas(
    sim: &mut SimWorld,
    particles: &mut ParticlesState,
    scratch: &SortScratch,
    signals: &mut SignalOutbox,
    reader: &mut SignalReader,
    relogio: &Relogio,
) {
    let ouvidos: Vec<String> = signals.read(reader).map(|s| s.name.to_string()).collect();
    let f = particles.frame(
        sim,
        relogio.playing,
        relogio.ticks,
        relogio.dt,
        &ouvidos,
        |e| scratch.rank(e),
    );
    for (bits, nome) in f.finished {
        signals.publish(ph2d_runtime::Signal::from_particles(&nome, bits));
    }
}
