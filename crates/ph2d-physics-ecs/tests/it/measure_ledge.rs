//! **A BEIRADA, medida antes de qualquer número** (`W-Ledge`, plano 08 §4.5).
//!
//! O plano exige uma decisão de desenho **antes do código** (*o mantle move o
//! corpo para um lugar que a física não escolheu*) e a §4.3 deixou nomeado que
//! achar a beirada é uma pergunta de **PERFIL**, não de varredura. Estas sondas
//! respondem as três perguntas de que essa decisão depende:
//!
//! 1. **o que acontece HOJE** quando um personagem cai ao lado de um patamar —
//!    porque uma wave que não sabe o que já existe constrói a segunda porta;
//! 2. **o flanco já diz que a parede ACABA?** — o [`WallProbe`] é um perfil de N
//!    alturas, e se ele bastasse não haveria sensor novo a construir;
//! 3. **onde está o lábio, e com que precisão** — o número que a janela de
//!    agarre e o alvo do mantle vão consumir.
//!
//! ⚠️ **A pergunta 2 é a que decide o tamanho da wave**, e é por isso que ela é
//! feita antes de uma linha de produto ser escrita.
//!
//! Rode: `cargo test -p ph2d-physics-ecs --test measure_ledge --release
//! -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics::{BodyDesc, PhysicsWorld, ShapeDesc};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    RigidBody,
};
use ph2d_platformer::{MAX_WALL_SAMPLES, odd_samples, wall_offsets};

// ── A GEOMETRIA DA CENA ──────────────────────────────────────────────────────
// Um bloco alto com um topo, e um personagem a cair rente à face dele.
/// A face vertical do bloco (o `x` em que a parede está).
const WALL_FACE: f32 = 0.5;
/// O TOPO do bloco — o lábio que esta wave existe para achar.
const LIP_Y: f32 = 3.5;
/// A meia-altura da cápsula do personagem (`half_height + radius`).
const HALF_H: f32 = 0.5;
/// A meia-largura dela.
const HALF_W: f32 = 0.2;
/// A altura de flutuação da fixture (a mesma do `platform_scene`).
const FLOAT_HEIGHT: f32 = 0.9;

/// Uma cena com chão, um bloco alto, e o personagem a cair ao lado dele.
///
/// ⚠️ **O personagem nasce ACIMA do lábio**, que é o gesto real: quem agarra uma
/// beirada está a cair por fora dela.
fn ledge_scene_at(start_y: f32, gap: f32, wall_armed: bool) -> (SimWorld, PhysicsBridge, Entity) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 40.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    // O bloco: face em `WALL_FACE`, topo em `LIP_Y`.
    let half_y = (LIP_Y - 0.5) * 0.5;
    sim.world_mut().spawn((
        Name::new("Block"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 1.0,
                half_y,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(WALL_FACE + 1.0, 0.5 + half_y)),
    ));
    let player = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: 0.2,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT_HEIGHT,
                wall_slide_speed: if wall_armed { 1.0 } else { 0.0 },
                wall_jump_height: if wall_armed { 2.0 } else { 0.0 },
                wall_grab_stamina: if wall_armed { 2.0 } else { 0.0 },
                ..PlatformPlayer::default()
            },
            // Rente à face, do lado de fora.
            Transform::from_translation(Vec2::new(WALL_FACE - HALF_W - gap, start_y)),
        ))
        .id();
    (sim, PhysicsBridge::new(), player)
}

/// A cena de sempre, rente à face (2 cm de folga).
fn ledge_scene(start_y: f32, wall_armed: bool) -> (SimWorld, PhysicsBridge, Entity) {
    ledge_scene_at(start_y, 0.02, wall_armed)
}

fn pose(sim: &SimWorld) -> (f32, f32) {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Player" {
            found = Some((t.translation.x, t.translation.y));
        }
    }
    found.expect("o player tem de existir")
}

