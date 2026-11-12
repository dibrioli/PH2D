//! Wheel · Car e Wheel · Motorbike.

use super::{UP, at_b, boxed, disc, rule, template};
use crate::{JointKind, JointRule, Motor, Role, Template, Tune};
use ph2d_physics_ecs::MotorMode;

/// Uma volta por segundo. NEGATIVO roda para `+x`, a frente do desenho (medido no smoke 57).
const SPIN: f32 = -std::f32::consts::TAU;
/// O torque do motor de trás, o do smoke 57: o carro de fábrica anda 9,1 m em 5 s (doc 02).
const DRIVE: f32 = 40.0;
/// O curso: 15 cm a descer (`min`, extensão) e 10 cm a subir (`max`, compressão).
const TRAVEL: [f32; 2] = [-0.15, 0.10];

const SUSPENSION: Tune = Tune {
    limits: Some(TRAVEL),
    ..Tune::NONE
};
const DRIVEN: Tune = Tune {
    motor: Some(Motor {
        mode: MotorMode::Velocity,
        value: SPIN,
        max_force: DRIVE,
    }),
    ..SUSPENSION
};

const fn wheel(name: &'static str, b: u16, axis: f32, tune: Tune) -> JointRule {
    rule(name, 0, b, JointKind::Wheel, at_b(0.0, 0.0))
        .axis(axis)
        .tuned(tune)
}

const CAR_ROLES: &[Role] = &[
    boxed("Body", [1.2, 0.25], [0.0, 0.55]),
    disc("Rear wheel", 0.30, [-0.70, 0.30]),
    disc("Front wheel", 0.30, [0.70, 0.30]),
];
const CAR_JOINTS: &[JointRule] = &[
    wheel("Rear wheel", 1, UP, DRIVEN),
    wheel("Front wheel", 2, UP, SUSPENSION),
];
pub(super) const CAR: Template = template(1, JointKind::Wheel, "Car", CAR_ROLES, CAR_JOINTS);

/// A inclinação do garfo da frente: 25° para trás.
const RAKE: f32 = 0.436;

const BIKE_ROLES: &[Role] = &[
    boxed("Body", [0.8, 0.2], [0.0, 0.80]),
    disc("Rear wheel", 0.32, [-0.65, 0.32]),
    disc("Front wheel", 0.32, [0.65, 0.32]),
];
const BIKE_JOINTS: &[JointRule] = &[
    wheel("Rear wheel", 1, UP, DRIVEN),
    wheel("Front wheel", 2, UP + RAKE, SUSPENSION),
];
pub(super) const MOTORBIKE: Template =
    template(2, JointKind::Wheel, "Motorbike", BIKE_ROLES, BIKE_JOINTS);
