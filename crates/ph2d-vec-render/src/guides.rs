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
    /// *Estou PRESO nesta guia* — um quadradinho cheio sobre a linha, como um alfinete.
    ///
    /// ⚠️ A marca é **independente do eixo** de propósito. Um traço perpendicular seria mais
    /// bonito e exigiria que este módulo soubesse a direção da guia — um segundo lugar onde a
    /// pergunta *"esta guia é horizontal?"* passaria a ser respondida, e o dia em que as duas
    /// respostas divergirem o alfinete aponta para o lado errado sem nada quebrar.
    GuideHit,
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
/// A guia do DOCUMENTO: o mesmo ciano, mais apagado. Ela é permanente e atravessa a tela, então
/// no brilho da guia de snap competiria com o desenho; recuar o alfa a põe atrás da arte sem
/// mudar a família de cor — as duas são a mesma espécie de afirmação (*alinhe-se aqui*).
const DOC_GUIDE: Color = Color::from_rgba8(90, 200, 235, 150);
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
        // As outras são pontos: `a == b`, uma marca só, e a figura diz qual espécie é.
        let mark = match g.kind {
            GuideKind::Curve => ring(a),
            GuideKind::Crossing => upright_cross(a),
            GuideKind::GuideHit => pin(a),
            GuideKind::Align | GuideKind::Grid => diagonal_cross(a),
        };
        ink(&mark, &pen);
    }
}

/// As **guias do DOCUMENTO** — as linhas que o artista arrastou da régua.
///
/// Traço **SÓLIDO e fino**, contra o tracejado das guias de snap: as duas são referências de
/// alinhamento e por isso partilham a cor, mas uma é permanente (foi autorada) e a outra vive
/// um frame (explica o encaixe vivo). Sólido × tracejado é a distinção que se lê sem legenda,
/// e o comprimento reforça: a do documento **atravessa a tela**, a de snap liga dois pontos.
///
/// `canvas` é o retângulo visível em pixels de tela (`[x, y, w, h]`); a linha é recortada a
/// ele. `transform` é o mundo→tela — a linha é construída a partir de UM ponto e da DIREÇÃO,
/// então ela continua correta se um dia a câmera girar (o que o par de extremos não daria).
pub fn draw_document_guides(
    guides: &[ph2d_guides::Guide],
    canvas: [f64; 4],
    transform: Affine,
    target: &mut VectorScene,
) {
    let thin = Stroke::new(1.0);
    for g in guides {
        // Um ponto QUALQUER da guia, e um segundo deslocado ao longo dela.
        let (p0, p1) = match g.axis {
            ph2d_guides::GuideAxis::Horizontal => ([0.0, g.pos], [1.0, g.pos]),
            ph2d_guides::GuideAxis::Vertical => ([g.pos, 0.0], [g.pos, 1.0]),
        };
        let a = transform * Point::new(p0[0], p0[1]);
        let b = transform * Point::new(p1[0], p1[1]);
        let Some((s, e)) = clip_line_to_rect(a, Point::new(b.x - a.x, b.y - a.y), canvas) else {
            continue;
        };
        let mut line = BezPath::new();
        line.move_to(s);
        line.line_to(e);
        target.inner_mut().stroke(
            &thin,
            Affine::IDENTITY,
            &Brush::Solid(DOC_GUIDE),
            None,
            &line,
        );
    }
}

/// Recorta a reta `p + t·d` ao retângulo `[x, y, w, h]`, devolvendo os dois pontos de saída.
/// `None` quando a reta não cruza o retângulo (ou `d` é degenerada).
///
/// Método das fatias (slab), o mesmo do Liang-Barsky: cada par de bordas dá um intervalo em
/// `t`, e a interseção dos dois é o trecho visível. Uma direção com componente zero num eixo
/// é o caso NORMAL aqui (toda guia é paralela a um eixo), então ele é tratado no ramo, não por
/// divisão por quase-zero.
fn clip_line_to_rect(p: Point, d: Point, rect: [f64; 4]) -> Option<(Point, Point)> {
    let (x0, y0, x1, y1) = (rect[0], rect[1], rect[0] + rect[2], rect[1] + rect[3]);
    let (mut lo, mut hi) = (f64::NEG_INFINITY, f64::INFINITY);
    for (num0, num1, den) in [(x0 - p.x, x1 - p.x, d.x), (y0 - p.y, y1 - p.y, d.y)] {
        if den == 0.0 {
            // Paralela a esta fatia: ou está dentro dela para todo `t`, ou nunca.
            if num0 > 0.0 || num1 < 0.0 {
                return None;
            }
            continue;
        }
        let (a, b) = (num0 / den, num1 / den);
        let (a, b) = if a <= b { (a, b) } else { (b, a) };
        lo = lo.max(a);
        hi = hi.min(b);
    }
    (lo <= hi && lo.is_finite() && hi.is_finite()).then(|| {
        (
            Point::new(p.x + d.x * lo, p.y + d.y * lo),
            Point::new(p.x + d.x * hi, p.y + d.y * hi),
        )
    })
}

/// O alfinete do encaixe numa guia: um quadradinho, a única das cinco figuras que é uma ÁREA.
fn pin(p: Point) -> BezPath {
    let r = TICK_PX * 0.6;
    let mut c = BezPath::new();
    c.move_to(Point::new(p.x - r, p.y - r));
    c.line_to(Point::new(p.x + r, p.y - r));
    c.line_to(Point::new(p.x + r, p.y + r));
    c.line_to(Point::new(p.x - r, p.y + r));
    c.close_path();
    c
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