/// **O que acontece HOJE quando se cai rente a um patamar** — e **quanto tempo a
/// janela de agarre dura**, que é o número que a dimensiona.
///
/// ⚠️ **A primeira fixture deste arquivo NÃO CONTINHA O FENÓMENO**, e vale
/// escrever porquê: ela largava o personagem *acima* do lábio a empurrar para
/// dentro, e o que ele fez foi **voar por cima do bloco** (medido: `x` 0,70 →
/// 8,66 em 90 tiques, sem nunca encostar na face). Empurrar contra uma parede
/// que ainda não está ao lado do corpo é deslocamento livre — o gesto real
/// começa com o corpo **já rente à face**.
///
/// ⚠️ **E a coluna `off` é o CONTROLO:** se as duas caíssem no mesmo tempo, a
/// parede armada estaria inerte e a tabela inteira falaria sobre uma lei que
/// nunca correu.
#[test]
#[ignore = "sonda de medicao"]
fn measure_what_happens_at_a_ledge_today() {
    println!("\n== cair rente a um patamar (labio em y = {LIP_Y:.2}) ==");
    println!("  parede   v ao cruzar   tiques na janela 0,1 / 0,2 / 0,4 m   y final");
    for armed in [false, true] {
        // O topo do corpo nasce ACIMA do lábio, com a maior parte já rente à
        // face — é assim que se chega a uma beirada.
        let (mut sim, mut bridge, player) = ledge_scene(LIP_Y - 0.1, armed);
        let mut prev = pose(&sim).1;
        let mut crossed = f32::NAN;
        let mut inside = [0_u32; 3];
        for t in 1..=300_u64 {
            bridge.set_player_input(
                player,
                PlayerInput {
                    drive: 1.0,
                    grab: true,
                    ..PlayerInput::default()
                },
            );
            bridge.dispatch(&mut sim, true, t);
            let y = pose(&sim).1;
            let top = y + HALF_H;
            if crossed.is_nan() && top < LIP_Y {
                crossed = (y - prev) * 60.0;
            }
            for (slot, g) in [0.1_f32, 0.2, 0.4].into_iter().enumerate() {
                if top < LIP_Y && top >= LIP_Y - g {
                    inside[slot] += 1;
                }
            }
            prev = y;
        }
        println!(
            "  {:>6}   {crossed:>9.3} m/s   {:>5} / {:>5} / {:>5}   {:>7.4}",
            if armed { "ON" } else { "off" },
            inside[0],
            inside[1],
            inside[2],
            pose(&sim).1
        );
    }
    println!("  (nenhuma das colunas PARA no labio — e' isso que a wave existe para mudar)");
}

/// **Quantos tiques a janela dura em QUEDA LIVRE** — o número que decide se o
/// agarre pode exigir empurrar contra a parede.
///
/// ⚠️ **A pergunta nasceu da tabela acima:** com a parede DESLIGADA o
/// personagem que empurra fica **preso pelo atrito** (mediu-se: ele nunca
/// desce), então um agarre que exigisse empurrar seria **inalcançável** em todo
/// personagem que não tenha `wall_slide_speed` autorado. A alternativa é apanhar
/// quem cai **sem** empurrar — e aí o que decide é a velocidade da queda.
#[test]
#[ignore = "sonda de medicao"]
fn measure_how_fast_the_window_goes_by_in_free_fall() {
    println!("\n== a janela em QUEDA LIVRE (sem empurrar), por altura de largada ==");
    println!("  queda   v ao cruzar   tiques na janela 0,1 / 0,2 / 0,4 / 0,8 m");
    for drop in [0.2_f32, 0.5, 1.0, 2.0, 4.0] {
        let (mut sim, mut bridge, player) = ledge_scene(LIP_Y - HALF_H + drop, false);
        let mut prev = pose(&sim).1;
        let mut crossed = f32::NAN;
        let mut inside = [0_u32; 4];
        for t in 1..=300_u64 {
            // ⚠️ **SEM `drive`** — é a metade que a tabela anterior não cobre.
            bridge.set_player_input(player, PlayerInput::default());
            bridge.dispatch(&mut sim, true, t);
            let y = pose(&sim).1;
            let top = y + HALF_H;
            if crossed.is_nan() && top < LIP_Y {
                crossed = (y - prev) * 60.0;
            }
            for (slot, g) in [0.1_f32, 0.2, 0.4, 0.8].into_iter().enumerate() {
                if top < LIP_Y && top >= LIP_Y - g {
                    inside[slot] += 1;
                }
            }
            prev = y;
        }
        println!(
            "  {drop:>5.1}   {crossed:>9.3} m/s   {:>5} / {:>5} / {:>5} / {:>5}",
            inside[0], inside[1], inside[2], inside[3]
        );
    }
    println!("  (uma janela de 1 ou 2 tiques e' uma moeda ao ar, nao uma assistencia)");
}

