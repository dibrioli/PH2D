//! As peças de LEITURA das barras do quadro (módulo filho de [`super`], o ficheiro estava no tecto
//! de LOC): o balão de cada controlo, o que está seleccionado, e os ícones — o contorno da forma e a
//! seta desenhados com a MESMA geometria do quadro.

use super::{FONT_SIZES, Item, OPACITIES, Selected, WIDTHS, node, palette};
use crate::icons::IconId;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, paint_text_centered, resolve};
use crate::zones::Rect;
use ph2d_board_edit::{Editor, Frame, Tool};
use ph2d_board_model::{Connector, Dash, Head, Rgba, Route, Shape, ShapeType, Style};
use ph2d_board_route::Dir;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Color, Stroke, VectorScene};

/// W18 — o triângulo de play do botão «Video», em fracções do lado do miolo, a partir do centro.
const ICON_PLAY: [[f64; 2]; 3] = [
    [-0.15, -0.2], // LITERAL-PX-OK: desenho do glifo (o play no ecrã)
    [0.225, 0.0],  // LITERAL-PX-OK: desenho do glifo
    [-0.15, 0.2],  // LITERAL-PX-OK: desenho do glifo
];

/// O tamanho da ponta nos ícones: o de nascença do catálogo (`Marker`, 1×).
const ICON_HEAD_SCALE: f64 = 1.0;

/// O balão de cada controlo (o nome e, quando há, o atalho de uma tecla).
pub(super) fn tooltip_key(it: Item) -> Option<&'static str> {
    if let Some(k) = super::notes::tooltip_key(it)
        .or_else(|| super::pen::tooltip_key(it))
        .or_else(|| super::organize::tooltip_key(it))
        .or_else(|| super::mind::tooltip_key(it))
        .or_else(|| super::fac::tooltip_key(it))
    {
        return Some(k);
    }
    Some(match it {
        Item::Tool(Tool::Select) => "board.tool.select",
        Item::Tool(Tool::Hand) => "board.tool.hand",
        Item::Tool(Tool::Connector) => "board.tool.arrow",
        Item::Route(Route::Straight) => "board.route.straight",
        Item::Route(Route::Elbow) => "board.route.elbow",
        Item::Route(Route::Curved) => "board.route.curved",
        // A curva do Excalidraw só chega pelo ficheiro: a barra não a oferece (`ROUTES`).
        Item::Route(Route::Spline) => return None,
        Item::Head(i, h) => return head_key(i, h),
        Item::Tool(Tool::Shape(ShapeType::Rectangle)) => "board.tool.rectangle",
        Item::Tool(Tool::Shape(ShapeType::Ellipse)) => "board.tool.ellipse",
        Item::Tool(Tool::Shape(ShapeType::Diamond)) => "board.tool.diamond",
        Item::Tool(Tool::Shape(_)) | Item::Pick(_) => return shape_key(it),
        Item::MoreShapes => "board.tool.more",
        Item::MoreHeads(0) => "board.head.start.more",
        Item::MoreHeads(_) => "board.head.end.more",
        Item::Reshape => "board.style.shape",
        Item::Gray => "board.style.gray",
        Item::Flip(0) => "board.style.flip_h",
        Item::Flip(_) => "board.style.flip_v",
        Item::Crop => "board.style.crop",
        Item::Video => "board.tool.video",
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
        Item::NoteColor(_)
        | Item::NoteSize(_)
        | Item::NoteWide(_)
        | Item::Bulk
        | Item::Mark(_)
        | Item::TextColor(_)
        | Item::Tool(
            Tool::Pen(_) | Tool::Eraser { .. } | Tool::Laser | Tool::Frame | Tool::MindMap,
        )
        | Item::Pen(_)
        | Item::Sketch
        | Item::Organize(_)
        | Item::Mind(_)
        | Item::Fac(_) => return None,
    })
}

