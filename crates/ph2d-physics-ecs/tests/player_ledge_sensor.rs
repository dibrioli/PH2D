//! **O SENSOR da beirada** (`W-LedgeSensor`) — a posição e a extensão, pela
//! porta do produto.
//!
//! ⚠️ **Irmão de `player_ledge.rs`, e o corte é por ASSUNTO:** aquele mede *a
//! lei* (agarrar, pendurar, subir) com o sensor na forma de antes desta wave —
//! um raio, janela `±grab` — e por isso é a **regressão-guarda**. Este mede *a
//! forma do sensor*: o que muda quando o artista move o `y` ou lhe dá extensão.
//!
//! ⚠️ **O oráculo é sempre o mesmo do irmão: a POSE.** Um gate sobre
//! `hanging == true` ficaria verde com o corpo a atravessar o patamar.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    RigidBody,
};

/// A face vertical do bloco.
const WALL_FACE: f32 = 0.5;
/// O TOPO do bloco — o lábio.
const LIP_Y: f32 = 3.5;
/// A meia-altura da cápsula (`half_height + radius`).
const HALF_H: f32 = 0.5;
/// A meia-largura dela.
const HALF_W: f32 = 0.2;
/// A altura de flutuação.
const FLOAT_HEIGHT: f32 = 0.9;

/// **A forma do sensor, como esta suíte a autora.**
#[derive(Copy, Clone)]
struct Sensor {
    grab: f32,
    reach_y: f32,
    span: f32,
}

/// Chão, um bloco de meia-largura `half_x`, e o personagem ao lado dele.
///
/// ⚠️ **A LARGURA do bloco é parte da fixture**, e é ela que separa esta suíte
/// da irmã: um patamar ESTREITO é onde um raio único cai no vazio para além da
/// borda de trás, que é o defeito que a extensão existe para cobrir.
fn scene(centre_y: f32, gap: f32, s: Sensor, half_x: f32) -> (SimWorld, PhysicsBridge, Entity) {
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
    let half_y = (LIP_Y - 0.5) * 0.5;
    sim.world_mut().spawn((
        Name::new("Block"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid { half_x, half_y },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(WALL_FACE + half_x, 0.5 + half_y)),
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
                ledge_grab: s.grab,
                ledge_reach_y: s.reach_y,
                ledge_span: s.span,
                ledge_speed: 3.0,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(WALL_FACE - HALF_W - gap, centre_y)),
        ))
        .id();
    (sim, PhysicsBridge::new(), player)
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

