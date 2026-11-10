//! **As marcas do Lint** (P13 U-Lint): o contorno de cada célula de um aviso escolhido na janela
//! LINT, por cima da arte. Um só caminho para todas as células (um `stroke` com halo).

use ph2d_vector::{Affine, BezPath, Color, Point, VectorScene};

use crate::view::stroke_halo;

/// Contorna as células `celulas` (`[x, y]` do sprite) com a cor `cor` (um token resolvido).
pub fn draw_lint_marks(
    scene: &mut VectorScene,
    celulas: &[[u32; 2]],
    to_screen: Affine,
    cor: Color,
    width_px: f64,
) {
    if celulas.is_empty() {
        return;
    }
    let mut path = BezPath::new();
    for &[x, y] in celulas {
        let (x, y) = (f64::from(x), f64::from(y));
        path.move_to(to_screen * Point::new(x, y));
        path.line_to(to_screen * Point::new(x + 1.0, y));
        path.line_to(to_screen * Point::new(x + 1.0, y + 1.0));
        path.line_to(to_screen * Point::new(x, y + 1.0));
        path.close_path();
    }
    stroke_halo(scene, &path, width_px, cor);
}
