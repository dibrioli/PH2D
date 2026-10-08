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
use ph2d_board_edit::Tool;
use ph2d_board_model::{Connector, Dash, Head, Rgba, Route, ShapeType, Style};
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Color, VectorScene};

/// ⛔ O salto tem os 16 bits do MEIO diferentes dos das abas de quadro (`…_0000_0003`): os dois
/// lados XOR-am números pequenos, e saltos que só diferem nos bits de baixo COLIDEM (medido: o
/// «mais formas» era a aba do quadro 1).
const SALT: u64 = 0xb0a2_d7ab_5ba2_0000;

/// O balão, a selecção e os ícones (módulo filho: o ficheiro estava no tecto de LOC).
#[path = "board_bar_look.rs"]
mod look;
pub use look::shape_name_key;
use look::{
    arrow_icon, ink, paint_panel, selected_arrow, selected_kind, selected_style, selection_box,
    shape_icon, shown, stroke, tooltip_key,
};

/// As notas e o texto (W3, módulo filho: o ficheiro estava no tecto de LOC).
#[path = "board_bar_notes.rs"]
mod notes;
pub use notes::{flyout_open, flyout_rects};

/// Onde fica cada controlo (módulo filho: o ficheiro estava no tecto de LOC).
#[path = "board_bar_layout.rs"]
mod layout;
use layout::{btn_px, gap};
pub use layout::{shapes_rects, style_rects, toolbar_rects};

/// Organizar (W5, módulo filho): o Frame, agrupar, travar, ordem, alinhar e o aviso das recusas.
#[path = "board_bar_organize.rs"]
mod organize;
pub use organize::OrgItem;

/// A caneta e o rascunho (W4, módulo filho: o ficheiro estava no tecto de LOC).
#[path = "board_bar_pen.rs"]
mod pen;
pub use pen::{PenItem, pen_box};

use ph2d_board_edit::{FONT_SIZES, STROKE_WIDTHS as WIDTHS};
/// As opacidades oferecidas (%).
const OPACITIES: [u8; 4] = [25, 50, 75, 100];
/// Os preenchimentos claros (os pastéis do marcador) e as tintas fortes do contorno.
const PASTELS: std::ops::Range<usize> = 0..5;
const VIVIDS: std::ops::Range<usize> = 5..9;
/// O PRETO do contorno (pedido do dono, 08/10: *«para setas e shapes não temos a opção de cor
/// preta»*) — um índice depois das cores do marcador; é preto do DOCUMENTO, nunca a tinta do tema
/// (que se desenha clara num quadro escuro).
const BLACK: usize = HIGHLIGHTER_RGBA.len();
// LITERAL-COLOR-OK: o preto do documento, uma cor do artista.
const BLACK_RGBA: [u8; 4] = [0, 0, 0, 255];

/// A cor de documento `i` das amostras: as do marcador e, a seguir, o preto.
fn palette(i: usize) -> Rgba {
    Rgba(HIGHLIGHTER_RGBA.get(i).copied().unwrap_or(BLACK_RGBA))
}

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
    /// A cor `i` das 16 das notas do Miro (`STICKY_COLORS`).
    NoteColor(usize),
    /// O tamanho P/M/G das notas (`NOTE_SCALES`).
    NoteSize(usize),
    /// Nota quadrada (`false`) ou larga (`true`).
    NoteWide(bool),
    /// O modo EM MASSA: uma ideia por linha, cada linha uma nota.
    Bulk,
    /// Negrito, itálico, sublinhado, riscado.
    Mark(ph2d_board_model::Mark),
    /// A cor da letra de uma FORMA: `None` = a da forma; `Some(i)` = a forte `i` do marcador.
    TextColor(Option<usize>),
    /// Um controlo da caneta (W4).
    Pen(PenItem),
    /// O botão «Rascunho ↔ Final»: a selecção, ou o quadro inteiro sem selecção.
    Sketch,
    /// Organizar (W5): agrupar, travar, ordem, alinhar, a cor de um frame.
    Organize(OrgItem),
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
    /// Só notas (ou pilhas).
    Notes,
    /// A escrever numa nota.
    TextNote,
    /// A escrever numa forma.
    TextShape,
    /// Só traços da caneta.
    Ink,
    /// Só frames (W5).
    Frames,
}

