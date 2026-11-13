//! Os gates da cena 111 (`W-Ledge`) — os NÚMEROS que a mensagem imprime,
//! afirmados antes de o artista os ler.
//!
//! ⚠️ **A cena inteira é um contraste**, então o gate corre os DOIS lados: um
//! gate que só afirmasse *"o da direita sobe"* passaria numa cena cujo patamar
//! alto fosse baixo demais para separar coisa alguma.

use super::{FLOAT, GRAB, HIGH_TOP, LANE_A, LANE_B, LANE_SPAN, LOW_TOP, build_ledge_scene};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

/// A meia-altura da cápsula do `spawn_player`.
const HALF_H: f32 = 0.5;

/// **O alcance de um pulo segurado é a altura PEDIDA** (`jump_height`, o ápice real desde
/// `f3a92736ab`) — encostado na parede ou não, desde que o corpo do player não raspa (Bug #14; até
/// lá o atrito na face comia 61 % e esta régua era um 0,745 medido).
fn reach_one() -> f32 {
    ph2d_physics_ecs::PlatformPlayer::default().jump_height
}

/// O topo do corpo no ápice de um pulo dado do chão.
fn peak_top() -> f32 {
    FLOAT + reach_one() + HALF_H
}

/// **A aritmética que a mensagem imprime está certa** — em tempo de compilação.
///
/// ⚠️ **É este gate que impede a cena de deixar de conter o próprio fenômeno**,
/// e ele tem de afirmar as DUAS pontas:
///
/// - baixar o patamar alto para dentro do alcance de um pulo faria os dois
///   subirem, e a cena não distinguiria nada;
/// - subi-lo para além de *um pulo mais o braço* faria a beirada nunca chegar à
///   janela, e o smoke leria como *"a feature não funciona"* uma cena em que ela
///   nunca foi oferecida.
#[test]
fn the_scene_delivers_the_numbers_its_message_prints() {
    // O baixo cabe num pulo: os PÉS chegam a `peak_top − 2·HALF_H`.
    assert!(LOW_TOP < peak_top() - 2.0 * HALF_H, "baixo {LOW_TOP}");
    // ⚠️ **No alto ninguém POUSA a pular, e a régua são os PÉS.**
    assert!(peak_top() - 2.0 * HALF_H < HIGH_TOP, "alto {HIGH_TOP}");
    // ⚠️ **E a JANELA do braço contém o lábio no ápice do gesto do passo 3** (pular colado à
    // face): o lábio fica acima da cabeça e a menos de `GRAB` dela.
    assert!(
        HIGH_TOP > peak_top(),
        "lábio {HIGH_TOP} × topo no ápice {}",
        peak_top()
    );
    assert!(
        HIGH_TOP - GRAB < peak_top(),
        "lábio {HIGH_TOP} fora do braço"
    );
    // E o braço existe — quanto tempo a janela dura é dinâmica, e quem a afirma
    // é o gate de comportamento ao lado.
    const _: () = assert!(GRAB > 0.0);
    // ⚠️ E as raias não se tocam: a raia mede 12 m de chão.
    const _: () = assert!(LANE_SPAN >= 12.0);
    const _: () = assert!(LANE_B - LANE_A == LANE_SPAN);
}

/// O `y` de uma entidade.
fn y(sim: &SimWorld, e: Entity) -> f32 {
    sim.world()
        .get::<Transform>(e)
        .expect("transform")
        .translation
        .y
}