/// Empurra contra a face e devolve a pose depois de `n` tiques.
fn press(sim: &mut SimWorld, bridge: &mut PhysicsBridge, player: Entity, n: u64) -> (f32, f32) {
    let mut tick = 0_u64;
    for _ in 0..n {
        bridge.set_player_input(
            player,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(sim, true, tick);
    }
    pose(sim)
}

/// **Está pendurado?** — o topo do corpo no lábio, e PARADO.
fn hanging(sim: &SimWorld) -> bool {
    (pose(sim).1 + HALF_H - LIP_Y).abs() < 0.08
}

/// **⚠️ O `reach_y` POSSUI a janela — e a prova é o par.**
///
/// A metade positiva sozinha não prova nada: um `reach_y` enorme apanharia
/// tudo. O que separa *"a janela é este número"* de *"a janela é grande"* é a
/// recusa — o MESMO corpo, a MESMA queda, com a janela curta demais para
/// alcançar o lábio, tem de **cair**.
#[test]
fn the_window_is_the_height_the_artist_wrote() {
    // O corpo começa com o topo 0,45 m abaixo do lábio.
    let drop = LIP_Y - HALF_H - 0.45;
    let wide = Sensor {
        grab: 0.6,
        reach_y: 0.6,
        span: 0.0,
    };
    let (mut sim, mut bridge, player) = scene(drop, 0.05, wide, 1.0);
    let _ = press(&mut sim, &mut bridge, player, 60);
    assert!(
        hanging(&sim),
        "com a janela em 0,60 um labio 0,45 acima da cabeca tem de ser apanhado: \
         topo em {:.3} contra o labio em {LIP_Y:.2}",
        pose(&sim).1 + HALF_H
    );

    let short = Sensor {
        reach_y: 0.2,
        ..wide
    };
    let (mut sim, mut bridge, player) = scene(drop, 0.05, short, 1.0);
    let end = press(&mut sim, &mut bridge, player, 60);
    assert!(
        !hanging(&sim),
        "e com a janela em 0,20 o MESMO labio esta' fora de alcance — ele tem de \
         cair: topo em {:.3}",
        end.1 + HALF_H
    );
}

/// **⚠️ O GATE DA WAVE: a extensão acha o lábio que um raio único ERRA.**
///
/// Num patamar ESTREITO o raio de antes cai no vazio para além da borda de
/// trás — e o defeito não é *"apanha tarde"*, é **não apanha**. O leque cobre o
/// mesmo `x` com amostras mais perto do corpo, e a mais próxima que bate **é a
/// beirada**.
///
/// ⚠️ **A metade de CONTROLE é a que dá sentido à outra:** sem ela, um gate que
/// só afirmasse *"com extensão ele agarra"* passaria numa fixture onde o raio
/// único também agarrava.
#[test]
fn a_span_catches_the_lip_a_single_ray_misses() {
    // ⚠️ **A aritmética é a fixture, e a primeira versão dela estava ERRADA** —
    // o CONTROLE reprovou porque o raio nu ainda pousava DENTRO do bloco.
    // Contas: a borda do corpo fica em `x = 0,45`; o raio nu pousa em
    // `0,45 + grab = 1,05`; o bloco vai de `0,50` a `0,50 + 2·half_x`. Para o
    // raio ERRAR é preciso `half_x < 0,275`, e para a amostra mais perto do
    // leque (`0,45 + grab − span/2 = 0,75`) ACERTAR é preciso `half_x ≥ 0,125`.
    // **0,20** fica no meio: o bloco vai a `0,90` e o raio nu passa 15 cm dele.
    let narrow = 0.2;
    let single = Sensor {
        grab: 0.6,
        reach_y: 0.6,
        span: 0.0,
    };
    let drop = LIP_Y - HALF_H - 0.45;
    let (mut sim, mut bridge, player) = scene(drop, 0.05, single, narrow);
    let _ = press(&mut sim, &mut bridge, player, 60);
    assert!(
        !hanging(&sim),
        "CONTROLE: com um raio unico este patamar estreito nao pode ser apanhado \
         — se ele for, a fixture nao contem o fenomeno"
    );

    let fanned = Sensor {
        span: 0.6,
        ..single
    };
    let (mut sim, mut bridge, player) = scene(drop, 0.05, fanned, narrow);
    let _ = press(&mut sim, &mut bridge, player, 60);
    assert!(
        hanging(&sim),
        "com extensao 0,60 o leque alcanca o topo do bloco: topo do corpo em \
         {:.3} contra o labio em {LIP_Y:.2}",
        pose(&sim).1 + HALF_H
    );
}

/// **⚠️ Uma amostra DENTRO da geometria recusa o leque INTEIRO.**
///
/// A rejeição de *"a parede continua acima da minha cabeça"* era **grátis**
/// enquanto o sensor era um PONTO (a origem cai dentro e o cast devolve
/// `distance == 0`). Com extensão ela deixa de ser, e é feita à mão: se a parede
/// continua acima da cabeça junto ao corpo, não há beirada a apanhar por mais
/// livre que esteja uma amostra lá à frente.
///
/// ⚠️ **A PRIMEIRA versão deste gate NÃO PODIA FALHAR pelo motivo que alegava**,
/// e a mutação provou-o: ela punha o corpo fundo contra um bloco largo, onde
/// **todas** as amostras nascem dentro da geometria — e aí trocar a recusa por
/// um `continue` dá exactamente o mesmo resultado (o leque acaba vazio de
/// qualquer maneira). A lei só é observável onde uma amostra está **dentro** e
/// outra **acha alguma coisa**.
///
/// A fixture é essa: uma parede FINA que continua acima da cabeça, e uma
/// prateleira mais baixa **atrás** dela, ao alcance da janela. Sem a recusa ele
/// agarra a prateleira **através da parede**.
#[test]
fn a_sample_inside_the_wall_refuses_the_whole_fan() {
    let fanned = Sensor {
        grab: 0.6,
        reach_y: 0.6,
        span: 0.6,
    };
    // ⚠️ **E a SEGUNDA versão também não o continha**, por outro motivo: com o
    // corpo àquela altura ele estava sobre o CHÃO, e `ledge_probe_wanted` nem
    // casta o sensor de quem está apoiado. A fixture tem de o pôr no AR.
    //
    // Geometria: parede FINA (x 0,50 → 0,90) que sobe até **6,00**, muito acima
    // da cabeça; prateleira (x 1,00 → 1,60) com o topo em **3,20**. Corpo com o
    // topo em 3,00 ⇒ a origem do leque fica em 3,60, **dentro da parede**, e a
    // janela `[2,40 · 3,60]` alcança a prateleira. As amostras pousam em
    // x = 0,75 · 0,90 (dentro da parede) · 1,05 · 1,20 · 1,35 (sobre a
    // prateleira).
    const SHELF_TOP: f32 = 3.2;
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
    sim.world_mut().spawn((
        Name::new("Wall"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.2,
                half_y: 2.75,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.7, 3.25)),
    ));
    sim.world_mut().spawn((
        Name::new("Shelf"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.3,
                half_y: 1.35,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(1.3, SHELF_TOP - 1.35)),
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
                ledge_grab: fanned.grab,
                ledge_reach_y: fanned.reach_y,
                ledge_span: fanned.span,
                ledge_speed: 3.0,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(WALL_FACE - HALF_W - 0.05, 3.0 - HALF_H)),
        ))
        .id();
    let mut bridge = PhysicsBridge::new();
    let end = press(&mut sim, &mut bridge, player, 40);
    assert!(
        (end.1 + HALF_H - SHELF_TOP).abs() > 0.15,
        "a parede continua acima da cabeca — a prateleira ATRAS dela nao pode ser \
         agarrada: topo do corpo em {:.3} contra o labio dela em {SHELF_TOP:.2}",
        end.1 + HALF_H
    );
}

