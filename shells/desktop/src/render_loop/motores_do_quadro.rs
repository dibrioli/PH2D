//! **OS MOTORES QUE CORREM NA JANELA DOS CÉREBROS** — os scripts do artista (TOP-20 #16), os
//! emissores de partículas (TOP-20 #18) e as vigias de contador, tirados do corpo da
//! [`super::fase_signal_outbox`].
//!
//! ⚠️ **São funções LIVRES e não fases-filhas**, e a razão é um empréstimo: o outbox segura o `sim`
//! (e o resto do `gfx`) do princípio ao fim do corpo, então um `self.fase_*(…)` a meio é um segundo
//! empréstimo de `self`. Cada motor recebe as peças de que precisa — e a CHAMADA fica no corpo do
//! outbox, que é o que o texto emendado do quadro colhe: a ORDEM continua medível.
//!
//! ⚠️ **Saíram por TECTO DE LOC** (`fase_signal_outbox` foi a `229` contra `200` ao ganhar o
//! terceiro motor) — cura por CORTE, nunca uma entrada nova de dívida.

use std::collections::BTreeMap;

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

impl Relogio {
    /// **O relógio deste quadro.**
    ///
    /// ⚠️ **Construir-se é assunto de quem é dono do tipo** — e o corte foi imposto pelo tecto de
    /// 200 LOC da `fase_signal_outbox` (ela chegou a `202` ao ganhar o terceiro motor). *A cura de
    /// um tecto é o CORTE, nunca uma entrada nova no `FN_OVERAGE_OK`.*
    pub(super) const fn do_quadro(playing: bool, ticks: u32, dt: f64) -> Self {
        Self { playing, ticks, dt }
    }
}

/// ⭐⭐⭐ **OS TRÊS MOTORES, pela ORDEM** — uma chamada só.
///
/// ⚠️ **A ordem é o contrato, e é o que o texto emendado do quadro mede:** os três falam ANTES de
/// a tabela de acções ler, que é o que faz uma porta abrir no MESMO quadro em que o botão é tocado.
///
/// ⚠️ **Agrupá-los foi imposto pelo tecto de 200 LOC da [`super::fase_signal_outbox`]** (ela chegou
/// a `202` ao ganhar o terceiro) **e é o certo por responsabilidade:** *correr os motores* é UM
/// passo do quadro, e o quarto entra aqui sem tocar na fase.
#[allow(clippy::too_many_arguments)]
pub(super) fn correm(
    sim: &mut SimWorld,
    script: &mut Option<ScriptHost>,
    particles: &mut ParticlesState,
    scratch: &SortScratch,
    drive: &mut ph2d_preview_drive::PreviewDrive,
    signals: &mut SignalOutbox,
    leitores: &mut crate::app_state::app_state_signal_readers::SignalReaders,
    relogio: &Relogio,
    accoes: &BTreeMap<String, ph2d_ecs::ActionSample>,
) {
    scripts(sim, script, drive, signals, &mut leitores.script, relogio);
    particulas(
        sim,
        particles,
        scratch,
        signals,
        &mut leitores.particles,
        relogio,
    );
    vigias(sim, signals, relogio);
    gatilhos(sim, signals, relogio, accoes);
}

/// ⭐⭐⭐ **OS SCRIPTS DO ARTISTA** (TOP-20 #16) — o que um script emite chega à tabela de acções
/// NESTE quadro.
///
/// ⚠️ O cursor lê em TODO quadro, mesmo parado (a lei do `ui_signal_reader`), e uma falha imprime
/// UMA linha — a mensagem fica no Inspector.
fn scripts(
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
fn particulas(
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

/// ⭐⭐⭐ **AS VIGIAS DE CONTADOR** — uma travessia de limiar chega à tabela de acções NESTE quadro.
///
/// ⚠️ **Ela não OUVE, só FALA** — logo não tem `SignalReader`, ao contrário dos dois irmãos acima.
/// É a única fonte desta janela cuja entrada é o estado do MUNDO e não o barramento.
///
/// ⚠️⚠️ **Ela corre AQUI, com os irmãos, e não depois da tabela de acções — e a escolha foi
/// MEDIDA, não herdada.** Ela vê o valor que o quadro ANTERIOR deixou, porque quem move um contador
/// é a tabela, que corre depois desta janela: isso custa `1` quadro na OBSERVAÇÃO e poupa `1` na
/// REACÇÃO. Falar depois da tabela inverteria os dois e daria **o mesmo total** ⇒ escolhe-se a
/// margem que o texto emendado do quadro já cobre, que é esta.
fn vigias(sim: &mut SimWorld, signals: &mut SignalOutbox, relogio: &Relogio) {
    let f = ph2d_app_components::counter_watch_bridge::frame(sim, relogio.playing, relogio.ticks);
    for (bits, row, nome) in f.disparos {
        signals.publish(ph2d_runtime::Signal::from_counter_watch(&nome, bits, row));
    }
}

/// ⭐⭐⭐ **O GATILHO — a mão de quem joga** (suplente #24). A tecla que o artista ligou no Input Map
/// chega à tabela de acções NESTE quadro.
///
/// ⚠️ **Ele não OUVE, só FALA** — como as vigias. E é o ÚNICO motor desta janela cuja entrada não é
/// o mundo nem o barramento: é o **teclado**, já resolvido em acções nomeadas.
///
/// ⚠️⚠️ **A cerca do relógio é a MESMA da fábrica, e sem ela o editor fica inutilizável:** as
/// teclas do jogo são as teclas do editor, e um gatilho ligado ao espaço publicaria o sinal dele a
/// cada espaço que o artista carrega a editar. *Play → a arma dispara · Stop → o teclado volta a
/// ser do editor.*
fn gatilhos(
    sim: &mut SimWorld,
    signals: &mut SignalOutbox,
    relogio: &Relogio,
    amostras: &BTreeMap<String, ph2d_ecs::ActionSample>,
) {
    if !relogio.playing {
        return;
    }
    let disparos = ph2d_ecs::dispara_gatilhos(sim.world_mut(), &|nome| {
        amostras.get(nome).copied().unwrap_or_default()
    });
    for d in disparos {
        signals.publish(ph2d_runtime::Signal::from_action(
            &d.signal,
            d.source.to_bits(),
            d.row,
        ));
    }
}

/// **As amostras de TODA acção do mapa, pelo nome** — a entrada do [`gatilhos`].
///
/// ⚠️⚠️ **Varre o MAPA e não os gatilhos, e é isso que dá a lei da acção inexistente de graça:** um
/// nome que o mapa não conhece simplesmente não está aqui, e o `unwrap_or_default` do motor
/// devolve silêncio. A alternativa — perguntar nome a nome ao mundo — poria a mesma decisão em
/// dois sítios, e o defeito mudo que ela abre é um `Release` a disparar em TODO quadro sobre uma
/// acção que ninguém ligou (porque `!pressed` é trivialmente verdade).
///
/// ⚠️ **`BTreeMap` e não `HashMap`** — a espinha do determinismo desta casa (lint estrutural).
pub(super) fn amostras_das_accoes(
    map: &ph2d_input::InputMap,
    estado: &ph2d_input::ActionState,
) -> BTreeMap<String, ph2d_ecs::ActionSample> {
    let input = ph2d_input::Input::new(map, estado);
    map.actions()
        .iter()
        .map(|a| {
            (
                a.name.clone(),
                ph2d_ecs::ActionSample {
                    pressed: input.pressed(&a.name),
                    just_pressed: input.just_pressed(&a.name),
                    just_released: input.just_released(&a.name),
                },
            )
        })
        .collect()
}
