//! **A geometria das formas do Quadro** (MiroClone, W1): o contorno de cada [`ShapeType`] numa
//! caixa, os cantos redondos, a caixa onde o texto vive, e o acerto de um ponto.
//!
//! Tudo em coordenadas LOCAIS da caixa `(0,0)–(w,h)`; [`to_world`] leva-as ao mundo (posição +
//! rotação à volta do centro). As formas do catálogo vectorial entram pelo `cook` dele (a mesma
//! geometria que o modo Vector desenha); rectângulo, losango e triângulo são daqui porque o canto
//! redondo do quadro segue outra regra ([`corner_radius`]).

use ph2d_board_model::{Element, Shape, ShapeType, VAlign};
use ph2d_vec_scene::{ShapeKind, cook};
use ph2d_vector::{
    Affine, BezPath, Line, ParamCurve, ParamCurveNearest, PathSeg, Point, Shape as _,
};

/// Folga entre o texto e a caixa onde ele vive, em unidades do mundo.
pub const TEXT_PADDING: f64 = 5.0;

/// O contorno de uma forma: o que se PREENCHE (contornos fechados) e as linhas de construção (a
/// tampa do cilindro, as barras do processo predefinido) que só se desenham.
#[derive(Clone, Debug)]
pub struct Outline {
    pub fill: BezPath,
    pub lines: BezPath,
}

/// O raio do canto redondo do RECTÂNGULO cujo lado menor é `min_side`: `min(S/4, 32)` (o «raio
/// adaptativo» — uma caixa grande não vira comprimido). Medido no oráculo (Excalidraw 0.18.1,
/// `roundness {type:3}`, o que o editor escreve ao desenhar um rectângulo): exacto em S = 20..400,
/// fixture `docs/MiroClone/ferramentas/excalidraw_oracle/saidas/formas_canto_retangulo.svg`; gate
/// em `oracle_tests`. O canto é uma QUADRÁTICA com o controlo no vértice, não um arco.
#[must_use]
pub fn corner_radius(min_side: f64) -> f64 {
    const FRACTION: f64 = 0.25;
    const CAP: f64 = 32.0;
    (min_side.abs() * FRACTION).min(CAP)
}

/// Quanto de cada aresta o canto redondo de um POLÍGONO (losango, triângulo) come a partir do
/// vértice. Medido no oráculo (Excalidraw 0.18.1, `roundness {type:2}`, o que o editor escreve ao
/// desenhar um losango): ¼ da meia-extensão em cada eixo, sem tecto — num losango é ¼ de cada
/// aresta. Fixture `saidas/formas_canto_losango.svg`; gate em `oracle_tests`.
const POLYGON_CORNER_CUT: f64 = 0.25;

/// O contorno de `shape` numa caixa `w × h` (local).
#[must_use]
pub fn outline(shape: &Shape, w: f64, h: f64) -> Outline {
    let round = shape.style.round;
    let polygon = |pts: &[[f64; 2]]| Outline {
        fill: if round {
            rounded_polygon(pts)
        } else {
            sharp_polygon(pts)
        },
        lines: BezPath::new(),
    };
    match shape.kind {
        // A nota e a pilha são rectângulos (o desenho põe a sombra e as folhas por baixo).
        ShapeType::Rectangle
        | ShapeType::Sticky
        | ShapeType::StickyStack
        | ShapeType::StickyWide => Outline {
            fill: if round {
                rounded_rect(w, h, corner_radius(w.min(h)))
            } else {
                sharp_polygon(&[[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]])
            },
            lines: BezPath::new(),
        },
        ShapeType::Diamond => {
            polygon(&[[w / 2.0, 0.0], [w, h / 2.0], [w / 2.0, h], [0.0, h / 2.0]])
        }
        ShapeType::Triangle => polygon(&[[w / 2.0, 0.0], [w, h], [0.0, h]]),
        other => {
            let kind = vec_kind(other);
            let path = cook(kind, [0.0, 0.0], [w, h], &kind.defaults());
            // ⛔ O catálogo vectorial é Y PARA CIMA (`space.rs`: `1 − 2v`); o quadro é Y para baixo.
            // Sem virar, a onda do Documento, a ponta do «fora da página» e o bico do balão ficavam
            // em cima (foto da cena 2, 06/10).
            let flip = Affine::new([1.0, 0.0, 0.0, -1.0, 0.0, h]);
            Outline {
                fill: flip * ph2d_vec_render::build_fill_bezpath(&path),
                lines: flip * ph2d_vec_render::build_lines_bezpath(&path),
            }
        }
    }
}

