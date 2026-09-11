//! **A static body is where the artist put it — including during PLAY.**
//!
//! `settle` (the paused branch) makes every rapier body track its authored
//! `Transform`, and `drive_kinematic`'s own comment declared the coverage
//! complete: *"a wall that has been moved by hand is caught by `settle`, while
//! paused"*. During play nothing caught it — the solver does not own a static
//! body's pose (`readback` skips it), the scene does not push it per tick
//! (`drive_kinematic` skips it), and `settle` only runs at `Ordering::Equal`
//! while paused. So dragging a wall with the clock running moved the DRAWING
//! and left the collider behind: a phantom collider, which is exactly what the
//! artist reported.
//!
//! The law: **a static body's pose has exactly one author, the authored
//! `Transform`** — the solver never writes it, so there is no second writer to
//! disagree with, and the pose can therefore be honoured on every dispatch
//! rather than only on the paused ones.
//!
//! The oracle is BEHAVIOURAL, because "phantom collider" is a claim about where
//! the collider *acts*, not about a number we could read from the same place
//! that wrote it.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

/// A ball resting on a static slab. The slab is what gets dragged.
fn resting_scene() -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let slab = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 4.0,
                    half_y: 0.5,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    let ball = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 1.5)),
        ))
        .id();
    (sim, slab, ball)
}

fn y_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).unwrap().translation.y
}

fn set_y(sim: &mut SimWorld, e: Entity, y: f32) {
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        t.translation.y = y;
    }
}

fn run(bridge: &mut PhysicsBridge, sim: &mut SimWorld, from: u64, ticks: u64) -> u64 {
    let mut tick = from;
    for _ in 0..ticks {
        tick += 1;
        bridge.dispatch(sim, true, tick);
    }
    tick
}

/// **The gate.** Drag the slab DOWN a metre with the clock running; the ball has
/// to come down and rest on the new top, because the collider is where the
/// drawing is.
///
/// ⚠️ The direction is not arbitrary and the first version of this fixture got
/// it wrong: dragging the slab UP *through* the ball jumps its whole span past
/// the ball (span `[-0.5, 0.5]` → `[1.5, 2.5]`, ball bottom at 0.499), so there
/// is no overlap to resolve, the ball is left with nothing under it and falls —
/// a correct outcome that the oracle called a failure. Downwards there is only
/// one physical answer: the ball follows the floor it is standing on.
///
/// Written RED-first: before the fix the ball hung at its old resting height
/// while the slab was drawn a metre below it.
///
/// # ⛔⛔ A premissa «genuinamente ADORMECIDA» não era AFIRMADA — e ela deixou de ser
/// barata em 2026-08-29
///
/// O doc dizia *«assenta até a bola estar genuinamente ADORMECIDA — o estado onde um bug de
/// acordar se esconde»*, e o teste corria 180 tiques e conferia `rest ≈ 0,8`: uma ALTURA, que
/// uma bola bem acordada também tem. ⚠️ A subida da `rapier2d` 0.35 baixou o
/// `sleep_linear_threshold` de `0,4` para **`0,05`** (`8×` mais difícil de adormecer), e esta
/// fixtura dá **3 s**. *Uma premissa que o gate não afirma é uma premissa que o gate perde em
/// silêncio.*
///
/// ⛔⛔ **E a pose BIT-A-BIT parada — o critério do [`resting_pose`] — NÃO PROVA o sono
/// AQUI, e isso mediu-se antes de se escrever a linha.** Naquele ficheiro a fixtura é uma
/// pilha de 12 corpos, que **treme** enquanto está acordada; aqui é **uma bola sobre uma
/// laje**, e ela fica exactamente parada muito antes de adormecer. Medido em 2026-08-30
/// (cena replicada em `ph2d-physics`, onde `RigidBody::is_sleeping` é alcançável):
///
/// | tique | pose | `|v|` | `is_sleeping` |
/// |---|---|---|---|
/// | 32 | **última alteração** | `0` | `false` |
/// | 33..144 | idêntica ao bit | `0` | **`false`** |
/// | **145** | idêntica ao bit | `0` | **`true`** ← adormece aqui |
/// | 180 (o fim do assentamento) | idêntica ao bit | `0` | `true` |
///
/// ⇒ *A bola fica bit-a-bit parada no tique 32 e só dorme no 145: o critério sozinho é
/// satisfeito **113 tiques antes** da premissa.* Uma fixtura de 60 tiques passaria por ele
/// com a bola bem acordada.
///
/// ⭐ **O que fecha é a DURAÇÃO, e ela é derivada do próprio produto:** a `rapier` adormece
/// um corpo que fique sob os limiares durante [`PhysicsSettings::time_until_sleep`]. Logo,
/// «parada há mais tiques do que esse tempo» **é** o critério dela, lido de fora. Aqui são
/// `180 − 32 = 148` tiques de imobilidade contra os `2,0 s = 120` exigidos — e o gate lê o
/// `time_until_sleep` do produto, de modo que subi-lo (ou encurtar o assentamento) põe este
/// gate VERMELHO em vez de o deixar perder a premissa em silêncio.
///
/// ⚠️ **A conta é CONSERVADORA:** o relógio da `rapier` arranca quando a VELOCIDADE desce
/// sob o limiar (tique ~25 aqui), antes de a pose congelar, então exigir `148 ≥ 120` pede
/// mais imobilidade do que ela pede. *Uma inexactidão que subestima é folga.*
///
/// ⚠️ **E o limiar não é a alavanca deste gate**, porque a velocidade em repouso é
/// **exactamente `0`**: qualquer `sleep_linear_threshold > 0` adormece-a, e só a sentinela
/// `0,0` — que significa *«proibido dormir»* — a impediria. A alavanca real é o par
/// «janela de assentamento × `time_until_sleep`», que é a que 2026-08-29 mexeu.
#[test]
fn a_static_body_dragged_during_play_carries_its_collider() {
    let (mut sim, slab, ball) = resting_scene();
    let mut bridge = PhysicsBridge::default();

    // Let it settle so the ball is genuinely ASLEEP on the slab — the state an
    // artist actually reaches before reaching for the wall, and the state a
    // wake-up bug hides in. ⚠️ The premise is ASSERTED, not assumed: see the doc.
    const SETTLE_TICKS: u64 = 180;
    let mut last_move = 0u64;
    let mut prev = bridge.body_pose(ball);
    for t in 1..=SETTLE_TICKS {
        bridge.dispatch(&mut sim, true, t);
        let now = bridge.body_pose(ball);
        if now != prev {
            last_move = t;
            prev = now;
        }
    }
    let tick = SETTLE_TICKS;
    // rapier's own sleep rule, read from outside: under the thresholds for
    // `time_until_sleep`. A pose that has not moved a single bit is under any
    // positive threshold, so "still for longer than that" implies asleep.
    let needed = (bridge.settings().time_until_sleep * 60.0).ceil() as u64;
    let still = SETTLE_TICKS - last_move;
    assert!(
        still >= needed,
        "fixture: the ball is NOT asleep when the drag happens. It last moved at tick \
         {last_move}, so it has been motionless for {still} ticks, and rapier needs \
         {needed} (time_until_sleep = {} s) before it sleeps. This gate is about a WAKE-UP \
         bug and an awake ball cannot expose one — lengthen SETTLE_TICKS",
        bridge.settings().time_until_sleep
    );
    // And the half that keeps "motionless" from being vacuous: the ball got there by
    // FALLING, so the scene is not one where nothing ever happened.
    assert!(
        last_move > 0,
        "fixture: the ball never moved at all — it was born at rest, so 'motionless' \
         says nothing about settling"
    );

    let rest = y_of(&sim, ball);
    assert!(
        (rest - 0.8).abs() < 0.01,
        "fixture: the ball should rest on top of the slab, got y = {rest:.4}"
    );

    // The gesture: the artist drags the slab down while the clock runs.
    set_y(&mut sim, slab, -1.0);
    run(&mut bridge, &mut sim, tick, 120);

    let carried = y_of(&sim, ball);
    assert!(
        (carried - (rest - 1.0)).abs() < 0.05,
        "the slab moved to y = -1.0 (top at -0.5) and the ball is at \
         y = {carried:.4} instead of {:.4}: the collider is not where the \
         drawing is",
        rest - 1.0
    );
    // And the collider itself reports the authored pose, not the spawn one.
    let (_, slab_y, _) = bridge.body_pose(slab).expect("slab has a body");
    assert!(
        (slab_y - -1.0).abs() < 1e-6,
        "the rapier slab is at y = {slab_y:.6}, authored y = -1.0"
    );
}

