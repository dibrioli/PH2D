//! A sonda da cena 104 + os gates que mantêm a mensagem dela honesta.
//!
//! ⚠️ **Uma cena cuja mensagem cita números tem de os medir**, senão a primeira
//! wave que mexer num default a transforma num folheto.
//!
//! ⚠️ **E os gates aqui julgam a CENA, não a lei.** A lei tem os dela
//! (`ph2d-platformer::kinematic_tests`), a consulta tem os dela
//! (`ph2d-physics/tests/buoyed_query.rs`) e o produto tem os dele
//! (`ph2d-physics-ecs/tests/player_in_water.rs`). O que só esta cena pode
//! afirmar é que **os três corpos que ela monta são de facto comparáveis** — se
//! eles diferirem em forma, densidade ou altura de largada, o artista olha para
//! uma diferença que não é a que a wave produziu.

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;

/// Monta a cena e simula `secs` segundos pela PORTA REAL (a ponte).
fn run(secs: f32) -> SimWorld {
    let mut sim = SimWorld::new();
    build_kin_water_scene(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let ticks = (secs * 60.0) as u64;
    for t in 0..=ticks {
        bridge.dispatch(&mut sim, true, t);
    }
    sim
}

fn y_of(sim: &SimWorld, who: &str) -> f32 {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == who {
            found = Some(t.translation.y);
        }
    }
    found.expect("o corpo tem de existir")
}