/// A forma do catálogo vectorial que desenha cada tipo do quadro (as três daqui não passam).
fn vec_kind(t: ShapeType) -> ShapeKind {
    match t {
        ShapeType::Rectangle
        | ShapeType::Sticky
        | ShapeType::StickyStack
        | ShapeType::StickyWide => ShapeKind::Rectangle,
        ShapeType::Ellipse => ShapeKind::Ellipse,
        ShapeType::Diamond => ShapeKind::Diamond,
        ShapeType::Triangle => ShapeKind::Polygon,
        ShapeType::Pill => ShapeKind::Pill,
        ShapeType::Parallelogram => ShapeKind::Parallelogram,
        // A operação manual ISO 5807 tem o lado LONGO em cima: no catálogo é o `TrapezoidFlip`.
        ShapeType::Trapezoid => ShapeKind::TrapezoidFlip,
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

/// Rectângulo com cada canto trocado por uma quadrática de `r` sobre cada lado e o controlo no
/// vértice (a construção do oráculo; `r ≤ S/4` nunca deixa dois cantos cruzarem-se).
fn rounded_rect(w: f64, h: f64, r: f64) -> BezPath {
    let mut p = BezPath::new();
    p.move_to((r, 0.0));
    p.line_to((w - r, 0.0));
    p.quad_to((w, 0.0), (w, r));
    p.line_to((w, h - r));
    p.quad_to((w, h), (w - r, h));
    p.line_to((r, h));
    p.quad_to((0.0, h), (0.0, h - r));
    p.line_to((0.0, r));
    p.quad_to((0.0, 0.0), (r, 0.0));
    p.close_path();
    p
}

/// Polígono com cada canto trocado por uma cúbica com OS DOIS controlos no vértice, das marcas a
/// [`POLYGON_CORNER_CUT`] de cada aresta (a construção do oráculo; ¼ + ¼ nunca se cruzam).
fn rounded_polygon(pts: &[[f64; 2]]) -> BezPath {
    let n = pts.len();
    let pt = |i: usize| Point::new(pts[i % n][0], pts[i % n][1]);
    let toward = |from: Point, to: Point| from + (to - from) * POLYGON_CORNER_CUT;
    let mut p = BezPath::new();
    for i in 0..n {
        let (prev, v, next) = (pt(i + n - 1), pt(i), pt(i + 1));
        let (a, b) = (toward(v, prev), toward(v, next));
        if i == 0 {
            p.move_to(a);
        } else {
            p.line_to(a);
        }
        p.curve_to(v, v, b);
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
        ShapeType::Rectangle
        | ShapeType::Sticky
        | ShapeType::StickyStack
        | ShapeType::StickyWide => [0.0, 0.0, 1.0, 1.0],
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

/// A largura que a forma precisa para uma linha de largura `text_w` caber na caixa do texto dela sem
/// quebrar (W6: um nó de mapa mental cresce de lado com o texto).
#[must_use]
pub fn width_for_text(t: ShapeType, text_w: f64) -> f64 {
    let [fx0, _, fx1, _] = text_frame(t);
    (text_w + 2.0 * TEXT_PADDING) / (fx1 - fx0)
}

/// Onde começa o bloco de texto (local): a caixa do texto, com o bloco no meio na vertical quando
/// cabe — ou no topo, ou no fundo (`valign`, o `verticalAlign` do Excalidraw, W9). Quando não cabe, a
/// forma cresce ([`height_for_text`]) e o topo encosta.
#[must_use]
pub fn text_origin(t: ShapeType, w: f64, h: f64, block_h: f64, valign: VAlign) -> [f64; 2] {
    let [x, y, _, rh] = text_rect(t, w, h);
    let room = (rh - block_h).max(0.0);
    let k = match valign {
        VAlign::Top => 0.0,
        VAlign::Middle => 0.5,
        VAlign::Bottom => 1.0,
    };
    [x, y + room * k]
}

/// ⭐ W14 — a forma que um elemento OCUPA: a dele, ou, numa imagem (W18: e num vídeo), o rectângulo da caixa (com os
/// cantos redondos dela) — clicar, prender uma seta e a faixa da seta tratam a imagem como uma caixa
/// (o Excalidraw prende setas a imagens). `None` = não ocupa uma forma (seta, traço, frame).
#[must_use]
pub fn shape_of(el: &Element) -> Option<std::borrow::Cow<'_, Shape>> {
    if let Some(s) = el.shape() {
        return Some(std::borrow::Cow::Borrowed(s));
    }
    // W18 — um vídeo ocupa a caixa como uma imagem.
    let style = el
        .image()
        .map(|i| &i.style)
        .or_else(|| el.embed().map(|e| &e.style))?;
    let mut style = style.clone();
    (style.fill, style.stroke) = (None, None);
    Some(std::borrow::Cow::Owned(Shape {
        kind: ShapeType::Rectangle,
        style,
        text: ph2d_board_model::RichText::default(),
    }))
}

/// Local → mundo: a caixa no sítio dela, rodada à volta do centro.
#[must_use]
pub fn to_world(el: &Element) -> Affine {
    let [cx, cy] = el.center();
    Affine::translate((cx, cy))
        * Affine::rotate(el.angle)
        * Affine::translate((-el.w / 2.0, -el.h / 2.0))
}

/// ⭐ W14/W16 — o PÍXEL NATURAL de uma imagem → a caixa local de `el` (`0..w × 0..h`): a parte
/// `source = [x, y, w, h]` da imagem natural (o recorte, ou ela inteira) estica-se à caixa e o
/// espelho vira à volta do centro dela. A lei do desenho e a do recorte pelo gesto.
#[must_use]
pub fn image_to_box(el: &Element, source: [f64; 4], flip: [bool; 2]) -> Affine {
    let [sx, sy, sw, sh] = source;
    let m = Affine::new([
        if flip[0] { -1.0 } else { 1.0 },
        0.0,
        0.0,
        if flip[1] { -1.0 } else { 1.0 },
        if flip[0] { el.w } else { 0.0 },
        if flip[1] { el.h } else { 0.0 },
    ]);
    m * Affine::scale_non_uniform(el.w / sw.max(1e-9), el.h / sh.max(1e-9))
        * Affine::translate((-sx, -sy))
}

/// O contorno de `el` já no mundo.
#[must_use]
pub fn world_outline(el: &Element) -> Option<Outline> {
    let shape = shape_of(el)?;
    let o = outline(&shape, el.w, el.h);
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
    let Some(shape) = shape_of(el) else {
        return false;
    };
    let [ux, uy] = el.unrotate(p);
    let local = Point::new(ux - el.x, uy - el.y);
    let o = outline(&shape, el.w, el.h);
    if o.fill.contains(local) {
        return true;
    }
    o.fill
        .segments()
        .chain(o.lines.segments())
        .any(|s: PathSeg| s.nearest(local, 1e-3).distance_sq <= tol * tol)
}

/// Onde o raio `from + t·dir` (mundo) SAI do contorno de `el`: o cruzamento de MAIOR `t` — numa
/// forma côncava (estrela, nuvem) o raio entra e sai várias vezes, e só o último garante que a
/// linha saiu de vez (a lei do `ph2d_vec_scene::boundary_hit`, aqui sobre a curva exacta).
/// `None` = o raio não cruza o contorno (ou não é uma forma).
#[must_use]
pub fn ray_exit(el: &Element, from: [f64; 2], dir: [f64; 2]) -> Option<[f64; 2]> {
    let o = world_outline(el)?;
    let len = dir[0].hypot(dir[1]);
    if len < 1e-12 {
        return None;
    }
    let d = [dir[0] / len, dir[1] / len];
    let [x0, y0, x1, y1] = el.aabb();
    let c = el.center();
    // Mais longe que qualquer ponto da forma, visto de `from`.
    let reach = (x1 - x0).hypot(y1 - y0) + (from[0] - c[0]).hypot(from[1] - c[1]) + 1.0;
    let line = Line::new(
        Point::new(from[0], from[1]),
        Point::new(from[0] + d[0] * reach, from[1] + d[1] * reach),
    );
    let t = o
        .fill
        .segments()
        .flat_map(|s| s.intersect_line(line))
        .map(|h| h.line_t)
        .fold(None, |best: Option<f64>, t| {
            Some(best.map_or(t, |b| b.max(t)))
        })?;
    Some([from[0] + d[0] * t * reach, from[1] + d[1] * t * reach])
}

/// ⭐ **Onde o caminho de `from` a `to` ENTRA na faixa de largura `gap` à volta de `el`** (W9, a lei XB1
/// do [estudo 07](../../../docs/MiroClone/07_fidelidade_excalidraw.md) §2): o 1.º ponto a `gap` do
/// contorno — na ELIPSE, a elipse de semi-eixos `+ gap` (a dele; a distância verdadeira daria 5,82 em vez
/// de 6 na diagonal, medido); nas outras, a distância verdadeira ao contorno (com os cantos redondos). Não
/// entra (o caminho chega a `to` por fora, ou `from` já está na faixa) ⇒ `to`.
#[must_use]
pub fn band_entry(el: &Element, from: [f64; 2], to: [f64; 2], gap: f64) -> [f64; 2] {
    let Some(shape) = shape_of(el) else {
        return to;
    };
    if shape.kind == ShapeType::Ellipse {
        return ellipse_entry(el, from, to, gap).unwrap_or(to);
    }
    let Some(o) = world_outline(el) else {
        return to;
    };
    let segs: Vec<PathSeg> = o.fill.segments().collect();
    let dist = |p: [f64; 2]| {
        let q = Point::new(p[0], p[1]);
        segs.iter()
            .map(|s| s.nearest(q, 1e-9).distance_sq)
            .fold(f64::INFINITY, f64::min)
            .sqrt()
    };
    let len = (to[0] - from[0]).hypot(to[1] - from[1]);
    if len < 1e-12 || o.fill.contains(Point::new(from[0], from[1])) || dist(from) <= gap {
        return to;
    }
    let u = [(to[0] - from[0]) / len, (to[1] - from[1]) / len];
    // A marcha por esferas: a distância ao contorno é 1-Lipschitz, um passo de `d − gap` nunca passa
    // a 1.ª entrada na faixa.
    let mut s = 0.0;
    for _ in 0..BAND_MARCH_STEPS {
        let p = [from[0] + u[0] * s, from[1] + u[1] * s];
        let step = dist(p) - gap;
        if step <= BAND_EPS {
            return p;
        }
        s += step;
        if s >= len {
            return to;
        }
    }
    [from[0] + u[0] * s, from[1] + u[1] * s]
}

/// A marcha pára a menos disto (mundo) da faixa — um milésimo de px a qualquer zoom de trabalho.
const BAND_EPS: f64 = 1e-6;
/// Tecto de passos da marcha (um raio rasante à faixa avança devagar; a 1e-6 nunca se chega perto).
const BAND_MARCH_STEPS: usize = 4096;

/// [`band_entry`] na elipse: a 1.ª raiz em `0..=1` da recta contra a elipse de semi-eixos `+ gap`, no
/// referencial da caixa (antes de rodar). `None` = não entra, ou `from` já está dentro.
fn ellipse_entry(el: &Element, from: [f64; 2], to: [f64; 2], gap: f64) -> Option<[f64; 2]> {
    let c = el.center();
    let (a, b) = (el.w / 2.0 + gap, el.h / 2.0 + gap);
    if a <= 0.0 || b <= 0.0 {
        return None;
    }
    let [fx, fy] = el.unrotate(from);
    let [tx, ty] = el.unrotate(to);
    let (px, py) = ((fx - c[0]) / a, (fy - c[1]) / b);
    let (dx, dy) = ((tx - fx) / a, (ty - fy) / b);
    let qa = dx * dx + dy * dy;
    let qb = 2.0 * (px * dx + py * dy);
    let qc = px * px + py * py - 1.0;
    if qc <= 0.0 || qa < 1e-24 {
        return None;
    }
    let disc = qb * qb - 4.0 * qa * qc;
    if disc < 0.0 {
        return None;
    }
    let t = (-qb - disc.sqrt()) / (2.0 * qa);
    (0.0..=1.0)
        .contains(&t)
        .then(|| el.rotate([fx + (tx - fx) * t, fy + (ty - fy) * t]))
}

/// ⭐ **Onde o raio `from + t·dir` SAI de vez da faixa de `gap` à volta de `el`** — a ponta presa ao
/// CENTRO com o vão (W9: o quadro inteiro pára antes da forma, decisão do dono 10/10): a entrada na faixa
/// vista de longe, pelo mesmo raio ([`band_entry`]). `None` = sem contorno.
#[must_use]
pub fn ray_exit_band(el: &Element, from: [f64; 2], dir: [f64; 2], gap: f64) -> Option<[f64; 2]> {
    let len = dir[0].hypot(dir[1]);
    if len < 1e-12 {
        return None;
    }
    if gap <= 0.0 {
        return ray_exit(el, from, dir);
    }
    let [x0, y0, x1, y1] = el.aabb();
    let c = el.center();
    let reach = (x1 - x0).hypot(y1 - y0) + (from[0] - c[0]).hypot(from[1] - c[1]) + 2.0 * gap + 1.0;
    let far = [
        from[0] + dir[0] / len * reach,
        from[1] + dir[1] / len * reach,
    ];
    world_outline(el)?;
    Some(band_entry(el, far, from, gap))
}

/// ⭐ **A normal do contorno de `el` em `p`** (mundo, unitária, para FORA) — a direcção em que uma seta
/// curva encaixa na forma (5.º smoke do dono, 06/10: «a seta na direcção da normal da curva da forma
/// onde se encaixa»). Numa junção LISA entre dois segmentos (o arredondado de um canto, a costura de
/// uma elipse) é a média; num CANTO vivo (o vértice de um losango) não há normal única ⇒ `None`, e
/// quem chama fica com a direcção do lado.
#[must_use]
pub fn outline_normal(el: &Element, p: [f64; 2]) -> Option<[f64; 2]> {
    /// Tangentes que diferem mais do que isto (graus) fazem um canto, não uma curva.
    const CORNER_DEG: f64 = 15.0;
    const H: f64 = 1e-4;
    let o = world_outline(el)?;
    let q = Point::new(p[0], p[1]);
    let segs: Vec<PathSeg> = o.fill.segments().collect();
    let (i, t) = segs
        .iter()
        .enumerate()
        .map(|(i, s)| (i, s.nearest(q, 1e-9)))
        .min_by(|a, b| a.1.distance_sq.total_cmp(&b.1.distance_sq))
        .map(|(i, n)| (i, n.t))?;
    let tangent = |s: &PathSeg, t: f64| {
        let (a, b) = ((t - H).max(0.0), (t + H).min(1.0));
        let v = s.eval(b) - s.eval(a);
        let l = v.hypot();
        (l > 1e-12).then(|| ph2d_vector::Vec2::new(v.x / l, v.y / l))
    };
    // As junções medem-se a partir do ponto DO CONTORNO (a auditoria W9: com a ponta a `g` da forma, a
    // comparação com `p` nunca casava e a curva entrava pelo eixo do ecrã numa elipse rodada).
    let on = segs[i].eval(t);
    let at = |pt: Point| (pt - on).hypot() < 1e-6;
    let tan = if t > H && t < 1.0 - H {
        tangent(&segs[i], t)?
    } else {
        // Numa junção: o segmento que chega e o que parte.
        let incoming = segs.iter().find(|s| at(s.end()))?;
        let outgoing = segs.iter().find(|s| at(s.start()))?;
        let (a, b) = (tangent(incoming, 1.0)?, tangent(outgoing, 0.0)?);
        if a.dot(b).clamp(-1.0, 1.0).acos().to_degrees() > CORNER_DEG {
            return None;
        }
        let m = a + b;
        ph2d_vector::Vec2::new(m.x / m.hypot(), m.y / m.hypot())
    };
    let n = [-tan.y, tan.x];
    // Para FORA: um passo ao longo da normal, a partir do ponto DO CONTORNO, sai do preenchimento.
    // ⛔ A partir de `p` não: uma ponta parada a `g` da forma (W9) já está fora, e os dois lados dela
    // também — a curva saía para dentro.
    let probe = Point::new(
        on.x + n[0] * 1e-3 * (1.0 + el.w.max(el.h)),
        on.y + n[1] * 1e-3 * (1.0 + el.w.max(el.h)),
    );
    Some(if o.fill.contains(probe) {
        [-n[0], -n[1]]
    } else {
        n
    })
}

/// O ponto do contorno de `el` mais perto de `p` (mundo) e a distância até ele.
#[must_use]
pub fn nearest_on_outline(el: &Element, p: [f64; 2]) -> Option<([f64; 2], f64)> {
    let o = world_outline(el)?;
    let q = Point::new(p[0], p[1]);
    o.fill
        .segments()
        .map(|s| {
            let n = s.nearest(q, 1e-6);
            (s.eval(n.t), n.distance_sq)
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(pt, d2)| ([pt.x, pt.y], d2.sqrt()))
}

/// `p` (mundo) está dentro do contorno de `el`?
#[must_use]
pub fn inside(el: &Element, p: [f64; 2]) -> bool {
    world_outline(el).is_some_and(|o| o.fill.contains(Point::new(p[0], p[1])))
}

#[cfg(test)]
mod oracle_tests;
#[cfg(test)]
mod tests;
