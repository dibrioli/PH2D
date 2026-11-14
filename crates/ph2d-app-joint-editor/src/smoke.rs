//! `PH2D_JOINT_EDITOR_SMOKE=<n>` — a aba aberta sobre a cena dos gabaritos (`PH2D_PHYSICS_SMOKE=123`):
//! `1` no «Car 1» montado · `2` num carro por fazer, com três peças soltas («Kart …») na Hierarchy para
//! arrastar · `3` na «Bridge 1» · `4` na «Door 1» · `5` o carro por fazer com a lista Template aberta (a foto
//! da maquete B1) · `6` o «Car 1» com o painel alargado pela divisória. É a cena 123 que monta o mundo; aqui só se abre a aba,
//! uma vez, no primeiro quadro em que o «Car 1» existe.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, ColliderShape, JointKind, RigidBody, nasce_com_forma};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::surface::JointEditorTab;

pub const ENV: &str = "PH2D_JOINT_EDITOR_SMOKE";
/// O maior nível que [`arm`] responde (conta-se no `match` abaixo).
pub const MAX_LEVEL: u32 = 7;

pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[ph2d_app_host::SmokeRouter {
    env: ENV,
    max_level: MAX_LEVEL,
}];

/// As peças soltas da cena 2: um carro por montar, à direita do «Car 1».
pub const KART: [(&str, [f32; 2]); 3] = [
    ("Kart Body", [18.0, 1.4]),
    ("Kart Rear Wheel", [17.3, 0.9]),
    ("Kart Front Wheel", [18.7, 0.9]),
];

#[must_use]
pub fn level() -> Option<u32> {
    std::env::var(ENV)
        .ok()?
        .trim()
        .parse()
        .ok()
        .filter(|n| (1..=MAX_LEVEL).contains(n))
}

fn find(sim: &mut SimWorld, name: &str) -> Option<u64> {
    let w = sim.world_mut();
    let mut q = w.query::<(ph2d_ecs::Entity, &Name, &ph2d_ecs::StableId)>();
    q.iter(w)
        .find(|(_, n, _)| n.as_str() == name)
        .map(|(_, _, s)| s.0)
}

/// As três peças do kart, com as formas-guia do gabarito Car (o corpo e as rodas de fábrica).
pub fn spawn_kart(sim: &mut SimWorld) {
    let Some(t) = ph2d_joint_templates::by_name(JointKind::Wheel, "Car") else {
        return;
    };
    let w = sim.world_mut();
    for ((name, at), role) in KART.iter().zip(t.roles) {
        let (shape, size, rgba) = match role.guide {
            ph2d_joint_templates::Guide::Box { half } => (
                ColliderShape::Cuboid {
                    half_x: half[0],
                    half_y: half[1],
                },
                [half[0] * 2.0, half[1] * 2.0],
                [0.55, 0.75, 0.95, 1.0],
            ),
            ph2d_joint_templates::Guide::Disc { radius } => (
                ColliderShape::Ball { radius },
                [radius * 2.0, radius * 2.0],
                [0.22, 0.22, 0.26, 1.0],
            ),
        };
        let e = w
            .spawn((
                Transform::from_translation(Vec2::new(at[0], at[1])),
                Sprite::atlas(WHITE_TILE_KEY, size, rgba),
                Name::new(*name),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
            ))
            .id();
        let _ = nasce_com_forma(w, e, shape);
    }
    ph2d_ecs::assign_missing_stable_ids(w);
    ph2d_ecs::assign_missing_root_order(w);
}

/// Abre a aba no estado do nível, uma vez.
pub fn arm(sim: &mut SimWorld, tab: &mut JointEditorTab) {
    if tab.smoke_done {
        return;
    }
    let Some(n) = level() else {
        tab.smoke_done = true;
        return;
    };
    let Some(car) = find(sim, "Car 1") else {
        return;
    };
    tab.smoke_done = true;
    tab.open();
    let on = match n {
        1 => car,
        6 => {
            tab.doc.set_panel_width(f32::MAX);
            car
        }
        7 => {
            tab.doc.set_panel_width(0.1);
            car
        }
        2 => {
            spawn_kart(sim);
            0
        }
        3 => find(sim, "Bridge 1").unwrap_or(car),
        4 => find(sim, "Door 1").unwrap_or(car),
        _ => 0,
    };
    tab.doc.set_snapshot(crate::model::read(sim));
    if on == 0 {
        tab.doc.choose_kind(JointKind::Wheel);
        if n == 5 {
            tab.doc.menu = Some(crate::doc::Menu::Template);
        }
    } else {
        tab.doc.open_on(on);
    }
}
