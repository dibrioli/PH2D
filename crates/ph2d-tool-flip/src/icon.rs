//! Ícone da tool Flip — duas páginas empilhadas (um *flipbook*) como
//! `kurbo::BezPath` num viewBox 24×24 (casa com `docs/design/icons/flip.svg`).
//! O chamador escala por `Affine::scale(chip_size / 24.0)` na hora de pintar.
//!
//! Duas molduras deslocadas na diagonal lêem como "pilha de quadros / animação
//! quadro-a-quadro" e são distintas dos outros glifos do topbar. Placeholder — o
//! Tool Studio pode trocar por um glifo mais rico depois.

use ph2d_vector::BezPath;

/// Ícone da tool Flip como `BezPath` num espaço 24×24: duas páginas empilhadas.
#[must_use]
pub fn flip_bezpath() -> BezPath {
    use ph2d_vector::Point;
    let mut p = BezPath::new();
    // Página de trás (deslocada pra cima-direita).
    p.move_to(Point::new(9.0, 4.0));
    p.line_to(Point::new(20.0, 4.0));
    p.line_to(Point::new(20.0, 15.0));
    p.line_to(Point::new(9.0, 15.0));
    p.close_path();
    // Página da frente (deslocada pra baixo-esquerda).
    p.move_to(Point::new(4.0, 9.0));
    p.line_to(Point::new(15.0, 9.0));
    p.line_to(Point::new(15.0, 20.0));
    p.line_to(Point::new(4.0, 20.0));
    p.close_path();
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_bezpath_has_expected_element_count() {
        // 2 × (MoveTo + LineTo×3 + ClosePath) = 10.
        assert_eq!(flip_bezpath().elements().len(), 10);
    }

    #[test]
    fn flip_bezpath_fits_in_24x24_viewbox() {
        use ph2d_vector::Shape;
        let bb = flip_bezpath().bounding_box();
        assert!(
            bb.x0 >= 0.0 && bb.y0 >= 0.0 && bb.x1 <= 24.0 && bb.y1 <= 24.0,
            "icon bbox {bb:?} outside 24x24 viewBox"
        );
    }
}
