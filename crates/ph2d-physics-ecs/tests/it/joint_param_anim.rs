//! **Um parâmetro de joint KEYFRAMADO é uma entrada por TICK** (W-JointAnim) —
//! os gates que provam que ele chega ao solver tick a tick, e que um scrub
//! reproduz o play que ele replaya.
//!
//! O irmão exato do `child_bodies`/`kinematic`: a auditoria do W4b já tinha
//! achado que uma pose dirigida por curva quebra o invariante *"o mundo é função
//! do tick, dado o repouso"* — e a cura foi o [`SceneAtTick`], perguntado uma vez
//! por tick. Um `motor_target` keyframado é a MESMA frase dita de outro número, e
//! o mesmo laço tinha de aprender a empurrá-lo.
//!
//! ⚠️ **Os oráculos aqui não conhecem função nenhuma desta wave.** São dois
//! fatos que o artista pode assistir: *tocar 60 ticks de uma vez dá a mesma cena
//! que tocá-los um a um*, e *arrastar a régua para trás e voltar dá a cena que
//! estava lá*. É por isso que eles pegam a doença sem saber onde ela mora.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, FrozenScene, JointKind, MotorMode, PhysicsBridge,
    PhysicsJoint, RigidBody, SceneAtTick,
};

/// O alvo do servo NO TICK `t` — a "curva" que o documento teria.
///
/// ⚠️ **Duas propriedades, e as duas são o que dá dentes aos gates.**
///
/// 1. Ela **varia por tick** na primeira metade. Uma constante seria satisfeita
///    por um push que nunca acontece, e o gate de catch-up ficaria verde sobre a
///    doença que ele existe para pegar.
/// 2. Ela tem um **DEGRAU no tick 61**, e o platô depois dele vale o MESMO que no
///    tick 120. Essa coincidência é a única forma de o memo do
///    `drive_joint_params` mentir sem ser notado: depois de um seed do ring, o
///    valor autorado do tick seguinte coincide com o que o memo guarda, o push
///    decide *"nada mudou"*, e o solver corre com o número do checkpoint. Sem o
///    degrau as duas mutações do push de replay SOBREVIVEM — medido.
fn target_at(tick: u64) -> f32 {
    /// O platô que o tick 61 e o tick 120 partilham.
    const PLATEAU: f32 = -0.30;
    if tick <= 60 {
        -1.2 * (tick as f32 / 60.0)
    } else {
        PLATEAU
    }
}

/// A metade que a shell implementa de verdade (`apply_from_doc`): põe a cena no
/// estado que ela tem no tick `t`. Aqui isso é uma linha, porque o único canal
/// animado é o alvo do servo.
struct Servo {
    joint: Entity,
}

impl SceneAtTick for Servo {
    fn put(&mut self, sim: &mut SimWorld, tick: u64) -> bool {
        if let Some(mut j) = sim.world_mut().get_mut::<PhysicsJoint>(self.joint) {
            j.motor_target = target_at(tick);
        }
        true
    }
}

/// Gancho estático, prancha de 1 m pendurada pela ponta esquerda, dobradiça com
/// **motor de POSIÇÃO** — um servo. O alvo é o número que se anima, e a
/// consequência dele é um ângulo que se mede.
fn arm() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Hook"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.05 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 6.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Plank"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.5,
                half_y: 0.1,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.5, 5.0)),
    ));
    let joint = sim
        .world_mut()
        .spawn((
            Name::new("Servo"),
            PhysicsJoint {
                body_a: stable_name_id("Hook"),
                body_b: stable_name_id("Plank"),
                kind: JointKind::Pin,
                motor_enabled: true,
                motor_mode: MotorMode::Position,
                motor_target: target_at(0),
                motor_max_force: 500.0,
                ..PhysicsJoint::default()
            },
            Transform::from_translation(Vec2::new(0.0, 5.0)),
        ))
        .id();
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    (sim, joint)
}

fn plank_pose(sim: &mut SimWorld) -> [f32; 3] {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    let e = q
        .iter(sim.world())
        .find(|(_, n)| n.as_str() == "Plank")
        .map(|(e, _)| e)
        .expect("plank");
    let t = sim.world().get::<Transform>(e).expect("transform");
    [t.translation.x, t.translation.y, t.rotation]
}

