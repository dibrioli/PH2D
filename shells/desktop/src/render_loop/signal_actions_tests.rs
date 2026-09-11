//! **Os gates da ponte do `SignalActions`** — irmão de [`crate::render_loop::signal_actions`] por CAP de LOC.
//!
//! ⚠️ **A RESOLUÇÃO tem os gates dela no `ph2d-ecs`** (casar nomes, achar o alvo, a ordem). Aqui
//! prova-se o que só existe com o mundo E o ledger: que a escrita chega, e que ela **não** chega
//! ao undo.

use super::*;
use ph2d_ecs::{Timer, TimerState, Transform};

fn um(name: &str, autostart: bool) -> Timer {
    Timer {
        name: name.into(),
        duration_us: 1_000_000,
        repeat: true,
        autostart,
        signal: "tic".into(),
    }
}

fn efeito(target: ph2d_ecs::Entity, verb: SignalVerb, arg: &str) -> SignalEffect {
    SignalEffect {
        target,
        verb,
        arg: arg.into(),
        source: target,
    }
}

/// Um mundo com um objecto que tem timers, já reconciliados (como o quadro os entrega).
fn com_timers(timers: Vec<Timer>) -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((Transform::default(), Timers(timers)))
        .id();
    super::super::start_autostart_timers(&mut sim);
    (sim, e)
}

fn rodando(sim: &SimWorld, e: ph2d_ecs::Entity) -> Vec<TimerState> {
    sim.world().get::<TimerRuntime>(e).cloned().expect("rt").0
}

/// ⭐⭐⭐ **UM SINAL ARRANCA UM TIMER QUE O `autostart` NUNCA ARRANCARIA.**
///
/// É o buraco que a W3 do `Timer` deixou nomeado, e que este componente fecha: até aqui o
/// `autostart` era o **único** caminho para um relógio começar, e um timer com ele desligado era
/// inalcançável por gesto nenhum.
///
/// **Mutação que deve sangrar:** trocar o braço `StartTimer` por um no-op.
#[test]
fn a_signal_starts_a_timer_that_autostart_never_would() {
    let (mut sim, e) = com_timers(vec![um("recarga", false)]);
    let mut drive = PreviewDrive::default();
    assert!(
        !rodando(&sim, e)[0].running,
        "a fixtura ja' arrancou o timer — ela nao mede o sinal"
    );
    let r = apply(
        &mut sim,
        &[efeito(e, SignalVerb::StartTimer, "")],
        &mut drive,
        None,
    );
    assert_eq!(r.applied, 1);
    assert!(
        rodando(&sim, e)[0].running,
        "o sinal nao arrancou o timer — o autostart continua a ser o unico caminho"
    );
}

/// **PARAR guarda o progresso** — o *Pause*, não o *Stop*. E arrancar volta ao princípio.
#[test]
fn stopping_by_signal_keeps_the_progress_and_starting_rewinds() {
    let (mut sim, e) = com_timers(vec![um("batida", true)]);
    let mut drive = PreviewDrive::default();
    if let Some(mut rt) = sim.world_mut().get_mut::<TimerRuntime>(e) {
        rt.0[0].elapsed_us = 700_000;
    }
    apply(
        &mut sim,
        &[efeito(e, SignalVerb::StopTimer, "")],
        &mut drive,
        None,
    );
    let s = rodando(&sim, e)[0];
    assert!(!s.running, "parar nao parou");
    assert_eq!(s.elapsed_us, 700_000, "parar ZEROU o progresso");

    apply(
        &mut sim,
        &[efeito(e, SignalVerb::StartTimer, "")],
        &mut drive,
        None,
    );
    let s = rodando(&sim, e)[0];
    assert!(s.running);
    assert_eq!(s.elapsed_us, 0, "arrancar nao voltou ao principio");
}

/// ⭐ **O `arg` escolhe UM timer; vazio escolhe TODOS.**
///
/// ⚠️ Sem o vazio-é-todos, o caso comum — *«um objecto, um relógio»* — obrigaria o artista a
/// repetir o nome do timer em cada linha da tabela, e a errá-lo é silêncio.
#[test]
fn the_argument_picks_one_timer_and_empty_picks_them_all() {
    let (mut sim, e) = com_timers(vec![um("a", false), um("b", false)]);
    let mut drive = PreviewDrive::default();

    apply(
        &mut sim,
        &[efeito(e, SignalVerb::StartTimer, "b")],
        &mut drive,
        None,
    );
    let r = rodando(&sim, e);
    assert!(!r[0].running, "o nome escolheu o timer errado");
    assert!(r[1].running, "o nome nao escolheu nenhum");

    apply(
        &mut sim,
        &[efeito(e, SignalVerb::StartTimer, "")],
        &mut drive,
        None,
    );
    assert!(
        rodando(&sim, e).iter().all(|s| s.running),
        "o vazio nao escolheu TODOS"
    );
}

