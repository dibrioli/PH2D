//! As barras do quadro para as NOTAS e o TEXTO (W3, módulo filho de [`super`]): o painel da
//! ferramenta Nota (as 16 cores do Miro, P/M/G, quadrada/larga, a pilha, o modo em massa), os
//! grupos da barra de estilo de uma selecção de notas, e os do texto — negrito, itálico, sublinhado,
//! riscado e, numa FORMA, a cor da letra (as notas do Miro não a mudam: help «Fonts»).

use super::{Item, Selected, VIVIDS, btn_px, doc_color, gap, toolbar_rects};
use crate::paint::{fill_rounded_rect, paint_text_centered};
use crate::widget::panel_chrome::HIGHLIGHTER_RGBA;
use crate::zones::Rect;
use ph2d_board_edit::{Editor, NOTE_SCALES, NoteStyle, Tool};
use ph2d_board_model::{BoardDoc, History, Mark, Rgba, STICKY_COLORS, ShapeType, Style};
use ph2d_i18n::tr;
use ph2d_tokens::{Spacing, TypeToken};
use ph2d_vector::{Color, VectorScene};

const MARKS: [Mark; 4] = [Mark::Bold, Mark::Italic, Mark::Underline, Mark::Strike];
/// Colunas do painel da ferramenta Nota (as 16 cores em 4 × 4).
const FLYOUT_COLS: usize = 4;
/// As proporções dos ÍCONES daqui, em fracções do miolo do botão — o desenho do glifo, não
/// espaçamento da interface.
const ICON_SHAPE_H: f32 = 0.6; // LITERAL-PX-OK: altura do glifo quadrada/larga
const ICON_LINE_STEP: f32 = 0.25; // LITERAL-PX-OK: três linhas a ¼, ½ e ¾ do miolo
const ICON_UNDERLINE_Y: f32 = 0.92; // LITERAL-PX-OK: o traço do «U» encostado ao fundo
const ICON_MARK_X: [f32; 2] = [0.25, 0.75]; // LITERAL-PX-OK: o traço do «U»/«S» na metade do meio
const ICON_SWATCH_BAR: f32 = 0.15; // LITERAL-PX-OK: a barra de cor debaixo do «A»
const ICON_NOTE_SIDE: [f32; 2] = [0.9, 0.75]; // LITERAL-PX-OK: a nota, e a de cima da pilha
const ICON_STACK_STEP: f32 = 0.15; // LITERAL-PX-OK: quanto cada folha da pilha espreita
const ICON_FOLD: f32 = 0.3; // LITERAL-PX-OK: a ponta dobrada da nota

/// O painel da ferramenta Nota, na ordem da grelha.
pub(super) fn flyout_items() -> Vec<Item> {
    (0..STICKY_COLORS.len())
        .map(Item::NoteColor)
        .chain((0..NOTE_SCALES.len()).map(Item::NoteSize))
        .chain([
            Item::NoteWide(false),
            Item::NoteWide(true),
            Item::Tool(Tool::Shape(ShapeType::StickyStack)),
            Item::Bulk,
        ])
        .collect()
}

/// A ferramenta activa é uma nota (ou a pilha)? Então o painel dela está aberto.
#[must_use]
pub fn flyout_open(tool: Tool) -> bool {
    matches!(tool, Tool::Shape(t) if t.is_note())
}

/// O painel à direita do botão Nota: as cores em 4 × 4, depois os tamanhos, depois as formas, a
/// pilha e o modo em massa — cada grupo numa linha nova.
#[must_use]
pub fn flyout_rects(area: Rect) -> Vec<(Item, Rect)> {
    let note = toolbar_rects(area)
        .into_iter()
        .find(|(it, _)| *it == Item::Tool(Tool::Shape(ShapeType::Sticky)))
        .map(|(_, r)| r)
        .expect("a barra tem a Nota");
    let (s, g) = (btn_px(), gap());
    let x0 = note.x + note.w + Spacing::Sm.px();
    let mut out = Vec::new();
    let mut row = 0.0;
    let mut place = |items: Vec<Item>, out: &mut Vec<(Item, Rect)>| {
        for chunk in items.chunks(FLYOUT_COLS) {
            for (c, it) in chunk.iter().enumerate() {
                let r = Rect::new(x0 + c as f32 * (s + g), note.y + row * (s + g), s, s);
                out.push((*it, r));
            }
            row += 1.0;
        }
    };
    let all = flyout_items();
    let colors = STICKY_COLORS.len();
    let sizes = colors + NOTE_SCALES.len();
    place(all[..colors].to_vec(), &mut out);
    place(all[colors..sizes].to_vec(), &mut out);
    place(all[sizes..].to_vec(), &mut out);
    out
}