/// Dirige os DOIS com a MESMA entrada, pelo roteiro do passo 3/4.
///
/// ⚠️ **A entrada é uma só, como no produto** (`hand_input_to_players` entrega a
/// todos): um harness que dirigisse cada um por sua vez mediria duas corridas e
/// perderia a propriedade que a cena existe para mostrar.
fn drive(mantle: bool) -> (f32, f32) {
    let mut sim = SimWorld::new();
    let (plain, ledge) = build_ledge_scene(sim.world_mut());
    // ⚠️ **Os dois são POSTOS diante do patamar alto**, e não conduzidos até lá:
    // a travessia da raia é o que o ARTISTA faz (e a sonda
    // `measure_where_the_lane_takes_him` mostra que ela funciona), enquanto o
    // que este gate afirma são os NÚMEROS do gesto do passo 3. Um gate que
    // dependesse de duzentos tiques de percurso mediria o percurso.
    for (e, lane) in [(plain, LANE_A), (ledge, LANE_B)] {
        let mut t = sim.world_mut().get_mut::<Transform>(e).expect("transform");
        t.translation.x = lane + 8.5 - 0.25;
        t.translation.y = FLOAT;
    }
    let mut bridge = PhysicsBridge::new();
    let mut tick = 0_u64;
    let mut step = |sim: &mut SimWorld, bridge: &mut PhysicsBridge, jump: bool, drive: f32| {
        let input = PlayerInput {
            jump,
            drive,
            ..PlayerInput::default()
        };
        bridge.set_player_input(plain, input);
        bridge.set_player_input(ledge, input);
        tick += 1;
        bridge.dispatch(sim, true, tick);
    };
    // Assenta a mola.
    for _ in 0..30 {
        step(&mut sim, &mut bridge, false, 0.0);
    }
    // Encosta na face.
    for _ in 0..30 {
        step(&mut sim, &mut bridge, false, 1.0);
    }
    // Pula contra ela SEGURANDO a direção — o passo 3.
    for _ in 0..40 {
        step(&mut sim, &mut bridge, true, 1.0);
    }
    for _ in 0..60 {
        step(&mut sim, &mut bridge, false, 1.0);
    }
    if mantle {
        // O passo 4: um aperto NOVO no pulo (o anterior já foi solto acima), e
        // depois o dedo larga tudo.
        //
        // ⚠️ **Largar é parte do gesto:** com o pulo preso ele salta no tique em
        // que a subida acaba (o pulo é mascarado na entrada enquanto a beirada
        // age, então o botão lê como aperto novo), e com a direção presa ele anda
        // até sair pelo outro lado do patamar.
        for _ in 0..2 {
            step(&mut sim, &mut bridge, true, 1.0);
        }
        for _ in 0..180 {
            step(&mut sim, &mut bridge, false, 0.0);
        }
    }
    (y(&sim, plain), y(&sim, ledge))
}

/// **O contraste que a cena É: com o mesmo gesto, só um fica PRESO.**
///
/// A metade de controlo é **o outro CAI**: sem atrito no corpo do player (Bug #14) nada o segura
/// na face, e no fim do gesto ele já está de pé no chão. (Até o #14 o atrito o prendia 17 cm
/// abaixo do lábio e este gate media o escorregar de 7,5 cm/s.)
#[test]
fn the_same_gesture_hangs_only_the_one_with_a_reach() {
    let ((plain0, ledge0), (plain1, ledge1)) = hang_then_wait();
    // Pendurado, o TOPO do corpo assenta no lábio…
    assert!(
        (ledge0 + HALF_H - HIGH_TOP).abs() < 0.08,
        "o da direita tinha de ficar pendurado com o topo no labio ({HIGH_TOP}): \
         topo em {:.3}",
        ledge0 + HALF_H
    );
    // …e continua lá um segundo depois.
    assert!(
        (ledge1 - ledge0).abs() < 0.02,
        "e tinha de FICAR: {ledge0:.3} -> {ledge1:.3}"
    );
    // O da esquerda não está seguro por nada: ele cai e fica de pé no chão.
    for y in [plain0, plain1] {
        assert!(
            (y - FLOAT).abs() < 0.02,
            "o da esquerda nao esta' preso a nada — tinha de estar no chao: {plain0:.3} -> {plain1:.3}"
        );
    }
}