/// **A extensão NÃO muda o mundo aprovado** — o neutro é o raio de antes.
///
/// ⚠️ É a redução literal que torna o degrau de schema barato: com `span = 0` o
/// leque tem **uma** amostra, na posição exacta do raio da `W-Ledge`, e a pose
/// final tem de ser a MESMA.
#[test]
fn a_zero_span_is_the_single_ray_of_before() {
    let base = Sensor {
        grab: 0.6,
        reach_y: 0.6,
        span: 0.0,
    };
    let drop = LIP_Y - HALF_H - 0.45;
    let (mut sim, mut bridge, player) = scene(drop, 0.05, base, 1.0);
    let a = press(&mut sim, &mut bridge, player, 60);
    // O MESMO, com a extensão explicitamente em zero por outro caminho (um
    // valor negativo é aparado para zero na porta).
    let (mut sim, mut bridge, player) = scene(drop, 0.05, Sensor { span: -1.0, ..base }, 1.0);
    let b = press(&mut sim, &mut bridge, player, 60);
    assert!(
        (a.0 - b.0).abs() < 1.0e-4 && (a.1 - b.1).abs() < 1.0e-4,
        "uma extensao aparada em zero tem de dar a MESMA pose: {a:?} contra {b:?}"
    );
}

/// **⚠️ VENCE O ACERTO MAIS PERTO DO CORPO**, e o oráculo é onde ele ATERRA.
///
/// Num patamar LARGO todas as amostras batem, e aí a pergunta *"qual venceu?"*
/// deixa de ser sobre achar e passa a ser sobre o `across` — o alvo do mantle.
/// A mais próxima é a beirada; a mais distante poria o corpo `span` metros para
/// dentro, num ponto que ninguém pediu.
///
/// ⚠️ **Este gate existe porque uma MUTAÇÃO sobreviveu aos outros quatro:**
/// trocar *primeiro acerto* por *último* não muda nada quando só uma amostra
/// bate, que é o caso das fixtures estreitas — a lei só é observável onde
/// várias batem.
#[test]
fn the_nearest_hit_wins_and_that_is_where_he_lands() {
    let fanned = Sensor {
        grab: 0.6,
        reach_y: 0.6,
        span: 0.6,
    };
    let (mut sim, mut bridge, player) = scene(LIP_Y - HALF_H - 0.15, 0.30, fanned, 1.0);
    let mut tick = 0_u64;
    let mut run = |sim: &mut SimWorld, b: &mut PhysicsBridge, n: u64, input: PlayerInput| {
        for _ in 0..n {
            b.set_player_input(player, input);
            tick += 1;
            b.dispatch(sim, true, tick);
        }
    };
    run(
        &mut sim,
        &mut bridge,
        90,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    run(
        &mut sim,
        &mut bridge,
        2,
        PlayerInput {
            drive: 1.0,
            jump: true,
            ..PlayerInput::default()
        },
    );
    run(&mut sim, &mut bridge, 180, PlayerInput::default());
    let (x, y) = pose(&sim);
    assert!(
        (y - (LIP_Y + FLOAT_HEIGHT)).abs() < 0.1,
        "premissa: ele TEM de acabar de pe' no patamar — y = {y:.4}"
    );
    // ⚠️ **A borda de dentro pousa no `x` da amostra VENCEDORA.** Com a mais
    // próxima, o corpo inteiro entra e para logo depois da face; com a mais
    // distante ele iria `span` metros mais fundo.
    let inner = x - HALF_W;
    assert!(
        inner > WALL_FACE - 1.0e-3 && inner < WALL_FACE + fanned.span,
        "a borda de dentro tem de pousar entre a face ({WALL_FACE:.2}) e um span \
         adiante ({:.2}) — o acerto mais LONGE poria {inner:.4} para la' disso",
        WALL_FACE + fanned.span
    );
}
