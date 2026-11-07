//! **A tinta:** o contorno da forma em edição e as alças, em píxeis de ecrã, com o tamanho constante em qualquer zoom.

use crate::{Alca, Medida, Pose, alcas, caixa, editando, no_ecra};
use bevy_ecs::world::World;
use ph2d_ecs::Entity;
use ph2d_editor_core::paint::resolve;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{Collider, ColliderShape, capsule_vertices, ellipse_vertices};
use ph2d_render::Camera2d;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Fill, Point, Rect, Stroke, VectorScene};

const ALCA_PX: f64 = 4.0; // LITERAL-PX-OK: meia-alça, em ecrã
const ALCA_SOB_PX: f64 = 6.0; // LITERAL-PX-OK: meia-alça sob o ponteiro
const MIOLO_PX: f64 = 3.0; // LITERAL-PX-OK: raio do ponto do miolo
const TRACO_PX: f64 = 1.5; // LITERAL-PX-OK: traço do contorno e das alças
const TRACEJADO_PX: [f64; 2] = [6.0, 4.0]; // LITERAL-PX-OK: o tracejado do Sensor

/// O contorno em ecrã: a mesma figura que o solver simula, pela mesma pose.
#[must_use]
pub fn contorno(pose: &Pose, m: &Medida, cam: &Camera2d, win: WindowSize) -> BezPath {
    let (c, h) = caixa(pose, m);
    let local: Vec<[f32; 2]> = match m.shape {
        ColliderShape::Cuboid { .. } => {
            vec![[-h[0], -h[1]], [h[0], -h[1]], [h[0], h[1]], [-h[0], h[1]]]
        }
        ColliderShape::Ball { .. } => ellipse_vertices(h[0], h[1]).to_vec(),
        ColliderShape::Capsule { half_height, .. } => {
            capsule_vertices(half_height * pose.s[1].abs(), h[0], h[1] - half_height * pose.s[1].abs())
                .to_vec()
        }
    };
    let mut path = BezPath::new();
    for (i, [x, y]) in local.iter().enumerate() {
        let (sx, sy) = cam.world_to_screen(pose.para_mundo([c[0] + x, c[1] + y]), win);
        let p = Point::new(f64::from(sx), f64::from(sy));
        if i == 0 {
            path.move_to(p);
        } else {
            path.line_to(p);
        }
    }
    path.close_path();
    path
}

/// Pinta o contorno (tracejado num Sensor) e as alças. Nada sem edição acesa.
pub fn desenhar(
    world: &World,
    cam: &Camera2d,
    win: WindowSize,
    ponteiro: (f32, f32),
    theme: Theme,
    scene: &mut VectorScene,
) {
    let Some(alvo) = editando(world) else {
        return;
    };
    let e = Entity::from_bits(alvo);
    let (Some(col), Some(pose)) = (world.get::<Collider>(e), Pose::de(world, e)) else {
        return;
    };
    let m = Medida::de(col);
    let cor = resolve(ColorToken::Selection, theme);
    let fundo = resolve(ColorToken::Bg0, theme);
    let s = scene.inner_mut();
    let traco = if col.is_sensor {
        Stroke::new(TRACO_PX).with_dashes(0.0, TRACEJADO_PX)
    } else {
        Stroke::new(TRACO_PX)
    };
    s.stroke(&traco, Affine::IDENTITY, &Brush::Solid(cor), None, &contorno(&pose, &m, cam, win));
    for a in alcas(m.shape) {
        let (x, y) = no_ecra(&pose, &m, a, cam, win);
        let (px, py) = (f64::from(x), f64::from(y));
        if a == Alca::Miolo {
            let c = ph2d_vector::Circle::new((px, py), MIOLO_PX);
            s.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(cor), None, &c);
            continue;
        }
        let sob = (x - ponteiro.0).hypot(y - ponteiro.1) <= crate::AGARRE_PX;
        let m = if sob { ALCA_SOB_PX } else { ALCA_PX };
        let r = Rect::new(px - m, py - m, px + m, py + m);
        s.fill(Fill::NonZero, Affine::IDENTITY, &Brush::Solid(fundo), None, &r);
        s.stroke(&Stroke::new(TRACO_PX), Affine::IDENTITY, &Brush::Solid(cor), None, &r);
    }
}
