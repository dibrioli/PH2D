//! A geometria de ecrã partilhada pelos três pintores de caixa (`paint.rs`): os quatro cantos
//! projectados, as zonas de rodar e o contorno. Uma conta só — eram três cópias da mesma
//! rotação dos cantos, e a afim do pai ([`super::model`]) entra aqui uma vez.

use super::hit::ids;
use super::model::MODEL_IDENTITY;
use super::paint::{GizmoView, HANDLE_SIZE_PX, ROTATE_HANDLE_OFFSET, world_to_screen};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_vector::VectorScene;

/// Os quatro cantos `[TL, TR, BL, BR]` em pixels de ecrã: rodados por `view.rotation` em torno do
/// centro da caixa e projectados por `câmara · model` — com um pai cisalhado, o paralelogramo.
pub(super) fn corners_screen(view: &GizmoView) -> [[f32; 2]; 4] {
    let cx_w = (view.bbox_min_world[0] + view.bbox_max_world[0]) * 0.5;
    let cy_w = (view.bbox_min_world[1] + view.bbox_max_world[1]) * 0.5;
    let hx_w = (view.bbox_max_world[0] - view.bbox_min_world[0]) * 0.5;
    let hy_w = (view.bbox_max_world[1] - view.bbox_min_world[1]) * 0.5;
    // Offsets locais: TL = (-hx, +hy), TR = (+hx, +hy), BL = (-hx, -hy), BR = (+hx, -hy) — Y de
    // mundo para cima; o `world_to_screen` faz a inversão.
    let cos_r = view.rotation.cos();
    let sin_r = view.rotation.sin();
    let rotate_world = |lx: f32, ly: f32| -> [f32; 2] {
        [
            cx_w + lx * cos_r - ly * sin_r,
            cy_w + lx * sin_r + ly * cos_r,
        ]
    };
    [
        world_to_screen(view, rotate_world(-hx_w, hy_w)),
        world_to_screen(view, rotate_world(hx_w, hy_w)),
        world_to_screen(view, rotate_world(-hx_w, -hy_w)),
        world_to_screen(view, rotate_world(hx_w, -hy_w)),
    ]
}

/// As quatro zonas de rodar `(id, rect)`, "para lá" de cada canto ao longo dos eixos LOCAIS da
/// caixa no ecrã. Sem pai, os eixos saem da rotação (o caminho de sempre); com pai, das ARESTAS
/// projectadas — sob cisalhamento os dois eixos deixam de ser perpendiculares.
pub(super) fn rotate_rects(view: &GizmoView, c: [[f32; 2]; 4]) -> [(NodeId, Rect); 4] {
    let [tl, tr, bl, br] = c;
    let (ux, uy) = if view.model == MODEL_IDENTITY {
        rotation_axes(view.rotation)
    } else {
        edge_axes(tl, tr, bl).unwrap_or_else(|| rotation_axes(view.rotation))
    };
    [
        (ids::GIZMO_ROTATE_TL, outer_rect(tl, ux, uy, -1.0, 1.0)),
        (ids::GIZMO_ROTATE_TR, outer_rect(tr, ux, uy, 1.0, 1.0)),
        (ids::GIZMO_ROTATE_BL, outer_rect(bl, ux, uy, -1.0, -1.0)),
        (ids::GIZMO_ROTATE_BR, outer_rect(br, ux, uy, 1.0, -1.0)),
    ]
}

/// Os eixos locais `(+X, +Y)` no ecrã de uma caixa rodada por `rotation`. O `world_to_screen`
/// inverte Y, então cada componente Y de mundo troca de sinal: +X = `(cos, -sin)`, +Y =
/// `(-sin, -cos)` (a versão antiga perdia o sinal de +X e a zona saía da quina).
fn rotation_axes(rotation: f32) -> ((f32, f32), (f32, f32)) {
    let cos_r = rotation.cos();
    let sin_r = rotation.sin();
    ((cos_r, -sin_r), (-sin_r, -cos_r))
}

/// Os eixos locais medidos nas arestas projectadas (`TL→TR` e `BL→TL`), ou `None` numa caixa
/// degenerada.
fn edge_axes(tl: [f32; 2], tr: [f32; 2], bl: [f32; 2]) -> Option<((f32, f32), (f32, f32))> {
    let unit = |a: [f32; 2], b: [f32; 2]| {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let n = (dx * dx + dy * dy).sqrt();
        (n > f32::EPSILON).then(|| (dx / n, dy / n))
    };
    Some((unit(tl, tr)?, unit(bl, tl)?))
}

/// Rect for the rotate-hover region just outside a corner handle.
/// `dx`/`dy` are unit-direction signs indicating which way "outside"
/// is for that corner (-1 = up/left, +1 = down/right). Kept as a
/// legacy helper used by the gizmo's tests + axis-aligned smoke
/// path before M14.7 polish added the oriented variant.
#[allow(dead_code)]
pub(super) fn corner_outer_rect(cx: f32, cy: f32, dx: f32, dy: f32) -> Rect {
    let half_handle = HANDLE_SIZE_PX * 0.5;
    let offset = half_handle + ROTATE_HANDLE_OFFSET;
    let x = if dx < 0.0 {
        cx - offset
    } else {
        cx + half_handle
    };
    let y = if dy < 0.0 {
        cy - offset
    } else {
        cy + half_handle
    };
    Rect::new(x, y, ROTATE_HANDLE_OFFSET, ROTATE_HANDLE_OFFSET)
}

/// A zona de rodar de um canto, centrada no ponto empurrado para FORA ao longo dos eixos locais
/// `(ux, uy)` do ecrã. A forma continua um AABB de ecrã (o `HitIndex` só testa `Rect::contains`);
/// a 45° as zonas vizinhas sobrepõem-se um pouco, o que só alarga o gesto de rodar.
fn outer_rect(corner: [f32; 2], ux: (f32, f32), uy: (f32, f32), lx: f32, ly: f32) -> Rect {
    let half_handle = HANDLE_SIZE_PX * 0.5;
    let offset = half_handle + ROTATE_HANDLE_OFFSET * 0.5;
    let off_x = lx * offset;
    let off_y = ly * offset;
    let center_x = corner[0] + ux.0 * off_x + uy.0 * off_y;
    let center_y = corner[1] + ux.1 * off_x + uy.1 * off_y;
    Rect::new(
        center_x - ROTATE_HANDLE_OFFSET * 0.5,
        center_y - ROTATE_HANDLE_OFFSET * 0.5,
        ROTATE_HANDLE_OFFSET,
        ROTATE_HANDLE_OFFSET,
    )
}

pub(super) fn midpoint(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5]
}

/// Stroke a 4-corner closed polygon as the gizmo's oriented bbox outline — with rotation (or a
/// sheared parent) the edges no longer align with screen axes, so the corner sequence is walked
/// directly via `BezPath`.
pub(super) fn paint_oriented_bbox(
    scene: &mut VectorScene,
    corners: [[f32; 2]; 4],
    width: f32,
    color: ph2d_vector::Color,
) {
    let mut path = ph2d_vector::BezPath::new();
    path.move_to((corners[0][0] as f64, corners[0][1] as f64));
    for c in corners.iter().skip(1) {
        path.line_to((c[0] as f64, c[1] as f64));
    }
    path.close_path();
    scene.inner_mut().stroke(
        &ph2d_vector::Stroke::new(width as f64),
        ph2d_vector::Affine::IDENTITY,
        color,
        None,
        &path,
    );
}