/// ⭐⭐⭐ **ESCONDER POR SINAL NÃO ENTRA NO UNDO** — a razão inteira de esta metade passar pelo
/// ledger.
///
/// A `Visibility` é um componente **registado**: sem esta declaração, cada porta que abre seria um
/// passo de `Ctrl+Z` que o artista não deu. A régua é a porta do produto — a substituição que a
/// captura faz — e não uma leitura do componente, que mediria a minha intenção.
///
/// **Mutação que deve sangrar:** apagar a chamada a `drive.driven(…)` do `set_visible`.
#[test]
fn hiding_by_signal_is_preview_and_the_capture_sees_the_authored_value() {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((Transform::default(), Visibility::visible()))
        .id();
    let mut drive = PreviewDrive::default();

    apply(
        &mut sim,
        &[efeito(e, SignalVerb::Hide, "")],
        &mut drive,
        None,
    );
    assert!(
        sim.world().get::<Visibility>(e).expect("vis").hidden,
        "o sinal nao escondeu nada — a cena tem de MOSTRAR o efeito"
    );

    // A porta que a captura usa: ela repõe o AUTORADO, fotografa, e devolve o vivo.
    let vivo = drive.substitute_authored(&mut sim);
    assert!(
        !sim.world().get::<Visibility>(e).expect("vis").hidden,
        "a captura fotografou o valor de PRE-VISUALIZACAO — cada porta que abre seria um passo \
         de undo"
    );
    PreviewDrive::restore_live(&mut sim, &vivo);
    assert!(
        sim.world().get::<Visibility>(e).expect("vis").hidden,
        "o vivo nao voltou depois da fotografia — a cena piscaria"
    );
}

/// **INVERTER lê o que está no ECRÃ**, nunca uma memória própria.
#[test]
fn toggling_inverts_what_the_scene_shows() {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((Transform::default(), Visibility::visible()))
        .id();
    let mut drive = PreviewDrive::default();
    for esperado in [true, false, true] {
        apply(
            &mut sim,
            &[efeito(e, SignalVerb::ToggleVisibility, "")],
            &mut drive,
            None,
        );
        assert_eq!(
            sim.world().get::<Visibility>(e).expect("vis").hidden,
            esperado,
            "inverter nao seguiu o que estava no ecra"
        );
    }
}

/// ⛔ **Um efeito sobre quem não tem o componente é INERTE e CONTADO** — nunca um estouro, e nunca
/// um silêncio.
///
/// ⚠️ Ligar um sinal a um objecto sem relógio é uma configuração **a meio**, não uma avaria: o
/// número existe para o painel o poder dizer, e não para virar um toast por quadro.
#[test]
fn an_effect_on_an_object_without_the_component_is_inert_and_counted() {
    let mut sim = SimWorld::default();
    let vazio = sim.world_mut().spawn((Transform::default(),)).id();
    let mut drive = PreviewDrive::default();
    let r = apply(
        &mut sim,
        &[
            efeito(vazio, SignalVerb::StartTimer, ""),
            efeito(vazio, SignalVerb::Hide, ""),
        ],
        &mut drive,
        None,
    );
    assert_eq!(r.applied, 0);
    assert_eq!(r.inert, 2, "os inertes nao foram contados");
}

/// **Um nome de timer que não casa com nenhum é inerte** — e não arranca os outros por engano.
#[test]
fn a_timer_name_that_matches_nothing_starts_nobody() {
    let (mut sim, e) = com_timers(vec![um("a", false)]);
    let mut drive = PreviewDrive::default();
    let r = apply(
        &mut sim,
        &[efeito(e, SignalVerb::StartTimer, "nao_existe")],
        &mut drive,
        None,
    );
    assert_eq!(r.inert, 1);
    assert!(
        !rodando(&sim, e)[0].running,
        "um nome que nao casa arrancou o timer errado"
    );
}
