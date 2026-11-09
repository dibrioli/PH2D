//! Os gates da cena 115 — e a SONDA que numerou o roteiro dela.

use super::*;

/// **A SONDA que escreveu os números da mensagem** — arranque, derrapagem e o
/// quanto a esteira leva, por raia.
///
/// `cargo test -p ph2d-host-desktop --bins measure_the_scene_surface -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_the_scene_surface() {
    use ph2d_ecs::SimWorld;
    use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

    let mut sim = SimWorld::new();
    let lanes = build_surface_scene(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let x = |sim: &SimWorld, e: Entity| {
        sim.world()
            .get::<ph2d_ecs::Transform>(e)
            .expect("transform")
            .translation
            .x
    };

    let mut tick = 0_u64;
    let mut run = |sim: &mut SimWorld, bridge: &mut PhysicsBridge, n: u64, drive: f32| {
        for _ in 0..n {
            tick += 1;
            for (_, p) in &lanes {
                bridge.set_player_input(
                    *p,
                    PlayerInput {
                        drive,
                        ..PlayerInput::default()
                    },
                );
            }
            bridge.dispatch(sim, true, tick);
        }
    };

    // ⚠️ A perna assenta com o eixo SOLTO — a janela do spawn é aérea, e uma
    // superfície de pouca tração colhe velocidade ali e a guarda.
    run(&mut sim, &mut bridge, 30, 0.0);
    let idle0: Vec<f32> = lanes.iter().map(|(_, p)| x(&sim, *p)).collect();
    run(&mut sim, &mut bridge, 60, 0.0);
    let idle: Vec<f32> = lanes
        .iter()
        .enumerate()
        .map(|(i, (_, p))| x(&sim, *p) - idle0[i])
        .collect();

    // ⚠️ **O GESTO DO ROTEIRO, e não um lançamento** — e a diferença derrubou a
    // primeira versão desta mensagem. A suíte da ponte LANÇA os dois à mesma
    // velocidade de propósito (senão o gelo, que nunca arranca, derrapa
    // *menos*); aqui o artista **corre e solta na marca**, então a sonda tem de
    // correr até a marca e soltar. Medir a derrapagem de um arranque de 1 s
    // dizia que o gelo derrapa 1,16 m contra 2,95 do controle — o contrário do
    // que a raia mostra.
    let start0: Vec<f32> = lanes.iter().map(|(_, p)| x(&sim, *p)).collect();
    run(&mut sim, &mut bridge, 60, 1.0);
    let started: Vec<f32> = lanes
        .iter()
        .enumerate()
        .map(|(i, (_, p))| x(&sim, *p) - start0[i])
        .collect();

    // Cada raia corre até PASSAR da marca, e só então solta. O relógio é o
    // mesmo para todas (uma raia lenta simplesmente demora mais a chegar).
    let mut released = vec![false; lanes.len()];
    let mut mark_x = vec![0.0_f32; lanes.len()];
    for _ in 0..2000 {
        tick += 1;
        for (i, (_, p)) in lanes.iter().enumerate() {
            let here = x(&sim, *p) - lane_x(i);
            if !released[i] && here >= MARK_X {
                released[i] = true;
                mark_x[i] = x(&sim, *p);
            }
            let drive = if released[i] { 0.0 } else { 1.0 };
            bridge.set_player_input(
                *p,
                PlayerInput {
                    drive,
                    ..PlayerInput::default()
                },
            );
        }
        bridge.dispatch(&mut sim, true, tick);
        if released.iter().all(|r| *r) {
            break;
        }
    }
    // E depois todas em roda-livre, tempo de sobra para assentarem (ou caírem).
    for _ in 0..300 {
        tick += 1;
        for (_, p) in &lanes {
            bridge.set_player_input(*p, PlayerInput::default());
        }
        bridge.dispatch(&mut sim, true, tick);
    }

    eprintln!("  raia        parado 1s   arranca 1s   derrapa (m)   caiu");
    for (i, (_, p)) in lanes.iter().enumerate() {
        let end = x(&sim, *p);
        let y = sim
            .world()
            .get::<ph2d_ecs::Transform>(*p)
            .expect("transform")
            .translation
            .y;
        eprintln!(
            "  {:<10}  {:9.2}   {:10.2}   {:11.2}   {}",
            SURFACES[i].1,
            idle[i],
            started[i],
            end - mark_x[i],
            if y < -1.0 { "SIM" } else { "nao" }
        );
    }
}

/// **A cena monta o que a mensagem afirma** — quatro raias, e só o CHÃO difere.
///
/// ⚠️ **A metade que carrega o gate é o CONTROLE:** a raia do meio não pode
/// carregar `WalkSurface` nenhum. Uma cena em que as quatro fossem autoradas
/// leria igual e não diria nada — três números sem régua.
#[test]
fn the_scene_differs_only_in_the_floor() {
    use ph2d_ecs::SimWorld;

    let mut sim = SimWorld::new();
    let lanes = build_surface_scene(sim.world_mut());
    assert_eq!(lanes.len(), SURFACES.len(), "uma raia por superficie");

    for (i, (deck, player)) in lanes.iter().enumerate() {
        let (want, tag) = SURFACES[i];
        assert_eq!(
            sim.world().get::<WalkSurface>(*deck).copied(),
            want,
            "a raia {tag} nao carrega a superficie que a mensagem afirma"
        );
        // ⚠️ E ela vive no DECK, nunca no personagem: uma superfície é
        // propriedade do CHÃO, e pô-la no corpo seria o modelo errado com o
        // mesmo efeito visível nesta cena.
        assert!(
            sim.world().get::<WalkSurface>(*player).is_none(),
            "a raia {tag} pos a superficie no PERSONAGEM"
        );
        let cfg = sim
            .world()
            .get::<PlatformPlayer>(*player)
            .expect("o player da raia");
        assert!(
            (cfg.speed - RUN_SPEED).abs() < 1e-6 && (cfg.acceleration - RUN_ACCEL).abs() < 1e-6,
            "os quatro personagens tem de ser IDENTICOS: {tag} diverge"
        );
    }
    assert!(
        lanes
            .iter()
            .enumerate()
            .any(|(i, (deck, _))| SURFACES[i].0.is_none()
                && sim.world().get::<WalkSurface>(*deck).is_none()),
        "a cena precisa de uma raia de CONTROLE, sem superficie nenhuma"
    );
}