pub(super) fn head_key(which: usize, h: Head) -> Option<&'static str> {
    // Uma linha por ponta do catálogo: `[início, fim]` (W23: a grelha oferece-as todas).
    let [start, end] = match h {
        Head::None => ["board.head.start.none", "board.head.end.none"],
        Head::Arrow => ["board.head.start.arrow", "board.head.end.arrow"],
        Head::Triangle => ["board.head.start.triangle", "board.head.end.triangle"],
        Head::Circle => ["board.head.start.circle", "board.head.end.circle"],
        Head::Bar => ["board.head.start.bar", "board.head.end.bar"],
        Head::TriangleOpen => [
            "board.head.start.triangle_open",
            "board.head.end.triangle_open",
        ],
        Head::Diamond => ["board.head.start.diamond", "board.head.end.diamond"],
        Head::DiamondOpen => [
            "board.head.start.diamond_open",
            "board.head.end.diamond_open",
        ],
        Head::CircleOpen => ["board.head.start.circle_open", "board.head.end.circle_open"],
        Head::One => ["board.head.start.one", "board.head.end.one"],
        Head::Many => ["board.head.start.many", "board.head.end.many"],
        Head::OneOrMany => ["board.head.start.one_or_many", "board.head.end.one_or_many"],
        Head::ExactlyOne => ["board.head.start.exactly_one", "board.head.end.exactly_one"],
        Head::ZeroOrOne => ["board.head.start.zero_or_one", "board.head.end.zero_or_one"],
        Head::ZeroOrMany => [
            "board.head.start.zero_or_many",
            "board.head.end.zero_or_many",
        ],
    };
    Some(if which == 0 { start } else { end })
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
        ShapeType::Sticky => "board.shape.sticky",
        ShapeType::StickyWide => "board.shape.sticky_wide",
        ShapeType::StickyStack => "board.shape.sticky_stack",
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
    if let Some(k) = super::notes::selected_kind(ed, doc) {
        return k;
    }
    if let Some(k) = super::mind::selected_kind(ed, doc) {
        return Some(k);
    }
    let els: Vec<_> = ed
        .selection()
        .iter()
        .filter_map(|id| doc.get(*id))
        .collect();
    if !els.is_empty() && els.iter().all(|el| el.as_frame().is_some()) {
        return Some(Selected::Frames);
    }
    if !els.is_empty() && els.iter().all(|el| el.image().is_some()) {
        return Some(Selected::Images);
    }
    if !els.is_empty() && els.iter().all(|el| el.embed().is_some()) {
        return Some(Selected::Videos);
    }
    let arrows = els.iter().any(|el| el.connector().is_some());
    let shapes = els.iter().any(|el| el.shape().is_some());
    let inks = els.iter().any(|el| el.ink().is_some());
    match (shapes, arrows) {
        (true, false) => Some(Selected::Shapes),
        (false, true) => Some(Selected::Arrows),
        (true, true) => Some(Selected::Both),
        (false, false) => inks.then_some(Selected::Ink),
    }
}

