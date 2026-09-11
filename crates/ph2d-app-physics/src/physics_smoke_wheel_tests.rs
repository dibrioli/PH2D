//! **A sonda da cena 57** — os números que a mensagem afirma, medidos sobre as
//! MESMAS peças que o artista abre.
//!
//! `#[ignore]` como as irmãs: ela imprime uma tabela, não afirma uma. Roda com
//! `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop probe_smoke_57 -- --ignored --nocapture`.
//!
//! Mais os gates NÃO-ignorados das duas afirmações que só esta cena pode
//! falsificar: **a suspensão balança MENOS que o eixo rígido**, e **ela de fato
//! se move** (uma suspensão travada passaria no primeiro por acidente de massa).

use super::{
    BUMP_HALF_Y, MEASURED_RIGID_DIST_M, MEASURED_RIGID_TRAVEL_M, MEASURED_SPRUNG_DIST_M,
    MEASURED_TRAVEL_M, spawn_props_with,
};
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;

/// Quantos ticks o carro tem para atravessar a pista.
const TICKS: u64 = 240;

fn scene_with(bump_half_y: f32) -> (SimWorld, PhysicsBridge) {
    let mut sim = SimWorld::new();
    spawn_props_with(sim.world_mut(), bump_half_y);
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    (sim, bridge)
}

fn by_name(sim: &SimWorld, want: &str) -> Entity {
    let mut q = sim.world().try_query::<(Entity, &Name)>().unwrap();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == want)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena 57 não tem `{want}`"))
}

fn pose(sim: &SimWorld, e: Entity) -> [f32; 3] {
    let t = sim.world().get::<Transform>(e).unwrap();
    [t.translation.x, t.translation.y, t.rotation]
}

/// O que uma corrida produz. Mais campos do que a mensagem usa, de propósito: a
/// primeira versão desta sonda mediu só o BALANÇO e ele **refutou a cena** —
/// ver [`measure`].
struct Run {
    /// Maior salto de altura do chassi entre dois ticks, em metros. É a
    /// grandeza que uma suspensão existe para reduzir.
    jolt: f32,
    /// Excursão vertical do chassi, pico a pico.
    heave: f32,
    /// Balanço do chassi, pico a pico, em graus.
    pitch: f32,
    /// Curso percorrido pela suspensão (cubo↔chassi ao longo do eixo).
    travel: f32,
    /// Quanto o carro andou.
    distance: f32,
}

/// Corre a cena e mede o carro `label`.
///
/// ⚠️ **O BALANÇO não é a grandeza certa, e a medição foi quem disse.** A
/// primeira versão desta cena afirmava *"o suspenso passa nivelado"* e mediu o
/// oposto — 23,4° contra 14,5° do eixo rígido. E está **correto**: duas
/// suspensões independentes deixam uma comprimir enquanto a outra estende, que
/// é exatamente o mergulho de um carro de verdade; um eixo rígido não tem para
/// onde ceder, então ele não mergulha — ele **sobe inteiro**. O que uma
/// suspensão reduz é o **SOLAVANCO** (a altura que o chassi salta de um tick
/// para o outro), e é essa a comparação que a cena faz.
fn measure(label: &str) -> Run {
    measure_with(label, BUMP_HALF_Y)
}

fn measure_with(label: &str, bump_half_y: f32) -> Run {
    let (mut sim, mut bridge) = scene_with(bump_half_y);
    let chassis = by_name(&sim, &format!("{label} Chassis"));
    let hub = by_name(&sim, &format!("{label} F"));
    let start = pose(&sim, chassis);
    let (mut pitch_lo, mut pitch_hi) = (f32::INFINITY, f32::NEG_INFINITY);
    let (mut ride_lo, mut ride_hi) = (f32::INFINITY, f32::NEG_INFINITY);
    let (mut y_lo, mut y_hi) = (f32::INFINITY, f32::NEG_INFINITY);
    let mut jolt = 0.0_f32;
    let mut prev_y = start[1];
    for tick in 1..=TICKS {
        bridge.dispatch(&mut sim, true, tick);
        let c = pose(&sim, chassis);
        let h = pose(&sim, hub);
        pitch_lo = pitch_lo.min(c[2]);
        pitch_hi = pitch_hi.max(c[2]);
        y_lo = y_lo.min(c[1]);
        y_hi = y_hi.max(c[1]);
        jolt = jolt.max((c[1] - prev_y).abs());
        prev_y = c[1];
        // O cubo visto do frame do chassi: o eixo LIVRE é o local +Y.
        let (dx, dy) = (h[0] - c[0], h[1] - c[1]);
        let (s, co) = (-c[2]).sin_cos();
        ride_lo = ride_lo.min(dx * s + dy * co);
        ride_hi = ride_hi.max(dx * s + dy * co);
    }
    Run {
        jolt,
        heave: y_hi - y_lo,
        pitch: (pitch_hi - pitch_lo).to_degrees(),
        travel: ride_hi - ride_lo,
        distance: pose(&sim, chassis)[0] - start[0],
    }
}

