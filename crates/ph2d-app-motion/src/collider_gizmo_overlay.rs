//! **O DESENHO do gizmo do colisor da forma** (doc 109 §5) — o contorno em cada peça de cada forma
//! que o mostra, e as alças numa.
//!
//! A geometria vive em [`super::collider_gizmo`]; aqui mora só tinta, com o vocabulário do gizmo de
//! warp — as MESMAS constantes, porque *um manipulador que o artista já aprendeu não reaprende a cor
//! nem o tamanho da alça*: cor, casing escuro, quadrado para canto e losango para lado.
//!
//! ⭐⭐ **À FRENTE da arte, e com o traço FORTE** (report do dono, 2026-09-13: *«o collider deve
//! aparecer na frente da shape (z-index maior)»*) — e eram **duas** coisas:
//!
//! - **a ORDEM**: esta tinta é chamada na `fase_vector_overlays`, logo a seguir à codificação das
//!   formas do Motion e na MESMA cena. Chamada de onde estava (a `fase_selection_highlight`, que
//!   corre ANTES), o contorno ficava por BAIXO dos quadrados — medido, e o gate
//!   `the_collider_outline_is_painted_after_the_shape_art` prende-o;
//! - **o PESO**: a 1.ª redacção pintava as peças sem alças com o traço fino e apagado dos braços do
//!   warp. Hoje todo contorno leva o traço forte com casing, e o que distingue a peça com alças é
//!   ter alças.
//!
//! ## ⚠️ Tudo em pixels de TELA
//!
//! O caminho é construído já em coordenadas de tela e traçado com `Affine::IDENTITY`, porque
//! `stroke` **multiplica** a espessura pelo transform (a lei do cabeçalho do `warp_overlay`).

use super::collider_gizmo::{ColliderGizmoView, Peca, handles};
use super::warp_overlay::{
    CASE_PX, CASE_RGBA, CORNER_PX, HANDLE_RGBA, OUTLINE_PX, TANGENT_PX, TANGENT_RGBA,
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
    // ⚠️ A janela da CENA, pela porta única — ver `warp_gizmo::scene_window`.
    let to_screen =
        camera.world_to_screen_affine(super::warp_gizmo::scene_window(center_split, full_window));
    let pt = |w: [f32; 2]| to_screen * Point::new(f64::from(w[0]), f64::from(w[1]));
    let case = Brush::Solid(Color::new(CASE_RGBA));
    let dim = Brush::Solid(Color::new(TANGENT_RGBA));
    let brush = Brush::Solid(Color::new(HANDLE_RGBA));

    // ── os CONTORNOS de todas as peças de todas as formas, num caminho só ──
    let mut contornos = BezPath::new();
    let mut houve = false;
    for grupo in &v.grupos {
        for peca in &grupo.pecas {
            contorno(&mut contornos, peca, &pt);
            houve = true;
        }
    }
    if !houve {
        return;
    }
    vector_scene.inner_mut().stroke(
        &Stroke::new(OUTLINE_PX + CASE_PX * 2.0),
        Affine::IDENTITY,
        &case,
        None,
        &contornos,
    );
    vector_scene.inner_mut().stroke(
        &Stroke::new(OUTLINE_PX),
        Affine::IDENTITY,
        &brush,
        None,
        &contornos,
    );

    // ── as ALÇAS, na peça da forma seleccionada ──
    let Some((_, ativa)) = v.peca_ativa() else {
        return;
    };
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
