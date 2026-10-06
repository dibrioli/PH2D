//! As peças de LEITURA das barras do quadro (módulo filho de [`super`], o ficheiro estava no tecto
//! de LOC): o balão de cada controlo, o que está seleccionado, e os ícones — o contorno da forma e a
//! seta desenhados com a MESMA geometria do quadro.

use super::{HEADS, Item, Selected};
use crate::zones::Rect;
use ph2d_board_edit::{Editor, Frame, Tool};
use ph2d_board_model::{Connector, Dash, Head, Rgba, Route, Shape, ShapeType, Style};
use ph2d_board_route::Dir;
use ph2d_tokens::Spacing;
use ph2d_vector::{Affine, Brush, Color, Stroke, VectorScene};

/// O tamanho da ponta nos ícones: o de nascença do catálogo (`Marker`, 1×).
const ICON_HEAD_SCALE: f64 = 1.0;

/// O balão de cada controlo (o nome e, quando há, o atalho de uma tecla).
pub(super) fn tooltip_key(it: Item) -> Option<&'static str> {
    Some(match it {
        Item::Tool(Tool::Select) => "board.tool.select",
        Item::Tool(Tool::Hand) => "board.tool.hand",
        Item::Tool(Tool::Connector) => "board.tool.arrow",
        Item::Route(Route::Straight) => "board.route.straight",
        Item::Route(Route::Elbow) => "board.route.elbow",
        Item::Route(Route::Curved) => "board.route.curved",
        Item::Head(i, h) => return head_key(i, h),
        Item::Tool(Tool::Shape(ShapeType::Rectangle)) => "board.tool.rectangle",
        Item::Tool(Tool::Shape(ShapeType::Ellipse)) => "board.tool.ellipse",
        Item::Tool(Tool::Shape(ShapeType::Diamond)) => "board.tool.diamond",
        Item::Tool(Tool::Shape(_)) | Item::Pick(_) => return shape_key(it),
        Item::MoreShapes => "board.tool.more",
        Item::Fill(None) => "board.style.no_fill",
        Item::Fill(Some(_)) => "board.style.fill",
        Item::Stroke(None) => "board.style.no_stroke",
        Item::Stroke(Some(_)) => "board.style.stroke",
        Item::Width(_) => "board.style.width",
        Item::Dash(_) => "board.style.dash",
        Item::Round(false) => "board.style.sharp",
        Item::Round(true) => "board.style.round",
        Item::Opacity(_) => "board.style.opacity",
        Item::Font(_) => "board.style.font",
    })
}

pub(super) fn head_key(which: usize, h: Head) -> Option<&'static str> {
    let keys = if which == 0 {
        [
            "board.head.start.none",
            "board.head.start.arrow",
            "board.head.start.triangle",
            "board.head.start.circle",
            "board.head.start.bar",
        ]
    } else {
        [
            "board.head.end.none",
            "board.head.end.arrow",
            "board.head.end.triangle",
            "board.head.end.circle",
            "board.head.end.bar",
        ]
    };
    HEADS.iter().position(|x| *x == h).map(|i| keys[i])
}

pub(super) fn shape_key(it: Item) -> Option<&'static str> {
    let (Item::Pick(t) | Item::Tool(Tool::Shape(t))) = it else {
        return None;
    };
    Some(shape_name_key(t))
}

/// A chave do NOME de cada forma (o balão da grelha; a cena de smoke escreve-o dentro da forma).
#[must_use]
pub fn shape_name_key(t: ShapeType) -> &'static str {
    match t {
        ShapeType::Rectangle => "board.shape.rectangle",
        ShapeType::Ellipse => "board.shape.ellipse",
        ShapeType::Diamond => "board.shape.diamond",
        ShapeType::Triangle => "board.shape.triangle",
        ShapeType::Pill => "board.shape.pill",
        ShapeType::Parallelogram => "board.shape.parallelogram",
        ShapeType::Trapezoid => "board.shape.trapezoid",
        ShapeType::Hexagon => "board.shape.hexagon",
        ShapeType::Cylinder => "board.shape.cylinder",
        ShapeType::Document => "board.shape.document",
        ShapeType::PredefinedProcess => "board.shape.predefined_process",
        ShapeType::OffPage => "board.shape.off_page",
        ShapeType::Delay => "board.shape.delay",
        ShapeType::Display => "board.shape.display",
        ShapeType::SpeechRect => "board.shape.speech",
        ShapeType::Cloud => "board.shape.cloud",
        ShapeType::Star => "board.shape.star",
        ShapeType::ArrowRight => "board.shape.arrow",
    }
}

/// O estilo do primeiro seleccionado (a barra mostra-o como «o actual»).
pub(super) fn selected_style(ed: &Editor, doc: &ph2d_board_model::BoardDoc) -> Option<Style> {
    ed.selection()
        .iter()
        .find_map(|id| doc.get(*id).map(|el| el.style().clone()))
}

/// A primeira seta seleccionada (a rota e as pontas «actuais»).
pub(super) fn selected_arrow(ed: &Editor, doc: &ph2d_board_model::BoardDoc) -> Option<Connector> {
    ed.selection()
        .iter()
        .find_map(|id| doc.get(*id)?.connector().cloned())
}

