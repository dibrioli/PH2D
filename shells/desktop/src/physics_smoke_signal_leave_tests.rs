//! A sonda da cena 76 + os gates que mantêm a mensagem dela honesta
//! (W-SignalLeave).

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

/// Monta a cena e simula `secs`, colhendo TODO sinal pela porta REAL, na ordem
/// em que a física os emitiu.
fn fired(secs: f32) -> Vec<String> {
    let mut sim = SimWorld::new();
    build_signal_leave_scene(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let mut out = Vec::new();
    for t in 0..=(secs * 60.0) as u64 {
        bridge.dispatch(&mut sim, true, t);
        for s in bridge.signal_events(&sim) {
            out.push(s.name);
        }
    }
    out
}

/// **A sonda.** `cargo test -p ph2d-host-desktop --release probe_smoke_76 --
/// --ignored --nocapture`
///
/// Roda ANTES de a mensagem da cena ser escrita: nesta linha duas cenas já
/// afirmaram coisas que a medição desmentiu.
#[test]
#[ignore = "sonda de medição"]
fn probe_smoke_76() {
    let all = fired(6.0);
    println!("\n=== cena 76 (6 s) — a sequencia inteira ===");
    println!("  {all:?}");
    for name in [
        "door_open",
        "door_close",
        "bell_hit",
        "bell_part",
        "half_open",
        "half_close",
    ] {
        println!(
            "  {name:<11} disparou {} vez(es)",
            all.iter().filter(|n| *n == name).count()
        );
    }
}

/// **A afirmação da cena:** a porta ABRE e depois FECHA.
///
/// O oráculo é a ORDEM, não a contagem — `close` antes de `open` é uma porta que
/// fecha antes de abrir, e uma contagem de dois não distingue os dois casos.
#[test]
fn the_door_opens_and_then_closes() {
    let all = fired(6.0);
    let open = all.iter().position(|n| n == "door_open");
    let close = all.iter().position(|n| n == "door_close");
    let (Some(open), Some(close)) = (open, close) else {
        panic!("a porta nao abriu e fechou -- {all:?}");
    };
    assert!(
        open < close,
        "a porta FECHOU antes de abrir -- {all:?}: os dois extremos estao trocados"
    );
}

/// **O extremo SÓLIDO também tem dois nomes** — bater e desencostar.
#[test]
fn the_solid_bell_shouts_the_hit_and_then_the_parting() {
    let all = fired(6.0);
    let hit = all.iter().position(|n| n == "bell_hit");
    let part = all.iter().position(|n| n == "bell_part");
    let (Some(hit), Some(part)) = (hit, part) else {
        panic!("o sino nao bateu e desencostou -- {all:?}");
    };
    assert!(hit < part, "o sino desencostou antes de bater -- {all:?}");
}

/// **O CONTROLE: marcada só na chegada, ela abre e NUNCA fecha.**
///
/// ⚠️ Sem esta pista a cena não distingue *"a saída disparou"* de *"tudo dispara
/// duas vezes"* — e o regime que ela protege é toda cena já autorada, que tem só
/// o componente de chegada.
#[test]
fn the_half_marked_door_opens_and_never_closes() {
    let all = fired(6.0);
    assert!(
        all.iter().any(|n| n == "half_open"),
        "a porta de controle nao abriu -- {all:?}"
    );
    // Ela não tem `SignalOnLeave`, então NENHUM nome pode sair dela ao ser
    // deixada: nem um segundo `half_open`, nem coisa nenhuma.
    assert_eq!(
        all.iter().filter(|n| *n == "half_open").count(),
        1,
        "a porta marcada so' na chegada gritou mais de uma vez -- {all:?}: um \
         extremo sem o componente dele tem de ser SILENCIO, nao o outro nome"
    );
}

/// **A cena imprime o que montou** — e as três pistas existem de fato.
///
/// A lição que esta linha pagou na cena do colorize: uma cena que afirma três
/// pistas e monta duas é indistinguível da feature quebrada.
#[test]
fn the_scene_builds_the_three_lanes_it_names() {
    let mut sim = SimWorld::new();
    build_signal_leave_scene(sim.world_mut());
    for name in LANE_NAMES {
        let found = sim
            .world_mut()
            .query::<&ph2d_ecs::Name>()
            .iter(sim.world())
            .any(|n| n.as_str() == name);
        assert!(found, "a cena 76 nomeia a pista '{name}' e nao a monta");
    }
}