/// Pinta o painel da ferramenta Nota (aberto): «actual» é o da PRÓXIMA nota.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_flyout(
    scene: &mut VectorScene,
    ts: &mut ph2d_text::TextSystem,
    theme: ph2d_tokens::Theme,
    hit: &mut crate::interaction::HitIndex,
    store: &crate::interaction::WidgetStore,
    area: Rect,
    tool: Tool,
    now: &Now,
) {
    let grid = flyout_rects(area);
    super::paint_panel(scene, &grid, theme);
    for (it, r) in grid {
        let on =
            matches!(it, Item::Tool(t) if t == tool) || is_current(it, None, now).unwrap_or(false);
        super::paint_item(scene, ts, theme, hit, store, it, r, on);
    }
}

fn marks() -> Vec<Item> {
    MARKS.iter().map(|m| Item::Mark(*m)).collect()
}

/// As cores da letra: a da forma e as fortes do marcador (os pastéis lêem-se mal sobre o fundo).
fn text_colors() -> Vec<Item> {
    std::iter::once(Item::TextColor(None))
        .chain(VIVIDS.map(|i| Item::TextColor(Some(i))))
        .collect()
}

/// Os grupos da barra de estilo de uma selecção de notas: as 16 cores, P/M/G, a forma, o texto.
pub(super) fn note_groups() -> Vec<Vec<Item>> {
    vec![
        (0..STICKY_COLORS.len()).map(Item::NoteColor).collect(),
        (0..NOTE_SCALES.len()).map(Item::NoteSize).collect(),
        vec![Item::NoteWide(false), Item::NoteWide(true)],
        marks(),
    ]
}

/// Os grupos a escrever: as marcas e, numa FORMA (`colour`), a cor da letra.
pub(super) fn text_groups(colour: bool) -> Vec<Vec<Item>> {
    if colour {
        vec![marks(), text_colors()]
    } else {
        vec![marks()]
    }
}

/// Os grupos do TEXTO que a barra de uma selecção de formas leva no fim.
pub(super) fn shape_text_groups() -> Vec<Vec<Item>> {
    vec![marks(), text_colors()]
}

/// Notas, ou a escrever (numa nota ou numa forma)? `None` = o resto decide.
pub(super) fn selected_kind(ed: &Editor, doc: &BoardDoc) -> Option<Option<Selected>> {
    if let Some(id) = ed.editing_id() {
        let el = doc.get(id)?;
        return Some(el.shape().map(|s| {
            if s.kind.is_note() {
                Selected::TextNote
            } else {
                Selected::TextShape
            }
        }));
    }
    let mut els = ed
        .selection()
        .iter()
        .filter_map(|id| doc.get(*id))
        .peekable();
    let notes = els.peek().is_some() && els.all(|el| el.shape().is_some_and(|s| s.kind.is_note()));
    notes.then_some(Some(Selected::Notes))
}

/// O que a barra mostra como «actual» nos controlos das notas e do texto.
pub(super) struct Now {
    pub marks: [bool; 4],
    pub text_color: Option<Option<Rgba>>,
    /// O tamanho P/M/G e a forma da primeira nota seleccionada.
    pub size: Option<usize>,
    pub kind: Option<ShapeType>,
    pub next: NoteStyle,
}

impl Now {
    pub(super) fn of(ed: &Editor, doc: &BoardDoc) -> Self {
        let first = ed
            .selection()
            .iter()
            .filter_map(|id| doc.get(*id))
            .find(|el| el.shape().is_some_and(|s| s.kind.is_note()));
        Self {
            marks: MARKS.map(|m| ed.has_mark(doc, m)),
            text_color: ed.text_color(doc),
            size: first.and_then(ph2d_board_edit::note_size),
            kind: first.and_then(|el| el.shape()).map(|s| s.kind),
            next: ed.notes,
        }
    }
}

