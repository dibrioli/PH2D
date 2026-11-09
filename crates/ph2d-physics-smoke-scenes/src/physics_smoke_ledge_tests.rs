//! Os gates da cena 111 (`W-Ledge`) — os NÚMEROS que a mensagem imprime,
//! afirmados antes de o artista os ler.
//!
//! ⚠️ **A cena inteira é um contraste**, então o gate corre os DOIS lados: um
//! gate que só afirmasse *"o da direita sobe"* passaria numa cena cujo patamar
//! alto fosse baixo demais para separar coisa alguma.

use super::{FLOAT, GRAB, HIGH_TOP, LANE_A, LANE_B, LANE_SPAN, LOW_TOP, build_ledge_scene};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{PhysicsBridge, PlayerInput};

/// O alcance MEDIDO de um pulo segurado (`measure_the_reachable_height`).
const REACH_ONE: f32 = 1.903;
/// A meia-altura da cápsula do `spawn_player`.
const HALF_H: f32 = 0.5;
/// **O topo do corpo no ápice de um pulo COLADO À PAREDE**, medido pela sonda
/// deste arquivo (`measure_what_a_jump_against_the_wall_reaches`): 0,745 m de
/// subida contra os 1,903 do ar livre — o atrito contra a face come **61%**.
///
/// ⚠️ **É esta a régua da cena, e não o `REACH_ONE`:** chega-se a um patamar
/// encostado nele.
const WALL_HUG_PEAK_TOP: f32 = 2.145;

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
    // ⚠️ **O baixo cabe num pulo COLADO À PAREDE**, e é essa a régua: os PÉS
    // chegam a `WALL_HUG_PEAK_TOP − 2·HALF_H`. A primeira versão desta cena o
    // pôs em 1,5 usando o número do ar livre, e ele ficou **intransponível** —
    // o personagem encostava na face e pulava no lugar para sempre.
    const _: () = assert!(LOW_TOP < WALL_HUG_PEAK_TOP - 2.0 * HALF_H);
    // E cabe num pulo de ar livre, obviamente.
    const _: () = assert!(LOW_TOP < REACH_ONE);
    // ⚠️ **No alto ninguém POUSA a pular, e a régua são os PÉS** — num pulo de
    // ar livre perfeito eles chegam a `FLOAT + REACH_ONE − HALF_H`.
    const _: () = assert!(FLOAT + REACH_ONE - HALF_H < HIGH_TOP);
    // ⚠️ **E a JANELA tem de estar no caminho do arco COLADO à parede**, que é o
    // gesto do passo 3 — as duas linhas que a primeira versão desta cena não
    // tinha, e por cuja falta o corpo nunca chegava a ver o lábio.
    const _: () = assert!(HIGH_TOP > WALL_HUG_PEAK_TOP);
    const _: () = assert!(HIGH_TOP - GRAB < WALL_HUG_PEAK_TOP);
    // O alto NÃO — um pulo sozinho não põe ninguém de pé nele (de pé é
    // `HIGH_TOP + FLOAT`, e o ápice é `FLOAT + REACH_ONE`).
    const _: () = assert!(HIGH_TOP > REACH_ONE);
    // …mas o TOPO DO CORPO no ápice desse pulo PASSA do lábio, e é isso que põe
    // a janela de agarre no caminho da subida.
    //
    // ⚠️ **A quarta asserção desta lista nasceu ERRADA e o compilador a
    // reprovou**, o que é o gate a fazer o seu trabalho: ela pedia que o lábio
    // estivesse *ao alcance do braço a partir do ápice*, e no ápice a cabeça
    // está **acima** do lábio — não abaixo. A janela é cruzada a SUBIR, antes do
    // ápice, e o que a torna alcançável é o corpo passar por ela, que é o que a
    // linha abaixo afirma.
    const _: () = assert!(HIGH_TOP < FLOAT + REACH_ONE + HALF_H);
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
/// ⚠️ **A metade de controlo NÃO é *"o outro cai"*, e a primeira versão deste
/// gate a escreveu assim e falhou.** Quem empurra contra uma parede sem
/// `wall_slide_speed` **fica preso pelo atrito** e desce devagarinho — medido,
/// ele para 17 cm abaixo do lábio, que é perto demais para distinguir seja o que
/// for. O que separa os dois não é a ALTURA, é **estar ou não SEGURO**: um fica
/// exatamente onde está, o outro continua a escorregar.
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
    // O da esquerda não está seguro por nada: ele escorrega.
    // ⚠️ **A barra é 0,08 e o número medido é 0,15 em dois segundos** — o
    // atrito deixa-o descer ~7,5 cm/s (é a mesma taxa que o doc do `wall_slide`
    // mediu ao derrubar a versão-teto daquela lei). Pedir mais seria pedir que a
    // parede o soltasse; pedir menos não distinguiria de estar preso.
    assert!(
        plain0 - plain1 > 0.08,
        "o da esquerda nao esta' preso a nada — tinha de escorregar: \
         {plain0:.3} -> {plain1:.3}"
    );
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
/// ⚠️ **O `REACH_ONE` de 1,903 m é de AR LIVRE**, e a cena não é: o gesto é
/// correr contra a parede e pular a segurar a direção, e aí o atrito contra a
/// face come parte da subida. Usar o número do ar livre foi como a primeira
/// versão desta cena nasceu — e o patamar alto ficou fora do caminho do corpo.
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
    println!("  em AR LIVRE ele subiria {REACH_ONE:.3} m");
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