/// O botão «Rascunho ↔ Final» aceso: com selecção, se tudo o que ele troca nela está à mão; sem
/// selecção, o modo do quadro.
pub(super) fn sketch_on(ed: &Editor, doc: &ph2d_board_model::BoardDoc, board: bool) -> bool {
    let sel: Vec<_> = ed
        .selection()
        .iter()
        .filter_map(|id| doc.get(*id))
        .filter(|el| ph2d_board_edit::sketchable(el))
        .collect();
    if ed.selection().is_empty() {
        board
    } else {
        !sel.is_empty() && sel.iter().all(|el| el.style().sketch)
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
        // A do Excalidraw com duas pontas é recta (XA1).
        Route::Straight | Route::Spline => vec![[x0, y0], [x1, y1]],
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
        text: Default::default(),
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

pub(super) fn paint_panel(scene: &mut VectorScene, rects: &[(Item, Rect)], theme: Theme) {
    super::super::float_bar::paint_panel(scene, rects.iter().map(|(_, r)| *r), theme);
}

/// A tinta «do tema» das barras: a do DOCUMENTO, que o quadro desenha pelo tema de cada momento.
pub(super) fn ink(_theme: Theme) -> Rgba {
    Rgba(ph2d_board_model::DEFAULT_INK)
}

/// Como o quadro desenhará a cor `c` neste tema (o quadradinho mostra o que o quadro vai mostrar).
pub(super) fn shown(c: Rgba, theme: Theme) -> Color {
    ph2d_board_render::ThemeInk::of(theme).color(c)
}

/// A cor de documento de um item de cor.
pub(super) fn swatch(it: Item, theme: Theme) -> Option<Rgba> {
    match it {
        Item::Fill(Some(i)) | Item::Stroke(Some(Some(i))) => Some(palette(i)),
        Item::Stroke(Some(None)) => Some(ink(theme)),
        _ => None,
    }
}

pub(super) fn doc_color(Rgba([r, g, b, a]): Rgba) -> Color {
    Color::from_rgba8(r, g, b, a) // LITERAL-COLOR-OK: cor do DOCUMENTO (dado do artista), não da UI
}

/// O item corresponde ao estilo actual da selecção?
pub(super) fn is_current(it: Item, s: &Style, arrow: Option<&Connector>, theme: Theme) -> bool {
    match it {
        Item::Route(r) => arrow.is_some_and(|c| c.route == r),
        Item::Head(i, h) => arrow.is_some_and(|c| c.heads[i] == h),
        Item::Fill(None) => s.fill.is_none(),
        Item::Fill(Some(_)) => s.fill == swatch(it, theme),
        Item::Stroke(None) => s.stroke.is_none(),
        Item::Stroke(Some(_)) => s.stroke == swatch(it, theme),
        Item::Width(i) => s.stroke_width == WIDTHS[i],
        Item::Dash(d) => s.dash == d,
        Item::Round(r) => s.round == r,
        Item::Opacity(i) => s.opacity == OPACITIES[i],
        Item::Font(i) => s.font_size == FONT_SIZES[i],
        _ => false,
    }
}

/// A constante das cúbicas que aproximam um quarto de círculo: `4(√2 − 1)/3`.
const QUARTER_CIRCLE_KAPPA: f64 = 0.552_284_75; // LITERAL-PX-OK: constante geométrica, não um tamanho

#[allow(clippy::too_many_arguments)]
/// Um controlo das barras: o botão e a cara dele.
pub(super) fn paint_item(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    it: Item,
    r: Rect,
    on: bool,
) {
    let id = node(it);
    let fg = super::super::float_bar::paint_button(
        scene,
        hit_index,
        store,
        theme,
        super::super::float_bar::Btn {
            id,
            rect: r,
            on,
            tint: ColorToken::Text2,
        },
    );
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    let inner = super::super::float_bar::inner(r);
    let line = f64::from(StrokeToken::Default.px());
    if super::notes::paint_icon(scene, text_system, theme, it, r, inner, fg, line)
        || super::pen::paint_icon(scene, theme, it, inner, fg, line)
        || super::organize::paint_icon(scene, theme, it, inner, fg, line)
        || super::mind::paint_icon(scene, text_system, it, r, inner, fg, line, on)
        || super::fac::paint_icon(scene, text_system, it, r, inner, fg, line)
    {
        return;
    }
    match it {
        Item::Tool(Tool::Select) => {
            paint_icon(scene, IconId::Select, inner, fg, StrokeToken::Default.px())
        }
        Item::Tool(Tool::Hand) => {
            paint_icon(scene, IconId::Pan, inner, fg, StrokeToken::Default.px())
        }
        Item::MoreShapes | Item::MoreHeads(_) => paint_icon(
            scene,
            IconId::MoreHorizontal,
            inner,
            fg,
            StrokeToken::Default.px(),
        ),
        Item::Tool(Tool::Shape(t)) | Item::Pick(t) => shape_icon(scene, t, inner, fg, line, false),
        Item::Tool(Tool::Connector) => {
            arrow_icon(
                scene,
                inner,
                fg,
                line,
                Route::Straight,
                Connector::DEFAULT_HEADS,
            );
        }
        Item::Route(rt) => arrow_icon(scene, inner, fg, line, rt, [Head::None, Head::None]),
        Item::Head(i, h) => {
            let heads = if i == 0 {
                [h, Head::None]
            } else {
                [Head::None, h]
            };
            arrow_icon(scene, inner, fg, line, Route::Straight, heads);
        }
        Item::Fill(_) | Item::Stroke(_) => {
            match swatch(it, theme) {
                Some(c) => {
                    let dot = crate::paint::frame_radius(theme, Radius::Sm.px());
                    fill_rounded_rect(scene, inner, dot, shown(c, theme));
                }
                // «Nenhum»: o quadrado vazio cortado pela diagonal.
                None => {
                    shape_icon(scene, ShapeType::Rectangle, inner, fg, line, false);
                    let mut p = ph2d_vector::BezPath::new();
                    p.move_to((f64::from(inner.x), f64::from(inner.y + inner.h)));
                    p.line_to((f64::from(inner.x + inner.w), f64::from(inner.y)));
                    stroke(scene, &p, fg, line, Dash::Solid);
                }
            }
            if matches!(it, Item::Stroke(Some(_))) {
                // O contorno lê-se como um ANEL: o miolo volta à cor do painel.
                // O miolo tem o lado de um `Sm`, centrado (com `Xs` de recuo o miolo sumia).
                let k = ((inner.w - Spacing::Sm.px()) / 2.0).max(0.0);
                let hole = Rect::new(
                    inner.x + k,
                    inner.y + k,
                    inner.w - 2.0 * k,
                    inner.h - 2.0 * k,
                );
                fill_rounded_rect(scene, hole, radius, resolve(ColorToken::BgElev, theme));
            }
        }
        Item::Width(i) => {
            let y = f64::from(inner.y + inner.h / 2.0);
            let mut p = ph2d_vector::BezPath::new();
            p.move_to((f64::from(inner.x), y));
            p.line_to((f64::from(inner.x + inner.w), y));
            stroke(scene, &p, fg, WIDTHS[i], Dash::Solid);
        }
        Item::Dash(d) => {
            let y = f64::from(inner.y + inner.h / 2.0);
            let mut p = ph2d_vector::BezPath::new();
            p.move_to((f64::from(inner.x), y));
            p.line_to((f64::from(inner.x + inner.w), y));
            stroke(scene, &p, fg, line, d);
        }
        Item::Round(round) => shape_icon(scene, ShapeType::Rectangle, inner, fg, line, round),
        // W18 — o vídeo: o ecrã (um rectângulo redondo) com o play dentro.
        Item::Video => {
            shape_icon(scene, ShapeType::Rectangle, inner, fg, line, true);
            let (cx, cy) = (
                f64::from(inner.x + inner.w / 2.0),
                f64::from(inner.y + inner.h / 2.0),
            );
            let s = f64::from(inner.w.min(inner.h));
            let mut tri = ph2d_vector::BezPath::new();
            for (k, [fx, fy]) in ICON_PLAY.iter().enumerate() {
                let p = (cx + fx * s, cy + fy * s);
                if k == 0 {
                    tri.move_to(p)
                } else {
                    tri.line_to(p)
                }
            }
            tri.close_path();
            scene.fill_path(&tri, &Brush::Solid(fg), Affine::IDENTITY);
        }
        // O preto e branco: o círculo do contraste, meio cheio.
        Item::Gray => {
            let (cx, cy) = (
                f64::from(inner.x + inner.w / 2.0),
                f64::from(inner.y + inner.h / 2.0),
            );
            let rad = f64::from(inner.w.min(inner.h) / 2.0) - line / 2.0;
            // Quartos de círculo em cúbicas: a metade esquerda cheia e, com a direita, o anel.
            let k = rad * QUARTER_CIRCLE_KAPPA;
            let mut half = ph2d_vector::BezPath::new();
            half.move_to((cx, cy - rad));
            half.curve_to((cx - k, cy - rad), (cx - rad, cy - k), (cx - rad, cy));
            half.curve_to((cx - rad, cy + k), (cx - k, cy + rad), (cx, cy + rad));
            let mut ring = half.clone();
            ring.curve_to((cx + k, cy + rad), (cx + rad, cy + k), (cx + rad, cy));
            ring.curve_to((cx + rad, cy - k), (cx + k, cy - rad), (cx, cy - rad));
            half.close_path();
            scene.fill_path(
                &half,
                &ph2d_vector::Brush::Solid(fg),
                ph2d_vector::Affine::IDENTITY,
            );
            stroke(scene, &ring, fg, line, Dash::Solid);
        }
        // W16 — espelhar: o eixo tracejado e os dois triângulos, o de trás cheio, o da frente vazio.
        Item::Flip(axis) => {
            let (x0, y0) = (f64::from(inner.x), f64::from(inner.y));
            let (w, h) = (f64::from(inner.w), f64::from(inner.h));
            // Em coordenadas do eixo `0` (horizontal); o vertical troca x e y.
            let at = |u: f64, v: f64| {
                if axis == 0 {
                    (x0 + u * w, y0 + v * h)
                } else {
                    (x0 + v * w, y0 + u * h)
                }
            };
            let mut mid = ph2d_vector::BezPath::new();
            mid.move_to(at(0.5, 0.0));
            mid.line_to(at(0.5, 1.0));
            stroke(scene, &mid, fg, line, Dash::Dashed);
            // Fracções do quadrado do ícone: o vão ao eixo, o topo e a base, a ponta do triângulo.
            let gap = 0.12; // LITERAL-PX-OK: fracção do ícone (o vão ao eixo)
            let (top, base) = (0.15, 0.85); // LITERAL-PX-OK: fracções do ícone (topo, base)
            let tip = 0.48; // LITERAL-PX-OK: fracção do ícone (a ponta do triângulo)
            for (side, full) in [(-1.0, true), (1.0, false)] {
                let mut t = ph2d_vector::BezPath::new();
                t.move_to(at(0.5 + side * gap, top));
                t.line_to(at(0.5 + side * gap, base));
                t.line_to(at(0.5 + side * tip, base));
                t.close_path();
                if full {
                    scene.fill_path(
                        &t,
                        &ph2d_vector::Brush::Solid(fg),
                        ph2d_vector::Affine::IDENTITY,
                    );
                } else {
                    stroke(scene, &t, fg, line, Dash::Solid);
                }
            }
        }
        // W16 — recortar: os dois esquadros cruzados do ícone de recorte.
        Item::Crop => {
            let (x0, y0) = (f64::from(inner.x), f64::from(inner.y));
            let (w, h) = (f64::from(inner.w), f64::from(inner.h));
            let at = |u: f64, v: f64| (x0 + u * w, y0 + v * h);
            let (a, b) = (0.25, 0.75); // LITERAL-PX-OK: fracções do quadrado do ícone
            let mut l = ph2d_vector::BezPath::new();
            l.move_to(at(a, 0.0));
            l.line_to(at(a, b));
            l.line_to(at(1.0, b));
            l.move_to(at(0.0, a));
            l.line_to(at(b, a));
            l.line_to(at(b, 1.0));
            stroke(scene, &l, fg, line, Dash::Solid);
        }
        Item::Opacity(i) => {
            let a = f32::from(OPACITIES[i]) / 100.0; // LITERAL-PX-OK: percentagem → fracção
            let c = resolve(ColorToken::Text1, theme).with_alpha(a);
            fill_rounded_rect(scene, inner, radius, c);
        }
        Item::Font(i) => {
            let label = tr([
                "board.font.s",
                "board.font.m",
                "board.font.l",
                "board.font.xl",
            ][i]);
            paint_text_centered(text_system, scene, label, r, TypeToken::Sm.px(), fg);
        }
        Item::NoteColor(_)
        | Item::NoteSize(_)
        | Item::NoteWide(_)
        | Item::Bulk
        | Item::Mark(_)
        | Item::TextColor(_)
        | Item::Tool(
            Tool::Pen(_) | Tool::Eraser { .. } | Tool::Laser | Tool::Frame | Tool::MindMap,
        )
        | Item::Pen(_)
        | Item::Sketch
        | Item::Organize(_)
        | Item::Mind(_)
        | Item::Fac(_)
        | Item::Reshape => {} // a cara do «Forma» é a forma actual: `shapes::paint_button`
    }
}