/// **A BEIRADA ARMADA, pela porta do produto** — onde ele para, e onde acaba se
/// pedir para subir.
///
/// ⚠️ **É o número que escolhe os defaults da cena de smoke**, e as duas colunas
/// são uma o controlo da outra: pendurado, o TOPO do corpo tem de assentar no
/// lábio; depois do mantle, o corpo tem de estar **em pé em cima** do patamar —
/// e a diferença entre os dois `y` é o gesto inteiro.
#[test]
#[ignore = "sonda de medicao"]
fn measure_the_ledge_armed() {
    println!("\n== a beirada armada (labio em {LIP_Y:.2}) ==");
    println!("  grab   speed   y pendurado   topo-labio   y apos o mantle   de pe'?");
    for grab in [0.2_f32, 0.4, 0.8] {
        for speed in [2.0_f32, 4.0] {
            // ⚠️ **Nasce AFASTADO da face**, e é a correção que a tabela de
            // cima obriga: rente a ela o atrito segura-o acima do lábio e a
            // janela nunca chega. Cair é o gesto; empurrar é a intenção.
            // ⚠️ **DENTRO da janela desde o primeiro tique**, e é a 3.ª correção
            // desta fixture: largado ACIMA do lábio com o dedo a empurrar, ele
            // **aterra em cima do bloco** (medido — o topo salta de 4,55 para
            // 4,90 no tique 10 e ele atravessa o patamar a andar). O que esta
            // tabela mede é a LEI, não a aproximação.
            let (mut sim, mut bridge, player) = ledge_scene_at(LIP_Y - HALF_H - 0.15, 0.30, false);
            if let Some(mut p) = sim.world_mut().get_mut::<PlatformPlayer>(player) {
                p.ledge_grab = grab;
                p.ledge_speed = speed;
            }
            let mut tick = 0_u64;
            // 90 tiques a empurrar contra a face: apanha e assenta.
            for _ in 0..90 {
                bridge.set_player_input(
                    player,
                    PlayerInput {
                        drive: 1.0,
                        ..PlayerInput::default()
                    },
                );
                tick += 1;
                bridge.dispatch(&mut sim, true, tick);
            }
            let hung = pose(&sim).1;
            // E agora o pedido de subir: empurrar + um TOQUE no pulo.
            //
            // ⚠️ **Um toque, e não o botão preso**, e o motivo é um fato do
            // produto que a sonda expôs: enquanto a beirada age, o pulo é
            // mascarado na ENTRADA, então o `was_held` fica falso — e no tique
            // em que a subida acaba, um botão ainda preso lê como aperto novo e
            // o personagem SALTA do patamar. É a mesma propriedade do nado, e
            // aqui ela é a fixture que teria mentido.
            //
            // ⚠️ **E o dedo LARGA a direção depois do toque**, pela mesma razão:
            // com ele preso o personagem sobe, chega ao patamar e **continua a
            // andar** até cair pelo outro lado (medido: o bloco acaba em
            // `x = 2,5` e ele saía por lá). O que esta coluna mede é o fim do
            // gesto, não o que o jogador faz a seguir.
            for i in 0..180 {
                bridge.set_player_input(
                    player,
                    PlayerInput {
                        drive: if i < 2 { 1.0 } else { 0.0 },
                        jump: i < 2,
                        ..PlayerInput::default()
                    },
                );
                tick += 1;
                bridge.dispatch(&mut sim, true, tick);
            }
            let (x, y) = pose(&sim);
            // De pé em cima: o centro à altura de repouso e já do lado de dentro.
            let standing = (y - (LIP_Y + FLOAT_HEIGHT)).abs() < 0.25 && x > WALL_FACE + HALF_W;
            println!(
                "  {grab:>4.1}   {speed:>5.1}   {hung:>11.4}   {:>+10.4}   {y:>15.4}   {}",
                hung + HALF_H - LIP_Y,
                if standing { "SIM" } else { "nao" }
            );
        }
    }
}

