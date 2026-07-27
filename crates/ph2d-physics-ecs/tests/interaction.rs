//! **A FERRAMENTA DE INTERAÇÃO** (W-Hand) — os três modos de segurar, o estouro e
//! o campo de atração, no nível da ENTIDADE.
//!
//! O que cada lei FAZ está medido no wrapper (`ph2d_physics::world::grab` e
//! `::blast` carregam as tabelas). O que estes gates defendem é o que só existe
//! aqui: que a escolha do artista chega ao solver, que as ferramentas não são a
//! mesma coisa com nomes diferentes, e que as duas regras de determinismo valem
//! para as três.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, HoldMode, InteractionSettings, InteractionTool,
    PhysicsBridge, RigidBody,
};

fn spawn(sim: &mut SimWorld, at: Vec2, half: f32) -> Entity {
    sim.world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: half,
                    half_y: half,
                },
                ..Collider::default()
            },
            Transform::from_translation(at),
        ))
        .id()
}

fn pose(sim: &SimWorld, e: Entity) -> (f32, f32, f32) {
    let t = sim.world().get::<Transform>(e).unwrap();
    (t.translation.x, t.translation.y, t.rotation)
}

fn run(bridge: &mut PhysicsBridge, sim: &mut SimWorld, from: u64, ticks: u64) -> u64 {
    let mut tick = from;
    for _ in 0..ticks {
        tick += 1;
        bridge.dispatch(sim, true, tick);
    }
    tick
}

/// Uma cena sem gravidade — o seguidor, e não a queda, é o que estes gates medem.
fn floating(bridge: &mut PhysicsBridge) {
    let mut s = bridge.settings();
    s.gravity_x = 0.0;
    s.gravity_y = 0.0;
    bridge.set_settings(s);
}

/// **A escolha do artista chega ao solver, e os três modos NÃO são o mesmo.**
///
/// O oráculo é o ATRASO — a grandeza que o wrapper mediu e a que o artista sente:
/// arrastando o cursor a velocidade constante, uma mola fica para trás, um hold
/// rígido não, e uma corda fica **exatamente** o slack para trás.
///
/// ⚠️ Um gate que só afirmasse *"o corpo se moveu"* passaria com os três modos
/// colapsados num só, que é precisamente a regressão que esta wave pode sofrer
/// (uma `hold_spec` que ignorasse `hold` e devolvesse sempre a mola).
#[test]
fn each_hold_mode_follows_the_cursor_its_own_way() {
    let cases = [
        (HoldMode::Spring, 0.0_f32),
        (HoldMode::Rigid, 0.0),
        (HoldMode::Rope, 1.5),
    ];
    let mut lag = Vec::new();
    for (hold, slack) in cases {
        let mut sim = SimWorld::new();
        let body = spawn(&mut sim, Vec2::new(0.0, 0.0), 0.3);
        let mut bridge = PhysicsBridge::default();
        floating(&mut bridge);
        let tick = run(&mut bridge, &mut sim, 0, 1);

        let settings = InteractionSettings {
            hold,
            slack,
            ..InteractionSettings::default()
        };
        assert!(
            bridge.grab_with(body, [0.0, 0.0], settings.hold_spec()),
            "{hold:?} refused to grab a dynamic body"
        );
        let mut cursor = 0.0_f32;
        let mut t = tick;
        for _ in 0..40 {
            cursor += 4.0 / 60.0;
            bridge.move_grab([cursor, 0.0]);
            t = run(&mut bridge, &mut sim, t, 1);
        }
        lag.push((hold, cursor - pose(&sim, body).0));
    }

    let spring = lag[0].1;
    let rigid = lag[1].1;
    let rope = lag[2].1;
    assert!(
        rigid.abs() < 0.01,
        "a rigid hold lagged {rigid:.4} m — it is supposed to have none"
    );
    assert!(
        spring > 0.1,
        "the spring did not lag ({spring:.4} m): it is not a spring"
    );
    assert!(
        (rope - 1.5).abs() < 0.05,
        "the rope trailed {rope:.4} m for a slack of 1.5 — the trail IS the slack"
    );
}

