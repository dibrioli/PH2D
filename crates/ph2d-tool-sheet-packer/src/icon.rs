//! Ícone do Sheet Packer — o glifo `BezPath` do pill.
//!
//! Uma **moldura com peças de tamanhos diferentes lá dentro**, que é literalmente o que a
//! ferramenta produz: uma folha e o arranjo. Desenhado no espaço 24×24 de sempre (Y para baixo,
//! origem no canto superior-esquerdo), traçado e não preenchido — a chrome desenha-o com
//! `Stroke::new(2.0)` e escala por `Affine::scale(chip_px / 24.0)`, igual aos irmãos da fila.
//!
//! ⚠️ **As peças de dentro têm tamanhos DIFERENTES de propósito**, pela mesma razão que o smoke:
//! com retângulos iguais o desenho leria como uma grade (uma tabela, um calendário), e o que
//! distingue esta ferramenta é justamente o encaixe de peças desiguais.
//!
//! O SVG canônico para o pipeline de design vive em `docs/design/icons/sheet-packer.svg`; este
//! `BezPath` espelha-o 1:1.

use ph2d_vector::{BezPath, Point};

/// Desenha um retângulo fechado no caminho.
fn rect(p: &mut BezPath, x0: f64, y0: f64, x1: f64, y1: f64) {
    p.move_to(Point::new(x0, y0));
    p.line_to(Point::new(x1, y0));
    p.line_to(Point::new(x1, y1));
    p.line_to(Point::new(x0, y1));
    p.close_path();
}

/// O glifo do Sheet Packer em espaço de desenho 24×24: a moldura da folha e três peças
/// desiguais encaixadas dentro dela.
pub fn sheet_packer_bezpath() -> BezPath {
    let mut p = BezPath::new();
    // A folha.
    rect(&mut p, 3.0, 3.0, 21.0, 21.0);
    // A peça grande, no canto superior-esquerdo — onde o empacotador de facto põe a maior.
    rect(&mut p, 6.0, 6.0, 13.0, 13.0);
    // A peça média, à direita dela.
    rect(&mut p, 15.0, 6.0, 18.0, 11.0);
    // A pequena, por baixo.
    rect(&mut p, 6.0, 15.0, 10.0, 18.0);
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector::Shape;

    /// O glifo tem de caber na caixa 24×24 que a chrome escala — um traço fora dela sairia
    /// recortado no pill, e o modo de falha é visual e mudo.
    #[test]
    fn the_glyph_fits_the_24x24_design_box() {
        let b = sheet_packer_bezpath().bounding_box();
        assert!(
            b.x0 >= 0.0 && b.y0 >= 0.0,
            "sai pelo canto superior-esquerdo"
        );
        assert!(
            b.x1 <= 24.0 && b.y1 <= 24.0,
            "sai pelo canto inferior-direito"
        );
    }

    /// ⚠️ As peças ficam DENTRO da moldura. Um desenho em que uma peça atravessa a borda leria
    /// como "espalhar", que é o oposto do que a ferramenta faz.
    #[test]
    fn the_pieces_sit_inside_the_sheet() {
        let b = sheet_packer_bezpath().bounding_box();
        // A moldura é o retângulo mais externo (3..21), então a caixa do glifo é a dela.
        assert!((b.x0 - 3.0).abs() < 1e-9 && (b.y0 - 3.0).abs() < 1e-9);
        assert!((b.x1 - 21.0).abs() < 1e-9 && (b.y1 - 21.0).abs() < 1e-9);
    }

    /// Quatro retângulos fechados: a folha e três peças.
    #[test]
    fn the_glyph_draws_four_closed_rectangles() {
        let n = sheet_packer_bezpath()
            .elements()
            .iter()
            .filter(|e| matches!(e, ph2d_vector::PathEl::ClosePath))
            .count();
        assert_eq!(n, 4);
    }
}
