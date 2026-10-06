//! ⭐ **As barras do quadro** (MiroClone, W1–W2) — a barra CURTA de ferramentas (decisão do dono 4)
//! encostada à esquerda da área, a grelha de todas as formas que ela abre, e a barra de ESTILO que
//! flutua por cima da selecção (preenchimento, contorno, espessura, traço, cantos, opacidade, letra;
//! numa seta, a rota e as pontas em vez do preenchimento e dos cantos).
//!
//! ⚠️ Os ids são DERIVADOS (salto XOR + índice na tabela [`items`]), como os das abas de quadro: a
//! tabela é a única fonte, e `populate` regista-os todos (pintado ⇒ clicável).
//! ⚠️ O ícone de cada forma é o CONTORNO dela (`ph2d_board_geom::outline`) — a mesma geometria que o
//! quadro desenha, nunca um segundo desenho.

use super::HeroScreen;
use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, paint_text_centered, resolve};
use crate::widget::ButtonState;
use crate::widget::panel_chrome::HIGHLIGHTER_RGBA;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_board_edit::{Editor, Frame, Tool};
use ph2d_board_model::{Connector, Dash, Head, Rgba, Route, Shape, ShapeType, Style};
use ph2d_board_route::Dir;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Color, Stroke, VectorScene};

/// ⛔ O salto tem os 16 bits do MEIO diferentes dos das abas de quadro (`…_0000_0003`): os dois
/// lados XOR-am números pequenos, e saltos que só diferem nos bits de baixo COLIDEM (medido: o
/// «mais formas» era a aba do quadro 1).
const SALT: u64 = 0xb0a2_d7ab_5ba2_0000;

use ph2d_board_edit::{FONT_SIZES, STROKE_WIDTHS as WIDTHS};
/// As opacidades oferecidas (%).
const OPACITIES: [u8; 4] = [25, 50, 75, 100];
/// Os preenchimentos claros (os pastéis do marcador) e as tintas fortes do contorno.
const PASTELS: std::ops::Range<usize> = 0..5;
const VIVIDS: std::ops::Range<usize> = 5..9;

/// Um controlo das barras.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Item {
    Tool(Tool),
    MoreShapes,
    Pick(ShapeType),
    /// `None` = sem preenchimento; `Some(i)` = o pastel `i` do marcador.
    Fill(Option<usize>),
    /// `None` = sem contorno; `Some(None)` = a tinta do tema; `Some(Some(i))` = a forte `i`.
    Stroke(Option<Option<usize>>),
    Width(usize),
    Dash(Dash),
    Round(bool),
    Opacity(usize),
    Font(usize),
    Route(Route),
    /// `(0 = início · 1 = fim, desenho)`.
    Head(usize, Head),
}

/// As pontas que a barra oferece: as do fluxograma e do diagrama de quadro. (O documento guarda as
/// oito do catálogo — as de UML entram pelo ficheiro, não pela barra estreita.)
const HEADS: [Head; 5] = [
    Head::None,
    Head::Arrow,
    Head::Triangle,
    Head::Circle,
    Head::Bar,
];
const ROUTES: [Route; 3] = [Route::Straight, Route::Elbow, Route::Curved];

/// O que está seleccionado decide que grupos a barra de estilo mostra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selected {
    Shapes,
    Arrows,
    Both,
}

/// Os atalhos da barra curta, na ordem em que se pintam.
const TOOLBAR: [Item; 8] = [
    Item::Tool(Tool::Select),
    Item::Tool(Tool::Hand),
    Item::Tool(Tool::Connector),
    Item::Tool(Tool::Shape(ShapeType::Rectangle)),
    Item::Tool(Tool::Shape(ShapeType::Ellipse)),
    Item::Tool(Tool::Shape(ShapeType::Diamond)),
    Item::Tool(Tool::Shape(ShapeType::Triangle)),
    Item::MoreShapes,
];