/// Tocar até `target`, um tick por dispatch — o play do relógio real —
/// **gravando a pose de CADA tick**.
///
/// ⚠️ A trajetória inteira, e não o endpoint: um servo é um sistema AMORTECIDO,
/// então ele converge para o alvo e ESQUECE por onde veio. Um gate que compara
/// só o fim fica verde sobre um replay que correu com os parâmetros errados —
/// medido: as duas mutações do push de replay sobreviveram a exactamente esse
/// gate antes de ele virar isto. É a mesma lição que o W1.5 já tinha pago.
fn play_recording(target: u64) -> (Vec<[f32; 3]>, PhysicsBridge, SimWorld, Entity) {
    let (mut sim, joint) = arm();
    let mut bridge = PhysicsBridge::new();
    let mut scene = Servo { joint };
    let mut poses = vec![plank_pose(&mut sim)];
    for t in 1..=target {
        bridge.dispatch_with_scene(&mut sim, true, t, &mut scene);
        poses.push(plank_pose(&mut sim));
    }
    (poses, bridge, sim, joint)
}

/// **Tocar 60 ticks de uma vez dá a mesma cena que tocá-los um a um.**
///
/// ⚠️ Este é o gate RED-first da wave. Sem o `drive_joint_params` no laço para a
/// frente, o `reconcile_joints` roda uma vez por DISPATCH: o dispatch de 60
/// ticks aplica a TODOS eles o alvo que o componente tinha no começo do quadro,
/// enquanto o tick-a-tick recebe um alvo novo por passo. Medido com o passe
/// desligado, os dois ângulos ficam a mais de um décimo de radiano.
#[test]
fn a_catch_up_plays_the_same_scene_as_a_tick_by_tick_play() {
    let (poses, _, _, _) = play_recording(60);
    let one_by_one = poses[60];

    let (mut sim, joint) = arm();
    let mut bridge = PhysicsBridge::new();
    let mut scene = Servo { joint };
    bridge.dispatch_with_scene(&mut sim, true, 60, &mut scene);
    let at_once = plank_pose(&mut sim);

    assert_eq!(
        one_by_one, at_once,
        "60 ticks de uma vez contra 60 dispatches: {one_by_one:?} contra {at_once:?}"
    );
    // CONTROLE: o alvo animado de fato move a prancha. Sem isto os dois lados
    // poderiam concordar sobre uma cena parada.
    let (mut still, joint) = arm();
    let mut b2 = PhysicsBridge::new();
    let mut frozen = FrozenScene;
    let _ = joint;
    for t in 1..=60 {
        b2.dispatch_with_scene(&mut still, true, t, &mut frozen);
    }
    let held = plank_pose(&mut still);
    assert!(
        (one_by_one[2] - held[2]).abs() > 0.2,
        "a rampa do servo tem de mover a prancha: {one_by_one:?} contra o controle {held:?}"
    );
}

/// **Arrastar a régua para trás devolve a cena que estava naquele tick.**
///
/// ⚠️ O replay não roda o `reconcile_joints` — ele é um laço de `step` puro —,
/// então sem o push por tick um scrub correria com os parâmetros que o solver
/// segurava quando a viagem começou. E com o ring habilitado há uma segunda
/// metade: o seed devolve ao solver o `data` do CHECKPOINT, que descreve outro
/// tick; é por isso que a primeira volta do laço empurra com `force`.
///
/// **Compara NO tick scrubado**, contra a pose que o play de fato mostrou ali —
/// nunca depois de voltar ao fim, onde o servo já convergiu e a diferença
/// evaporou.
#[test]
fn a_scrub_shows_the_pose_the_play_showed_at_that_tick() {
    let (poses, mut bridge, mut sim, joint) = play_recording(120);
    let mut scene = Servo { joint };

    // Alvos que caem ENTRE âncoras do ring (`STRIDE = 10`), para o laço de
    // replay de fato rodar: um alvo em cima de uma âncora replaya zero passos e
    // não exercita o push nenhum.
    for &t in &[97u64, 64, 43] {
        bridge.dispatch_with_scene(&mut sim, false, t, &mut scene);
        let got = plank_pose(&mut sim);
        let want = poses[t as usize];
        let d = (got[2] - want[2]).abs();
        assert!(
            d < 1e-4,
            "scrub para o tick {t} tem de mostrar a pose do play: {want:?} contra {got:?} (delta {d})"
        );
    }
}