/// O gesto do passo 3 e, depois dele, mais DOIS segundos de dedo preso.
fn hang_then_wait() -> ((f32, f32), (f32, f32)) {
    let mut sim = SimWorld::new();
    let (plain, ledge) = build_ledge_scene(sim.world_mut());
    for (e, lane) in [(plain, LANE_A), (ledge, LANE_B)] {
        let mut t = sim.world_mut().get_mut::<Transform>(e).expect("transform");
        t.translation.x = lane + 8.5 - 0.25;
        t.translation.y = FLOAT;
    }
    let mut bridge = PhysicsBridge::new();
    let mut tick = 0_u64;
    let mut step = |sim: &mut SimWorld, bridge: &mut PhysicsBridge, jump: bool| {
        let input = PlayerInput {
            jump,
            drive: 1.0,
            ..PlayerInput::default()
        };
        bridge.set_player_input(plain, input);
        bridge.set_player_input(ledge, input);
        tick += 1;
        bridge.dispatch(sim, true, tick);
    };
    for _ in 0..30 {
        step(&mut sim, &mut bridge, false);
    }
    for _ in 0..40 {
        step(&mut sim, &mut bridge, true);
    }
    for _ in 0..60 {
        step(&mut sim, &mut bridge, false);
    }
    let first = (y(&sim, plain), y(&sim, ledge));
    for _ in 0..120 {
        step(&mut sim, &mut bridge, false);
    }
    (first, (y(&sim, plain), y(&sim, ledge)))
}

/// **E o passo 4 é VERDADE: ele acaba DE PÉ em cima do patamar.**
///
/// ⚠️ **A altura de repouso é `lábio + float_height`, e não *"acima do
/// lábio"***: a perna é uma mola e o personagem PAIRA, então um gate que
/// pedisse só *"passou por cima"* ficaria verde com ele a subir para sempre.
#[test]
fn the_mantle_leaves_him_standing_on_the_high_shelf() {
    let (plain, ledge) = drive(true);
    assert!(
        (ledge - (HIGH_TOP + FLOAT)).abs() < 0.15,
        "de pe' no patamar seria y = {:.3}; ele esta' em {ledge:.3}",
        HIGH_TOP + FLOAT
    );
    assert!(
        plain < HIGH_TOP,
        "e o da esquerda continua embaixo dele: {plain:.3}"
    );
}

