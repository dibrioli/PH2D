//! **A geometria das formas do Quadro** (MiroClone, W1): o contorno de cada [`ShapeType`] numa
//! caixa, os cantos redondos, a caixa onde o texto vive, e o acerto de um ponto.
//!
//! Tudo em coordenadas LOCAIS da caixa `(0,0)–(w,h)`; [`to_world`] leva-as ao mundo (posição +
//! rotação à volta do centro). As formas do catálogo vectorial entram pelo `cook` dele (a mesma
//! geometria que o modo Vector desenha); rectângulo, losango e triângulo são daqui porque o canto
//! redondo do quadro segue outra regra ([`corner_radius`]).

use ph2d_board_model::{Element, Shape, ShapeType};
use ph2d_vec_scene::{ShapeKind, cook};
use ph2d_vector::{Affine, BezPath, ParamCurveNearest, PathSeg, Point, RoundedRect, Shape as _};

/// Folga entre o texto e a caixa onde ele vive, em unidades do mundo.
pub const TEXT_PADDING: f64 = 5.0;

/// O contorno de uma forma: o que se PREENCHE (contornos fechados) e as linhas de construção (a
/// tampa do cilindro, as barras do processo predefinido) que só se desenham.
#[derive(Clone, Debug)]
pub struct Outline {
    pub fill: BezPath,
    pub lines: BezPath,
}

/// O raio do canto redondo para uma caixa cujo lado menor é `min_side`: um quarto do lado, até
/// um tecto fixo (o «raio adaptativo»), para que uma caixa grande não vire um comprimido.
/// ⚠️ Regra a conferir contra o oráculo (Excalidraw, `roughness 0`) no passo do oráculo da W1.
#[must_use]
pub fn corner_radius(min_side: f64) -> f64 {
    const FRACTION: f64 = 0.25;
    const CAP: f64 = 32.0;
    (min_side.abs() * FRACTION).min(CAP)
}

/// O contorno de `shape` numa caixa `w × h` (local).
#[must_use]
pub fn outline(shape: &Shape, w: f64, h: f64) -> Outline {
    let round = shape.style.round;
    let polygon = |pts: &[[f64; 2]]| Outline {
        fill: if round {
            rounded_polygon(pts, corner_radius(w.min(h)))
        } else {
            sharp_polygon(pts)
        },
        lines: BezPath::new(),
    };
    match shape.kind {
        ShapeType::Rectangle => {
            let r = if round { corner_radius(w.min(h)) } else { 0.0 };
            Outline {
                fill: RoundedRect::new(0.0, 0.0, w, h, r).to_path(0.1),
                lines: BezPath::new(),
            }
        }
        ShapeType::Diamond => {
            polygon(&[[w / 2.0, 0.0], [w, h / 2.0], [w / 2.0, h], [0.0, h / 2.0]])
        }
        ShapeType::Triangle => polygon(&[[w / 2.0, 0.0], [w, h], [0.0, h]]),
        other => {
            let kind = vec_kind(other);
            let path = cook(kind, [0.0, 0.0], [w, h], &kind.defaults());
            Outline {
                fill: ph2d_vec_render::build_fill_bezpath(&path),
                lines: ph2d_vec_render::build_lines_bezpath(&path),
            }
        }
    }
}

/// A forma do catálogo vectorial que desenha cada tipo do quadro (as três daqui não passam).
fn vec_kind(t: ShapeType) -> ShapeKind {
    match t {
        ShapeType::Rectangle => ShapeKind::Rectangle,
        ShapeType::Ellipse => ShapeKind::Ellipse,
        ShapeType::Diamond => ShapeKind::Diamond,
        ShapeType::Triangle => ShapeKind::Polygon,
        ShapeType::Pill => ShapeKind::Pill,
        ShapeType::Parallelogram => ShapeKind::Parallelogram,
        ShapeType::Trapezoid => ShapeKind::Trapezoid,
        ShapeType::Hexagon => ShapeKind::HexagonFlat,
        ShapeType::Cylinder => ShapeKind::Cylinder,
        ShapeType::Document => ShapeKind::Document,
        ShapeType::PredefinedProcess => ShapeKind::PredefinedProcess,
        ShapeType::OffPage => ShapeKind::OffPage,
        ShapeType::Delay => ShapeKind::Delay,
        ShapeType::Display => ShapeKind::Display,
        ShapeType::SpeechRect => ShapeKind::SpeechRect,
        ShapeType::Cloud => ShapeKind::Cloud,
        ShapeType::Star => ShapeKind::Star,
        ShapeType::ArrowRight => ShapeKind::ArrowRight,
    }
}

fn sharp_polygon(pts: &[[f64; 2]]) -> BezPath {
    let mut p = BezPath::new();
    for (i, q) in pts.iter().enumerate() {
        let q = Point::new(q[0], q[1]);
        if i == 0 {
            p.move_to(q);
        } else {
            p.line_to(q);
        }
    }
    p.close_path();
    p
}

/// Polígono com cada canto trocado por uma curva quadrática de raio `r` (limitado a metade de cada
/// aresta, para dois cantos vizinhos nunca se cruzarem).
fn rounded_polygon(pts: &[[f64; 2]], r: f64) -> BezPath {
    let n = pts.len();
    let pt = |i: usize| Point::new(pts[i % n][0], pts[i % n][1]);
    let toward = |from: Point, to: Point| {
        let d = to - from;
        let len = d.hypot();
        if len <= 0.0 {
            from
        } else {
            from + d * (r.min(len / 2.0) / len)
        }
    };
    let mut p = BezPath::new();
    for i in 0..n {
        let (prev, v, next) = (pt(i + n - 1), pt(i), pt(i + 1));
        let (a, b) = (toward(v, prev), toward(v, next));
        if i == 0 {
            p.move_to(a);
        } else {
            p.line_to(a);
        }
        p.quad_to(v, b);
    }
    p.close_path();
    p
}

