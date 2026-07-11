//! Smart guides + grid do editor vetorial (ADR-0108): o feedback visual do snap.
//!
//! Uma guia é o segmento **entre o ponto que encaixou e o ponto em que encaixou**.
//! Desenhá-la assim (em vez de uma linha infinita) mostra *com o quê* a forma
//! alinhou, não só *que* alinhou. Encaixe de grade não tem contraparte do outro
//! lado, então vira uma cruz no ponto.
//!
//! Tudo em screen-space depois do `transform` da câmera: a espessura e o tracejado
//! ficam constantes em pixels, como o resto dos overlays.
//!
//! Aqui NÃO se desenha grade: o editor já tem o overlay universal
//! (`ph2d_editor_core::grid_snap::render`), com nove tipos de grade. Este módulo só
//! explica o encaixe.

use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Stroke, VectorScene};

/// Um segmento de guia em world-space. `a == b` (encaixe de grade) vira só a cruz.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Guide {
    pub a: [f64; 2],
    pub b: [f64; 2],
    /// Encaixe de grade (sem forma-alvo do outro lado).
    pub grid: bool,
}

/// Ciano das guias — mesmo tom dos vértices selecionados nos overlays.
const GUIDE: Color = Color::from_rgba8(90, 200, 235, 230);
/// Meia-aresta da cruz que marca a ponta de uma guia, em pixels de tela.
const TICK_PX: f64 = 4.0;

/// Cursor de texto (modo Text): um segmento vertical SÓLIDO no ponto de inserção,
/// screen-space via `transform`. `a`/`b` são as pontas em world (base e topo).
pub fn draw_text_caret(a: [f64; 2], b: [f64; 2], transform: Affine, target: &mut VectorScene) {
    let pa = transform * Point::new(a[0], a[1]);
    let pb = transform * Point::new(b[0], b[1]);
    let mut line = BezPath::new();
    line.move_to(pa);
    line.line_to(pb);
    target.inner_mut().stroke(
        &Stroke::new(1.5),
        Affine::IDENTITY,
        &Brush::Solid(GUIDE),
        None,
        &line,
    );
}

/// Desenha as guias de snap ativas neste frame (screen-space, via `transform`).
pub fn draw_snap_guides(guides: &[Guide], transform: Affine, target: &mut VectorScene) {
    let dashed = Stroke::new(1.0).with_dashes(0.0, [4.0, 3.0]);
    for g in guides {
        let a = transform * Point::new(g.a[0], g.a[1]);
        let b = transform * Point::new(g.b[0], g.b[1]);
        if !g.grid {
            let mut line = BezPath::new();
            line.move_to(a);
            line.line_to(b);
            target
                .inner_mut()
                .stroke(&dashed, Affine::IDENTITY, &Brush::Solid(GUIDE), None, &line);
        }
        // Cruz nas duas pontas (uma só quando coincidem).
        for p in [a, b] {
            let mut cross = BezPath::new();
            cross.move_to(Point::new(p.x - TICK_PX, p.y - TICK_PX));
            cross.line_to(Point::new(p.x + TICK_PX, p.y + TICK_PX));
            cross.move_to(Point::new(p.x - TICK_PX, p.y + TICK_PX));
            cross.line_to(Point::new(p.x + TICK_PX, p.y - TICK_PX));
            target.inner_mut().stroke(
                &Stroke::new(1.5),
                Affine::IDENTITY,
                &Brush::Solid(GUIDE),
                None,
                &cross,
            );
        }
    }
}
