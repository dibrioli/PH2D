//! Slider · Elevator, Piston, Sliding door.

use super::{UP, at_b, boxed, rule, template};
use crate::{JointKind, JointRule, Motor, Role, Template, Tune};
use ph2d_physics_ecs::MotorMode;

const fn rail(stroke: f32, motor: Option<Motor>) -> Tune {
    Tune {
        limits: Some([0.0, stroke]),
        motor,
        ..Tune::NONE
    }
}

const fn push(speed: f32, max_force: f32) -> Option<Motor> {
    Some(Motor {
        mode: MotorMode::Velocity,
        value: speed,
        max_force,
    })
}

// Um andar (3 m) a 1 m/s: a plataforma de fábrica sobe 3,00 m e para no batente (doc 02).
const ELEVATOR_ROLES: &[Role] = &[
    boxed("Rail", [0.05, 2.0], [-1.2, 2.0]).scenery(),
    boxed("Platform", [1.0, 0.1], [0.0, 0.1]),
];
const ELEVATOR_JOINTS: &[JointRule] = &[rule("Rail", 0, 1, JointKind::Slider, at_b(0.0, 0.0))
    .axis(UP)
    .tuned(rail(3.0, push(1.0, 200.0)))];
pub(super) const ELEVATOR: Template = template(
    10,
    JointKind::Slider,
    "Elevator",
    ELEVATOR_ROLES,
    ELEVATOR_JOINTS,
);

const PISTON_ROLES: &[Role] = &[
    boxed("Cylinder", [0.5, 0.2], [-0.5, 1.0]).scenery(),
    boxed("Rod", [0.5, 0.1], [0.3, 1.0]),
];
const PISTON_JOINTS: &[JointRule] = &[rule("Stroke", 0, 1, JointKind::Slider, at_b(-1.0, 0.0))
    .axis(0.0)
    .tuned(rail(0.8, push(0.5, 100.0)))];
pub(super) const PISTON: Template =
    template(11, JointKind::Slider, "Piston", PISTON_ROLES, PISTON_JOINTS);

// A porta corre a própria largura e não tem motor: abre quem a empurra (ou um script).
const SLIDING_ROLES: &[Role] = &[
    boxed("Track", [1.0, 0.05], [0.5, 2.15]).scenery(),
    boxed("Door", [0.5, 1.0], [0.0, 1.05]),
];
const SLIDING_JOINTS: &[JointRule] = &[rule("Track", 0, 1, JointKind::Slider, at_b(0.0, 1.0))
    .axis(0.0)
    .tuned(rail(1.0, None))];
pub(super) const SLIDING_DOOR: Template = template(
    12,
    JointKind::Slider,
    "Sliding door",
    SLIDING_ROLES,
    SLIDING_JOINTS,
);