/// Formas, setas ou as duas?
pub(super) fn selected_kind(ed: &Editor, doc: &ph2d_board_model::BoardDoc) -> Option<Selected> {
    let els: Vec<_> = ed
        .selection()
        .iter()
        .filter_map(|id| doc.get(*id))
        .collect();
    let arrows = els.iter().any(|el| el.connector().is_some());
    let shapes = els.iter().any(|el| el.shape().is_some());
    match (shapes, arrows) {
        (true, false) => Some(Selected::Shapes),
        (false, true) => Some(Selected::Arrows),
        (true, true) => Some(Selected::Both),
        (false, false) => None,
    }
}

/// A caixa no ecrã da selecção: a moldura das formas e as rotas das setas.
pub(super) fn selection_box(
    ed: &mut Editor,
    doc: &ph2d_board_model::BoardDoc,
    camera: &ph2d_board_model::Camera,
    area: [f64; 4],
) -> Option<Rect> {
    let frame = ed.frame(doc).map(|f| screen_box(&f, camera, area));
    let ids: Vec<_> = ed.selection().iter().copied().collect();
    let routes = ed.routes(doc);
    let wires = ids.iter().filter_map(|id| routes.get(*id)).map(|r| {
        let a = camera.to_screen(area, [r.bbox[0], r.bbox[1]]);
        let b = camera.to_screen(area, [r.bbox[2], r.bbox[3]]);
        Rect::new(
            a[0] as f32,
            a[1] as f32,
            (b[0] - a[0]) as f32,
            (b[1] - a[1]) as f32,
        )
    });
    frame.into_iter().chain(wires).reduce(|a, b| {
        let (x0, y0) = (a.x.min(b.x), a.y.min(b.y));
        let (x1, y1) = ((a.x + a.w).max(b.x + b.w), (a.y + a.h).max(b.y + b.h));
        Rect::new(x0, y0, x1 - x0, y1 - y0)
    })
}

/// A caixa no ecrã que contém uma moldura rodada.
pub(super) fn screen_box(f: &Frame, camera: &ph2d_board_model::Camera, area: [f64; 4]) -> Rect {
    let (hw, hh) = (f.w / 2.0, f.h / 2.0);
    let pts =
        [[-hw, -hh], [hw, -hh], [hw, hh], [-hw, hh]].map(|l| camera.to_screen(area, f.point(l)));
    let x0 = pts.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
    let y0 = pts.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
    let x1 = pts.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
    let y1 = pts.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);
    Rect::new(x0 as f32, y0 as f32, (x1 - x0) as f32, (y1 - y0) as f32)
}

/// Um traço de ícone com o MESMO tracejado que a forma desenha (`ph2d_board_render::style_stroke`).
pub(super) fn stroke(scene: &mut VectorScene, p: &ph2d_vector::BezPath, c: Color, w: f64, d: Dash) {
    let s = ph2d_board_render::style_stroke(w, d);
    scene
        .inner_mut()
        .stroke(&s, Affine::IDENTITY, &Brush::Solid(c), None, p);
}

/// O ícone de uma seta: a MESMA geometria que o quadro desenha (`ph2d_board_route::drawn`), de
/// canto a canto de `r` — recta na diagonal, cotovelo/curva em Z.
pub(super) fn arrow_icon(
    scene: &mut VectorScene,
    r: Rect,
    c: Color,
    w: f64,
    route: Route,
    heads: [Head; 2],
) {
    let (x0, y0) = (f64::from(r.x), f64::from(r.y + r.h));
    let (x1, y1) = (f64::from(r.x + r.w), f64::from(r.y));
    let pts = match route {
        Route::Straight => vec![[x0, y0], [x1, y1]],
        Route::Elbow | Route::Curved => {
            let mx = (x0 + x1) / 2.0;
            vec![[x0, y0], [mx, y0], [mx, y1], [x1, y1]]
        }
    };
    let routed = ph2d_board_route::Routed::from_points(pts, route, [Dir::East, Dir::West]);
    // A ponta do catálogo no tamanho de nascença (4 × a linha), não o do quadro: cabe no botão.
    let d = ph2d_board_route::drawn_at(&routed, heads, w, ICON_HEAD_SCALE);
    let brush = Brush::Solid(c);
    let s = Stroke::new(w)
        .with_caps(ph2d_vector::Cap::Round)
        .with_join(ph2d_vector::Join::Round);
    scene
        .inner_mut()
        .stroke(&s, Affine::IDENTITY, &brush, None, &d.line);
    for (head, filled) in &d.heads {
        if *filled {
            scene.fill_path(head, &brush, Affine::IDENTITY);
        } else {
            scene
                .inner_mut()
                .stroke(&s, Affine::IDENTITY, &brush, None, head);
        }
    }
}

/// O ícone de uma forma: o contorno dela, encaixado em `r`.
pub(super) fn shape_icon(
    scene: &mut VectorScene,
    t: ShapeType,
    r: Rect,
    c: Color,
    w: f64,
    round: bool,
) {
    let ink = Rgba([0, 0, 0, 255]);
    let mut style = Style::new(None, Some(ink), ink);
    style.round = round;
    let shape = Shape {
        kind: t,
        style,
        text: String::new(),
    };
    let (bw, bh) = (f64::from(r.w), f64::from(r.h - Spacing::Xs.px()));
    let o = ph2d_board_geom::outline(&shape, bw, bh);
    let at = Affine::translate((f64::from(r.x), f64::from(r.y) + (f64::from(r.h) - bh) / 2.0));
    let s = Stroke::new(w);
    scene
        .inner_mut()
        .stroke(&s, at, &Brush::Solid(c), None, &o.fill);
    if !o.lines.is_empty() {
        scene
            .inner_mut()
            .stroke(&s, at, &Brush::Solid(c), None, &o.lines);
    }
}