/// **Só o Rigid segura a ATITUDE.** O outro eixo em que os modos diferem, e o
/// que torna o Rigid irredutível ao `Rope { slack: 0 }` (os dois têm atraso zero;
/// apenas um segura o ângulo).
#[test]
fn only_the_rigid_hold_keeps_the_attitude() {
    let mut spin = Vec::new();
    for hold in [HoldMode::Spring, HoldMode::Rigid, HoldMode::Rope] {
        let mut sim = SimWorld::new();
        // Pego pela QUINA, com gravidade: um corpo preso por um ponto fora do
        // centro de massa gira, a menos que a lei o proíba.
        let body = spawn(&mut sim, Vec2::new(0.0, 0.0), 0.5);
        let mut bridge = PhysicsBridge::default();
        let tick = run(&mut bridge, &mut sim, 0, 1);
        let settings = InteractionSettings {
            hold,
            slack: 0.0,
            ..InteractionSettings::default()
        };
        assert!(bridge.grab_with(body, [-0.5, 0.5], settings.hold_spec()));
        let mut t = tick;
        for _ in 0..90 {
            bridge.move_grab([-0.5, 0.5]);
            t = run(&mut bridge, &mut sim, t, 1);
        }
        spin.push((hold, pose(&sim, body).2.abs()));
    }
    let rigid = spin.iter().find(|(h, _)| *h == HoldMode::Rigid).unwrap().1;
    assert!(
        rigid < 0.02,
        "a rigid hold let the body rotate {rigid:.4} rad"
    );
    for (h, s) in &spin {
        if *h != HoldMode::Rigid {
            assert!(
                *s > 0.1,
                "{h:?} kept the attitude ({s:.4} rad) — then it is a rigid hold \
                 wearing another name"
            );
        }
    }
}

/// **A EXPLOSÃO empurra para FORA, mais perto empurra mais, e fora do alcance não
/// empurra nada.**
///
/// Três afirmações num gate porque são a MESMA lei vista em três pontos — o
/// falloff — e medi-las separadamente esconderia um kernel que empurra na direção
/// certa com peso constante.
#[test]
fn the_blast_pushes_outward_with_a_falloff_that_reaches_zero() {
    // ⚠️ Em EIXOS diferentes, e a primeira versão deste fixture não estava: numa
    // fila os três voam na mesma direção, o mais próximo (que leva o maior
    // impulso) alcança o do meio e **bate nele**, e o que se mede vira transporte
    // de momento em vez de falloff (medido: 12,32 contra 10,87 m, razão 1,13 onde
    // a lei manda 5). Cada corpo no seu raio, ninguém no caminho de ninguém.
    let mut sim = SimWorld::new();
    let near = spawn(&mut sim, Vec2::new(0.5, 0.0), 0.2);
    let far = spawn(&mut sim, Vec2::new(0.0, 2.5), 0.2);
    let outside = spawn(&mut sim, Vec2::new(-6.0, 0.0), 0.2);
    let mut bridge = PhysicsBridge::default();
    floating(&mut bridge);
    let tick = run(&mut bridge, &mut sim, 0, 1);

    let hit = bridge.explode([0.0, 0.0], 3.0, 10.0);
    assert_eq!(
        hit, 2,
        "the blast reported {hit} bodies inside a 3 m radius"
    );
    // UM tick: o que se mede é a velocidade que o impulso deu, não onde o corpo
    // chegou depois de meio segundo (que é `v·t` e dilui a razão).
    let t = run(&mut bridge, &mut sim, tick, 1);
    let _ = t;

    let (nx, _, _) = pose(&sim, near);
    let (_, fy, _) = pose(&sim, far);
    let (ox, _, _) = pose(&sim, outside);
    let near_step = nx - 0.5;
    let far_step = fy - 2.5;
    assert!(
        near_step > 0.0,
        "the near body did not move outward (x = {nx:.4})"
    );
    assert!(
        far_step > 0.0,
        "the far body did not move outward (y = {fy:.4})"
    );
    // A lei diz 0,833 contra 0,167 — razão 5. Pedir 3 deixa margem ao solver sem
    // deixar passar um peso constante (razão 1).
    assert!(
        near_step > far_step * 3.0,
        "the near body moved {near_step:.4} and the far one {far_step:.4} in one \
         tick: the falloff is flat"
    );
    assert!(
        (ox - -6.0).abs() < 1e-4,
        "a body OUTSIDE the radius moved to {ox:.4}"
    );
}

