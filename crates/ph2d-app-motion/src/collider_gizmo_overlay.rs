//! **O DESENHO do gizmo do colisor da forma** (doc 109 §5) — os contornos em todas as peças e as
//! alças numa.
//!
//! A geometria vive em [`super::collider_gizmo`]; aqui mora só tinta, com o vocabulário do gizmo de
//! warp — as MESMAS constantes, porque *um manipulador que o artista já aprendeu não reaprende a cor
//! nem o tamanho da alça*: cor, casing escuro, quadrado para canto e losango para lado.
//!
//! ## ⚠️ Tudo em pixels de TELA
//!
//! O caminho é construído já em coordenadas de tela e traçado com `Affine::IDENTITY`, porque
//! `stroke` **multiplica** a espessura pelo transform (a lei do cabeçalho do `warp_overlay`).

use super::collider_gizmo::{ColliderGizmoView, Peca, handles};
use super::warp_overlay::{
    ARM_PX, CASE_PX, CASE_RGBA, CORNER_PX, HANDLE_RGBA, OUTLINE_PX, TANGENT_PX, TANGENT_RGBA,
};
use ph2d_contact::Forma;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Shape, Stroke, VectorScene};

/// A tolerância, em pixels, com que o círculo vira curvas.
const CIRCULO_TOL_PX: f64 = 0.1;

/// **Desenha o gizmo publicado.** No-op sem peças.
pub fn draw(
    v: &ColliderGizmoView,
    camera: &Camera2d,
    center_split: ph2d_editor_core::screens::layout::CenterSplit,
    full_window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    let Some(ativa) = v.pecas.first() else {
        return;
    };
    // ⚠️ A janela da CENA, pela porta única — ver `warp_gizmo::scene_window`.
    let to_screen =
        camera.world_to_screen_affine(super::warp_gizmo::scene_window(center_split, full_window));
    let pt = |w: [f32; 2]| to_screen * Point::new(f64::from(w[0]), f64::from(w[1]));
    let case = Brush::Solid(Color::new(CASE_RGBA));
    let dim = Brush::Solid(Color::new(TANGENT_RGBA));
    let brush = Brush::Solid(Color::new(HANDLE_RGBA));
    let traca = |scene: &mut VectorScene, path: &BezPath, px: f64, cor: &Brush| {
        scene.inner_mut().stroke(
            &Stroke::new(px + CASE_PX * 2.0),
            Affine::IDENTITY,
            &case,
            None,
            path,
        );
        scene
            .inner_mut()
            .stroke(&Stroke::new(px), Affine::IDENTITY, cor, None, path);
    };

    // ── as OUTRAS peças: o contorno, mais fino e apagado ──
    let mut outras = BezPath::new();
    for peca in &v.pecas[1..] {
        contorno(&mut outras, peca, &pt);
    }
    if !v.pecas[1..].is_empty() {
        traca(vector_scene, &outras, ARM_PX, &dim);
    }

    // ── a peça com as ALÇAS ──
    let mut dela = BezPath::new();
    contorno(&mut dela, ativa, &pt);
    traca(vector_scene, &dela, OUTLINE_PX, &brush);
    for h in handles(ativa) {
        let c = pt(h.world);
        let mut marca = BezPath::new();
        let canto =
            h.alca.x != 0 && h.alca.y != 0 || matches!(ativa.colisor.forma, Forma::Disco(_));
        if canto {
            marca.move_to(Point::new(c.x - CORNER_PX, c.y - CORNER_PX));
            marca.line_to(Point::new(c.x + CORNER_PX, c.y - CORNER_PX));
            marca.line_to(Point::new(c.x + CORNER_PX, c.y + CORNER_PX));
            marca.line_to(Point::new(c.x - CORNER_PX, c.y + CORNER_PX));
        } else {
            marca.move_to(Point::new(c.x, c.y - TANGENT_PX));
            marca.line_to(Point::new(c.x + TANGENT_PX, c.y));
            marca.line_to(Point::new(c.x, c.y + TANGENT_PX));
            marca.line_to(Point::new(c.x - TANGENT_PX, c.y));
        }
        marca.close_path();
        let cor = if canto { &brush } else { &dim };
        vector_scene
            .inner_mut()
            .fill(Fill::NonZero, Affine::IDENTITY, cor, None, &marca);
        vector_scene.inner_mut().stroke(
            &Stroke::new(CASE_PX * 2.0),
            Affine::IDENTITY,
            &case,
            None,
            &marca,
        );
    }
}

/// O contorno de uma peça, acrescentado a `path`, já em tela.
fn contorno(path: &mut BezPath, peca: &Peca, pt: &impl Fn([f32; 2]) -> Point) {
    match peca.colisor.forma {
        Forma::Caixa { .. } => {
            let Some(k) = peca.colisor.cantos(peca.p) else {
                return;
            };
            path.move_to(pt(k[0]));
            for c in &k[1..] {
                path.line_to(pt(*c));
            }
            path.close_path();
        }
        Forma::Disco(r) => {
            let c = peca.colisor.centro(peca.p);
            let (centro, borda) = (pt(c), pt([c[0] + r, c[1]]));
            let raio = (borda.x - centro.x).hypot(borda.y - centro.y);
            path.extend(Circle::new(centro, raio).path_elements(CIRCULO_TOL_PX));
        }
    }
}
