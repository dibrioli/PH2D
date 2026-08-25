//! A sonda da cena 106 + os gates que mantêm a mensagem dela honesta.
//!
//! ⚠️ **Uma cena cuja mensagem cita números tem de os medir**, senão a primeira wave que
//! mexer num default a transforma num folheto.
//!
//! ⚠️ **E os gates aqui julgam a CENA, não a lei.** A lei tem os dela
//! (`ph2d-platformer::kinematic_tests`), a porta tem os dela
//! (`ph2d-physics/tests/zone_push.rs`) e o produto tem os dele
//! (`ph2d-physics-ecs/tests/player_zone_force.rs`). O que só esta cena pode afirmar é
//! que **os quatro corpos que ela monta são comparáveis** — se diferirem em forma,
//! densidade, freio ou ponto de partida, o artista olha para uma diferença que não é a
//! que a wave produziu.

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::{AreaEffector, PhysicsBridge, PhysicsSettings, PlayerInput};

/// Monta a cena e simula `secs` segundos pela PORTA REAL (a ponte).
///
/// ⚠️ **Sem gravidade, como a cena a arma:** o `physics_smoke_zone_force` escreve
/// `gravity_y = 0` no `AppGfx` antes de montar, e uma sonda que a esquecesse mediria
/// quatro corpos em queda livre.
fn run(secs: f32) -> SimWorld {
    let mut sim = SimWorld::new();
    build_zone_force_scene(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(PhysicsSettings {
        gravity_y: 0.0,
        ..Default::default()
    });
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let ticks = (secs * 60.0) as u64;
    for t in 0..=ticks {
        bridge.dispatch(&mut sim, true, t);
    }
    sim
}

fn x_of(sim: &SimWorld, who: &str) -> f32 {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == who {
            found = Some(t.translation.x);
        }
    }
    found.expect("o corpo tem de existir")
}

const SUBJECTS: [&str; 4] = [
    "Loose Crate",
    "Dynamic Player",
    "Kinematic Player",
    "Pure Player",
];

/// **A sonda.** `cargo test -p ph2d-host-desktop --release probe_smoke_106 --
/// --ignored --nocapture`
#[test]
#[ignore = "sonda de medição"]
fn probe_smoke_106() {
    println!("\n=== cena 106: quanto a correnteza de {FORCE:.0} N leva cada um ===");
    print!("{:<8}", "t (s)");
    for who in SUBJECTS {
        print!("{who:>18}");
    }
    println!();
    for secs in [0.5_f32, 1.0, 2.0, 4.0] {
        let s = run(secs);
        print!("{secs:<8.1}");
        for who in SUBJECTS {
            print!("{:>18.4}", x_of(&s, who));
        }
        println!();
    }
    println!(
        "\nLEITURA: os TRES players tem de concordar. Se o AZUL e o ROXO ficarem em\n\
         0,0000, a forca da zona voltou a nao existir para o modo cinematico.\n"
    );
}