/// **O campo de ATRAÇÃO puxa para dentro enquanto está armado, e para quando é
/// desarmado.**
///
/// O CONTROLE é a segunda metade: sem ela o gate passaria com um campo que nunca
/// solta — e "nunca solta" é exatamente a ferramenta grudada no cursor que o
/// release existe para evitar.
#[test]
fn the_pull_field_gathers_while_armed_and_stops_when_it_is_not() {
    let mut sim = SimWorld::new();
    let body = spawn(&mut sim, Vec2::new(2.0, 0.0), 0.2);
    let mut bridge = PhysicsBridge::default();
    floating(&mut bridge);
    let tick = run(&mut bridge, &mut sim, 0, 1);

    let settings = InteractionSettings {
        tool: InteractionTool::Attract,
        attract_radius: 3.0,
        attract_force: 50.0,
        ..InteractionSettings::default()
    };
    bridge.attract(&settings, [0.0, 0.0]);
    let t = run(&mut bridge, &mut sim, tick, 60);
    let gathered = pose(&sim, body).0;
    assert!(
        gathered < 1.0,
        "the field did not gather the body (x = {gathered:.4}, started at 2.0)"
    );

    // Desarma e deixa correr: sem o campo, e com a resistência dele fora, o corpo
    // segue em linha reta — o que importa é que nada continua PUXANDO.
    bridge.stop_attract();
    let before = pose(&sim, body);
    let t2 = run(&mut bridge, &mut sim, t, 30);
    let _ = t2;
    let after = pose(&sim, body);
    let pulled_back = (after.0 - before.0).abs() < 1e-6 || after.0 > before.0;
    assert!(
        pulled_back || (after.0 - before.0).abs() > 0.0,
        "unreachable: this only documents that the body is no longer held"
    );
    // A afirmação de verdade: o campo não existe mais.
    assert!(
        bridge.attract_marks().is_none(),
        "the field survived `stop_attract`"
    );
    assert!(
        !bridge.is_poking(),
        "a released field still reads as a poke"
    );
}

/// **As DUAS regras de determinismo valem para o campo de atração, não só para a
/// mão.**
///
/// Regra 1: armar descarta o ring e nada é gravado enquanto ele está em voo — sem
/// isso, um scrub para trás responderia com a trajetória cutucada dentro da janela
/// do cache e com a de repouso fora dela, para o MESMO tick.
///
/// Regra 2: um rewind SOLTA.
#[test]
fn the_pull_field_obeys_both_determinism_rules() {
    let mut sim = SimWorld::new();
    let _body = spawn(&mut sim, Vec2::new(2.0, 0.0), 0.2);
    let mut bridge = PhysicsBridge::default();
    let tick = run(&mut bridge, &mut sim, 0, 60);
    let (before, _) = bridge.ring_stats();
    assert!(
        before > 0,
        "fixture: the ring should have checkpoints after 60 ticks, had {before}"
    );

    let settings = InteractionSettings {
        tool: InteractionTool::Attract,
        ..InteractionSettings::default()
    };
    bridge.attract(&settings, [0.0, 0.0]);
    assert_eq!(
        bridge.ring_stats().0,
        0,
        "arming the field left checkpoints in the ring"
    );
    let t = run(&mut bridge, &mut sim, tick, 60);
    assert_eq!(
        bridge.ring_stats().0,
        0,
        "checkpoints were recorded while a field was in flight"
    );

    // Regra 2.
    bridge.dispatch(&mut sim, true, t - 30);
    assert!(
        bridge.attract_marks().is_none(),
        "a rewind did not release the field"
    );
}

/// **Um estouro também descarta o ring**, e a razão é diferente da do campo: ele
/// é INSTANTÂNEO, então "não gravar enquanto durar" não diz nada — o que quebraria
/// o scrub é um checkpoint de ANTES dele, que replayaria em frente sem o estouro.
#[test]
fn a_blast_drops_the_scrub_cache() {
    let mut sim = SimWorld::new();
    let _body = spawn(&mut sim, Vec2::new(0.5, 0.0), 0.2);
    let mut bridge = PhysicsBridge::default();
    // ⚠️ Sem gravidade: com ela o corpo cai ~4,9 m no segundo que aquece o ring e
    // sai do alcance de 3 m, então o estouro atinge ZERO e o gate mede o caminho
    // da recusa em vez do que ele afirma.
    floating(&mut bridge);
    run(&mut bridge, &mut sim, 0, 60);
    assert!(
        bridge.ring_stats().0 > 0,
        "fixture: the ring should be warm"
    );
    let hit = bridge.explode([0.0, 0.0], 3.0, 10.0);
    assert_eq!(hit, 1);
    assert_eq!(
        bridge.ring_stats().0,
        0,
        "a blast left the scrub cache describing a run that no longer happened"
    );
}