/// Os atalhos da barra curta, na ordem em que se pintam (a Nota logo a seguir à mão: é o coração
/// do brainstorm).
const TOOLBAR: [Item; 12] = [
    Item::Tool(Tool::Select),
    Item::Tool(Tool::Hand),
    Item::Tool(Tool::Shape(ShapeType::Sticky)),
    organize::FRAME_TOOL,
    Item::Tool(Tool::Connector),
    pen::PEN_TOOL,
    Item::Tool(Tool::Shape(ShapeType::Rectangle)),
    Item::Tool(Tool::Shape(ShapeType::Ellipse)),
    Item::Tool(Tool::Shape(ShapeType::Diamond)),
    Item::Tool(Tool::Shape(ShapeType::Triangle)),
    Item::MoreShapes,
    Item::Sketch,
];

/// ⭐ **A tabela** — todo controlo das barras, por grupo da barra de estilo. O índice é o id.
#[must_use]
pub fn items() -> Vec<Item> {
    let mut v: Vec<Item> = TOOLBAR.to_vec();
    v.extend(ShapeType::ALL.iter().map(|t| Item::Pick(*t)));
    v.extend(notes::flyout_items());
    v.extend(pen::flyout_items());
    for it in [
        Selected::Shapes,
        Selected::Arrows,
        Selected::Notes,
        Selected::TextShape,
        Selected::Ink,
        Selected::Frames,
    ]
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
        .chain(std::iter::once(Item::Stroke(Some(Some(BLACK)))))
        .collect();
    let width: Vec<Item> = (0..WIDTHS.len()).map(Item::Width).collect();
    let dash = vec![
        Item::Dash(Dash::Solid),
        Item::Dash(Dash::Dashed),
        Item::Dash(Dash::Dotted),
    ];
    let opacity: Vec<Item> = (0..OPACITIES.len()).map(Item::Opacity).collect();
    let font: Vec<Item> = (0..FONT_SIZES.len()).map(Item::Font).collect();
    let org = organize::groups(sel);
    let mut groups: Vec<Vec<Item>> = match sel {
        Selected::Shapes => vec![
            fill,
            stroke,
            width,
            dash,
            vec![Item::Round(false), Item::Round(true)],
            opacity,
            font,
        ]
        .into_iter()
        .chain(notes::shape_text_groups())
        .collect(),
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
        Selected::Notes => notes::note_groups(),
        Selected::TextNote => notes::text_groups(false),
        Selected::TextShape => notes::text_groups(true),
        Selected::Ink => pen::ink_groups(),
        Selected::Frames => vec![organize::frame_fill_group()],
    };
    groups.extend(org);
    groups
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
    let now = notes::Now::of(ed, &board.doc);
    let org = organize::Now::of(ed, &board.doc);
    if let Some(n) = ed.take_notice() {
        live.notice = Some((n.key(), std::time::Instant::now()));
    }
    let notice = live
        .notice
        .filter(|(_, t)| t.elapsed().as_secs_f32() < crate::toast::Toast::DEFAULT_TTL_S);
    let area_w = super::board_view::area_of(area);
    // A escrever, a barra fica (as marcas do texto); só um arrasto a esconde.
    let sel_box = (!ed.is_busy())
        .then(|| selection_box(ed, &board.doc, &board.camera, area_w))
        .flatten();
    let tools = toolbar_rects(area);
    paint_panel(scene, &tools, theme);
    let note_tool = Item::Tool(Tool::Shape(ShapeType::Sticky));
    let sketch_on = look::sketch_on(ed, &board.doc, board.sketch);
    for (it, r) in tools {
        let on = matches!(it, Item::Tool(t) if t == tool)
            || (it == Item::MoreShapes && shapes_open)
            || (it == note_tool && notes::flyout_open(tool))
            || (it == pen::PEN_TOOL && pen::flyout_open(tool))
            || (it == Item::Sketch && sketch_on)
            || (it == organize::FRAME_TOOL && tool == Tool::Frame);
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
    if notes::flyout_open(tool) {
        let (hit, store) = (&mut hero.hit_index, &hero.store);
        notes::paint_flyout(scene, text_system, theme, hit, store, area, tool, &now);
    }
    if pen::flyout_open(tool) {
        let (hit, store) = (&mut hero.hit_index, &hero.store);
        pen::paint_flyout(scene, text_system, theme, hit, store, area, ed);
    }
    if let Some((key, _)) = notice {
        organize::paint_notice(scene, text_system, theme, area, key);
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
    let groups: Vec<Vec<Item>> = style_groups(what)
        .into_iter()
        .map(|g| {
            g.into_iter()
                .filter(|it| org.shows(*it) != Some(false))
                .collect()
        })
        .filter(|g: &Vec<Item>| !g.is_empty())
        .collect();
    let bar = layout::style_rects_of(area, sel, groups);
    paint_panel(scene, &bar, theme);
    for (it, r) in bar {
        let on = org
            .is_on(it, style.fill)
            .or_else(|| notes::is_current(it, Some(&style), &now))
            .or_else(|| pen::is_current(it, ed, Some(style.stroke_width), theme))
            .unwrap_or_else(|| is_current(it, &style, arrow.as_ref(), theme));
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

/// A cor de documento de um item de cor.
fn swatch(it: Item, theme: Theme) -> Option<Rgba> {
    match it {
        Item::Fill(Some(i)) | Item::Stroke(Some(Some(i))) => Some(palette(i)),
        Item::Stroke(Some(None)) => Some(ink(theme)),
        _ => None,
    }
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
    let fg = super::float_bar::paint_button(
        scene,
        hit_index,
        store,
        theme,
        super::float_bar::Btn {
            id,
            rect: r,
            on,
            tint: ColorToken::Text2,
        },
    );
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    let inner = super::float_bar::inner(r);
    let line = f64::from(StrokeToken::Default.px());
    if notes::paint_icon(scene, text_system, theme, it, r, inner, fg, line)
        || pen::paint_icon(scene, theme, it, inner, fg, line)
        || organize::paint_icon(scene, theme, it, inner, fg, line)
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
        | Item::Tool(Tool::Pen(_) | Tool::Eraser { .. } | Tool::Laser | Tool::Frame)
        | Item::Pen(_)
        | Item::Sketch
        | Item::Organize(_) => {}
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
    if it == Item::Organize(OrgItem::Tag) {
        super::board_nav::open_tag_field(hero);
        return true;
    }
    let Some((board, live)) = hero.documents.active_parts() else {
        return true;
    };
    let history = live.histories.entry(board.id).or_default();
    let ed = super::board_view::editor(&mut live.editor, theme);
    let center = [board.camera.center_x, board.camera.center_y];
    let (doc, sketch) = (&mut board.doc, &mut board.sketch);
    if notes::apply(ed, doc, history, it, center)
        || pen::apply(ed, doc, history, sketch, theme, it)
        || organize::apply(ed, doc, history, it)
    {
        if let Some(n) = ed.take_notice() {
            live.notice = Some((n.key(), std::time::Instant::now()));
        }
        return true;
    }
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
                || super::board_view::default_style().text_color,
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
        Item::MoreShapes
        | Item::NoteColor(_)
        | Item::NoteSize(_)
        | Item::NoteWide(_)
        | Item::Bulk
        | Item::Mark(_)
        | Item::TextColor(_)
        | Item::Pen(_)
        | Item::Sketch
        | Item::Organize(_) => {}
    }
    true
}

#[cfg(test)]
#[path = "board_bar_tests.rs"]
mod tests;
