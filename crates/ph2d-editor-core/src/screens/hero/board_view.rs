//! ⭐ **A VISTA de um quadro** (MiroClone) — com uma aba de quadro activa, a área central é dele:
//! a roda dá zoom à volta do cursor e arrastar (botão esquerdo ou do meio) move a vista. Nenhum
//! destes gestos chega à cena.
//!
//! A área é o `last_canvas` que o último `paint_hero_screen` publicou — a MESMA onde
//! [`ph2d_board_render::paint`] desenhou o quadro.

use super::HeroScreen;
use crate::zones::Rect;
use ph2d_board_model::Area;
use ph2d_host::{PointerButton, PointerKind};

/// Factor de zoom por píxel de roda: o mesmo da câmara da cena (`0.9` por linha de 16 px).
const WHEEL_ZOOM_PER_LINE: f64 = 0.9;
const WHEEL_LINE_PX: f64 = 16.0;

fn area_of(r: Rect) -> Area {
    [f64::from(r.x), f64::from(r.y), f64::from(r.w), f64::from(r.h)]
}

fn inside(r: Rect, x: f32, y: f32) -> bool {
    x >= r.x && y >= r.y && x < r.x + r.w && y < r.y + r.h
}

/// A roda sobre a área de um quadro activo. `true` = consumida (a cena não a vê).
pub fn wheel(hero: &mut HeroScreen, x: f32, y: f32, dy: f32) -> bool {
    let area = hero.last_canvas;
    if !inside(area, x, y) {
        return false;
    }
    let Some(board) = hero.documents.active_board_mut() else { return false };
    let factor = WHEEL_ZOOM_PER_LINE.powf(-f64::from(dy) / WHEEL_LINE_PX);
    board.camera.zoom_about(area_of(area), [f64::from(x), f64::from(y)], factor);
    true
}

/// Um botão sobre a área de um quadro activo. `on_canvas` = nenhum painel nem controlo por baixo.
/// `true` = consumido.
pub fn pointer(
    hero: &mut HeroScreen,
    kind: PointerKind,
    button: PointerButton,
    x: f32,
    y: f32,
    on_canvas: bool,
) -> bool {
    if hero.documents.active().is_none() {
        return false;
    }
    match kind {
        PointerKind::Down if on_canvas && inside(hero.last_canvas, x, y) => {
            if matches!(button, PointerButton::Primary | PointerButton::Middle) {
                hero.documents.pan_from = Some([f64::from(x), f64::from(y)]);
            }
            true
        }
        PointerKind::Up => hero.documents.pan_from.take().is_some(),
        _ => false,
    }
}

/// O cursor mexeu-se. `true` = um arrasto da vista do quadro está em curso e consumiu-o.
pub fn pointer_move(hero: &mut HeroScreen, x: f32, y: f32) -> bool {
    let Some(from) = hero.documents.pan_from else { return false };
    let to = [f64::from(x), f64::from(y)];
    if let Some(board) = hero.documents.active_board_mut() {
        board.camera.pan_by_screen(to[0] - from[0], to[1] - from[1]);
    }
    hero.documents.pan_from = Some(to);
    true
}

#[cfg(test)]
#[path = "board_view_tests.rs"]
mod tests;