/// **A subida ANDA à velocidade AUTORADA** — a régua do cancelamento de
/// gravidade.
///
/// ⚠️ **Esta sonda existe porque o pendurar NÃO consegue medir esse termo:** o
/// servo re-corrige em todo tique (o alvo é `lip_rise / dt`), então a gravidade
/// de um tique é absorvida pelo tique seguinte e o assentamento move
/// **0,1 mm** com o termo removido — abaixo de qualquer oráculo honesto. A
/// subida é outra coisa: o alvo dela é uma **CONSTANTE** (`speed`), então a
/// gravidade não cancelada sai do número que o artista escreveu e **fica** lá.
#[test]
#[ignore = "sonda de medicao"]
fn measure_whether_the_climb_walks_at_the_authored_speed() {
    println!("\n== a subida contra o numero autorado ==");
    println!("  speed   subiu em 10 tiques   esperado   razao");
    for speed in [2.0_f32, 3.0, 4.0] {
        let (mut sim, mut bridge, player) = ledge_scene_at(LIP_Y - HALF_H - 0.15, 0.30, false);
        if let Some(mut p) = sim.world_mut().get_mut::<PlatformPlayer>(player) {
            p.ledge_grab = 0.4;
            p.ledge_speed = speed;
        }
        let mut tick = 0_u64;
        for _ in 0..90 {
            bridge.set_player_input(
                player,
                PlayerInput {
                    drive: 1.0,
                    ..PlayerInput::default()
                },
            );
            tick += 1;
            bridge.dispatch(&mut sim, true, tick);
        }
        let before = pose(&sim).1;
        // O toque que pede a subida, e depois DEZ tiques a subir.
        for i in 0..12 {
            bridge.set_player_input(
                player,
                PlayerInput {
                    drive: if i < 2 { 1.0 } else { 0.0 },
                    jump: i < 2,
                    ..PlayerInput::default()
                },
            );
            tick += 1;
            bridge.dispatch(&mut sim, true, tick);
        }
        let rose = pose(&sim).1 - before;
        // 10 dos 12 tiques sobem (os dois primeiros ainda pedem).
        let want = speed * 10.0 / 60.0;
        println!(
            "  {speed:>5.1}   {rose:>17.4}   {want:>8.4}   {:>5.3}",
            rose / want
        );
    }
}

// ── O MUNDO CRU, para as perguntas de SENSOR ─────────────────────────────────

/// O mesmo bloco, num mundo sem ECS — para castar os raios à mão.
fn raw() -> (PhysicsWorld, ph2d_physics::RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    let half_y = (LIP_Y - 0.5) * 0.5;
    w.add_static_cuboid(WALL_FACE + 1.0, 0.5 + half_y, 1.0, half_y);
    let body = w.spawn_body(BodyDesc {
        body_type: ph2d_physics::RigidBodyType::Dynamic,
        x: WALL_FACE - HALF_W - 0.02,
        y: 0.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Capsule {
            half_height: 0.3,
            radius: 0.2,
        },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: true,
        offset: [0.0, 0.0],
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
    });
    // ⚠️ **UM passo antes de castar, e não é higiene:** o cast lê o BVH que o
    // último `step` deixou (o contrato que o `bridge::player` documenta no
    // topo). Sem ele TODO raio desta sonda devolve `None` — e a primeira
    // corrida deste arquivo devolveu exatamente isso, uma tabela inteira de
    // *"nao achou"* que se lê como *"não há beirada nenhuma"*.
    w.step();
    (w, body)
}

/// Põe o corpo com o CENTRO em `y` e devolve o handle.
fn place(w: &mut PhysicsWorld, body: ph2d_physics::RigidBodyHandle, y: f32) {
    w.set_body_pose(body, WALL_FACE - HALF_W - 0.02, y, 0.0, true);
}

/// **O flanco já sabe que a parede ACABA?**
///
/// ⚠️ **É esta a pergunta que decide o tamanho da wave.** O [`WallProbe`] é um
/// perfil de N alturas: se *"a amostra de cima não vê nada e a de baixo vê"*
/// bastasse para dizer onde está o lábio, não haveria sensor novo — haveria uma
/// leitura nova do que já se casta.
#[test]
#[ignore = "sonda de medicao"]
fn measure_what_the_flank_says_at_a_ledge() {
    println!("\n== o perfil do FLANCO, por altura do corpo (labio em {LIP_Y:.2}) ==");
    println!("  (o . e' um raio que nao viu nada; o # viu parede)");
    for n in [3_usize, 5, 9] {
        println!(
            "\n  {n} amostras — passo de {:.3} m",
            2.0 * HALF_H / (n - 1) as f32
        );
        println!("     topo do corpo   perfil (de baixo p/ cima)   banda cega");
        for k in -4..=4 {
            let top = LIP_Y + 0.1 * k as f32;
            let cy = top - HALF_H;
            let (mut w, body) = raw();
            place(&mut w, body, cy);
            let offs = wall_offsets(HALF_H, n, 1.0);
            let m = odd_samples(n, MAX_WALL_SAMPLES);
            // As alturas saem embaralhadas (o meio primeiro); ordena para ler.
            let mut sorted: Vec<f32> = offs[..m].to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mut row = String::new();
            for off in &sorted {
                let hit = w
                    .cast_ray(
                        [WALL_FACE - HALF_W - 0.02, cy + off],
                        [1.0, 0.0],
                        HALF_W + 0.1,
                        Some(body),
                        0,
                    )
                    .is_some();
                row.push(if hit { '#' } else { '.' });
            }
            println!(
                "     {top:>13.2}   {row:<26}   {:.3} m",
                2.0 * HALF_H / (m - 1) as f32
            );
        }
    }
}

