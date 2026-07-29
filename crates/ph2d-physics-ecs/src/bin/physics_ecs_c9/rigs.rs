//! **Os dois rigs articulados do c9** — a roda (W-Wheel) e a polia (W-Pulley).
//!
//! Irmão de `main.rs`, separado dele pelo cap de 700 LOC. O corte é por assunto:
//! uma lane de zona é um corpo com um flag, um RIG é uma montagem de vários
//! corpos mais o vínculo que os une.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, PulleyWheel, RigidBody, WrapSide,
};

/// Monta as duas lanes.
pub fn spawn(sim: &mut SimWorld) {
    // Uma RODA (W-Wheel): o unico joint do kit que deixa DOIS graus de liberdade
    // livres, entao ele e o unico caminho de solver com um motor de posicao (a
    // suspensao, em `LinX`) e um limite BILATERAL no mesmo joint. Ela roda no
    // CHAO de proposito -- uma suspensao so comprime quando a roda esta APOIADA e
    // o peso do chassi desce sobre ela; solta no ar os dois corpos caem juntos e
    // a lane mediria uma distancia constante.
    sim.world_mut().spawn((
        Name::new("C9 Wheel Ground"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 3.0,
                half_y: 0.25,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-56.0, -0.25)),
    ));
    sim.world_mut().spawn((
        Name::new("C9 Wheel Chassis"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.8,
                half_y: 0.2,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-56.0, 0.8)),
    ));
    sim.world_mut().spawn((
        Name::new("C9 Wheel Hub"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.3 },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-56.0, 0.3)),
    ));
    sim.world_mut().spawn((
        Name::new("C9 Wheel"),
        PhysicsJoint {
            body_a: stable_name_id("C9 Wheel Chassis"),
            body_b: stable_name_id("C9 Wheel Hub"),
            kind: JointKind::Wheel,
            // Com curso ARMADO: o batente e um caminho de solver a mais (limite
            // bilateral nao-acoplado), e ele so percorre se estiver ligado.
            limits_enabled: true,
            limit_min: -0.15,
            limit_max: 0.15,
            // E com TRACAO: o motor angular e o outro eixo do mesmo joint, e a
            // roda girando mantem a lane VIVA ate o fim dos passos em vez de
            // dormir no primeiro segundo.
            motor_enabled: true,
            motor_speed: -4.0,
            ..PhysicsJoint::default()
        },
        // A suspensao aponta para CIMA (o eixo e a rotacao do proprio joint).
        Transform {
            translation: Vec2::new(-56.0, 0.3),
            rotation: std::f32::consts::FRAC_PI_2,
            ..Transform::IDENTITY
        },
    ));

    // W-Pulley: um elevador com contrapeso. A POLIA e o primeiro vinculo que NAO
    // e um joint do rapier -- ela e um passe de impulso por sub-passo, com massa
    // efetiva computada a partir do `effective_inv_mass` (por eixo) e do
    // `effective_world_inv_inertia_sqrt` do proprio rapier. Entra no hash porque
    // MOVE corpos: se aquela aritmetica divergisse entre OSes, e aqui que se ve.
    //
    // Massas DIFERENTES (4 kg contra 1) de proposito: com massas iguais a lane
    // fica parada e o hash nao veria a corda trabalhar.
    for (name, x, density) in [
        (
            "C9 Pulley Load",
            -64.0_f32,
            4.0_f32 / (std::f32::consts::PI * 0.04),
        ),
        (
            "C9 Pulley Counterweight",
            -60.0,
            1.0 / (std::f32::consts::PI * 0.04),
        ),
    ] {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                density,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, 6.0)),
        ));
    }
    sim.world_mut().spawn((
        Name::new("C9 Pulley"),
        PhysicsJoint {
            body_a: stable_name_id("C9 Pulley Load"),
            body_b: stable_name_id("C9 Pulley Counterweight"),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(-64.0, 6.0)),
    ));
    // As ROLDANAS (W-Pulley W1): entidades, com raio. O raio entra no hash por
    // DOIS caminhos -- a tangencia (por onde a corda passa) e o ARCO (quanto dela
    // existe), e o arco e o unico ponto do passe que chama um transcendental
    // (`libm::atan2f`, pinado cross-OS pela lei 6). Uma lane de raio ZERO nao
    // exercitaria nenhum dos dois.
    //
    // E a SEGUNDA delas e um TAMBOR (W2): `motor_speed` encurta o comprimento de
    // repouso a `w*r` por segundo, o que move as duas poses -- entao ele TEM de
    // entrar no hash. Ele tambem e a unica lane que exercita o `pulley_payout`,
    // a integral que o checkpoint carrega.
    for (order, x, radius, motor) in [(0_u16, -64.0_f32, 0.4_f32, 0.0_f32), (1, -60.0, 0.25, 1.5)] {
        sim.world_mut().spawn((
            Name::new(format!("C9 Pulley Wheel {}", order + 1)),
            PulleyWheel {
                rope: stable_name_id("C9 Pulley"),
                order,
                radius,
                wrap: WrapSide::Auto,
                motor_speed: motor,
                // Roldanas de CENARIO: a lane da talha (W3) e a de baixo, e
                // separa-las e o que torna esta a linha de regressao do modelo
                // antigo -- ela tem de deixar o hash onde estava.
                body: 0,
                local: [0.0, 0.0],
                mounted: false,
                break_enabled: false,
                break_force: PulleyWheel::DEFAULT_BREAK_FORCE,
            },
            Transform::from_translation(Vec2::new(x, 9.0)),
        ));
    }
}