/// `Some(ligado)` para os controlos das notas e do texto; `None` = não é daqui. `style` = o da
/// selecção (`None` no painel da ferramenta: lá, «actual» é o da PRÓXIMA nota).
pub(super) fn is_current(it: Item, style: Option<&Style>, now: &Now) -> Option<bool> {
    Some(match it {
        Item::NoteColor(i) => {
            let c = Rgba(STICKY_COLORS[i]);
            style.map_or(now.next.color == c, |s| s.fill == Some(c))
        }
        Item::NoteSize(i) => style.map_or(now.next.size == i, |_| now.size == Some(i)),
        Item::NoteWide(w) => style.map_or(now.next.wide == w, |_| {
            now.kind
                == Some(if w {
                    ShapeType::StickyWide
                } else {
                    ShapeType::Sticky
                })
        }),
        Item::Mark(m) => {
            now.marks[MARKS
                .iter()
                .position(|x| *x == m)
                .expect("é uma das quatro")]
        }
        Item::TextColor(c) => now.text_color == Some(c.map(|i| Rgba(HIGHLIGHTER_RGBA[i]))),
        Item::Bulk => false,
        _ => return None,
    })
}

/// O balão de cada controlo daqui.
pub(super) fn tooltip_key(it: Item) -> Option<&'static str> {
    Some(match it {
        Item::Tool(Tool::Shape(ShapeType::Sticky | ShapeType::StickyWide)) => "board.tool.note",
        Item::Tool(Tool::Shape(ShapeType::StickyStack)) => "board.tool.stack",
        Item::NoteColor(_) => "board.note.color",
        Item::NoteSize(_) => "board.note.size",
        Item::NoteWide(false) => "board.note.square",
        Item::NoteWide(true) => "board.note.wide",
        Item::Bulk => "board.note.bulk",
        Item::Mark(Mark::Bold) => "board.mark.bold",
        Item::Mark(Mark::Italic) => "board.mark.italic",
        Item::Mark(Mark::Underline) => "board.mark.underline",
        Item::Mark(Mark::Strike) => "board.mark.strike",
        Item::TextColor(None) => "board.text.auto_color",
        Item::TextColor(Some(_)) => "board.text.color",
        _ => return None,
    })
}

/// Pinta o ícone de `it` em `inner` (o botão é `r`). `false` = não é daqui.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_icon(
    scene: &mut VectorScene,
    ts: &mut ph2d_text::TextSystem,
    theme: ph2d_tokens::Theme,
    it: Item,
    r: Rect,
    inner: Rect,
    fg: Color,
    line: f64,
) -> bool {
    let radius = crate::paint::frame_radius(theme, ph2d_tokens::Radius::Sm.px());
    match it {
        Item::Tool(Tool::Shape(ShapeType::Sticky | ShapeType::StickyWide)) => {
            note_icon(scene, inner, fg, line, false);
        }
        Item::Tool(Tool::Shape(ShapeType::StickyStack)) => note_icon(scene, inner, fg, line, true),
        Item::NoteColor(i) => {
            fill_rounded_rect(scene, inner, radius, doc_color(Rgba(STICKY_COLORS[i])));
            // As claras (o cinzento, o amarelo-claro) somem no painel sem um fio à volta.
            super::stroke(
                scene,
                &rect_path(inner),
                fg,
                line / 2.0,
                ph2d_board_model::Dash::Solid,
            );
        }
        Item::NoteSize(i) => {
            let label = tr(["board.font.s", "board.font.m", "board.font.l"][i]);
            paint_text_centered(ts, scene, label, r, TypeToken::Sm.px(), fg);
        }
        Item::NoteWide(wide) => {
            // A quadrada e a larga com a proporção do Miro (199 × 199 e 350 × 199).
            let h = inner.h * ICON_SHAPE_H;
            let aspect = (ph2d_board_model::STICKY_WIDE / ph2d_board_model::STICKY_SIDE) as f32;
            let w = if wide { h * aspect } else { h }.min(inner.w);
            let b = Rect::new(
                inner.x + (inner.w - w) / 2.0,
                inner.y + (inner.h - h) / 2.0,
                w,
                h,
            );
            super::stroke(
                scene,
                &rect_path(b),
                fg,
                line,
                ph2d_board_model::Dash::Solid,
            );
        }
        Item::Bulk => {
            // Três ideias, uma por linha.
            for k in 0..3 {
                let y = inner.y + inner.h * ICON_LINE_STEP * (1 + k) as f32;
                let mut p = ph2d_vector::BezPath::new();
                p.move_to((f64::from(inner.x), f64::from(y)));
                p.line_to((f64::from(inner.x + inner.w), f64::from(y)));
                super::stroke(scene, &p, fg, line, ph2d_board_model::Dash::Solid);
            }
        }
        Item::Mark(m) => {
            let key = match m {
                Mark::Bold => "board.mark.b",
                Mark::Italic => "board.mark.i",
                Mark::Underline => "board.mark.u",
                Mark::Strike => "board.mark.s",
            };
            paint_text_centered(ts, scene, tr(key), r, TypeToken::Md.px(), fg);
            let y = match m {
                Mark::Underline => Some(inner.y + inner.h * ICON_UNDERLINE_Y),
                Mark::Strike => Some(inner.y + inner.h / 2.0),
                _ => None,
            };
            if let Some(y) = y {
                let mut p = ph2d_vector::BezPath::new();
                let [a, b] = ICON_MARK_X.map(|f| f64::from(inner.x + inner.w * f));
                p.move_to((a, f64::from(y)));
                p.line_to((b, f64::from(y)));
                super::stroke(scene, &p, fg, line, ph2d_board_model::Dash::Solid);
            }
        }
        Item::TextColor(c) => {
            let ink = c.map_or(fg, |i| doc_color(Rgba(HIGHLIGHTER_RGBA[i])));
            paint_text_centered(ts, scene, tr("board.text.a"), r, TypeToken::Md.px(), ink);
            let bh = inner.h * ICON_SWATCH_BAR;
            let bar = Rect::new(inner.x, inner.y + inner.h - bh, inner.w, bh);
            fill_rounded_rect(scene, bar, 0.0, ink);
        }
        _ => return false,
    }
    true
}