/// **O gelo CAI e a borracha PÁRA** — a consequência que a mensagem promete,
/// pelo gesto que ela manda fazer.
///
/// ⚠️ **O oráculo é o gesto do roteiro, não um lançamento** — e é a lição que
/// custou a primeira versão da mensagem: a suíte da ponte lança os dois à mesma
/// velocidade de propósito, e aqui o artista *corre e solta*, onde o gelo (que
/// nunca arranca) derraparia MENOS. Correr até a marca é o que separa os dois.
#[test]
fn the_ice_lane_falls_into_the_pit_and_the_rubber_one_stops_short() {
    use ph2d_ecs::SimWorld;
    use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

    let mut sim = SimWorld::new();
    let lanes = build_surface_scene(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let pos = |sim: &SimWorld, e: Entity| {
        let t = sim
            .world()
            .get::<ph2d_ecs::Transform>(e)
            .expect("transform");
        (t.translation.x, t.translation.y)
    };

    let mut tick = 0_u64;
    let mut released = vec![false; lanes.len()];
    let mut mark = vec![0.0_f32; lanes.len()];
    for _ in 0..2400 {
        tick += 1;
        for (i, (_, p)) in lanes.iter().enumerate() {
            let here = pos(&sim, *p).0 - lane_x(i);
            if !released[i] && here >= MARK_X {
                released[i] = true;
                mark[i] = pos(&sim, *p).0;
            }
            bridge.set_player_input(
                *p,
                PlayerInput {
                    drive: if released[i] { 0.0 } else { 1.0 },
                    ..PlayerInput::default()
                },
            );
        }
        bridge.dispatch(&mut sim, true, tick);
        if released.iter().all(|r| *r) {
            break;
        }
    }
    assert!(
        released.iter().all(|r| *r),
        "toda raia tem de conseguir chegar a' marca — senao a cena mede outra coisa"
    );
    for _ in 0..300 {
        tick += 1;
        for (_, p) in &lanes {
            bridge.set_player_input(*p, PlayerInput::default());
        }
        bridge.dispatch(&mut sim, true, tick);
    }

    let by = |tag: &str| {
        let i = SURFACES
            .iter()
            .position(|(_, t)| *t == tag)
            .expect("a raia");
        let (x, y) = pos(&sim, lanes[i].1);
        (x - mark[i], y)
    };
    let (ice, ice_y) = by("Ice");
    let (normal, _) = by("Normal");
    let (rubber, _) = by("Rubber");
    assert!(
        ice_y < -1.0,
        "o gelo tem de CAIR no poco — a consequencia e' o que impede 'grip baixo' \
         de ser confundido com 'o knob esta' quebrado' (derrapou {ice:.2} m)"
    );
    assert!(
        rubber < normal && normal < ice,
        "a ordem das derrapagens e' a lei: borracha < controle < gelo, saiu \
         {rubber:.2} / {normal:.2} / {ice:.2}"
    );
}

/// **A esteira leva quem está PARADO** — o passo 1 da mensagem, o único que não
/// pede um toque no teclado.
#[test]
fn the_belt_lane_carries_a_player_who_does_nothing() {
    use ph2d_ecs::SimWorld;
    use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

    let mut sim = SimWorld::new();
    let lanes = build_surface_scene(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let x = |sim: &SimWorld, e: Entity| {
        sim.world()
            .get::<ph2d_ecs::Transform>(e)
            .expect("transform")
            .translation
            .x
    };
    // A perna assenta primeiro; só então a medição começa.
    let mut tick = 0_u64;
    let mut idle = |sim: &mut SimWorld, bridge: &mut PhysicsBridge, n: u64| {
        for _ in 0..n {
            tick += 1;
            for (_, p) in &lanes {
                bridge.set_player_input(*p, PlayerInput::default());
            }
            bridge.dispatch(sim, true, tick);
        }
    };
    idle(&mut sim, &mut bridge, 30);
    let before: Vec<f32> = lanes.iter().map(|(_, p)| x(&sim, *p)).collect();
    idle(&mut sim, &mut bridge, 60);

    for (i, (_, p)) in lanes.iter().enumerate() {
        let moved = x(&sim, *p) - before[i];
        let (surface, tag) = SURFACES[i];
        let belt = surface.map_or(0.0, |s| s.belt);
        if belt == 0.0 {
            assert!(
                moved.abs() < 0.05,
                "a raia {tag} nao tem correia e nao pode andar sozinha: {moved:.3} m"
            );
        } else {
            assert!(
                moved > 2.5,
                "a raia {tag} tem correia de {belt} m/s e tem de LEVAR quem esta' \
                 parado: {moved:.3} m em 1 s"
            );
        }
    }
}
