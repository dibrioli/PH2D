//! A grelha de pontos do fundo do quadro (módulo filho de [`crate`]: o ficheiro estava no tecto de LOC).

use ph2d_board_model::{Area, Camera};
use ph2d_tokens::Spacing;
use ph2d_vector::{BezPath, Rect, Shape as _};

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