/// ⭐ **A tabela** — todo controlo das barras, por grupo da barra de estilo. O índice é o id.
#[must_use]
pub fn items() -> Vec<Item> {
    let mut v: Vec<Item> = TOOLBAR.to_vec();
    v.extend(ShapeType::ALL.iter().map(|t| Item::Pick(*t)));
    for it in [Selected::Shapes, Selected::Arrows]
        .into_iter()
        .flat_map(style_groups)
        .flatten()
    {
        if !v.contains(&it) {
            v.push(it);
        }
    }
    v
}

/// Os grupos da barra de estilo, da esquerda para a direita, para o que está seleccionado.
#[must_use]
pub fn style_groups(sel: Selected) -> Vec<Vec<Item>> {
    let fill: Vec<Item> = std::iter::once(Item::Fill(None))
        .chain(PASTELS.map(|i| Item::Fill(Some(i))))
        .collect();
    let stroke: Vec<Item> = [Item::Stroke(None), Item::Stroke(Some(None))]
        .into_iter()
        .chain(VIVIDS.map(|i| Item::Stroke(Some(Some(i)))))
        .collect();
    let width: Vec<Item> = (0..WIDTHS.len()).map(Item::Width).collect();
    let dash = vec![
        Item::Dash(Dash::Solid),
        Item::Dash(Dash::Dashed),
        Item::Dash(Dash::Dotted),
    ];
    let opacity: Vec<Item> = (0..OPACITIES.len()).map(Item::Opacity).collect();
    let font: Vec<Item> = (0..FONT_SIZES.len()).map(Item::Font).collect();
    match sel {
        Selected::Shapes => vec![
            fill,
            stroke,
            width,
            dash,
            vec![Item::Round(false), Item::Round(true)],
            opacity,
            font,
        ],
        Selected::Arrows => vec![
            // Uma seta sem traço não se vê: o «sem contorno» não entra.
            stroke
                .into_iter()
                .filter(|it| *it != Item::Stroke(None))
                .collect(),
            width,
            dash,
            ROUTES.iter().map(|r| Item::Route(*r)).collect(),
            HEADS.iter().map(|h| Item::Head(0, *h)).collect(),
            HEADS.iter().map(|h| Item::Head(1, *h)).collect(),
            opacity,
            font,
        ],
        Selected::Both => vec![
            stroke
                .into_iter()
                .filter(|it| *it != Item::Stroke(None))
                .collect(),
            width,
            dash,
            opacity,
            font,
        ],
    }
}

/// O id do controlo `item`.
#[must_use]
pub fn node(item: Item) -> NodeId {
    let i = items()
        .iter()
        .position(|x| *x == item)
        .expect("está na tabela") as u64;
    NodeId(SALT ^ (i + 1))
}

fn item_of(id: NodeId) -> Option<Item> {
    let i = (id.0 ^ SALT).checked_sub(1)?;
    items().get(usize::try_from(i).ok()?).copied()
}

/// Regista todos os controlos (pintado ⇒ clicável). Chamado pelo `document_tabs::populate`.
pub fn populate(store: &mut WidgetStore) {
    for it in items() {
        store.register(
            node(it),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        if let Some(key) = tooltip_key(it) {
            store.set_tooltip(node(it), tr(key));
        }
    }
}

/// O balão de cada controlo (o nome e, quando há, o atalho de uma tecla).
fn tooltip_key(it: Item) -> Option<&'static str> {
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

fn head_key(which: usize, h: Head) -> Option<&'static str> {
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

fn shape_key(it: Item) -> Option<&'static str> {
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

fn btn_px() -> f32 {
    Spacing::Xl2.px()
}

fn gap() -> f32 {
    ph2d_tokens::control_gap_px()
}

/// O rectângulo de cada atalho da barra curta, de cima para baixo, encostado à esquerda de `area`.
#[must_use]
pub fn toolbar_rects(area: Rect) -> Vec<(Item, Rect)> {
    let (s, g) = (btn_px(), gap());
    let x = area.x + Spacing::Md.px();
    let mut y = area.y + Spacing::Md.px();
    TOOLBAR
        .iter()
        .map(|it| {
            let r = Rect::new(x, y, s, s);
            y += s + g;
            (*it, r)
        })
        .collect()
}

/// A grelha de todas as formas, à direita do botão «mais formas» (três colunas).
#[must_use]
pub fn shapes_rects(area: Rect) -> Vec<(Item, Rect)> {
    let more = toolbar_rects(area)
        .into_iter()
        .find(|(it, _)| *it == Item::MoreShapes)
        .map(|(_, r)| r)
        .expect("a barra tem o «mais formas»");
    let (s, g) = (btn_px(), gap());
    let x0 = more.x + more.w + Spacing::Sm.px();
    ShapeType::ALL
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let (col, row) = ((i % 3) as f32, (i / 3) as f32);
            let r = Rect::new(x0 + col * (s + g), more.y + row * (s + g), s, s);
            (Item::Pick(*t), r)
        })
        .collect()
}