/// **OS QUATRO CORPOS SÃO COMPARÁVEIS** — o gate que só esta cena pode afirmar.
///
/// ⚠️ **Sem ele a cena mente sem quebrar nada:** uma densidade diferente, uma cápsula
/// mais gorda, um freio distinto ou um `x` de partida distinto produziriam quatro
/// distâncias diferentes, o artista veria a diferença e concluiria que a wave falhou —
/// ou, pior, que ela funcionou quando não funcionou.
#[test]
fn the_four_subjects_differ_only_in_the_mode() {
    let mut sim = SimWorld::new();
    build_zone_force_scene(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    let world = sim.world();

    let mut shapes = Vec::new();
    let mut starts = Vec::new();
    let mut brakes = Vec::new();
    let mut modes = Vec::new();
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
        starts.push(t.translation.x);
        if let Some(p) = p {
            brakes.push((p.acceleration, p.air_acceleration));
        }
        modes.push((n.as_str().to_string(), rb.kind, m.copied()));
    }

    assert_eq!(shapes.len(), 4, "a cena tem de montar os QUATRO sujeitos");
    assert!(
        shapes.windows(2).all(|w| w[0] == w[1]),
        "forma e densidade têm de ser idênticas nos quatro: {shapes:?}"
    );
    assert!(
        starts.windows(2).all(|w| (w[0] - w[1]).abs() < 1.0e-6),
        "os quatro partem do MESMO x: {starts:?}"
    );
    assert_eq!(brakes.len(), 3, "três têm lei de player; o caixote não tem");
    assert!(
        brakes.windows(2).all(|w| w[0] == w[1]),
        "e o freio da caminhada é o MESMO nos três: {brakes:?}"
    );

    // E o que DIFERE é exactamente uma coisa: o modo de cada player.
    for (name, want_kind, want_mode) in [
        ("Loose Crate", BodyKind::Dynamic, None),
        (
            "Dynamic Player",
            BodyKind::Dynamic,
            Some(PlayerMode::Dynamic),
        ),
        (
            "Kinematic Player",
            BodyKind::Kinematic,
            Some(PlayerMode::Kinematic),
        ),
        ("Pure Player", BodyKind::Kinematic, Some(PlayerMode::Pure)),
    ] {
        let got = modes
            .iter()
            .find(|(n, ..)| n == name)
            .unwrap_or_else(|| panic!("{name} tem de existir"));
        assert_eq!(got.1, want_kind, "{name}: RigidBody.kind");
        assert_eq!(got.2, want_mode, "{name}: PlayerMode");
    }
}

/// **A CENA ENTREGA O QUE A MENSAGEM PROMETE: os três modos viajam, e viajam juntos.**
///
/// ⚠️ **Nasceu VERMELHO** — antes da `W-ZoneForce` o azul e o roxo andavam `0,0000 m`
/// nesta cena, em qualquer força, enquanto o âmbar viajava 20,6.
///
/// ⚠️ **E a metade que separa *anda* de *anda o mesmo*:** o primeiro `assert` diz que
/// eles saíram do lugar, o segundo que os três concordam. Sem o segundo, um empurrão de
/// magnitude errada (por exemplo, a força em vez da aceleração — a massa esquecida)
/// passaria com folga.
#[test]
fn the_scene_delivers_the_numbers_its_message_prints() {
    let sim = run(2.0);
    for (i, who) in SUBJECTS.iter().enumerate() {
        let x = x_of(&sim, who);
        assert!(
            (x - CARRIED[i]).abs() < 0.5,
            "{who} andou {x:.4} m e a mensagem promete {:.4}",
            CARRIED[i]
        );
    }

    let dynamic = x_of(&sim, "Dynamic Player");
    for who in ["Kinematic Player", "Pure Player"] {
        let x = x_of(&sim, who);
        assert!(
            x > 5.0,
            "{who} tem de ser levado pela correnteza; andou {x:.4} m (antes da wave: 0,0000)"
        );
        let ratio = x / dynamic;
        assert!(
            (0.94..=1.06).contains(&ratio),
            "{who} andou {x:.4} m contra {dynamic:.4} do dinâmico (razão {ratio:.4})"
        );
    }

    // E o caixote é o TETO: a caminhada resiste, então nenhum player o alcança.
    let crate_x = x_of(&sim, "Loose Crate");
    assert!(
        crate_x > dynamic * 1.5,
        "o caixote solto tem de andar bem mais que um player: {crate_x:.4} contra {dynamic:.4}"
    );
}

/// **A ZONA É LARGA O BASTANTE PARA NINGUÉM SAIR DELA** — a premissa da cena.
///
/// ⚠️ Um sujeito que sai da correnteza para de acelerar, e *"andou menos"* passaria a
/// significar *"saiu"* em vez de *"o freio dele resistiu"* — que é a pergunta da cena.
/// Uma premissa só escrita em prosa é uma que a próxima edição apaga.
#[test]
fn nobody_leaves_the_current_during_the_run() {
    let sim = run(2.0);
    for who in SUBJECTS {
        let x = x_of(&sim, who);
        assert!(
            x < ZONE_HALF[0],
            "{who} chegou a {x:.4} e a zona acaba em {:.1}",
            ZONE_HALF[0]
        );
    }
}

