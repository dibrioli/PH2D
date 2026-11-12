//! O catálogo. A ordem de [`ALL`] é a do seletor; o `id` é o que se grava e nunca muda.
//! Cada número de fábrica tem a sua medição em `docs/JointEditor/02_gabaritos_medidos.md`.

mod hinges;
mod lines;
mod rails;
mod wheels;

use crate::{
    Anchor, Guide, JointKind, JointRule, Link, Many, Role, Spot, Template, TemplateId, Tune,
};

/// «Para cima»: a rotação da entidade-junta que põe o eixo na vertical.
const UP: f32 = std::f32::consts::FRAC_PI_2;

const fn boxed(name: &'static str, half: [f32; 2], at: [f32; 2]) -> Role {
    Role {
        name,
        guide: Guide::Box { half },
        at,
        many: None,
        may_be_world: false,
        fixed: false,
    }
}

const fn disc(name: &'static str, radius: f32, at: [f32; 2]) -> Role {
    Role {
        name,
        guide: Guide::Disc { radius },
        at,
        many: None,
        may_be_world: false,
        fixed: false,
    }
}

impl Role {
    /// Uma peça parada que o cenário pode substituir (o poste, o trilho, o gancho).
    const fn scenery(mut self) -> Self {
        self.may_be_world = true;
        self.fixed = true;
        self
    }

    const fn row(mut self, count: u16, step: [f32; 2]) -> Self {
        self.many = Some(Many { count, step });
        self
    }
}

const fn rule(name: &'static str, a: u16, b: u16, kind: JointKind, anchor: Anchor) -> JointRule {
    JointRule {
        name,
        a,
        b,
        link: Link::Pair,
        kind,
        anchor,
        axis: None,
        tune: Tune::NONE,
        group: 0,
    }
}

impl JointRule {
    const fn link(mut self, link: Link) -> Self {
        self.link = link;
        self
    }
    const fn axis(mut self, rad: f32) -> Self {
        self.axis = Some(rad);
        self
    }
    const fn tuned(mut self, tune: Tune) -> Self {
        self.tune = tune;
        self
    }
}

const fn template(
    id: u16,
    kind: JointKind,
    name: &'static str,
    roles: &'static [Role],
    joints: &'static [JointRule],
) -> Template {
    Template {
        id: TemplateId(id),
        kind,
        name,
        roles,
        joints,
    }
}

const fn at_b(u: f32, v: f32) -> Anchor {
    Anchor::At(Spot::b(u, v))
}

pub const ALL: &[Template] = &[
    wheels::CAR,
    wheels::MOTORBIKE,
    hinges::DOOR,
    hinges::SEESAW,
    hinges::PENDULUM,
    hinges::FLIPPER,
    lines::CHAIN,
    lines::BRIDGE,
    lines::HANGING,
    rails::ELEVATOR,
    rails::PISTON,
    rails::SLIDING_DOOR,
    lines::TRAMPOLINE,
    lines::SOFT_PLATFORM,
    hinges::GLUE,
    hinges::BREAKABLE,
    lines::RIGID_PENDULUM,
    lines::LIFT,
];