/// A barra de estilo por cima de `sel` (o rectângulo da selecção no ecrã), presa dentro de `area`;
/// por baixo da selecção quando não cabe por cima.
#[must_use]
pub fn style_rects(area: Rect, sel: Rect, what: Selected) -> Vec<(Item, Rect)> {
    let (s, g) = (Spacing::Xl.px(), gap());
    let sep = Spacing::Sm.px();
    let groups = style_groups(what);
    let n: usize = groups.iter().map(Vec::len).sum();
    let width = n as f32 * (s + g) + (groups.len() - 1) as f32 * sep;
    let above = sel.y - s - Spacing::Lg.px();
    let y = if above >= area.y {
        above
    } else {
        (sel.y + sel.h + Spacing::Lg.px()).min(area.y + area.h - s)
    };
    let mut x = (sel.x + sel.w / 2.0 - width / 2.0)
        .max(area.x)
        .min(area.x + area.w - width);
    let mut out = Vec::with_capacity(n);
    for grp in groups {
        for it in grp {
            out.push((it, Rect::new(x, y, s, s)));
            x += s + g;
        }
        x += sep;
    }
    out
}

/// ⭐ Pinta as barras do quadro activo. `area` é a área do quadro no ecrã.
#[allow(clippy::too_many_arguments)]
pub fn paint(
    hero: &mut HeroScreen,
    area: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
) {
    let theme = hero.theme;
    let shapes_open = hero.documents.live.shapes_open;
    let Some((board, live)) = hero.documents.active_parts() else {
        return;
    };
    let ed = super::board_view::editor(&mut live.editor, theme);
    let tool = ed.tool;
    let sel_style = selected_style(ed, &board.doc);
    let arrow = selected_arrow(ed, &board.doc);
    let what = selected_kind(ed, &board.doc);
    let area_w = super::board_view::area_of(area);
    let sel_box = (!ed.is_busy() && !ed.is_editing_text())
        .then(|| selection_box(ed, &board.doc, &board.camera, area_w))
        .flatten();
    let tools = toolbar_rects(area);
    paint_panel(scene, &tools, theme);
    for (it, r) in tools {
        let on =
            matches!(it, Item::Tool(t) if t == tool) || (it == Item::MoreShapes && shapes_open);
        paint_item(
            scene,
            text_system,
            theme,
            &mut hero.hit_index,
            &hero.store,
            it,
            r,
            on,
        );
    }
    if shapes_open {
        let grid = shapes_rects(area);
        paint_panel(scene, &grid, theme);
        for (it, r) in grid {
            let on = matches!(it, Item::Pick(t) if tool == Tool::Shape(t));
            paint_item(
                scene,
                text_system,
                theme,
                &mut hero.hit_index,
                &hero.store,
                it,
                r,
                on,
            );
        }
    }
    let (Some(mut sel), Some(style), Some(what)) = (sel_box, sel_style, what) else {
        return;
    };
    // A pega de RODAR e os pontos azuis ficam acima da moldura: a barra sobe acima deles (na foto
    // de 06/10 tapava a pega).
    let m = super::board_view::metrics();
    let knob = (m.dot + m.handle) as f32;
    sel.y -= knob;
    sel.h += knob;
    let bar = style_rects(area, sel, what);
    paint_panel(scene, &bar, theme);
    for (it, r) in bar {
        let on = is_current(it, &style, arrow.as_ref(), theme);
        paint_item(
            scene,
            text_system,
            theme,
            &mut hero.hit_index,
            &hero.store,
            it,
            r,
            on,
        );
    }
}