/// **E um scrub que acerta uma ÂNCORA do ring em cheio também.**
///
/// O caso em que o laço de replay roda ZERO vezes: o solver fica com os
/// parâmetros do checkpoint (que são os certos) e o memo com os do tick de onde
/// viemos (que não são). O tick seguinte comparia contra a resposta errada.
///
/// A cena tem um PLATÔ de propósito — o alvo é o mesmo em 120 e em 61 —, que é
/// exactamente a coincidência que faz o memo mentir sem ser notado.
#[test]
fn a_scrub_onto_a_cached_tick_leaves_the_solver_and_the_memo_agreeing() {
    let (poses, mut bridge, mut sim, joint) = play_recording(120);
    let mut scene = Servo { joint };

    bridge.dispatch_with_scene(&mut sim, false, 60, &mut scene);
    // E um tick para a frente: é aqui que um memo mentiroso vira uma pose errada.
    bridge.dispatch_with_scene(&mut sim, true, 61, &mut scene);
    let got = plank_pose(&mut sim);
    let want = poses[61];
    let d = (got[2] - want[2]).abs();
    assert!(
        d < 1e-4,
        "o tick depois de um seed exato tem de reproduzir o play: {want:?} contra {got:?} (delta {d})"
    );
}

/// **Um parâmetro que anda a cada tick NÃO joga fora o cache de scrub.**
///
/// A consequência de projeto da wave, e a que só um relógio enxerga: um edit de
/// parâmetro ia pela rota de remover-e-inserir, e QUALQUER item nas listas
/// estruturais limpa o ring. Com um alvo keyframado isso acontecia em todo tick
/// de play, então o scrub bit-exato do W1.5 morria pelo resto da cena — e nada
/// parecia quebrado, só lento.
///
/// ⚠️ **Quem o salva AQUI é o `drive_joint_params`, não a rota de retune do
/// reconcile** — e a diferença foi medida, não suposta: neutralizar a rota do
/// reconcile deixa este gate VERDE, porque o push por tick já escreveu o número
/// e atualizou o memo antes de o reconcile do quadro seguinte olhar. A rota do
/// reconcile é a do SLIDER do Inspector, e tem gate próprio logo abaixo.
///
/// O oráculo é o número de `step` que o replay cobra, que é exatamente o que o
/// ring existe para cortar.
#[test]
fn a_keyframed_parameter_does_not_throw_away_the_scrub_cache() {
    let (_, mut bridge, mut sim, joint) = play_recording(120);
    let mut scene = Servo { joint };
    let (cached, _) = bridge.ring_stats();
    assert!(
        cached > 0,
        "o ring tem de ter guardado alguma coisa depois de 120 ticks com um param animado"
    );

    let before = bridge.steps_taken();
    bridge.dispatch_with_scene(&mut sim, false, 110, &mut scene);
    let replayed = bridge.steps_taken() - before;
    assert!(
        replayed <= 10,
        "um scrub de 120 para 110 tem de replayar o resto do STRIDE, não a cena inteira: {replayed} passos"
    );
}

/// **Afinar um joint na mão também não joga fora o cache de scrub** — a rota do
/// SLIDER, irmã da de cima.
///
/// Aqui não há curva nenhuma: o artista arrasta o `Motor Target` na §12 e o
/// `reconcile_joints` do quadro seguinte é quem vê a diferença. Antes desta wave
/// ele respondia remover-e-inserir, o que limpa o ring — então **arrastar um
/// slider matava o scrub bit-exato a cada quadro**, muito antes de existir um
/// parâmetro keyframado.
///
/// ⚠️ Este é o gate que a mutação M5 (voltar o edit de parâmetro a ser
/// estrutural) sangra; a irmã acima sobrevive a ela, e foi assim que se soube
/// que as duas rotas precisavam de um gate cada
/// ([[feedback_layered_defenses_need_per_layer_gates]]).
#[test]
fn tuning_a_joint_by_hand_does_not_throw_away_the_scrub_cache() {
    let (mut sim, joint) = arm();
    let mut bridge = PhysicsBridge::new();
    let mut frozen = FrozenScene;
    for t in 1..=120 {
        bridge.dispatch_with_scene(&mut sim, true, t, &mut frozen);
    }
    assert!(bridge.ring_stats().0 > 0, "o ring tem de estar cheio");

    // O gesto do Inspector: escrever o campo no componente, e nada mais.
    sim.world_mut()
        .get_mut::<PhysicsJoint>(joint)
        .expect("servo")
        .motor_target = -0.9;
    bridge.dispatch_with_scene(&mut sim, true, 121, &mut frozen);

    let before = bridge.steps_taken();
    bridge.dispatch_with_scene(&mut sim, false, 111, &mut frozen);
    let replayed = bridge.steps_taken() - before;
    assert!(
        replayed <= 10,
        "afinar um joint na mão não pode custar a cena inteira num scrub: {replayed} passos"
    );
}