/// **A MENSAGEM É HONESTA SOBRE O GESTO QUE ELA MANDA FAZER.**
///
/// ⚠️ **Nasceu porque a primeira versão dela mandava fazer um gesto que eu nunca tinha
/// rodado:** o passo 3 dizia *"ele tem de conseguir progredir contra a correnteza"* e o
/// Enio reportou que não progredia. Não progride, e não podia — esta cena não tem chão,
/// então o player usa a `air_acceleration` (**20 m/s²**) contra os **43,76** desta
/// correnteza (16 N sobre 0,3657 kg).
///
/// A regra que esta cena já carregava — *uma cena cuja mensagem cita NÚMEROS tem de
/// medir os dela* — vale igual para os **GESTOS** que ela manda fazer. Este gate afirma
/// as DUAS metades, porque só as duas juntas descrevem o que o artista vê:
///
/// 1. com o dedo em `A` ele ainda é levado a favor (o mesmo SINAL de quando está solto),
///    só mais devagar — se algum dia isto virar negativo, a mensagem passou a mentir ao
///    contrário;
/// 2. e a ablação que a mensagem oferece **funciona**: abaixo dos 20 m/s² da caminhada
///    ele progride CONTRA.
#[test]
fn the_message_is_honest_about_walking_against_the_current() {
    let against = walked_x(-1.0, FORCE);
    let idle = walked_x(0.0, FORCE);
    let along = walked_x(1.0, FORCE);

    assert!(
        against > 0.0,
        "nesta cena o dedo NAO inverte o lado: com `A` ele ainda foi para {against:.3} m"
    );
    assert!(
        against < idle && idle < along,
        "mas o dedo tem de MUDAR o quanto ele e' levado: {against:.3} < {idle:.3} < {along:.3}"
    );
    for (i, got) in [against, idle, along].iter().enumerate() {
        assert!(
            (got - WALKED[i]).abs() < 0.5,
            "e os numeros publicados tem de ser os medidos: {got:.3} contra {:.3}",
            WALKED[i]
        );
    }

    // A ABLAÇÃO da mensagem: `7 N` põe a correnteza em 19,1 m/s², logo abaixo dos 20 da
    // caminhada no ar — e aí ele vence. ⚠️ **É esta metade que torna o passo 3b um
    // experimento em vez de uma frase**: sem ela a mensagem mandaria mexer num slider
    // sem ninguém ter conferido o outro lado da fronteira.
    let won = walked_x(-1.0, 7.0);
    assert!(
        won < -5.0,
        "abaixo da autoridade da caminhada ele tem de progredir CONTRA: {won:.3} m"
    );
}

/// Dois segundos com o dedo em `drive`, na cena com a força `force`.
///
/// ⚠️ **A entrada vai para TODO player**, como a ponte faz — `hand_input_to_players`
/// entrega a mesma a cada um, porque há um teclado e logo um dedo. Uma fixture que a
/// desse só a um mediria uma cena que o artista não consegue produzir.
fn walked_x(drive: f32, force: f32) -> f32 {
    let mut sim = SimWorld::new();
    build_zone_force_scene(sim.world_mut());
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    if (force - FORCE).abs() > f32::EPSILON {
        let zone = {
            let mut q = sim
                .world()
                .try_query::<(ph2d_ecs::Entity, &Name)>()
                .unwrap();
            q.iter(sim.world())
                .find(|(_, n)| n.as_str() == "Current")
                .map(|(e, _)| e)
                .expect("a cena tem uma correnteza")
        };
        sim.world_mut().entity_mut(zone).insert(AreaEffector {
            force: [force, 0.0],
        });
    }
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(PhysicsSettings {
        gravity_y: 0.0,
        ..Default::default()
    });
    let players: Vec<_> = {
        let mut q = sim
            .world()
            .try_query::<(ph2d_ecs::Entity, &PlatformPlayer)>()
            .unwrap();
        q.iter(sim.world()).map(|(e, _)| e).collect()
    };
    for e in players {
        bridge.set_player_input(
            e,
            PlayerInput {
                drive,
                ..PlayerInput::default()
            },
        );
    }
    for t in 0..=120u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    x_of(&sim, "Kinematic Player")
}
