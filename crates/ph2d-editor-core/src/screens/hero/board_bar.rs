//! ⭐ **As barras do quadro** (MiroClone, W1) — a barra CURTA de ferramentas (decisão do dono 4)
//! encostada à esquerda da área, a grelha de todas as formas que ela abre, e a barra de ESTILO que
//! flutua por cima da selecção (preenchimento, contorno, espessura, traço, cantos, opacidade, letra).
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
use ph2d_board_model::{Dash, Rgba, Shape, ShapeType, Style};
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Affine, Brush, Color, Stroke, VectorScene};

/// ⛔ O salto tem os 16 bits do MEIO diferentes dos das abas de quadro (`…_0000_0003`): os dois
/// lados XOR-am números pequenos, e saltos que só diferem nos bits de baixo COLIDEM (medido: o
/// «mais formas» era a aba do quadro 1).
const SALT: u64 = 0xb0a2_d7ab_5ba2_0000;

/// Os tamanhos de letra que a barra oferece (unidades do mundo): P · M · G · GG (os do Excalidraw).
const FONT_SIZES: [f64; 4] = [16.0, 20.0, 28.0, 36.0];
/// As espessuras de contorno (unidades do mundo): fino · normal · grosso.
const WIDTHS: [f64; 3] = [1.0, 2.0, 4.0];
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
}

/// Os atalhos da barra curta, na ordem em que se pintam.
const TOOLBAR: [Item; 7] = [
    Item::Tool(Tool::Select),
    Item::Tool(Tool::Hand),
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
    v.extend(style_groups().into_iter().flatten());
    v
}

/// Os grupos da barra de estilo, da esquerda para a direita.
fn style_groups() -> Vec<Vec<Item>> {
    vec![
        std::iter::once(Item::Fill(None))
            .chain(PASTELS.map(|i| Item::Fill(Some(i))))
            .collect(),
        [Item::Stroke(None), Item::Stroke(Some(None))]
            .into_iter()
            .chain(VIVIDS.map(|i| Item::Stroke(Some(Some(i)))))
            .collect(),
        (0..WIDTHS.len()).map(Item::Width).collect(),
        vec![
            Item::Dash(Dash::Solid),
            Item::Dash(Dash::Dashed),
            Item::Dash(Dash::Dotted),
        ],
        vec![Item::Round(false), Item::Round(true)],
        (0..OPACITIES.len()).map(Item::Opacity).collect(),
        (0..FONT_SIZES.len()).map(Item::Font).collect(),
    ]
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
    Spacing::Xxs.px()
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
pub fn style_rects(area: Rect, sel: Rect) -> Vec<(Item, Rect)> {
    let (s, g) = (Spacing::Xl.px(), gap());
    let sep = Spacing::Sm.px();
    let groups = style_groups();
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
    let frame = (!ed.is_busy() && !ed.is_editing_text())
        .then(|| ed.frame(&board.doc))
        .flatten();
    let camera = board.camera;
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
    let (Some(f), Some(style)) = (frame, sel_style) else {
        return;
    };
    let sel = screen_box(&f, &camera, super::board_view::area_of(area));
    let bar = style_rects(area, sel);
    paint_panel(scene, &bar, theme);
    for (it, r) in bar {
        let on = is_current(it, &style, theme);
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

/// O estilo da primeira forma seleccionada (a barra mostra-o como «o actual»).
fn selected_style(ed: &Editor, doc: &ph2d_board_model::BoardDoc) -> Option<Style> {
    ed.selection()
        .iter()
        .find_map(|id| doc.get(*id)?.shape().map(|s| s.style.clone()))
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
    Color::from_rgba8(r, g, b, a)
}

/// O item corresponde ao estilo actual da selecção?
fn is_current(it: Item, s: &Style, theme: Theme) -> bool {
    match it {
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
                let k = inner.w * 0.3;
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
            let a = (f32::from(OPACITIES[i]) / 100.0 * 255.0) as u8;
            let c = ColorToken::Text1.resolve(theme);
            fill_rounded_rect(scene, inner, radius, Color::from_rgba8(c.r, c.g, c.b, a));
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

fn stroke(scene: &mut VectorScene, p: &ph2d_vector::BezPath, c: Color, w: f64, d: Dash) {
    let s = match d {
        Dash::Solid => Stroke::new(w),
        Dash::Dashed => Stroke::new(w).with_dashes(0.0, [w * 3.0, w * 2.0]),
        Dash::Dotted => Stroke::new(w)
            .with_caps(ph2d_vector::Cap::Round)
            .with_dashes(0.0, [0.0, w * 2.5]),
    };
    scene
        .inner_mut()
        .stroke(&s, Affine::IDENTITY, &Brush::Solid(c), None, p);
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
    let (bw, bh) = (f64::from(r.w), f64::from(r.h) * 0.75);
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
            let c = swatch(it, theme);
            ed.set_style(doc, history, |s| s.fill = c);
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
        Item::MoreShapes => {}
    }
    true
}

#[cfg(test)]
#[path = "board_bar_tests.rs"]
mod tests;