fn rect_path(r: Rect) -> ph2d_vector::BezPath {
    let (x0, y0) = (f64::from(r.x), f64::from(r.y));
    let (x1, y1) = (f64::from(r.x + r.w), f64::from(r.y + r.h));
    let mut p = ph2d_vector::BezPath::new();
    p.move_to((x0, y0));
    p.line_to((x1, y0));
    p.line_to((x1, y1));
    p.line_to((x0, y1));
    p.close_path();
    p
}

/// A nota: um quadrado com a ponta de baixo dobrada; a pilha, duas folhas por baixo dela.
fn note_icon(scene: &mut VectorScene, r: Rect, c: Color, w: f64, stack: bool) {
    let side = r.w.min(r.h) * ICON_NOTE_SIDE[usize::from(stack)];
    let (x0, y0) = (r.x + (r.w - side) / 2.0, r.y + (r.h - side) / 2.0);
    if stack {
        let d = side * ICON_STACK_STEP;
        for k in [2.0, 1.0] {
            let b = Rect::new(x0 + d * k, y0 + d * k, side, side);
            super::stroke(scene, &rect_path(b), c, w, ph2d_board_model::Dash::Solid);
        }
    }
    let fold = side * ICON_FOLD;
    let (x0, y0, x1, y1) = (
        f64::from(x0),
        f64::from(y0),
        f64::from(x0 + side),
        f64::from(y0 + side),
    );
    let f = f64::from(fold);
    let mut p = ph2d_vector::BezPath::new();
    p.move_to((x0, y0));
    p.line_to((x1, y0));
    p.line_to((x1, y1 - f));
    p.line_to((x1 - f, y1));
    p.line_to((x0, y1));
    p.close_path();
    p.move_to((x1, y1 - f));
    p.line_to((x1 - f, y1 - f));
    p.line_to((x1 - f, y1));
    super::stroke(scene, &p, c, w, ph2d_board_model::Dash::Solid);
}

/// O clique num controlo daqui. `center` = o centro da vista (onde o modo em massa abre). `false` =
/// não é daqui.
pub(super) fn apply(
    ed: &mut Editor,
    doc: &mut BoardDoc,
    history: &mut History,
    it: Item,
    center: [f64; 2],
) -> bool {
    match it {
        Item::Tool(Tool::Shape(ShapeType::Sticky | ShapeType::StickyWide)) => {
            ed.tool = Tool::Shape(ed.notes.kind());
        }
        Item::NoteColor(i) => ed.set_note_color(doc, history, Rgba(STICKY_COLORS[i])),
        Item::NoteSize(i) => ed.set_note_size(doc, history, i),
        Item::NoteWide(w) => {
            ed.set_note_wide(doc, history, w);
            if matches!(
                ed.tool,
                Tool::Shape(ShapeType::Sticky | ShapeType::StickyWide)
            ) {
                ed.tool = Tool::Shape(ed.notes.kind());
            }
        }
        Item::Bulk => {
            ed.begin_bulk(doc, history, center);
            ed.tool = Tool::Select;
        }
        Item::Mark(m) => {
            ed.toggle_mark(doc, history, m);
        }
        Item::TextColor(c) => {
            ed.set_text_color(doc, history, c.map(|i| Rgba(HIGHLIGHTER_RGBA[i])));
        }
        _ => return false,
    }
    true
}