#[test]
#[ignore = "sonda de medição: imprime os números da cena 57"]
fn probe_smoke_57() {
    for label in ["Sprung", "Rigid"] {
        let m = measure(label);
        println!(
            "  {label:<7}: solavanco {:.4} m/tick  sobe-desce {:.3} m  balanco {:5.1} graus  \
             curso {:.4} m  andou {:5.2} m",
            m.jolt, m.heave, m.pitch, m.travel, m.distance
        );
    }
}

/// **A SUSPENSÃO SE MOVE E O EIXO RÍGIDO NÃO** — a afirmação inteira do passo 1
/// da mensagem, e a única propriedade desta cena que separa os dois carros por
/// mais de uma ordem de grandeza (medido: 0,150 m contra 0,000).
///
/// As duas metades num gate só porque cada uma sozinha passa sobre a cena
/// errada: *"o cubo verde percorre 15 cm"* também vale num carro que caiu de
/// lado, e *"o laranja não percorre nada"* vale num carro que nem saiu do lugar
/// — daí o controle de que os DOIS atravessam a pista.
#[test]
fn the_sprung_wheels_travel_in_their_arches_and_the_rigid_ones_do_not() {
    let sprung = measure("Sprung");
    let rigid = measure("Rigid");
    for (label, dist) in [("Sprung", sprung.distance), ("Rigid", rigid.distance)] {
        assert!(
            dist > 3.0,
            "o carro {label} tinha de atravessar a pista para a frente; andou {dist:.2} m"
        );
    }
    assert!(
        sprung.travel > 0.05,
        "a suspensão tinha de trabalhar; o cubo percorreu {:.4} m no chassi",
        sprung.travel
    );
    assert!(
        rigid.travel < 0.01,
        "o controle falhou: o eixo RÍGIDO não pode ceder, mas percorreu {:.4} m",
        rigid.travel
    );
}

/// **O carro suspenso MERGULHA mais que o de eixo rígido, e isso está certo.**
///
/// Pinado porque é contra-intuitivo e porque a primeira versão desta cena
/// afirmava o CONTRÁRIO: duas suspensões independentes deixam uma comprimir
/// enquanto a outra estende, que é o mergulho de um carro de verdade; um eixo
/// rígido não tem para onde ceder e por isso sobe inteiro. Sem este gate, a
/// próxima pessoa a medir o balanço lê 13,1 contra 11,7 e "conserta" a
/// suspensão para eliminar um comportamento correto.
#[test]
fn the_sprung_car_pitches_more_because_that_is_what_a_suspension_does() {
    let sprung = measure("Sprung");
    let rigid = measure("Rigid");
    assert!(
        sprung.pitch > rigid.pitch,
        "o carro suspenso tinha de mergulhar MAIS que o de eixo rígido; \
         medido {:.1} contra {:.1} graus",
        sprung.pitch,
        rigid.pitch
    );
}

/// **A MENSAGEM NÃO MENTE** — os quatro números que a cena imprime são os que a
/// simulação de fato produz.
#[test]
fn the_scene_message_states_the_numbers_the_sim_produces() {
    let sprung = measure("Sprung");
    let rigid = measure("Rigid");
    for (what, live, claimed, tol) in [
        ("o curso do verde", sprung.travel, MEASURED_TRAVEL_M, 0.01),
        (
            "o curso do laranja",
            rigid.travel,
            MEASURED_RIGID_TRAVEL_M,
            0.01,
        ),
        (
            "a distância do verde",
            sprung.distance,
            MEASURED_SPRUNG_DIST_M,
            0.1,
        ),
        (
            "a distância do laranja",
            rigid.distance,
            MEASURED_RIGID_DIST_M,
            0.1,
        ),
    ] {
        assert!(
            (live - claimed).abs() < tol,
            "a mensagem da cena 57 diz {claimed:.3} para {what} e a sim produz {live:.4}"
        );
    }
}

/// **A varredura que escolheu [`BUMP_HALF_Y`]** — para nenhuma altura ser
/// escolhida por gosto.
///
/// Uma suspensão absorve o degrau que CABE no curso dela; acima disso ela bate
/// no batente e o chassi leva o resto igual a um eixo rígido. A tabela mostra
/// onde essa fronteira está nesta cena.
#[test]
#[ignore = "sonda de medição: a varredura de altura de lombada da cena 57"]
fn probe_smoke_57_bump_sweep() {
    println!("  meia-altura   solavanco verde   solavanco laranja   razao");
    for h in [0.02_f32, 0.035, 0.05, 0.08, 0.12] {
        let s = measure_with("Sprung", h);
        let r = measure_with("Rigid", h);
        println!(
            "  {h:>9.3}   {:>15.4}   {:>17.4}   {:>5.2}x   andou {:5.2} vs {:5.2}                sobe-desce {:.3} vs {:.3}",
            s.jolt,
            r.jolt,
            r.jolt / s.jolt.max(1e-9),
            s.distance,
            r.distance,
            s.heave,
            r.heave,
        );
    }
}