/// **Os dois sobem o patamar BAIXO** — o controle que torna a falha acima uma
/// falta de BRAÇO, e não um personagem quebrado.
#[test]
fn both_of_them_clear_the_low_shelf() {
    let mut sim = SimWorld::new();
    let (plain, ledge) = build_ledge_scene(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let mut tick = 0_u64;
    let mut hi = (f32::MIN, f32::MIN);
    for i in 0..90_u64 {
        let input = PlayerInput {
            jump: (30..55).contains(&i),
            ..PlayerInput::default()
        };
        bridge.set_player_input(plain, input);
        bridge.set_player_input(ledge, input);
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
        hi = (hi.0.max(y(&sim, plain)), hi.1.max(y(&sim, ledge)));
    }
    for (who, v) in [("sem braco", hi.0), ("com braco", hi.1)] {
        assert!(
            v > LOW_TOP,
            "o {who} tem de passar do topo do patamar baixo ({LOW_TOP}): {v:.3}"
        );
    }
}

/// **Quanto um pulo COLADO À PAREDE alcança** — o número que escolhe o patamar
/// alto.
///
/// Desde o Bug #14 é o mesmo do ar livre (`reach_one`); até lá o atrito contra a face comia 61 %.
#[test]
#[ignore = "sonda de medicao"]
fn measure_what_a_jump_against_the_wall_reaches() {
    let mut sim = SimWorld::new();
    let (_, ledge) = build_ledge_scene(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    let mut tick = 0_u64;
    for _ in 0..30 {
        tick += 1;
        bridge.set_player_input(ledge, PlayerInput::default());
        bridge.dispatch(&mut sim, true, tick);
    }
    let rest = y(&sim, ledge);
    for _ in 0..190 {
        tick += 1;
        bridge.set_player_input(
            ledge,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, tick);
    }
    let at_wall = y(&sim, ledge);
    let mut peak = f32::MIN;
    for i in 0..120_u64 {
        tick += 1;
        bridge.set_player_input(
            ledge,
            PlayerInput {
                drive: 1.0,
                jump: i < 40,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, tick);
        peak = peak.max(y(&sim, ledge));
    }
    println!("\n== o pulo COLADO A' PAREDE (labio em {HIGH_TOP:.2}) ==");
    println!(
        "  repouso            y = {rest:.4}  (topo {:.4})",
        rest + HALF_H
    );
    println!("  encostado na face  y = {at_wall:.4}");
    println!(
        "  pico do pulo       y = {peak:.4}  (topo {:.4}), sobe {:.4} m",
        peak + HALF_H,
        peak - rest
    );
    println!("  em AR LIVRE ele subiria {:.3} m", reach_one());
}

/// **O PERCURSO da raia** — onde ele chega, com o gesto do artista.
///
/// ⚠️ **Correr para a direita NÃO o leva ao patamar alto**, e foi assim que os
/// gates desta cena nasceram vermelhos: o patamar BAIXO está no caminho, e um
/// dedo que só empurra encosta-o na face dele. O gesto real pulsa o pulo.
#[test]
#[ignore = "sonda de medicao"]
fn measure_where_the_lane_takes_him() {
    let mut sim = SimWorld::new();
    let (_, ledge) = build_ledge_scene(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    println!("\n== o percurso da raia da direita (baixo {LOW_TOP:.1}, alto {HIGH_TOP:.1}) ==");
    for t in 1..=600_u64 {
        bridge.set_player_input(
            ledge,
            PlayerInput {
                drive: 1.0,
                // Pulsa: 25 tiques preso, 15 solto — o que uma mão faz.
                jump: t % 40 < 25,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
        if t % 25 == 0 {
            let tr = sim.world().get::<Transform>(ledge).expect("t").translation;
            println!(
                "  t={t:>3}  x={:>6.3}  y={:>6.3}  topo={:>6.3}",
                tr.x,
                tr.y,
                tr.y + HALF_H
            );
        }
    }
}

/// **SONDA de diagnóstico: quão perto da parede ele chega.**
///
/// Report do smoke: *"o objeto não encosta na parede"*. A pergunta é um número —
/// a face do patamar está num `x` conhecido, e a meia-largura do corpo também.
#[test]
#[ignore = "sonda de diagnostico"]
fn how_close_to_the_wall_he_gets() {
    /// A meia-largura da cápsula do `spawn_player` (o `radius`).
    const HALF_W: f32 = 0.2;
    let mut sim = SimWorld::new();
    let (_, ledge) = build_ledge_scene(sim.world_mut());
    let mut bridge = PhysicsBridge::new();
    // A face ESQUERDA do patamar baixo, na raia da direita.
    let face = LANE_B + 3.0;
    println!("\n== quao perto da parede ele chega (face em x = {face:.2}) ==");
    let mut best = f32::INFINITY;
    for t in 1..=420_u64 {
        bridge.set_player_input(
            ledge,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
        let tr = sim.world().get::<Transform>(ledge).expect("t").translation;
        let gap = (face - HALF_W) - tr.x;
        best = best.min(gap.abs());
        if t % 30 == 0 {
            println!(
                "  t={t:>3}  x={:>7.3}  y={:>6.3}  vao ate' a face = {:>7.4} m",
                tr.x, tr.y, gap
            );
        }
    }
    println!("  MENOR vao alcancado: {best:.4} m  (0 = encostado)");
}

/// **SONDA de diagnóstico: PENDURADO, quão longe da parede ele fica.**
///
/// Report do smoke (com foto): *"o objeto não encosta na parede"* — e na foto a
/// cabeça está à altura do lábio, que é a pose do pendurar. A pergunta é o `x`.
#[test]
#[ignore = "sonda de diagnostico"]
fn how_far_from_the_wall_he_hangs() {
    /// A meia-largura da cápsula do `spawn_player` (o `radius`).
    const HALF_W: f32 = 0.2;
    let mut sim = SimWorld::new();
    let (_, ledge) = build_ledge_scene(sim.world_mut());
    let face = LANE_B + 8.5;
    {
        let mut t = sim.world_mut().get_mut::<Transform>(ledge).expect("t");
        t.translation.x = face - 0.25;
        t.translation.y = FLOAT;
    }
    let mut bridge = PhysicsBridge::new();
    let mut tick = 0_u64;
    let mut step = |sim: &mut SimWorld, bridge: &mut PhysicsBridge, jump: bool, drive: f32| {
        bridge.set_player_input(
            ledge,
            PlayerInput {
                jump,
                drive,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(sim, true, tick);
    };
    println!("\n== pendurado: onde ele fica (face do patamar em x = {face:.2}) ==");
    for _ in 0..30 {
        step(&mut sim, &mut bridge, false, 0.0);
    }
    for _ in 0..30 {
        step(&mut sim, &mut bridge, false, 1.0);
    }
    for i in 0..140 {
        step(&mut sim, &mut bridge, i < 40, 1.0);
        if i % 20 == 0 || i == 139 {
            let tr = sim.world().get::<Transform>(ledge).expect("t").translation;
            println!(
                "  i={i:>3}  x={:>7.3}  y={:>6.3}  topo={:>6.3}  vao ate' a face = {:>7.4} m",
                tr.x,
                tr.y,
                tr.y + HALF_H,
                (face - HALF_W) - tr.x
            );
        }
    }
}

/// **SONDA de diagnóstico: encostar no patamar BAIXO vindo da DIREITA.**
///
/// ⚠️ **É o lado que os gates não visitam.** O passo 8 do roteiro afirma que um
/// degrau que se sobe a pé **não** é beirada, e a fixture dele aproxima-se pela
/// ESQUERDA. Vindo da direita a cabeça também passa o lábio, e se a lei engatar
/// o servo segura o corpo a uma distância fixa da face — que é exactamente o que
/// *"o objeto não encosta na parede"* descreve.
#[test]
#[ignore = "sonda de diagnostico"]
fn how_close_he_gets_walking_left_into_the_low_step() {
    /// A meia-largura da cápsula do `spawn_player` (o `radius`).
    const HALF_W: f32 = 0.2;
    for (tag, grab) in [("sem beirada", 0.0_f32), ("com beirada 0.60", 0.6)] {
        let mut sim = SimWorld::new();
        let (_, ledge) = build_ledge_scene(sim.world_mut());
        {
            let mut p = sim
                .world_mut()
                .get_mut::<ph2d_physics_ecs::PlatformPlayer>(ledge)
                .expect("player");
            p.ledge_grab = grab;
            p.ledge_on = grab > 0.0;
        }
        {
            let mut t = sim.world_mut().get_mut::<Transform>(ledge).expect("t");
            // No chão, à DIREITA do patamar baixo (face direita em `+5.0`).
            t.translation.x = LANE_B + 6.5;
            t.translation.y = FLOAT;
        }
        let mut bridge = PhysicsBridge::new();
        for t in 1..=300_u64 {
            bridge.set_player_input(
                ledge,
                PlayerInput {
                    drive: -1.0,
                    ..PlayerInput::default()
                },
            );
            bridge.dispatch(&mut sim, true, t);
        }
        let tr = sim.world().get::<Transform>(ledge).expect("t").translation;
        let face = LANE_B + 5.0;
        println!(
            "  {tag:<18}  x={:>7.3}  y={:>6.3}  vao ate' a face = {:>7.4} m",
            tr.x,
            tr.y,
            tr.x - (face + HALF_W)
        );
    }
}