/// O estilo do primeiro seleccionado (a barra mostra-o como «o actual»).
fn selected_style(ed: &Editor, doc: &ph2d_board_model::BoardDoc) -> Option<Style> {
    ed.selection()
        .iter()
        .find_map(|id| doc.get(*id).map(|el| el.style().clone()))
}

/// A primeira seta seleccionada (a rota e as pontas «actuais»).
fn selected_arrow(ed: &Editor, doc: &ph2d_board_model::BoardDoc) -> Option<Connector> {
    ed.selection()
        .iter()
        .find_map(|id| doc.get(*id)?.connector().cloned())
}

/// Formas, setas ou as duas?
fn selected_kind(ed: &Editor, doc: &ph2d_board_model::BoardDoc) -> Option<Selected> {
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
fn selection_box(
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
fn screen_box(f: &Frame, camera: &ph2d_board_model::Camera, area: [f64; 4]) -> Rect {
    let (hw, hh) = (f.w / 2.0, f.h / 2.0);
    let pts =
        [[-hw, -hh], [hw, -hh], [hw, hh], [-hw, hh]].map(|l| camera.to_screen(area, f.point(l)));
    let x0 = pts.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
    let y0 = pts.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
    let x1 = pts.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
    let y1 = pts.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);
    Rect::new(x0 as f32, y0 as f32, (x1 - x0) as f32, (y1 - y0) as f32)
}

fn paint_panel(scene: &mut VectorScene, rects: &[(Item, Rect)], theme: Theme) {
    let Some(first) = rects.first().map(|(_, r)| *r) else {
        return;
    };
    let (mut x0, mut y0, mut x1, mut y1) = (first.x, first.y, first.x + first.w, first.y + first.h);
    for (_, r) in rects {
        x0 = x0.min(r.x);
        y0 = y0.min(r.y);
        x1 = x1.max(r.x + r.w);
        y1 = y1.max(r.y + r.h);
    }
    let pad = Spacing::Xs.px();
    let panel = Rect::new(x0 - pad, y0 - pad, x1 - x0 + 2.0 * pad, y1 - y0 + 2.0 * pad);
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, panel, radius, resolve(ColorToken::BgElev, theme));
    crate::paint::stroke_frame(
        scene,
        panel,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        1.0,
        resolve(ColorToken::Border, theme),
    );
}

/// A cor de documento de um item de cor.
fn swatch(it: Item, theme: Theme) -> Option<Rgba> {
    match it {
        Item::Fill(Some(i)) | Item::Stroke(Some(Some(i))) => Some(Rgba(HIGHLIGHTER_RGBA[i])),
        Item::Stroke(Some(None)) => Some(ink(theme)),
        _ => None,
    }
}

fn ink(theme: Theme) -> Rgba {
    let c = ColorToken::Text1.resolve(theme);
    Rgba([c.r, c.g, c.b, c.a])
}

fn doc_color(Rgba([r, g, b, a]): Rgba) -> Color {
    Color::from_rgba8(r, g, b, a) // LITERAL-COLOR-OK: cor do DOCUMENTO (dado do artista), não da UI
}

