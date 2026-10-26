//! **A selecção por cima da arte** (P6): as formigas a marchar pela borda da máscara (as arestas
//! das células, em células do documento). A moldura solta é o gizmo do sprite da casa (P6b).
//! Cores: o núcleo claro e o anel escuro do Draw — o fundo é a arte, não o chrome.

use ph2d_pixel_core::selection::Mask;
use ph2d_vector::{Affine, BezPath, Point, Stroke as Pen, VectorScene};

use crate::view::{CURSOR_CORE, HALO, HALO_PX};

/// O comprimento de cada traço (e de cada vão) das formigas, em píxeis de ECRÃ.
pub const ANTS_DASH_PX: f64 = 4.0; // LITERAL-PX-OK: padrão das formigas do Aseprite, em píxeis de ecrã

/// Um segmento de aresta entre cantos de células `((x0, y0), (x1, y1))`.
pub type Edge = ((i32, i32), (i32, i32));

/// **A borda da máscara**: os segmentos MÁXIMOS onde uma célula seleccionada encosta numa que não
/// é (fora do sprite conta como não seleccionada), horizontais e depois verticais.
#[must_use]
pub fn mask_edges(m: &Mask) -> Vec<Edge> {
    let (w, h) = (m.width() as i32, m.height() as i32);
    let mut out = Vec::new();
    for y in 0..=h {
        let mut run: Option<i32> = None;
        for x in 0..=w {
            let edge = x < w && m.get(x, y - 1) != m.get(x, y);
            match (edge, run) {
                (true, None) => run = Some(x),
                (false, Some(x0)) => {
                    out.push(((x0, y), (x, y)));
                    run = None;
                }
                _ => {}
            }
        }
    }
    for x in 0..=w {
        let mut run: Option<i32> = None;
        for y in 0..=h {
            let edge = y < h && m.get(x - 1, y) != m.get(x, y);
            match (edge, run) {
                (true, None) => run = Some(y),
                (false, Some(y0)) => {
                    out.push(((x, y0), (x, y)));
                    run = None;
                }
                _ => {}
            }
        }
    }
    out
}

/// **As formigas**: as arestas `edges` (células do documento) pelo afim `to_screen`, o anel escuro
/// cheio por baixo e o traço claro tracejado por cima, deslocado `phase_px` (o relógio do host).
pub fn draw_marching_ants(
    scene: &mut VectorScene,
    edges: &[Edge],
    to_screen: Affine,
    phase_px: f64,
    width_px: f64,
) {
    if edges.is_empty() {
        return;
    }
    let mut path = BezPath::new();
    for &((x0, y0), (x1, y1)) in edges {
        path.move_to(to_screen * Point::new(f64::from(x0), f64::from(y0)));
        path.line_to(to_screen * Point::new(f64::from(x1), f64::from(y1)));
    }
    let nucleo = 2.0 * width_px;
    let s = scene.inner_mut();
    s.stroke(
        &Pen::new(nucleo + 2.0 * HALO_PX),
        Affine::IDENTITY,
        HALO,
        None,
        &path,
    );
    let dash = Pen::new(nucleo).with_dashes(phase_px, [ANTS_DASH_PX, ANTS_DASH_PX]);
    s.stroke(&dash, Affine::IDENTITY, CURSOR_CORE, None, &path);
}

#[cfg(test)]
mod tests {
    use super::mask_edges;
    use ph2d_pixel_core::selection::Mask;

    #[test]
    fn the_outline_of_a_box_is_four_maximal_edges_and_a_hole_adds_four() {
        let mut m = Mask::marquee(8, 8, (1, 1), (4, 3));
        assert_eq!(
            mask_edges(&m),
            vec![
                ((1, 1), (5, 1)),
                ((1, 4), (5, 4)),
                ((1, 1), (1, 4)),
                ((5, 1), (5, 4))
            ]
        );
        m.set(2, 2, false);
        assert_eq!(mask_edges(&m).len(), 4 + 4);
        assert!(mask_edges(&Mask::empty(4, 4)).is_empty());
        // a borda do sprite também é borda
        assert_eq!(mask_edges(&Mask::full(3, 2)).len(), 4);
    }
}
