//! **Desenhar um quadro** (MiroClone) numa área do ecrã: o fundo, a grelha de pontos e os
//! elementos vivos em ordem de z, recortados à área.
//!
//! ⚠️ W0: reconstrói tudo a cada quadro. O alvo (100 mil elementos, `docs/MiroClone/02_plano.md`
//! §2) pede fragmentos em cache por elemento e índice espacial — a W1 mede antes de os escrever.

use ph2d_board_model::{Area, Board, Camera, ElementKind, Rgba};
use ph2d_tokens::{ColorToken, Spacing, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Color, Rect, Shape, VectorScene};

/// Pinta `board` em `area` (`[x, y, w, h]` em px de ecrã).
pub fn paint(board: &Board, area: Area, scene: &mut VectorScene, theme: Theme) {
    let [x, y, w, h] = area;
    if !(w > 0.0 && h > 0.0) {
        return;
    }
    let clip = Rect::new(x, y, x + w, y + h);
    scene.push_clip(&clip);
    scene.fill_rect(clip, token(ColorToken::Bg1, theme));
    let dots = dot_grid(&board.camera, area);
    scene.fill_path(
        &dots,
        &Brush::Solid(token(ColorToken::GridLine, theme)),
        Affine::IDENTITY,
    );
    for el in board.doc.live_in_z_order() {
        let a = board.camera.to_screen(area, [el.x, el.y]);
        let b = board.camera.to_screen(area, [el.x + el.w, el.y + el.h]);
        let r = Rect::new(a[0], a[1], b[0], b[1]);
        if r.intersect(clip).area() <= 0.0 {
            continue;
        }
        match &el.kind {
            ElementKind::Rect { fill } => scene.fill_rect(r, doc_color(*fill)),
        }
    }
    scene.pop_layer();
}

/// O passo da grelha no MUNDO para esta vista: parte do passo de base e dobra/divide por 2 até o
/// espaçamento no ecrã ficar em `[mínimo, 2·mínimo)` — a mesma densidade de pontos em qualquer zoom.
#[must_use]
pub fn grid_step_world(camera: &Camera) -> f64 {
    let min_px = f64::from(Spacing::Xl.px());
    let mut step = f64::from(Spacing::Xl2.px());
    while step * camera.zoom < min_px {
        step *= 2.0;
    }
    while step * camera.zoom >= 2.0 * min_px {
        step /= 2.0;
    }
    step
}

/// Os pontos da grelha visíveis em `area`, como UM caminho (uma chamada de preenchimento).
#[must_use]
pub fn dot_grid(camera: &Camera, area: Area) -> BezPath {
    let [x, y, w, h] = area;
    let step = grid_step_world(camera);
    let half = f64::from(Spacing::Xxs.px()) / 2.0;
    let lo = camera.to_world(area, [x, y]);
    let hi = camera.to_world(area, [x + w, y + h]);
    let mut path = BezPath::new();
    let mut gx = (lo[0] / step).ceil() * step;
    while gx <= hi[0] {
        let mut gy = (lo[1] / step).ceil() * step;
        while gy <= hi[1] {
            let [sx, sy] = camera.to_screen(area, [gx, gy]);
            let dot = Rect::new(sx - half, sy - half, sx + half, sy + half);
            path.extend(dot.path_elements(0.1));
            gy += step;
        }
        gx += step;
    }
    path
}

fn token(t: ColorToken, theme: Theme) -> Color {
    let c = t.resolve(theme);
    Color::from_rgba8(c.r, c.g, c.b, c.a)
}

fn doc_color(Rgba([r, g, b, a]): Rgba) -> Color {
    Color::from_rgba8(r, g, b, a)
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
