//! Pin · Door, Seesaw, Pendulum, Flipper — e Weld · Glue, Breakable.

use super::{at_b, boxed, disc, rule, template};
use crate::{Anchor, JointKind, JointRule, Motor, Role, Spot, Template, Tune};
use ph2d_physics_ecs::{MotorMode, PhysicsJoint};
use std::f32::consts::FRAC_PI_2;

const fn limited(lo: f32, hi: f32) -> Tune {
    Tune {
        limits: Some([lo, hi]),
        ..Tune::NONE
    }
}

// A porta de vaivém: pendurada pelo topo, um quarto de volta para cada lado.
const DOOR_ROLES: &[Role] = &[
    boxed("Frame", [0.6, 0.1], [0.0, 2.15]).scenery(),
    boxed("Door", [0.1, 1.0], [0.0, 1.05]),
];
const DOOR_JOINTS: &[JointRule] =
    &[rule("Hinge", 0, 1, JointKind::Pin, at_b(0.0, 1.0)).tuned(limited(-FRAC_PI_2, FRAC_PI_2))];
pub(super) const DOOR: Template = template(3, JointKind::Pin, "Door", DOOR_ROLES, DOOR_JOINTS);

/// 20°: a ponta da tábua de fábrica (1,5 m do pino) desce 0,51 m e toca o chão do desenho.
const SEESAW_TILT: f32 = 0.35;
const SEESAW_ROLES: &[Role] = &[
    boxed("Base", [0.15, 0.3], [0.0, 0.3]).scenery(),
    boxed("Plank", [1.5, 0.08], [0.0, 0.68]),
];
const SEESAW_JOINTS: &[JointRule] =
    &[rule("Pivot", 0, 1, JointKind::Pin, at_b(0.0, 0.0))
        .tuned(limited(-SEESAW_TILT, SEESAW_TILT))];
pub(super) const SEESAW: Template =
    template(4, JointKind::Pin, "Seesaw", SEESAW_ROLES, SEESAW_JOINTS);

pub(super) const SWING_ROLES: &[Role] = &[
    boxed("Pivot", [0.1, 0.1], [0.0, 2.5]).scenery(),
    disc("Bob", 0.25, [0.0, 1.0]),
];
const PENDULUM_JOINTS: &[JointRule] = &[rule(
    "Pivot",
    0,
    1,
    JointKind::Pin,
    Anchor::At(Spot::a(0.0, 0.0)),
)];
pub(super) const PENDULUM: Template =
    template(5, JointKind::Pin, "Pendulum", SWING_ROLES, PENDULUM_JOINTS);

/// A palheta descansa em −0,5 rad e o jogo manda-a a +0,5: um servo, que cede a uma bola pesada.
const FLIPPER_REST: f32 = -0.5;
const FLIPPER_ROLES: &[Role] = &[
    boxed("Base", [0.1, 0.1], [-0.5, 1.0]).scenery(),
    boxed("Paddle", [0.6, 0.08], [0.0, 1.0]),
];
const FLIPPER_JOINTS: &[JointRule] = &[rule("Pivot", 0, 1, JointKind::Pin, at_b(-0.85, 0.0))
    .tuned(Tune {
        motor: Some(Motor {
            mode: MotorMode::Position,
            value: FLIPPER_REST,
            max_force: 60.0,
        }),
        ..limited(FLIPPER_REST, -FLIPPER_REST)
    })];
pub(super) const FLIPPER: Template =
    template(6, JointKind::Pin, "Flipper", FLIPPER_ROLES, FLIPPER_JOINTS);

const WELD_ROLES: &[Role] = &[
    boxed("Part A", [0.4, 0.4], [-0.4, 0.4]),
    boxed("Part B", [0.4, 0.4], [0.4, 0.4]),
];
const GLUE_JOINTS: &[JointRule] = &[rule("Glue", 0, 1, JointKind::Weld, Anchor::Seam)];
pub(super) const GLUE: Template = template(15, JointKind::Weld, "Glue", WELD_ROLES, GLUE_JOINTS);

const BREAKABLE_JOINTS: &[JointRule] =
    &[
        rule("Glue", 0, 1, JointKind::Weld, Anchor::Seam).tuned(Tune {
            // «Aguenta uns dez quilos», a escala medida das constantes de quebra do tipo.
            breaks: Some([
                PhysicsJoint::DEFAULT_BREAK_FORCE,
                PhysicsJoint::DEFAULT_BREAK_TORQUE,
            ]),
            ..Tune::NONE
        }),
    ];
pub(super) const BREAKABLE: Template = template(
    16,
    JointKind::Weld,
    "Breakable",
    WELD_ROLES,
    BREAKABLE_JOINTS,
);