/// The CONTROL, and it is the half that keeps the fix from becoming a bug: a
/// static body nobody touched must not be teleported (and re-woken) every
/// dispatch. `settle` earned this guard while paused — teleporting
/// unconditionally zeroes velocity, and doing it per frame during play would
/// re-wake every sleeping stack forever.
#[test]
fn an_untouched_static_body_is_not_disturbed_during_play() {
    let (mut sim, slab, ball) = resting_scene();
    let mut bridge = PhysicsBridge::default();
    let tick = run(&mut bridge, &mut sim, 0, 180);
    let rest = y_of(&sim, ball);
    let before = bridge.body_pose(slab).expect("slab has a body");

    run(&mut bridge, &mut sim, tick, 120);

    assert_eq!(
        bridge.body_pose(slab).expect("slab has a body"),
        before,
        "an untouched static body moved"
    );
    let after = y_of(&sim, ball);
    assert!(
        (after - rest).abs() < 1e-4,
        "the ball drifted from {rest:.6} to {after:.6} with nothing touched"
    );
}

/// A DYNAMIC body is not covered by this law, and the distinction is the whole
/// reason it is safe: the solver owns a dynamic pose, so honouring an authored
/// `Transform` during play would be a second author — and the one that ran last
/// would win in silence, which is the frame-order bug W4 documented.
///
/// Here the ball is falling and its `Transform` is overwritten every dispatch by
/// `readback`. Writing to it mid-play must not teleport the body.
#[test]
fn a_dynamic_body_is_not_settled_during_play() {
    let mut sim = SimWorld::new();
    let ball = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 10.0)),
        ))
        .id();
    let mut bridge = PhysicsBridge::default();
    let tick = run(&mut bridge, &mut sim, 0, 30);
    let falling = y_of(&sim, ball);

    // Someone writes the Transform of a body the solver owns. The next dispatch
    // must ignore it: the fall continues from where the SOLVER left it.
    set_y(&mut sim, ball, 100.0);
    run(&mut bridge, &mut sim, tick, 1);
    let next = y_of(&sim, ball);
    assert!(
        next < falling,
        "a dynamic body was teleported by an authored Transform mid-play \
         ({falling:.4} -> {next:.4})"
    );
}
