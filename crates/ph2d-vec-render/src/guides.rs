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

/// O que a guia está DIZENDO. As quatro espécies são afirmações diferentes, então têm marcas
/// diferentes — uma marca só para todas obrigaria o artista a adivinhar por que o ponto parou
/// ali (plano 25 §9, a W6).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GuideKind {
    /// *Alinhei com aquilo* — a linha tracejada entre os dois pontos alinhados.
    Align,
    /// *Caí na régua* — só a cruz diagonal no ponto de rede.
    Grid,
    /// *Estou SOBRE esta linha* — um anel no ponto de pouso.
    Curve,
    /// *Estou onde duas linhas se cruzam* — um mais, que é a figura do próprio fato.
    Crossing,
}

/// Um segmento de guia em world-space. Com `a == b` (as espécies de POSIÇÃO e a grade) só a
/// marca é desenhada — não há dois pontos a ligar.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Guide {
    pub a: [f64; 2],
    pub b: [f64; 2],
    pub kind: GuideKind,
}

/// Ciano das guias — mesmo tom dos vértices selecionados nos overlays.
const GUIDE: Color = Color::from_rgba8(90, 200, 235, 230);
/// Meia-aresta da cruz que marca a ponta de uma guia, em pixels de tela.
const TICK_PX: f64 = 4.0;
/// Raio do anel que marca um pouso SOBRE a curva, em pixels de tela.
const RING_PX: f64 = 4.5;

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
    let pen = Stroke::new(1.5);
    for g in guides {
        let a = transform * Point::new(g.a[0], g.a[1]);
        let b = transform * Point::new(g.b[0], g.b[1]);
        let mut ink = |p: &BezPath, s: &Stroke| {
            target
                .inner_mut()
                .stroke(s, Affine::IDENTITY, &Brush::Solid(GUIDE), None, p);
        };
        if g.kind == GuideKind::Align {
            let mut line = BezPath::new();
            line.move_to(a);
            line.line_to(b);
            ink(&line, &dashed);
            // Cruz nas duas pontas: a guia LIGA dois pontos, e ambos merecem marca.
            for p in [a, b] {
                ink(&diagonal_cross(p), &pen);
            }
            continue;
        }
        // As outras três são pontos: `a == b`, uma marca só, e a figura diz qual espécie é.
        let mark = match g.kind {
            GuideKind::Curve => ring(a),
            GuideKind::Crossing => upright_cross(a),
            GuideKind::Align | GuideKind::Grid => diagonal_cross(a),
        };
        ink(&mark, &pen);
    }
}

/// O ✕ da grade (e das pontas de uma guia de alinhamento).
fn diagonal_cross(p: Point) -> BezPath {
    let mut c = BezPath::new();
    c.move_to(Point::new(p.x - TICK_PX, p.y - TICK_PX));
    c.line_to(Point::new(p.x + TICK_PX, p.y + TICK_PX));
    c.move_to(Point::new(p.x - TICK_PX, p.y + TICK_PX));
    c.line_to(Point::new(p.x + TICK_PX, p.y - TICK_PX));
    c
}

/// O `+` do cruzamento — duas linhas que se cruzam, que é literalmente o fato relatado.
fn upright_cross(p: Point) -> BezPath {
    let mut c = BezPath::new();
    c.move_to(Point::new(p.x - TICK_PX, p.y));
    c.line_to(Point::new(p.x + TICK_PX, p.y));
    c.move_to(Point::new(p.x, p.y - TICK_PX));
    c.line_to(Point::new(p.x, p.y + TICK_PX));
    c
}

/// O anel do pouso sobre a curva. Quatro cúbicos com o `kappa` de sempre — a crate não
/// importa `Circle` de propósito (o overlay inteiro é `BezPath` em espaço de tela).
fn ring(p: Point) -> BezPath {
    const K: f64 = 0.552_284_749_83;
    let (r, k) = (RING_PX, RING_PX * K);
    let mut c = BezPath::new();
    c.move_to(Point::new(p.x + r, p.y));
    c.curve_to(
        Point::new(p.x + r, p.y + k),
        Point::new(p.x + k, p.y + r),
        Point::new(p.x, p.y + r),
    );
    c.curve_to(
        Point::new(p.x - k, p.y + r),
        Point::new(p.x - r, p.y + k),
        Point::new(p.x - r, p.y),
    );
    c.curve_to(
        Point::new(p.x - r, p.y - k),
        Point::new(p.x - k, p.y - r),
        Point::new(p.x, p.y - r),
    );
    c.curve_to(
        Point::new(p.x + k, p.y - r),
        Point::new(p.x + r, p.y - k),
        Point::new(p.x + r, p.y),
    );
    c.close_path();
    c
}
