//! **OS MOTORES QUE CORREM NA JANELA DOS CÉREBROS** — os scripts do artista (TOP-20 #16), os
//! emissores de partículas (TOP-20 #18), as vigias de contador, o gatilho, as armas e as barras de
//! vida, chamados pela `fase_signal_outbox` da shell.
//!
//! ⚠️ **São funções LIVRES e não fases-filhas**, e a razão é um empréstimo: o outbox segura o `sim`
//! (e o resto do `gfx`) do princípio ao fim do corpo. Cada motor recebe as peças de que precisa — e
//! a CHAMADA fica no corpo do outbox, que é o que o texto emendado do quadro colhe: a ORDEM continua
//! medível (`shells/desktop/tests/it/o_cerebro_fala_antes_de_a_tabela_ouvir.rs`).
//!
//! ⚠️ **Moraram na shell** (`render_loop/motores_do_quadro.rs`) até a `line/CodeEditor` pagar as
//! costuras do editor de scripts movendo-os para a família onde as pontes deles já viviam.

use std::collections::BTreeMap;

use crate::health_bar_bridge::HealthBarsState;
use crate::particles_bridge::ParticlesState;
use ph2d_ecs::SimWorld;
use ph2d_ecs::sort_key::SortScratch;
use ph2d_runtime::{SignalOutbox, SignalReader};
use ph2d_script::ScriptHost;

/// **O relógio do quadro**, para os dois motores: se a corrida anda, quantos passos fixos e quanto
/// dura cada um.
pub struct Relogio {
    /// A corrida está a andar (`playhead.is_playing()`)?
    pub playing: bool,
    /// Quantos passos fixos este quadro deve.
    pub ticks: u32,
    /// Quanto dura um passo fixo.
    pub dt: f64,
}

impl Relogio {
    /// **O relógio deste quadro.**
    ///
    /// ⚠️ **Construir-se é assunto de quem é dono do tipo** — e o corte foi imposto pelo tecto de
    /// 200 LOC da `fase_signal_outbox` (ela chegou a `202` ao ganhar o terceiro motor). *A cura de
    /// um tecto é o CORTE, nunca uma entrada nova no `FN_OVERAGE_OK`.*
    #[must_use]
    pub const fn do_quadro(playing: bool, ticks: u32, dt: f64) -> Self {
        Self { playing, ticks, dt }
    }
}

/// Os cursores dos motores que OUVEM — um por motor: partilhar um apagaria os sinais do outro.
pub struct Leitores<'a> {
    pub script: &'a mut SignalReader,
    pub particles: &'a mut SignalReader,
    pub weapon: &'a mut SignalReader,
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
pub fn correm(
    sim: &mut SimWorld,
    script: &mut Option<ScriptHost>,
    particles: &mut ParticlesState,
    health_bars: &mut HealthBarsState,
    scratch: &SortScratch,
    drive: &mut ph2d_preview_drive::PreviewDrive,
    signals: &mut SignalOutbox,
    leitores: Leitores<'_>,
    relogio: &Relogio,
    accoes: &BTreeMap<String, ph2d_ecs::ActionSample>,
    portas: &mut crate::script_bridge::ScriptDoors<'_, '_>,
) -> Vec<ph2d_ecs::Death> {
    let mortes = scripts(
        sim,
        script,
        drive,
        signals,
        leitores.script,
        relogio,
        portas,
    );
    particulas(
        sim,
        particles,
        scratch,
        signals,
        leitores.particles,
        relogio,
    );
    vigias(sim, signals, relogio);
    gatilhos(sim, signals, relogio, accoes);
    // ⭐⭐⭐ **As ARMAS correm DEPOIS do gatilho, e a ordem É a lei:** elas ouvem o que ele acabou
    // de publicar, logo carregar na tecla e a bala nascer acontecem no MESMO quadro. Correndo
    // antes, cada tiro chegava um quadro atrasado — a lei que o cabeçalho do outbox já escreve.
    armas(sim, signals, leitores.weapon, relogio);
    barras(sim, health_bars, relogio);
    // Ciclo 19 do Motion: as entradas do grafo (o `Motion Input`, a vida, o dano e os sinais) —
    // DEPOIS de todos os que falam, e antes do Motion, que coze neste mesmo quadro.
    crate::motion_input_bridge::frame(sim, signals);
    mortes
}

/// ⭐ **AS BARRAS DE VIDA** (plano 28, W4) — elas só LÊEM (a vida, a pose) e não falam no
/// barramento, logo a posição delas nesta lista não é lei de ordem.
///
/// ⚠️ **Elas correm DEPOIS do passo da física**, que é quem escreve o `HealthNow`: a barra mostra a
/// vida deste quadro e não a do anterior.
///
/// ⚠️ **O tempo do rasto é o de JOGO** (passos devidos × passo fixo): com o relógio parado o rasto
/// espera, que é o que faz um scrub parado não o escorrer.
fn barras(sim: &mut SimWorld, health_bars: &mut HealthBarsState, relogio: &Relogio) {
    #[allow(clippy::cast_possible_truncation)]
    let dt = if relogio.playing {
        (f64::from(relogio.ticks) * relogio.dt) as f32
    } else {
        0.0
    };
    health_bars.frame(sim, dt);
}