/// O item corresponde ao estilo actual da selecção?
fn is_current(it: Item, s: &Style, arrow: Option<&Connector>, theme: Theme) -> bool {
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

#[allow(clippy::too_many_arguments)]
fn paint_item(
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
    let hover = matches!(
        store.button_state(id),
        Some(ButtonState::Hovered | ButtonState::Pressed | ButtonState::Focused)
    );
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    if on {
        fill_rounded_rect(scene, r, radius, resolve(ColorToken::AccentSoft, theme));
    } else if hover {
        fill_rounded_rect(scene, r, radius, resolve(ColorToken::Bg2, theme));
    }
    let fg = resolve(
        if on {
            ColorToken::Accent
        } else {
            ColorToken::Text2
        },
        theme,
    );
    let inset = Spacing::Xs.px();
    let inner = Rect::new(
        r.x + inset,
        r.y + inset,
        r.w - 2.0 * inset,
        r.h - 2.0 * inset,
    );
    let line = f64::from(StrokeToken::Default.px());
    match it {
        Item::Tool(Tool::Select) => {
            paint_icon(scene, IconId::Select, inner, fg, StrokeToken::Default.px())
        }
        Item::Tool(Tool::Hand) => {
            paint_icon(scene, IconId::Pan, inner, fg, StrokeToken::Default.px())
        }
        Item::MoreShapes => paint_icon(
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
                    fill_rounded_rect(scene, inner, dot, doc_color(c));
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
    }
    hit_index.register(id, r);
}

/// Um traço de ícone com o MESMO tracejado que a forma desenha (`ph2d_board_render::style_stroke`).
fn stroke(scene: &mut VectorScene, p: &ph2d_vector::BezPath, c: Color, w: f64, d: Dash) {
    let s = ph2d_board_render::style_stroke(w, d);
    scene
        .inner_mut()
        .stroke(&s, Affine::IDENTITY, &Brush::Solid(c), None, p);
}

/// O ícone de uma seta: a MESMA geometria que o quadro desenha (`ph2d_board_route::drawn`), de
/// canto a canto de `r` — recta na diagonal, cotovelo/curva em Z.
fn arrow_icon(scene: &mut VectorScene, r: Rect, c: Color, w: f64, route: Route, heads: [Head; 2]) {
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
    let d = ph2d_board_route::drawn(&routed, heads, w);
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
fn shape_icon(scene: &mut VectorScene, t: ShapeType, r: Rect, c: Color, w: f64, round: bool) {
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

/// ⭐ Os cliques nas barras. Corre no pré-despacho (ids derivados não chegam a um handler de chrome).
pub fn apply_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let (WidgetEvent::Click(id) | WidgetEvent::DoubleClick(id)) = event else {
        return false;
    };
    let Some(it) = item_of(id) else {
        return false;
    };
    let theme = hero.theme;
    if it == Item::MoreShapes {
        hero.documents.live.shapes_open = !hero.documents.live.shapes_open;
        return true;
    }
    let Some((board, live)) = hero.documents.active_parts() else {
        return true;
    };
    let history = live.histories.entry(board.id).or_default();
    let ed = super::board_view::editor(&mut live.editor, theme);
    let doc = &mut board.doc;
    match it {
        Item::Tool(t) => ed.tool = t,
        Item::Pick(t) => {
            ed.tool = Tool::Shape(t);
            live.shapes_open = false;
        }
        Item::Fill(_) => {
            // A letra acompanha o fundo: escolher um preenchimento escolhe a tinta que se lê nele
            // (sem fundo, a do tema — a mesma do quadro).
            let c = swatch(it, theme);
            let ink = c.map_or_else(
                || super::board_view::default_style(theme).text_color,
                Rgba::readable_ink,
            );
            ed.set_style(doc, history, |s| {
                s.fill = c;
                s.text_color = ink;
            });
        }
        Item::Stroke(_) => {
            let c = swatch(it, theme);
            ed.set_style(doc, history, |s| s.stroke = c);
        }
        Item::Width(i) => ed.set_style(doc, history, |s| s.stroke_width = WIDTHS[i]),
        Item::Dash(d) => ed.set_style(doc, history, |s| s.dash = d),
        Item::Round(r) => ed.set_style(doc, history, |s| s.round = r),
        Item::Opacity(i) => ed.set_style(doc, history, |s| s.opacity = OPACITIES[i]),
        Item::Font(i) => ed.set_style(doc, history, |s| s.font_size = FONT_SIZES[i]),
        Item::Route(r) => ed.set_route(doc, history, r),
        Item::Head(i, h) => ed.set_head(doc, history, i, h),
        Item::MoreShapes => {}
    }
    true
}

#[cfg(test)]
#[path = "board_bar_tests.rs"]
mod tests;
