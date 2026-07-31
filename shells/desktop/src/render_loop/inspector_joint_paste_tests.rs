//! **Colar as propriedades de um joint** (W-JointCopy) — a metade de shell.
//!
//! Irmão de `inspector_joint_tests.rs`, separado dele por assunto (e porque o
//! arquivo lá bate o cap de 600 do shell com pouco mais). O que a `ph2d-
//! physics-ecs` já prova é a LEI — quais campos são uma propriedade
//! (`joint::properties`, 4 mutações). O que se prova aqui é a **porta**: que ela
//! escreve pela fila e pelo clamp de sempre, e que ela atravessa em silêncio uma
//! entidade que não é joint — a condição que torna o fan-out sobre a seleção
//! seguro, porque uma seleção mista tem sprites dentro.

use ph2d_core::Vec2;
use ph2d_ecs::scene::{EditorCommandQueue, apply_editor_commands};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{JointKind, MotorMode, PhysicsJoint};

use super::inspector_joint::paste_joint_properties;
use super::inspector_joint_tests::{registry, two_bodies};

/// Um joint com a afinação toda longe do default — a FONTE.
fn tuned(body_a: u64, body_b: u64) -> PhysicsJoint {
    PhysicsJoint {
        body_a,
        body_b,
        kind: JointKind::Spring,
        rest_length: 2.75,
        stiffness: 400.0,
        damping: 12.0,
        break_enabled: true,
        break_force: 900.0,
        motor_mode: MotorMode::Position,
        motor_target: 1.25,
        collide_connected: true,
        local_a: [0.5, 0.5],
        local_b: [0.25, 0.25],
        anchored: true,
        ..PhysicsJoint::default()
    }
}

/// Uma cena com DOIS joints entre os mesmos corpos: uma fonte afinada e um alvo
/// no default, com âncoras próprias. Dois joints sobre um par é legítimo (o
/// motivo de um joint ser ENTIDADE, W3) e é a fixture mais curta que tem uma
/// fonte e um alvo de verdade.
fn source_and_target() -> (SimWorld, Entity, Entity) {
    let (mut sim, hook, plank) = two_bodies(true);
    let a = ph2d_ecs::stable_name_id("Hook");
    let b = ph2d_ecs::stable_name_id("Plank");
    let src = sim
        .world_mut()
        .spawn((
            tuned(a, b),
            Transform::from_translation(Vec2::new(0.0, 5.5)),
        ))
        .id();
    let dst = sim
        .world_mut()
        .spawn((
            PhysicsJoint {
                body_a: a,
                body_b: b,
                local_a: [-1.0, -2.0],
                local_b: [-3.0, -4.0],
                anchored: true,
                active: false,
                ..PhysicsJoint::default()
            },
            Transform::from_translation(Vec2::new(0.0, 5.5)),
        ))
        .id();
    let _ = (hook, plank);
    (sim, src, dst)
}

/// Aplica o paste pela porta real e drena a fila, como o laço da shell faz.
fn paste(sim: &mut SimWorld, dst: Entity, source: &PhysicsJoint) {
    let reg = registry();
    let queue = EditorCommandQueue::default();
    paste_joint_properties(sim, dst.to_bits(), source, &queue, &reg);
    apply_editor_commands(sim.world_mut(), &queue, &reg).expect("commit");
}

