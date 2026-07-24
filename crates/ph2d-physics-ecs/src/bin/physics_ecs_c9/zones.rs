//! **As lanes da familia das ZONAS** do harness `physics_ecs_c9` -- vento, arrasto, empuxo,
//! arrasto de forma, mesa giratoria, frame, falloff e espelho.
//!
//! Irmao do `physics_ecs_c9.rs` pelo cap de 700 LOC, e o corte e o que a cena ja vinha
//! fazendo: a familia das zonas cresceu uma lane por wave desde o W-Area e continua
//! crescendo, enquanto o resto do harness (queda, pilha, joints, constraints) esta estavel.
//! A proxima wave de zona acrescenta a lane AQUI.
//!
//! Cada lane existe para levar UM caminho do codigo ao hash: se ela some, o hash deixa de
//! provar aquele caminho e nada fica vermelho -- e por isso elas moram no harness e nao
//! numa suite de testes.

use ph2d_core::Vec2;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_physics_ecs::{
    AreaBuoyancy, AreaDrag, AreaEffector, AreaFalloff, AreaFormDrag, AreaTorque, BodyKind,
    Collider, ColliderShape, InitialVelocity, RigidBody,
};

/// Acrescenta as oito lanes de zona a cena do harness.
pub fn spawn(sim: &mut SimWorld) {
    // One WIND COLUMN and the ball falling through it (W-Area): the zone is a static
    // SENSOR carrying an `AreaEffector`, so the impulse is read back from the narrow
    // phase's INTERSECTION graph each substep and folded into the ball's velocity —
    // a path no other body here travels (every other fold happens before the pipeline
    // or inside the contact solver). CI proves that `f32` fold is bit-identical
    // cross-OS, the same guarantee gravity scale gets. Its own lane, far left.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 1.0,
                half_y: 3.0,
            },
            density: 1.0,
            is_sensor: true,
            ..Collider::default()
        },
        AreaEffector { force: [2.0, 0.0] },
        Transform::from_translation(Vec2::new(-58.0, 2.0)),
    ));
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-58.0, 4.0)),
    ));

    // One DRAG POOL and the ball sinking through it (W-AreaDrag): the zone carries an
    // `AreaDrag` and no force, so it reaches the world through the same `AreaEffect`
    // bundle by the OTHER half — and its decay is applied at the top of the substep,
    // outside the point where rapier applies its own damping. That is a different `f32`
    // fold from every other body here, and CI proves it bit-identical cross-OS. Its own
    // lane, far left.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 1.0,
                half_y: 2.0,
            },
            density: 1.0,
            is_sensor: true,
            ..Collider::default()
        },
        AreaDrag(6.0),
        Transform::from_translation(Vec2::new(-64.0, 1.0)),
    ));
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-64.0, 4.0)),
    ));

    // Uma POÇA e a caixa boiando nela (W-Buoyancy): o empuxo recorta o polígono do corpo
    // contra a superfície (Sutherland–Hodgman + shoelace) e aplica o impulso NUM PONTO, o
    // que gera torque — dois folds de `f32` que nenhum outro corpo do harness percorre, e
    // que CI prova bit-idênticos cross-OS. A caixa entra INCLINADA, então o momento
    // restaurador entra no hash também. Lane própria, na ponta esquerda.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 2.0,
                half_y: 1.5,
            },
            density: 1.0,
            is_sensor: true,
            ..Collider::default()
        },
        AreaBuoyancy(4.0),
        Transform::from_translation(Vec2::new(-70.0, -1.5)),
    ));
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.4,
                half_y: 0.2,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform {
            translation: Vec2::new(-70.0, 1.0),
            rotation: 0.6,
            scale: Vec2::new(1.0, 1.0),
            skew_x: 0.0,
            skew_y: 0.0,
        },
    ));

    // Uma zona de arrasto de FORMA e um tronco girando dentro dela (W-FormDrag): a
    // resistencia e somada aresta por aresta, com `omega x r` por sub-amostra, entao ela
    // percorre um fold de `f32` que nenhum outro corpo do harness toca. O tronco entra
    // GIRANDO e de trave, para que as duas metades (seccao e freio de rotacao) entrem no
    // hash. Lane propria, na ponta esquerda.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 2.0,
                half_y: 2.0,
            },
            density: 1.0,
            is_sensor: true,
            ..Collider::default()
        },
        AreaFormDrag(3.0),
        Transform::from_translation(Vec2::new(-76.0, 0.0)),
    ));
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.9,
                half_y: 0.2,
            },
            density: 1.0,
            ..Collider::default()
        },
        InitialVelocity {
            linvel: [1.0, 0.0],
            angvel: 5.0,
        },
        Transform::from_translation(Vec2::new(-76.0, 1.0)),
    ));

    // Uma MESA GIRATORIA e um tronco girando com ela (W-AreaTorque): o torque entra por
    // `apply_torque_impulse`, resistido pelo MOMENTO DE INERCIA do corpo -- um fold de
    // `f32` (torque * dt) que nenhum outro corpo do harness percorre, e o corpo entra
    // parado para que a rampa de omega venha inteira do torque da zona. Lane propria, na
    // ponta esquerda, sem gravidade local (a zona so gira).
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 2.0,
                half_y: 2.0,
            },
            density: 1.0,
            is_sensor: true,
            ..Collider::default()
        },
        AreaTorque(8.0),
        Transform::from_translation(Vec2::new(-82.0, 0.0)),
    ));
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.9,
                half_y: 0.2,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-82.0, 0.0)),
    ));

    // Uma zona de forca ROTACIONADA (W-AreaFrame): a forca e autorada no frame DELA e
    // rodada pela pose dela, entao este e o unico corpo do harness cujo impulso passa
    // pelo `zone_force_world` -- um par (sin, cos) vindo do `UnitComplex` do rapier e
    // dois produtos de `f32` por sub-passo. Meia volta de giro (nem eixo, nem 45 graus)
    // para que seno e cosseno sejam os dois nao-triviais e um erro de ulp em qualquer um
    // deles mova o hash. Lane propria a esquerda da mesa giratoria.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 2.5,
                half_y: 2.5,
            },
            density: 1.0,
            is_sensor: true,
            ..Collider::default()
        },
        AreaEffector { force: [3.0, 0.0] },
        Transform {
            translation: Vec2::new(-92.0, 0.0),
            rotation: 0.9,
            scale: Vec2::new(1.0, 1.0),
            skew_x: 0.0,
            skew_y: 0.0,
        },
    ));
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.3 },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-92.0, 0.0)),
    ));

    // Uma zona com FALLOFF (W-AreaFalloff): o fator entra no caminho determinista por uma
    // transformacao inversa de isometria mais a regua `radial_fraction`, que numa CAPSULA
    // e o ramo com `sqrt` (as calotas) -- a forma mais cara das cinco, escolhida de
    // proposito para o hash cobrir o ramo que um `hypot` de plataforma envenenaria. O corpo
    // nasce FORA do eixo para que a regua leia os dois ramos e nao so o flanco reto. Lane
    // propria a esquerda da zona rotacionada.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 2.0,
                radius: 1.5,
            },
            density: 1.0,
            is_sensor: true,
            ..Collider::default()
        },
        AreaEffector { force: [2.0, 1.0] },
        AreaFalloff(0.8),
        Transform::from_translation(Vec2::new(-102.0, 0.0)),
    ));
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.3 },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-101.2, 0.7)),
    ));

    // Uma zona ESPELHADA (W-AreaMirror): o sinal da escala entra no caminho determinista
    // por dois produtos de `f32` na forca e um no torque (o pseudoescalar). Espelhada num
    // eixo SO -- um espelho de verdade, nao a rotacao de 180 graus que espelhar os dois
    // seria -- e girada, para que o espelho e a rotacao componham na ordem que o kernel
    // afirma. Lane propria a esquerda da zona com falloff.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 2.5,
                half_y: 2.5,
            },
            density: 1.0,
            is_sensor: true,
            ..Collider::default()
        },
        AreaEffector { force: [2.5, 0.0] },
        AreaTorque(1.5),
        Transform {
            translation: Vec2::new(-112.0, 0.0),
            rotation: 0.6,
            scale: Vec2::new(-1.0, 1.0),
            skew_x: 0.0,
            skew_y: 0.0,
        },
    ));
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.3 },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-112.0, 0.0)),
    ));
}
