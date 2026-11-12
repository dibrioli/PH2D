//! As juntas de duas pontas: Rope · Chain, Bridge, Hanging — Spring · Trampoline, Soft platform —
//! Rod · Rigid pendulum — Pulley · Lift.

use super::hinges::SWING_ROLES;
use super::{boxed, rule, template};
use crate::{Anchor, JointKind, JointRule, Length, Link, Role, Spot, Template, Tune};

const fn ends(a: (f32, f32), b: (f32, f32)) -> Anchor {
    Anchor::Ends(Spot::a(a.0, a.1), Spot::b(b.0, b.1))
}

/// O comprimento é a distância entre as pontas como estão na cena.
const AS_PLACED: Tune = Tune {
    length: Some(Length::Gap { slack: 0.0 }),
    ..Tune::NONE
};

const CHAIN_ROLES: &[Role] = &[
    boxed("Hook", [0.1, 0.1], [0.0, 3.0]).scenery(),
    boxed("Link", [0.06, 0.18], [0.0, 2.7]).row(6, [0.0, -0.4]),
];
const CHAIN_JOINTS: &[JointRule] = &[
    rule(
        "Top link",
        0,
        1,
        JointKind::Rope,
        ends((0.0, 0.0), (0.0, 1.0)),
    )
    .link(Link::ToFirst)
    .tuned(AS_PLACED),
    rule("Link", 1, 1, JointKind::Rope, ends((0.0, -1.0), (0.0, 1.0)))
        .link(Link::Chain)
        .tuned(AS_PLACED),
];
pub(super) const CHAIN: Template = template(7, JointKind::Rope, "Chain", CHAIN_ROLES, CHAIN_JOINTS);

// Oito tábuas de 0,5 m com 2 cm de folga entre elas; os postes seguram as pontas a 1 m do chão.
const BRIDGE_ROLES: &[Role] = &[
    boxed("Left post", [0.15, 0.5], [-2.24, 0.6]).scenery(),
    boxed("Plank", [0.25, 0.05], [-1.82, 1.0]).row(8, [0.52, 0.0]),
    boxed("Right post", [0.15, 0.5], [2.24, 0.6]).scenery(),
];
const BRIDGE_JOINTS: &[JointRule] = &[
    rule(
        "Left end",
        0,
        1,
        JointKind::Rope,
        ends((1.0, 0.8), (-1.0, 0.0)),
    )
    .link(Link::ToFirst)
    .tuned(AS_PLACED),
    rule(
        "Plank",
        1,
        1,
        JointKind::Rope,
        ends((1.0, 0.0), (-1.0, 0.0)),
    )
    .link(Link::Chain)
    .tuned(AS_PLACED),
    rule(
        "Right end",
        2,
        1,
        JointKind::Rope,
        ends((-1.0, 0.8), (1.0, 0.0)),
    )
    .link(Link::ToLast)
    .tuned(AS_PLACED),
];
pub(super) const BRIDGE: Template =
    template(8, JointKind::Rope, "Bridge", BRIDGE_ROLES, BRIDGE_JOINTS);

const HANGING_ROLES: &[Role] = &[
    boxed("Hook", [0.1, 0.1], [0.0, 2.5]).scenery(),
    boxed("Load", [0.3, 0.3], [0.0, 1.0]),
];
const HANGING_JOINTS: &[JointRule] =
    &[rule("Rope", 0, 1, JointKind::Rope, ends((0.0, 0.0), (0.0, 1.0))).tuned(AS_PLACED)];
pub(super) const HANGING: Template =
    template(9, JointKind::Rope, "Hanging", HANGING_ROLES, HANGING_JOINTS);

// A lona esticada entre dois postes por molas em TRAÇÃO (30 cm de pré-carga cada): puxada pelos
// dois lados, ela assenta 3 mm abaixo do desenho e não gira (doc 02).
const TRAMPOLINE_SPRING: Tune = Tune {
    spring: Some([300.0, 2.0]),
    length: Some(Length::Gap { slack: -0.3 }),
    ..Tune::NONE
};
const TRAMPOLINE_ROLES: &[Role] = &[
    boxed("Left post", [0.1, 0.45], [-1.6, 0.45]).scenery(),
    boxed("Bed", [1.0, 0.05], [0.0, 0.81]),
    boxed("Right post", [0.1, 0.45], [1.6, 0.45]).scenery(),
];
const TRAMPOLINE_JOINTS: &[JointRule] = &[
    rule(
        "Left spring",
        0,
        1,
        JointKind::Spring,
        ends((0.0, 0.8), (-1.0, 0.0)),
    )
    .tuned(TRAMPOLINE_SPRING),
    rule(
        "Right spring",
        2,
        1,
        JointKind::Spring,
        ends((0.0, 0.8), (1.0, 0.0)),
    )
    .tuned(TRAMPOLINE_SPRING),
];
pub(super) const TRAMPOLINE: Template = template(
    13,
    JointKind::Spring,
    "Trampoline",
    TRAMPOLINE_ROLES,
    TRAMPOLINE_JOINTS,
);

// A plataforma pendurada em duas molas bem amortecidas: assenta 8 mm abaixo do desenho (doc 02).
const SOFT_SPRING: Tune = Tune {
    spring: Some([200.0, 10.0]),
    ..AS_PLACED
};
const SOFT_ROLES: &[Role] = &[
    boxed("Ceiling", [1.0, 0.1], [0.0, 2.5]).scenery(),
    boxed("Platform", [0.8, 0.1], [0.0, 1.0]),
];
const SOFT_JOINTS: &[JointRule] = &[
    rule(
        "Left spring",
        0,
        1,
        JointKind::Spring,
        ends((-0.7, -1.0), (-0.875, 1.0)),
    )
    .tuned(SOFT_SPRING),
    rule(
        "Right spring",
        0,
        1,
        JointKind::Spring,
        ends((0.7, -1.0), (0.875, 1.0)),
    )
    .tuned(SOFT_SPRING),
];
pub(super) const SOFT_PLATFORM: Template = template(
    14,
    JointKind::Spring,
    "Soft platform",
    SOFT_ROLES,
    SOFT_JOINTS,
);

const RIGID_JOINTS: &[JointRule] =
    &[rule("Rod", 0, 1, JointKind::Rod, ends((0.0, 0.0), (0.0, 0.0))).tuned(AS_PLACED)];
pub(super) const RIGID_PENDULUM: Template = template(
    17,
    JointKind::Rod,
    "Rigid pendulum",
    SWING_ROLES,
    RIGID_JOINTS,
);

// A corda e as roldanas saem da regra de sempre da polia (`pulley_rig`): acima das duas peças.
const LIFT_ROLES: &[Role] = &[
    boxed("Load", [0.3, 0.3], [-1.0, 0.3]),
    boxed("Counterweight", [0.3, 0.3], [1.0, 0.3]),
];
const LIFT_JOINTS: &[JointRule] = &[rule("Rope", 0, 1, JointKind::Pulley, Anchor::Seam)];
pub(super) const LIFT: Template = template(18, JointKind::Pulley, "Lift", LIFT_ROLES, LIFT_JOINTS);