/// **A afinação chega ao componente VIVO** — e a identidade, as âncoras e o
/// interruptor do experimento continuam os do alvo.
///
/// O oráculo é o componente depois do commit, não o retorno da função pura: é
/// aqui que se prova que a porta ENFILEIRA e que a fila é drenada, que foi
/// exatamente o defeito *"às vezes funciona"* que o laço da §12 já pagou uma vez.
#[test]
fn pasting_lands_on_the_live_component() {
    let (mut sim, src, dst) = source_and_target();
    let source = *sim.world().get::<PhysicsJoint>(src).expect("fonte");
    let before = *sim.world().get::<PhysicsJoint>(dst).expect("alvo");
    assert_eq!(before.kind, JointKind::Pin, "premissa: o alvo era um Pin");

    paste(&mut sim, dst, &source);

    let after = *sim.world().get::<PhysicsJoint>(dst).expect("alvo");
    assert_eq!(after.kind, JointKind::Spring, "o tipo tem de viajar");
    assert!((after.stiffness - 400.0).abs() < 1e-4);
    assert!((after.rest_length - 2.75).abs() < 1e-4);
    assert!(after.break_enabled);
    assert_eq!(after.motor_mode, MotorMode::Position);
    // …e o que NÃO é propriedade continua do alvo.
    assert_eq!(
        after.local_a,
        [-1.0, -2.0],
        "a âncora é a COLOCAÇÃO do alvo"
    );
    assert_eq!(after.local_b, [-3.0, -4.0]);
    assert!(
        !after.active,
        "o alvo estava desligado de propósito — colar não pode re-ligá-lo"
    );
    assert!(
        !after.anchored,
        "o tipo mudou, logo a política de âncora mudou: o reconcile tem de \
         re-derivar as duas locais"
    );
}

/// **A porta atravessa em silêncio o que não é um joint.**
///
/// É esta linha que torna o fan-out seguro: o Paste é a única edição da §12 que
/// se espalha sobre a seleção, e uma seleção mista tem sprites dentro. Sem o
/// early-return o paste tentaria escrever `PhysicsJoint` num sprite — e o
/// `queue_set` o CRIARIA, deixando na cena um joint sem corpos que o overlay
/// desenharia e que o reconcile recusaria em silêncio.
#[test]
fn pasting_onto_something_that_is_not_a_joint_writes_nothing() {
    let (mut sim, hook, _plank) = two_bodies(true);
    let source = tuned(1, 2);
    paste(&mut sim, hook, &source);
    assert!(
        sim.world().get::<PhysicsJoint>(hook).is_none(),
        "o paste inventou um joint num CORPO — o fan-out sobre uma selecao \
         mista faria isso uma vez por sprite"
    );
}

/// **Colar duas vezes é colar uma vez** — a idempotência que um fan-out sobre
/// dez joints precisa ter, e o controle de que a porta não acumula nada.
#[test]
fn pasting_twice_is_pasting_once() {
    let (mut sim, src, dst) = source_and_target();
    let source = *sim.world().get::<PhysicsJoint>(src).expect("fonte");
    paste(&mut sim, dst, &source);
    let once = *sim.world().get::<PhysicsJoint>(dst).expect("alvo");
    paste(&mut sim, dst, &source);
    let twice = *sim.world().get::<PhysicsJoint>(dst).expect("alvo");
    assert_eq!(once, twice);
}

/// **Um número impossível na fonte é aparado na chegada** — o paste sai pelo
/// MESMO `clamped()` que toda edição da §12, então ele não pode autorar um
/// estado que digitar à mão não poderia.
///
/// ⚠️ A fonte aqui é construída à mão com um `NaN`, que a UI não produz; o que
/// o gate pina é que a porta não é um atalho POR VOLTA do clamp — se algum dia
/// uma fonte vier de um arquivo, ela chega pela mesma peneira.
#[test]
fn a_pasted_number_goes_through_the_same_clamp() {
    let (mut sim, _src, dst) = source_and_target();
    let poisoned = PhysicsJoint {
        stiffness: f32::NAN,
        max_length: -5.0,
        ..tuned(1, 2)
    };
    paste(&mut sim, dst, &poisoned);
    let after = *sim.world().get::<PhysicsJoint>(dst).expect("alvo");
    assert!(
        after.stiffness.is_finite(),
        "um NaN de rigidez envenena a pose do corpo pelo solver"
    );
    assert!(
        after.max_length >= PhysicsJoint::MIN_LENGTH,
        "rapier exige comprimento estritamente positivo numa corda"
    );
}