/// **Onde está o lábio, e com que precisão** — o raio para BAIXO, à frente.
///
/// ⚠️ **A origem é o alvo do mantle**, e não um ponto qualquer: o `x` em que se
/// pergunta *"a que altura é o chão do outro lado?"* é o mesmo `x` em que o
/// corpo vai pousar. Uma segunda escolha de `x` daria um sensor que mede uma
/// beirada e um mantle que aterra noutra.
#[test]
#[ignore = "sonda de medicao"]
fn measure_the_down_ray_in_front() {
    println!("\n== o raio para BAIXO, a' frente (labio real em {LIP_Y:.3}) ==");
    println!("  topo do corpo   janela   achou?   labio medido   erro");
    for grab in [0.2_f32, 0.4, 0.8] {
        for k in [-2_i32, 0, 2, 4, 8] {
            let top = LIP_Y - 0.1 * k as f32;
            let cy = top - HALF_H;
            let (mut w, body) = raw();
            place(&mut w, body, cy);
            // O `x` do alvo: o centro atravessa a face e mais meia-largura.
            let x = WALL_FACE + HALF_W;
            let from = [x, top + grab];
            let hit = w.cast_ray(from, [0.0, -1.0], grab, Some(body), 0);
            match hit {
                Some(h) => {
                    let lip = from[1] - h.distance;
                    println!(
                        "  {top:>13.3}   {grab:>6.2}   {:>6}   {lip:>12.4}   {:+.4}",
                        "SIM",
                        lip - LIP_Y
                    );
                }
                None => println!("  {top:>13.3}   {grab:>6.2}   {:>6}", "nao"),
            }
        }
        println!();
    }
}

/// **O corpo CABE lá em cima?** — a varredura que o mantle vai querer.
#[test]
#[ignore = "sonda de medicao"]
fn measure_whether_the_body_fits_on_top() {
    println!("\n== varrer o corpo ate' ao destino do mantle ==");
    println!("  destino (x, y)        coube?");
    for extra in [0.0_f32, 0.3, 0.6] {
        let (mut w, body) = raw();
        let top = LIP_Y;
        let cy = top - HALF_H;
        place(&mut w, body, cy);
        // Sobe primeiro, atravessa depois — o L do mantle.
        let rise = LIP_Y + FLOAT_HEIGHT - cy + extra;
        let across = WALL_FACE + HALF_W - (WALL_FACE - HALF_W - 0.02);
        let up = w.sweep_body(body, [0.0, 1.0], rise, 0);
        place(&mut w, body, cy + rise);
        let side = w.sweep_body(body, [1.0, 0.0], across, 0);
        println!(
            "  subir {rise:>5.3}, andar {across:>5.3}   subida {:<6} travessia {}",
            if up.is_none() { "livre" } else { "BATE" },
            if side.is_none() { "livre" } else { "BATE" }
        );
    }
}

/// **O sensor, tique a tique** — a sonda de atribuição.
#[test]
#[ignore = "sonda de medicao"]
fn measure_the_ledge_probe_tick_by_tick() {
    use ph2d_physics_ecs::{ProbeKind, ProbeState};
    println!("\n== o raio da beirada, tique a tique (labio {LIP_Y:.2}, grab 0,4) ==");
    let (mut sim, mut bridge, player) = ledge_scene_at(LIP_Y - HALF_H - 0.15, 0.30, false);
    if let Some(mut p) = sim.world_mut().get_mut::<PlatformPlayer>(player) {
        p.ledge_grab = 0.4;
        p.ledge_speed = 3.0;
    }
    for t in 1..=140_u64 {
        bridge.set_player_input(
            player,
            PlayerInput {
                drive: 1.0,
                // Depois de assentar, pede para SUBIR.
                jump: t > 20,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
        let (x, y) = pose(&sim);
        let marks: Vec<_> = bridge
            .player_probe_marks()
            .iter()
            .filter(|m| m.kind == ProbeKind::Ledge)
            .map(|m| m.state)
            .collect();
        let tag = if marks.contains(&ProbeState::Hit) {
            "HIT"
        } else if marks.contains(&ProbeState::Clear) {
            "clear"
        } else if marks.is_empty() {
            "-nenhum-"
        } else {
            "idle"
        };
        if t <= 25 || t % 5 == 0 {
            println!("  t={t:>3}  x={x:>6.3}  topo={:>6.3}  {tag}", y + HALF_H);
        }
    }
}