/// A oscilação de `who` entre o 3.º e o 6.º segundo — a régua que o `MEASURED`
/// da cena publica.
///
/// ⚠️ **A primeira metade é DESCARTADA de propósito:** ela contém a entrada na
/// água, que é um transiente e não uma oscilação. Medi-las juntas misturaria as
/// duas e daria um número que não descreve nem uma nem outra.
fn bob_of(who: &str) -> f32 {
    let mut sim = SimWorld::new();
    build_kin_water_scene(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for t in 0..=360u64 {
        bridge.dispatch(&mut sim, true, t);
        if t > 180 {
            let y = y_of(&sim, who);
            lo = lo.min(y);
            hi = hi.max(y);
        }
    }
    hi - lo
}

const SUBJECTS: [&str; 3] = ["Control Capsule", "Dynamic Player", "Kinematic Player"];

/// **A sonda.** `cargo test -p ph2d-host-desktop --release probe_smoke_104 --
/// --ignored --nocapture`
#[test]
#[ignore = "sonda de medição"]
fn probe_smoke_104() {
    println!("\n=== cena 104: percurso desde a largada (y = {DROP_Y:.2}) ===");
    println!("{:<18} {:>10} {:>12}", "sujeito", "y", "percorreu");
    let sim = run(4.0);
    for who in SUBJECTS {
        let y = y_of(&sim, who);
        println!("{who:<18} {y:>10.4} {:>12.4}", y - DROP_Y);
    }

    println!("\n=== a trajetoria, para a oscilacao ser visivel no numero ===");
    print!("{:<8}", "t (s)");
    for who in SUBJECTS {
        print!("{who:>18}");
    }
    println!();
    for secs in [1.0_f32, 2.0, 3.0, 4.0, 5.0, 6.0, 10.0] {
        let s = run(secs);
        print!("{secs:<8.1}");
        for who in SUBJECTS {
            print!("{:>18.4}", y_of(&s, who));
        }
        println!();
    }

    println!("\n=== amplitude entre o 3o e o 6o segundo ===");
    for who in SUBJECTS {
        println!("{who:<18} {:>10.4}", bob_of(who));
    }
    println!(
        "\nLEITURA: o AMBAR e o AZUL tem de concordar. Se o AZUL afundar sozinho,\n\
         a agua voltou a nao existir para o modo cinematico.\n"
    );
}

/// **OS TRÊS CORPOS SÃO COMPARÁVEIS** — o gate que só esta cena pode afirmar.
///
/// ⚠️ **Sem ele a cena mente sem quebrar nada:** uma densidade diferente, uma
/// cápsula mais gorda ou uma altura de largada distinta produziriam três linhas
/// d'água diferentes, o artista veria a diferença e concluiria que a wave falhou
/// — ou, pior, que ela funcionou quando não funcionou. O oráculo da cena é a
/// COMPARAÇÃO, e uma comparação exige que só uma coisa difira.
#[test]
fn the_three_subjects_differ_only_in_what_the_wave_changes() {
    let mut sim = SimWorld::new();
    build_kin_water_scene(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let world = sim.world();

    let mut shapes = Vec::new();
    let mut kinds = Vec::new();
    let mut drops = Vec::new();
    let mut players = 0;
    let mut q = world
        .try_query::<(
            &Name,
            &Transform,
            &Collider,
            &RigidBody,
            Option<&PlatformPlayer>,
            Option<&PlayerMode>,
        )>()
        .unwrap();
    for (n, t, c, rb, p, m) in q.iter(world) {
        if !SUBJECTS.contains(&n.as_str()) {
            continue;
        }
        shapes.push((c.shape, c.density));
        kinds.push((n.as_str().to_string(), rb.kind, m.copied()));
        drops.push(t.translation.y);
        if p.is_some() {
            players += 1;
        }
    }

    assert_eq!(shapes.len(), 3, "a cena tem de montar os TRÊS sujeitos");
    assert!(
        shapes.windows(2).all(|w| w[0] == w[1]),
        "forma e densidade têm de ser idênticas nos três: {shapes:?}"
    );
    assert!(
        drops.windows(2).all(|w| (w[0] - w[1]).abs() < 1.0e-6),
        "os três são largados da MESMA altura: {drops:?}"
    );
    assert_eq!(
        players, 2,
        "dois têm lei de player; o CONTROLE não tem nenhuma"
    );

    // E o que DIFERE é exactamente uma coisa: o modo do player azul.
    let kin = kinds
        .iter()
        .find(|(n, ..)| n == "Kinematic Player")
        .expect("o sujeito cinemático");
    assert_eq!(kin.1, BodyKind::Kinematic);
    assert_eq!(
        kin.2,
        Some(PlayerMode::Kinematic),
        "os DOIS campos, como o chip da §14 os escreve"
    );
    for (n, kind, mode) in &kinds {
        if n == "Kinematic Player" {
            continue;
        }
        assert_eq!(*kind, BodyKind::Dynamic, "{n} tem de ser dinâmico");
        assert!(mode.is_none(), "{n} não carrega PlayerMode");
    }
}

/// **A CENA ENTREGA O QUE A MENSAGEM PROMETE: os três boiam, e os dois players
/// concordam.**
///
/// ⚠️ **Nasceu VERMELHO** — antes da `W-KinFluid` o azul afundava `139,67 m`
/// nesta cena enquanto os outros dois subiam ~1,09.
///
/// ⚠️ **A tolerância não é folga, é a precisão do número publicado:** a mensagem
/// cita três casas, e um gate mais apertado que isso reprovaria a cena por
/// ruído de agendamento em vez de por regressão.
#[test]
fn the_scene_delivers_the_numbers_its_message_prints() {
    let sim = run(4.0);
    for (i, who) in SUBJECTS.iter().enumerate() {
        let y = y_of(&sim, who);
        let sank = DROP_Y - y;
        assert!(
            (sank - SANK[i]).abs() < 0.01,
            "{who} afundou {sank:.4} e a mensagem promete {:.4}",
            SANK[i]
        );
        // ⚠️ **A metade que separa *parar na água* de *afundar devagar*:** o
        // número acima diz QUANTO ele desceu, e só este diz que ele PAROU —
        // acima da superfície, que é onde a poça começa (`y = 0`).
        assert!(
            y > 0.0,
            "{who} tem de parar ACIMA da superfície e parou em {y:.4}"
        );
    }

    let dynamic = bob_of("Dynamic Player");
    let kinematic = bob_of("Kinematic Player");
    assert!(
        (dynamic - BOB[0]).abs() < 0.05 && (kinematic - BOB[1]).abs() < 0.05,
        "a amplitude publicada tem de ser a medida: {dynamic:.4}/{kinematic:.4} \
         contra {:.4}/{:.4}",
        BOB[0],
        BOB[1]
    );
    assert!(
        (dynamic - kinematic).abs() < 0.1,
        "e os dois modos têm de concordar: {dynamic:.4} contra {kinematic:.4}"
    );
}

/// **A POÇA É FUNDA** — nada ao alcance da perna, senão a cena mede outra coisa.
///
/// ⚠️ Com fundo alcançável o sensor de chão responde, o personagem fica **de
/// pé** e a água deixa de ser a única coisa a agi-lo. O gate afirma a premissa
/// que a cena declara no doc, porque uma premissa só escrita em prosa é uma que
/// a próxima edição apaga.
#[test]
fn the_pool_is_deep_enough_that_nothing_stands_in_it() {
    let mut sim = SimWorld::new();
    build_kin_water_scene(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    // O fundo da poça, e a folga até o ponto mais baixo que qualquer sujeito
    // alcança em dez segundos.
    let floor = -2.0 * POOL_HALF[1];
    let mut lowest = f32::MAX;
    let mut bridge = PhysicsBridge::new();
    for t in 0..=600u64 {
        bridge.dispatch(&mut sim, true, t);
        for who in SUBJECTS {
            lowest = lowest.min(y_of(&sim, who));
        }
    }
    assert!(
        lowest - floor > FLOAT + 1.0,
        "ninguém pode chegar perto do fundo: o mais baixo foi {lowest:.4} e o \
         fundo está em {floor:.4} (a perna alcança {FLOAT:.2})"
    );
}