/// ⭐⭐⭐ **OS SCRIPTS DO ARTISTA** (TOP-20 #16) — o que um script emite chega à tabela de acções
/// NESTE quadro.
///
/// ⚠️ O cursor lê em TODO quadro, mesmo parado (a lei do `ui_signal_reader`), e uma falha imprime
/// UMA linha — a mensagem fica no Inspector. Devolve as mortes que os scripts pediram
/// (`ph2d.despawn`), para o dreno único da fábrica.
fn scripts(
    sim: &mut SimWorld,
    script: &mut Option<ScriptHost>,
    drive: &mut ph2d_preview_drive::PreviewDrive,
    signals: &mut SignalOutbox,
    reader: &mut SignalReader,
    relogio: &Relogio,
    portas: &mut crate::script_bridge::ScriptDoors<'_, '_>,
) -> Vec<ph2d_ecs::Death> {
    let ouvidos: Vec<String> = signals.read(reader).map(|s| s.name.to_string()).collect();
    let Some(host) = script.as_mut() else {
        return Vec::new();
    };
    let nomes: Vec<&str> = ouvidos.iter().map(String::as_str).collect();
    let f = crate::script_bridge::frame(
        host,
        sim,
        drive,
        relogio.playing,
        relogio.ticks,
        relogio.dt,
        &nomes,
        portas,
    );
    for (bits, nome) in f.emitted {
        signals.publish(ph2d_runtime::Signal::from_script(&nome, bits));
    }
    for (bits, msg) in f.failed {
        eprintln!("[script] o objecto {bits} parou: {msg}");
    }
    f.despawned
        .into_iter()
        .map(|entity| ph2d_ecs::Death {
            entity,
            signal: String::new(),
            why: ph2d_ecs::DeathCause::Killed,
        })
        .collect()
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
    let f = crate::counter_watch_bridge::frame(sim, relogio.playing, relogio.ticks);
    for (bits, row, nome) in f.disparos {
        signals.publish(ph2d_runtime::Signal::from_counter_watch(&nome, bits, row));
    }
}

/// ⭐⭐⭐ **AS ARMAS** — o que o gatilho pediu vira um tiro NESTE quadro.
///
/// ⚠️ **Ela OUVE e FALA**, como os scripts — e é o único motor desta janela que faz as duas coisas
/// sobre o mesmo barramento. É por isso que ela tem cursor PRÓPRIO: a tabela de acções pode estar a
/// ouvir o mesmo `fire`, e com um cursor partilhado quem lesse primeiro apagava o outro.
///
/// ⚠️⚠️ **Ela corre DEPOIS do [`gatilhos`], e a ordem É a lei:** correndo antes, a arma leria o
/// `fire` do quadro ANTERIOR e cada tiro chegava um quadro atrasado. *A janela de graça do outbox
/// esconderia o atraso de um toast e não o de uma bala.*
///
/// ⚠️ **O `dt` é o do QUADRO INTEIRO** (`ticks × dt`), como o das vigias: a cadência é um intervalo
/// de simulação, e um quadro que deve três passos fixos gastou três.
fn armas(
    sim: &mut SimWorld,
    signals: &mut SignalOutbox,
    reader: &mut SignalReader,
    relogio: &Relogio,
) {
    // ⚠️ **O cursor lê em TODO quadro, mesmo parado** — a lei do `ui_signal_reader`: um leitor que
    // salta quadros acumula `missed` e passa a ver o passado. A cerca da corrida vive na PONTE.
    let ouvidos: Vec<String> = signals.read(reader).map(|s| s.name.to_string()).collect();
    let refs: Vec<&str> = ouvidos.iter().map(String::as_str).collect();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let dt_us = (f64::from(relogio.ticks) * relogio.dt * 1e6) as u64;
    let t = crate::weapon_bridge::frame(sim, relogio.playing, dt_us, &refs);
    for (e, nome) in t.disparos.into_iter().chain(t.secas).chain(t.recarregadas) {
        signals.publish(ph2d_runtime::Signal::from_weapon(&nome, e.to_bits()));
    }
}

/// ⭐⭐⭐ **O GATILHO — a mão de quem joga** (suplente #24). A tecla que o artista ligou no Input Map
/// chega à tabela de acções NESTE quadro.
///
/// ⚠️ **Ele não OUVE, só FALA** — como as vigias. E é o ÚNICO motor desta janela cuja entrada não é
/// o mundo nem o barramento: é o **teclado**, já resolvido em acções nomeadas.
///
/// ⚠️ **A LEI vive na crate da família** ([`crate::trigger_bridge::frame`]) — a cerca
/// do relógio incluída, que é o molde da irmã das vigias. Aqui fica a COMPOSIÇÃO: *onde* no quadro
/// este motor corre, e a publicação.
fn gatilhos(
    sim: &mut SimWorld,
    signals: &mut SignalOutbox,
    relogio: &Relogio,
    amostras: &BTreeMap<String, ph2d_ecs::ActionSample>,
) {
    for d in crate::trigger_bridge::frame(sim, relogio.playing, amostras) {
        signals.publish(ph2d_runtime::Signal::from_action(
            &d.signal,
            d.source.to_bits(),
            d.row,
        ));
    }
}
