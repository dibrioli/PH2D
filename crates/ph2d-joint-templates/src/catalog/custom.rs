//! **Custom** — o último gabarito de cada família (maquete B1): duas peças quaisquer e UMA junta do
//! tipo, com os números de fábrica do tipo. É a junta feita à mão de sempre, pela mesma porta de
//! soltar peças: o artista traz as duas e afina nos cartões.

use super::{UP, boxed, disc, rule};
use crate::{Anchor, JointKind, JointRule, Length, Role, Spot, Template, TemplateId, Tune};

/// As duas peças lado a lado, a encostar no meio: a emenda é o ponto da junta.
const SIDE_BY_SIDE: &[Role] = &[
    boxed("Part A", [0.4, 0.15], [-0.4, 0.6]),
    boxed("Part B", [0.4, 0.15], [0.4, 0.6]),
];
/// Uma em cima da outra, afastadas: a junta de duas pontas liga a base de A ao topo de B.
const ONE_ABOVE: &[Role] = &[
    boxed("Part A", [0.3, 0.15], [0.0, 2.0]),
    boxed("Part B", [0.3, 0.15], [0.0, 1.0]),
];
/// Lado a lado e afastadas: a polia passa a corda por cima das duas.
const APART: &[Role] = &[
    boxed("Part A", [0.3, 0.3], [-1.0, 0.3]),
    boxed("Part B", [0.3, 0.3], [1.0, 0.3]),
];
const WHEEL_ROLES: &[Role] = &[
    boxed("Body", [0.6, 0.2], [0.0, 0.75]),
    disc("Wheel", 0.3, [0.0, 0.3]),
];
const RAIL_ROLES: &[Role] = &[
    boxed("Rail", [0.8, 0.1], [0.0, 0.4]),
    boxed("Part B", [0.25, 0.2], [0.0, 0.7]),
];

const ENDS: Anchor = Anchor::Ends(Spot::a(0.0, -1.0), Spot::b(0.0, 1.0));
/// O comprimento é a distância entre as pontas como estão na cena.
const AS_PLACED: Tune = Tune {
    length: Some(Length::Gap { slack: 0.0 }),
    ..Tune::NONE
};

const PIN: &[JointRule] = &[rule("Pin", 0, 1, JointKind::Pin, Anchor::Seam)];
const WELD: &[JointRule] = &[rule("Weld", 0, 1, JointKind::Weld, Anchor::Seam)];
const FREE: &[JointRule] = &[rule("Joint", 0, 1, JointKind::Custom, Anchor::Seam)];
const WHEEL: &[JointRule] = &[rule(
    "Wheel",
    0,
    1,
    JointKind::Wheel,
    Anchor::At(Spot::b(0.0, 0.0)),
)
.axis(UP)];
const SLIDER: &[JointRule] = &[rule(
    "Slider",
    0,
    1,
    JointKind::Slider,
    Anchor::At(Spot::b(0.0, 0.0)),
)
.axis(0.0)];
const SPRING: &[JointRule] = &[rule("Spring", 0, 1, JointKind::Spring, ENDS)];
const ROPE: &[JointRule] = &[rule("Rope", 0, 1, JointKind::Rope, ENDS).tuned(AS_PLACED)];
const ROD: &[JointRule] = &[rule("Rod", 0, 1, JointKind::Rod, ENDS).tuned(AS_PLACED)];
const PULLEY: &[JointRule] = &[rule("Rope", 0, 1, JointKind::Pulley, Anchor::Seam)];

const fn custom(
    id: u16,
    kind: JointKind,
    roles: &'static [Role],
    joints: &'static [JointRule],
) -> Template {
    Template {
        id: TemplateId(id),
        kind,
        name: "Custom",
        roles,
        joints,
        hand_made: true,
    }
}

pub(super) const WHEEL_C: Template = custom(19, JointKind::Wheel, WHEEL_ROLES, WHEEL);
pub(super) const PIN_C: Template = custom(20, JointKind::Pin, SIDE_BY_SIDE, PIN);
pub(super) const ROPE_C: Template = custom(21, JointKind::Rope, ONE_ABOVE, ROPE);
pub(super) const SLIDER_C: Template = custom(22, JointKind::Slider, RAIL_ROLES, SLIDER);
pub(super) const SPRING_C: Template = custom(23, JointKind::Spring, ONE_ABOVE, SPRING);
pub(super) const WELD_C: Template = custom(24, JointKind::Weld, SIDE_BY_SIDE, WELD);
pub(super) const ROD_C: Template = custom(25, JointKind::Rod, ONE_ABOVE, ROD);
pub(super) const PULLEY_C: Template = custom(26, JointKind::Pulley, APART, PULLEY);
pub(super) const FREE_C: Template = custom(27, JointKind::Custom, SIDE_BY_SIDE, FREE);