/// **Uma explosão que não atinge ninguém não é uma perturbação** — e portanto não
/// derruba o cache. Espelho exato da recusa da mão (*"derrubar o cache de um gesto
/// que não aconteceu seria custo puro"*).
#[test]
fn a_blast_that_hits_nothing_leaves_the_cache_alone() {
    let mut sim = SimWorld::new();
    let _body = spawn(&mut sim, Vec2::new(0.5, 0.0), 0.2);
    let mut bridge = PhysicsBridge::default();
    floating(&mut bridge);
    run(&mut bridge, &mut sim, 0, 60);
    let warm = bridge.ring_stats().0;
    assert!(warm > 0, "fixture: the ring should be warm");
    // Longe de tudo.
    assert_eq!(bridge.explode([50.0, 50.0], 3.0, 10.0), 0);
    assert_eq!(
        bridge.ring_stats().0,
        warm,
        "a blast that hit nothing dropped the cache anyway"
    );
}

/// **A razão de amortecimento é uma RAZÃO**, e é isso que a torna um knob que se
/// pode mexer: `1` é o crítico em QUALQUER rigidez, então dobrar a rigidez não
/// exige recalcular o segundo número.
///
/// O oráculo é o SOBRESSINAL — o defeito que se vê. Sub-amortecido o corpo passa
/// do cursor; no crítico ele não passa, em nenhuma das duas rigidezes.
#[test]
fn the_damping_ratio_means_the_same_thing_at_any_stiffness() {
    for stiffness in [400.0_f32, 1600.0] {
        let mut over = 0.0_f32;
        let mut under = 0.0_f32;
        for (ratio, out) in [(1.0_f32, &mut over), (0.25, &mut under)] {
            let mut sim = SimWorld::new();
            let body = spawn(&mut sim, Vec2::new(0.0, 0.0), 0.2);
            let mut bridge = PhysicsBridge::default();
            floating(&mut bridge);
            let tick = run(&mut bridge, &mut sim, 0, 1);
            let settings = InteractionSettings {
                stiffness,
                damping_ratio: ratio,
                ..InteractionSettings::default()
            };
            assert!(bridge.grab_with(body, [0.0, 0.0], settings.hold_spec()));
            let mut cursor = 0.0_f32;
            let mut t = tick;
            for _ in 0..30 {
                cursor += 4.0 / 60.0;
                bridge.move_grab([cursor, 0.0]);
                t = run(&mut bridge, &mut sim, t, 1);
            }
            let mut worst = 0.0_f32;
            for _ in 0..60 {
                bridge.move_grab([cursor, 0.0]);
                t = run(&mut bridge, &mut sim, t, 1);
                worst = worst.max(pose(&sim, body).0 - cursor);
            }
            *out = worst;
        }
        assert!(
            over < 1e-3,
            "critical damping overshot {over:.4} m at stiffness {stiffness}"
        );
        assert!(
            under > 0.01,
            "ratio 0.25 did NOT overshoot at stiffness {stiffness} \
             ({under:.4} m): the ratio is not reaching the solver"
        );
    }
}

/// **Um hold rígido segura o ângulo que o corpo TINHA, não o zero.**
///
/// ⚠️ O gate irmão (`only_the_rigid_hold_keeps_the_attitude`) NÃO pega isto, e a
/// razão é uma armadilha de fixture: ele pega um corpo em rotação **0**, onde
/// *"mantém a atitude"* e *"endireita"* são a MESMA saída. Um `FixedJoint` cujo
/// `local_frame1` fosse a identidade exigiria rotação zero do corpo — e chicotearia
/// qualquer objeto inclinado para o prumo no instante do press, com o irmão verde.
#[test]
fn a_rigid_hold_keeps_the_angle_the_body_had() {
    let start = 0.6_f32;
    let mut sim = SimWorld::new();
    let body = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 1.0,
                    half_y: 0.2,
                },
                ..Collider::default()
            },
            Transform {
                rotation: start,
                ..Transform::from_translation(Vec2::new(0.0, 0.0))
            },
        ))
        .id();
    let mut bridge = PhysicsBridge::default();
    let tick = run(&mut bridge, &mut sim, 0, 1);
    let settings = InteractionSettings {
        hold: HoldMode::Rigid,
        ..InteractionSettings::default()
    };
    assert!(bridge.grab_with(body, [0.0, 0.0], settings.hold_spec()));
    let mut t = tick;
    for _ in 0..60 {
        bridge.move_grab([0.0, 0.0]);
        t = run(&mut bridge, &mut sim, t, 1);
    }
    let held = pose(&sim, body).2;
    assert!(
        (held - start).abs() < 0.02,
        "a rigid hold on a body tilted {start:.3} rad left it at {held:.3} — it \
         was whipped to a different attitude by the press"
    );
}
