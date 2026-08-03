//! As duas lanes de JOINT do harness de determinismo que nao sao rigs -- o
//! SERVO (W-J6) e o CUSTOM (W-JointCustom).
//!
//! Irmao do `main.rs` pelo cap de 700 LOC, cortado pelo mesmo assunto que o
//! `rigs.rs` e o `zones.rs` ja usam: um vinculo de DOIS corpos com um parametro
//! cada, contra uma montagem de varios corpos e contra um corpo com um flag.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    AxisMode, AxisSpec, BodyKind, Collider, ColliderShape, CustomAxis, JointKind, MotorMode,
    PhysicsJoint, RigidBody,
};

pub(super) fn spawn(sim: &mut SimWorld) {
    // Um SERVO (W-J6): a mesma dobradica do resto do repo, mas mirando um LUGAR
    // em vez de uma taxa. Entra no hash porque o motor de POSICAO e um caminho de
    // solver proprio -- `set_motor` com stiffness diferente de zero, resolvido
    // junto com os contatos -- e porque ele SEGURA (o corpo nao dorme), entao
    // toda divergencia de plataforma continua acumulando ate o fim dos passos em
    // vez de ser congelada pelo sono no primeiro segundo.
    sim.world_mut().spawn((
        Name::new("C9 Servo Hook"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.05 },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-64.0, 6.0)),
    ));
    sim.world_mut().spawn((
        Name::new("C9 Servo Arm"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.5,
                half_y: 0.1,
            },
            density: 1.0,
            ..Collider::default()
        },
        // Pendurado: a gravidade puxa para LONGE do alvo o tempo todo, que e o
        // que torna o servo observavel em vez de coincidente.
        Transform::from_translation(Vec2::new(-64.0, 5.5)),
    ));
    sim.world_mut().spawn((
        Name::new("C9 Servo"),
        PhysicsJoint {
            body_a: stable_name_id("C9 Servo Hook"),
            body_b: stable_name_id("C9 Servo Arm"),
            kind: JointKind::Pin,
            motor_enabled: true,
            motor_mode: MotorMode::Position,
            // 1 rad, um angulo que nao e nem 0 nem um multiplo de pi/4.
            motor_target: 1.0,
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(-64.0, 6.0)),
    ));

    // Um CUSTOM (W-JointCustom): a configuracao de eixos AUTORADA. Entra no hash
    // porque ele e o unico tipo cujo `GenericJoint` nao vem de um builder
    // especializado -- a mascara de travamento e os batentes por eixo sao
    // montados campo a campo -- e porque a combinacao que ele monta aqui (um
    // eixo LIMITADO, um TRAVADO, um LIVRE) nao e alcancavel por nenhum preset.
    // A carga bate nos dois batentes e SEGURA, entao a divergencia continua
    // acumulando em vez de ser congelada pelo sono.
    sim.world_mut().spawn((
        Name::new("C9 Custom Post"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.05 },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-58.0, 6.0)),
    ));
    sim.world_mut().spawn((
        Name::new("C9 Custom Block"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.25,
                half_y: 0.25,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-58.0, 6.0)),
    ));
    sim.world_mut().spawn((
        Name::new("C9 Custom"),
        {
            let mut j = PhysicsJoint {
                body_a: stable_name_id("C9 Custom Post"),
                body_b: stable_name_id("C9 Custom Block"),
                kind: JointKind::Custom,
                motor_enabled: true,
                motor_mode: MotorMode::Velocity,
                motor_speed: 1.7,
                motor_max_force: 40.0,
                ..PhysicsJoint::default()
            };
            // X limitado (o carrinho bate nos dois batentes), Y travado (nao cai),
            // rotacao livre (o bloco gira enquanto desliza) -- e o motor no eixo
            // LINEAR, que e a escolha que nenhum preset oferece.
            *j.custom.axis_mut(CustomAxis::X) = AxisSpec {
                mode: AxisMode::Limited,
                min: -0.6,
                max: 0.6,
            };
            j.custom.axis_mut(CustomAxis::Y).mode = AxisMode::Locked;
            j.custom.axis_mut(CustomAxis::Rotation).mode = AxisMode::Free;
            j.custom.motor_axis = CustomAxis::X;
            j
        },
        Transform::from_translation(Vec2::new(-58.0, 6.0)),
    ));
}
