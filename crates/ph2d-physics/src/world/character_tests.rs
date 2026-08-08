//! Os gates da porta do controlador cinemático (W-KinMove).
use super::*;
use crate::world::desc::BodyDesc;
use crate::world::shape::ShapeDesc;
use rapier2d::dynamics::RigidBodyType;

const UP: [f32; 2] = [0.0, 1.0];

fn params() -> CharacterParams {
    CharacterParams {
        up: UP,
        snap_distance: 0.0,
        max_slope_deg: 45.0,
        step_height: 0.0,
    }
}

/// ⚠️ `BodyDesc` não tem `Default` de propósito (cada campo é uma decisão do
/// chamador), então a fixture o escreve inteiro — o molde do `servo_gain_sweep`.
fn desc(body_type: RigidBodyType, x: f32, y: f32, shape: ShapeDesc, is_sensor: bool) -> BodyDesc {
    BodyDesc {
        body_type,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        shape,
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    }
}

fn capsule() -> ShapeDesc {
    ShapeDesc::Capsule {
        half_height: 0.3,
        radius: 0.2,
    }
}

/// Um mundo com chão em `y ∈ [−0.5, 0.5]` e um personagem-cápsula.
fn scene(at: [f32; 2]) -> (PhysicsWorld, RigidBodyHandle) {
    let mut w = PhysicsWorld::new();
    w.add_static_cuboid(0.0, 0.0, 10.0, 0.5);
    let me = w.spawn_body(desc(
        RigidBodyType::KinematicPositionBased,
        at[0],
        at[1],
        capsule(),
        false,
    ));
    w.step();
    (w, me)
}

/// **O que não encontra nada anda inteiro** — o controle de toda a suíte.
#[test]
fn a_clear_path_is_travelled_whole() {
    let (w, me) = scene([0.0, 3.0]);
    let got = w.move_character(me, [0.5, 0.0], params(), None, 0);
    assert!(
        (got.translation[0] - 0.5).abs() < 1.0e-4 && got.translation[1].abs() < 1.0e-4,
        "no ar, o deslocamento pedido e' o efetivo: {:?}",
        got.translation
    );
}

/// **E o que encontra o chão NÃO o atravessa** — a razão de existir da porta.
///
/// ⚠️ O oráculo é a folga do controlador (`offset`), não zero: o rapier preserva
/// deliberadamente um vão entre o personagem e o mundo, e exigir contato exato
/// seria o gate a medir uma coisa que a biblioteca promete não fazer.
#[test]
fn the_floor_is_not_crossed() {
    // Centro em `y = 1.0` ⇒ pé em `0.5` ⇒ 0.0 acima do topo... não: o topo do
    // chão é `0.5`, o pé da cápsula é `1.0 − 0.5 = 0.5`. Sobe meio metro.
    let (w, me) = scene([0.0, 1.5]);
    let got = w.move_character(me, [0.0, -2.0], params(), None, 0);
    assert!(
        got.translation[1] > -0.51 && got.translation[1] < -0.4,
        "a queda tem de parar no chao (pedidos 2 m, cabem ~0.5): {}",
        got.translation[1]
    );
    assert!(got.grounded, "e o rapier tem de o dizer no diagnostico");
}

/// **Um SENSOR não é uma parede** — a mesma frase que o `cast_ray` escreve, do
/// lado do controlador.
///
/// ⚠️ Sem a exclusão, um personagem cinemático seria **parado por um volume de
/// gatilho**: a face pior do defeito que fazia o dinâmico ficar de pé sobre a
/// água.
#[test]
fn a_trigger_volume_does_not_stop_a_character() {
    let mut w = PhysicsWorld::new();
    w.spawn_body(desc(
        RigidBodyType::Fixed,
        1.0,
        3.0,
        ShapeDesc::Cuboid {
            half_x: 2.0,
            half_y: 2.0,
        },
        true,
    ));
    let me = w.spawn_body(desc(
        RigidBodyType::KinematicPositionBased,
        -2.0,
        3.0,
        capsule(),
        false,
    ));
    w.step();
    let got = w.move_character(me, [1.0, 0.0], params(), None, 0);
    assert!(
        (got.translation[0] - 1.0).abs() < 1.0e-3,
        "um gatilho nao pode bloquear: {:?}",
        got.translation
    );
}

/// **Entrada degenerada e handle morto são no-op silencioso** — o chamador é um
/// laço por-entidade.
#[test]
fn degenerate_input_moves_nothing() {
    let (w, me) = scene([0.0, 3.0]);
    assert_eq!(
        w.move_character(me, [f32::NAN, 0.0], params(), None, 0)
            .translation,
        [0.0, 0.0]
    );
    let dead = RigidBodyHandle::invalid();
    assert_eq!(
        w.move_character(dead, [1.0, 0.0], params(), None, 0)
            .translation,
        [0.0, 0.0]
    );
}