/// A caixa onde o texto de cada tipo vive, em FRACÇÕES da caixa `[x0, y0, x1, y1]` — a parte da
/// forma onde uma linha inteira cabe sem tocar no contorno (o miolo da elipse e do losango, o
/// corpo do balão sem o bico, o cilindro sem a tampa…), tirada dos parâmetros de nascença do
/// catálogo vectorial.
#[must_use]
pub fn text_frame(t: ShapeType) -> [f64; 4] {
    // 1/2 − 1/(2√2): o rectângulo inscrito numa elipse com a mesma proporção.
    const IN_ELLIPSE: f64 = 0.146_446_609_406_726_24;
    match t {
        ShapeType::Rectangle => [0.0, 0.0, 1.0, 1.0],
        ShapeType::Ellipse => [IN_ELLIPSE, IN_ELLIPSE, 1.0 - IN_ELLIPSE, 1.0 - IN_ELLIPSE],
        ShapeType::Diamond => [0.25, 0.25, 0.75, 0.75],
        ShapeType::Triangle => [0.25, 0.5, 0.75, 1.0],
        ShapeType::Pill => [0.15, 0.0, 0.85, 1.0],
        ShapeType::Parallelogram | ShapeType::Trapezoid | ShapeType::Hexagon => {
            [0.2, 0.0, 0.8, 1.0]
        }
        ShapeType::Cylinder => [0.0, 0.2, 1.0, 0.9],
        ShapeType::Document => [0.0, 0.0, 1.0, 0.85],
        ShapeType::PredefinedProcess => [0.12, 0.0, 0.88, 1.0],
        ShapeType::OffPage => [0.0, 0.0, 1.0, 0.7],
        ShapeType::Delay => [0.0, 0.0, 0.8, 1.0],
        ShapeType::Display => [0.2, 0.0, 0.9, 1.0],
        ShapeType::SpeechRect => [0.0, 0.0, 1.0, 0.7],
        ShapeType::Cloud => [0.15, 0.2, 0.85, 0.8],
        ShapeType::Star => [0.3, 0.35, 0.7, 0.75],
        ShapeType::ArrowRight => [0.0, 0.3, 0.6, 0.7],
    }
}

/// A caixa local `[x, y, w, h]` onde o texto quebra e se centra, já com a folga.
#[must_use]
pub fn text_rect(t: ShapeType, w: f64, h: f64) -> [f64; 4] {
    let [fx0, fy0, fx1, fy1] = text_frame(t);
    let (x0, y0) = (fx0 * w + TEXT_PADDING, fy0 * h + TEXT_PADDING);
    let (x1, y1) = (fx1 * w - TEXT_PADDING, fy1 * h - TEXT_PADDING);
    [x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0)]
}

/// A altura que a forma precisa para o texto de altura `text_h` caber na caixa dela — o «a forma
/// cresce» (cresce para baixo, o topo fica onde está). Nunca menos que `h`.
#[must_use]
pub fn height_for_text(t: ShapeType, h: f64, text_h: f64) -> f64 {
    let [_, fy0, _, fy1] = text_frame(t);
    let need = (text_h + 2.0 * TEXT_PADDING) / (fy1 - fy0);
    h.max(need)
}

/// Onde começa o bloco de texto (local): a caixa do texto, com o bloco centrado na vertical quando
/// cabe (quando não cabe, a forma cresce — [`height_for_text`] — e o topo encosta).
#[must_use]
pub fn text_origin(t: ShapeType, w: f64, h: f64, block_h: f64) -> [f64; 2] {
    let [x, y, _, rh] = text_rect(t, w, h);
    [x, y + ((rh - block_h) / 2.0).max(0.0)]
}

/// Local → mundo: a caixa no sítio dela, rodada à volta do centro.
#[must_use]
pub fn to_world(el: &Element) -> Affine {
    let [cx, cy] = el.center();
    Affine::translate((cx, cy))
        * Affine::rotate(el.angle)
        * Affine::translate((-el.w / 2.0, -el.h / 2.0))
}

/// O contorno de `el` já no mundo.
#[must_use]
pub fn world_outline(el: &Element) -> Option<Outline> {
    let shape = el.shape()?;
    let o = outline(shape, el.w, el.h);
    let t = to_world(el);
    Some(Outline {
        fill: t * o.fill,
        lines: t * o.lines,
    })
}

/// O ponto `p` (mundo) acerta em `el`? Dentro do contorno, ou a menos de `tol` (mundo) dele — a
/// borda de uma forma sem preenchimento continua agarrável.
#[must_use]
pub fn hit(el: &Element, p: [f64; 2], tol: f64) -> bool {
    let Some(shape) = el.shape() else {
        return false;
    };
    let [ux, uy] = el.unrotate(p);
    let local = Point::new(ux - el.x, uy - el.y);
    let o = outline(shape, el.w, el.h);
    if o.fill.contains(local) {
        return true;
    }
    o.fill
        .segments()
        .chain(o.lines.segments())
        .any(|s: PathSeg| s.nearest(local, 1e-3).distance_sq <= tol * tol)
}

#[cfg(test)]
mod tests;
