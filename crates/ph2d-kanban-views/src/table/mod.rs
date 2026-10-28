//! ⭐ A vista **Tabela** (a do Trello Premium): uma linha por cartão à vista (o filtro da aba) e as
//! colunas Card, List, Labels, Members, Due date e uma por campo personalizado. O cabeçalho fica
//! fixo e ordena (ascendente → descendente → a ordem do quadro); a rolagem é virtual (só as linhas
//! visíveis se pintam); cada célula edita no lugar por um popover, e o título abre o verso.

pub mod edit;
pub mod layout;
mod metrics;
mod paint;

pub use edit::Pop;
pub use layout::{Column, Grid, Sort};

use ph2d_editor_core::doc_surface::{Key, KeyInput};
use ph2d_editor_core::widget::{
    DatePickerEvent, DatePickerLayout, ScrollbarState,
    scrollbar_delta_for_drag, scrollbar_thumb_rect, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_dates::civil;
use ph2d_kanban_model::{
    Card, CardId, CardRef, FieldKind, FieldValue, Kanban, KanbanId, KanbanSet, Plan,
};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

use crate::common::{Ctx, PointerAt, PointerKind, Reply, people};
use crate::datepick::Picker;
use metrics as m;

/// O estado da tabela (da VISTA, nunca do documento).
#[derive(Debug, Default)]
pub struct TableState {
    /// A ordem pedida (`None` = a do quadro).
    pub sort: Option<Sort>,
    /// A rolagem `(x, y)` em px.
    pub scroll: (f32, f32),
    /// O popover aberto.
    pub pop: Option<Pop>,
    /// A linha e a coluna sob o rato.
    pub hover: Option<(usize, Column)>,
    /// O polegar da barra a ser arrastado: `(y do rato, rolagem)` ao carregar.
    thumb_drag: Option<(f32, f32)>,
    thumb_hover: bool,
    /// O que o último quadro pintou (o rato lê a MESMA geometria).
    painted: Painted,
    /// As linhas do último quadro e a impressão do que as decidiu (o documento, a ordem, o filtro e
    /// o minuto): igual ⇒ não se refiltram nem reordenam 20 000 cartões a cada quadro.
    cache: Option<(Stamp, Vec<CardId>)>,
}

/// O que decide as linhas da tabela.
#[derive(Clone, Debug, PartialEq)]
struct Stamp {
    kanban: KanbanId,
    /// Quantos registos e a soma das versões de cartões, listas, etiquetas e campos (cada `put`
    /// sobe a versão do registo).
    records: [u64; 2],
    sort: Option<Sort>,
    filter: ph2d_kanban_filter::Filter,
    me: ph2d_kanban_model::Member,
    minute: i64,
}

impl Stamp {
    fn of(k: &Kanban, cx: &Ctx<'_>, sort: Option<Sort>) -> Self {
        let n = (k.cards.len() + k.lists.len() + k.labels.len() + k.fields.len()) as u64;
        let v = k.cards.values().map(|c| c.meta.version).sum::<u64>()
            + k.lists.values().map(|l| l.meta.version).sum::<u64>()
            + k.labels.values().map(|l| l.meta.version).sum::<u64>()
            + k.fields.values().map(|f| f.meta.version).sum::<u64>();
        Self {
            kanban: k.id,
            records: [n, v],
            sort,
            filter: cx.filter.clone(),
            me: cx.me.clone(),
            minute: cx.now.0.div_euclid(MINUTE_MS),
        }
    }
}

const MINUTE_MS: i64 = 60_000;

/// O que o último quadro pintou.
#[derive(Clone, Debug, Default)]
struct Painted {
    area: Rect,
    grid: Grid,
    rows: Vec<CardId>,
    /// As linhas que se pintaram (a rolagem virtual): `[first, end)`.
    drawn: std::ops::Range<usize>,
}

impl TableState {
    /// As linhas do último quadro, pela ordem à vista.
    #[must_use]
    pub fn rows(&self) -> &[CardId] {
        &self.painted.rows
    }

    /// As linhas que o último quadro pintou (a rolagem virtual).
    #[must_use]
    pub fn drawn(&self) -> std::ops::Range<usize> {
        self.painted.drawn.clone()
    }

    /// A geometria do último quadro.
    #[must_use]
    pub fn grid(&self) -> &Grid {
        &self.painted.grid
    }

    /// Clicar no cabeçalho de `col`: ascendente → descendente → a ordem do quadro.
    pub fn toggle_sort(&mut self, col: Column) {
        self.sort = match self.sort {
            Some(s) if s.by == col && !s.desc => Some(Sort { by: col, desc: true }),
            Some(s) if s.by == col => None,
            _ => Some(Sort {
                by: col,
                desc: false,
            }),
        };
    }
}

/// Pinta a tabela de `kanban` em `area`.
#[allow(clippy::too_many_arguments)]
pub fn paint(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    area: Rect,
    st: &mut TableState,
    set: &KanbanSet,
    kanban: KanbanId,
    cx: &Ctx<'_>,
) {
    let Some(k) = set.get(kanban) else {
        return;
    };
    let cols = layout::columns(k);
    let stamp = Stamp::of(k, cx, st.sort);
    let rows = match st.cache.take() {
        Some((s, rows)) if s == stamp => rows,
        _ => layout::rows(k, cx, st.sort),
    };
    let grid = Grid::new(area, &cols, rows.len(), st.scroll);
    st.scroll = (grid.scroll_x, grid.scroll_y);
    let drawn = grid.visible();
    let look = paint::Look {
        hover: st.hover,
        sort: st.sort,
        thumb: if st.thumb_drag.is_some() {
            ScrollbarState::Dragging
        } else if st.thumb_hover {
            ScrollbarState::Hovered
        } else {
            ScrollbarState::Normal
        },
    };
    paint::sheet(out, ts, theme, k, cx, &grid, &rows, drawn.clone(), &look);
    if let Some(pop) = &st.pop {
        paint::pop(out, ts, theme, k, cx, pop);
    }
    st.cache = Some((stamp, rows.clone()));
    st.painted = Painted {
        area,
        grid,
        rows,
        drawn,
    };
}

fn card_of<'k>(k: &'k Kanban, id: CardId) -> Option<&'k Card> {
    k.cards.get(&id).filter(|c| !c.meta.deleted)
}

fn track(g: &Grid) -> Rect {
    scrollbar_track_rect(g.body)
}

/// Abre o popover da célula `(i, col)`, ou edita já (a caixa de um campo), ou abre o verso.
fn press_cell(st: &mut TableState, k: &Kanban, cx: &Ctx<'_>, i: usize, col: Column) -> Reply {
    let p = &st.painted;
    let (Some(&id), Some(cell)) = (p.rows.get(i), p.grid.cell(i, col)) else {
        return Reply::taken();
    };
    let Some(card) = card_of(k, id) else {
        return Reply::taken();
    };
    let bounds = p.area;
    st.pop = match col {
        Column::Card => return Reply::open(CardRef { kanban: k.id, card: id }),
        Column::List => Some(Pop::List {
            card: id,
            dd: edit::list_dropdown(k, card),
            chip: cell,
        }),
        Column::Labels => {
            let n = k.labels.values().filter(|l| !l.meta.deleted).count();
            Some(Pop::Labels {
                card: id,
                at: edit::place(cell, edit::list_pop_h(n), bounds),
            })
        }
        Column::Members => {
            let n = people(k, cx).len();
            Some(Pop::Members {
                card: id,
                at: edit::place(cell, edit::list_pop_h(n), bounds),
            })
        }
        Column::Due => Some(Pop::Dates {
            card: id,
            picker: Box::new(Picker::new(
                id.0,
                card.start,
                card.due.map(|d| (d.at, d.remind_before_min)),
                cx.now,
                cx.zone,
                cx.week_start,
            )),
            at: edit::place(cell, DatePickerLayout::preferred_height(), bounds),
        }),
        Column::Field(f) => match k.fields.get(&f).map(|d| &d.kind) {
            Some(FieldKind::Checkbox) => {
                let on = matches!(card.fields.get(&f), Some(FieldValue::Checkbox(true)));
                let v = (!on).then_some(FieldValue::Checkbox(true));
                return Reply::with_plan(edit::set_field(k, card, f, v));
            }
            Some(FieldKind::Dropdown(_)) => edit::option_dropdown(k, card, f).map(|dd| Pop::Option {
                card: id,
                field: f,
                dd,
                chip: cell,
            }),
            _ => return Reply::open(CardRef { kanban: k.id, card: id }),
        },
    };
    Reply::taken()
}

/// Um evento do seletor de datas.
fn picker_event(
    st: &mut TableState,
    k: &Kanban,
    cx: &Ctx<'_>,
    card: CardId,
    ev: DatePickerEvent,
) -> Plan {
    let Some(Pop::Dates { picker, .. }) = st.pop.as_mut() else {
        return Plan::default();
    };
    let today = civil(cx.now.0, cx.zone).date;
    match ev {
        DatePickerEvent::Month(e) => picker.month_event(e, cx.now, cx.zone),
        DatePickerEvent::Pick { slot, date } => picker.picked(slot, date),
        DatePickerEvent::TextCommitted { field, text } => picker.typed(field, &text, today),
        DatePickerEvent::Save => {
            let (start, due) = picker.read(cx.zone);
            st.pop = None;
            return card_of(k, card).map_or_else(Plan::default, |c| edit::set_dates(k, c, start, due));
        }
        DatePickerEvent::Remove => {
            st.pop = None;
            return card_of(k, card).map_or_else(Plan::default, |c| edit::set_dates(k, c, None, None));
        }
        DatePickerEvent::Close => st.pop = None,
        DatePickerEvent::Toggled { .. } | DatePickerEvent::Reminder(_) => {}
    }
    Plan::default()
}

fn merge(into: &mut Plan, p: Plan) {
    for (k, ops) in p.into_steps() {
        into.extend(k, ops);
    }
}

/// O rato sobre o popover aberto. `None` = o rato não era dele.
fn pointer_pop(st: &mut TableState, k: &Kanban, cx: &Ctx<'_>, p: PointerAt) -> Option<Reply> {
    let pop = st.pop.as_mut()?;
    let card = pop.card();
    let c = card_of(k, card)?;
    if p.kind == PointerKind::Move {
        if let Pop::Dates { picker, at, .. } = pop {
            picker.widget.pointer_move(*at, p.x, p.y);
        }
        return Some(Reply::default());
    }
    if p.kind == PointerKind::Up {
        return Some(Reply::default());
    }
    let mut reply = Reply::taken();
    match pop {
        Pop::List { dd, chip, .. } => {
            let chip = *chip;
            let hit = (0..dd.options.len()).find(|&i| dd.option_rect(chip, i).contains(p.x, p.y));
            let chosen = hit.map(|i| dd.options[i].value);
            st.pop = None;
            if let Some(to) = chosen {
                reply.plan = edit::move_to(k, c, to);
            } else if chip.contains(p.x, p.y) {
                return Some(reply);
            }
        }
        Pop::Option { dd, chip, field, .. } => {
            let hit = (0..dd.options.len()).find(|&i| dd.option_rect(*chip, i).contains(p.x, p.y));
            let (f, chosen) = (*field, hit.map(|i| dd.options[i].value));
            st.pop = None;
            if let Some(v) = chosen {
                reply.plan = edit::set_field(k, c, f, v.map(FieldValue::Option));
            }
        }
        Pop::Labels { at, .. } => {
            if !at.contains(p.x, p.y) {
                st.pop = None;
                return Some(reply);
            }
            let labels = paint::pop_labels(k);
            if let Some(l) = (0..labels.len())
                .find(|&i| edit::pop_row(*at, i + 1).contains(p.x, p.y))
                .map(|i| labels[i].id)
            {
                reply.plan = edit::toggle_label(k, c, l);
            }
        }
        Pop::Members { at, .. } => {
            if !at.contains(p.x, p.y) {
                st.pop = None;
                return Some(reply);
            }
            let who = people(k, cx);
            if let Some(i) = (0..who.len()).find(|&i| edit::pop_row(*at, i + 1).contains(p.x, p.y)) {
                reply.plan = edit::toggle_member(k, c, &who[i]);
            }
        }
        Pop::Dates { picker, at, .. } => {
            if !at.contains(p.x, p.y) {
                st.pop = None;
                return Some(reply);
            }
            let evs = picker.widget.pointer_down(*at, p.x, p.y);
            for ev in evs {
                let plan = picker_event(st, k, cx, card, ev);
                merge(&mut reply.plan, plan);
            }
        }
    }
    Some(reply)
}

/// O rato em px da vista.
pub fn pointer(
    st: &mut TableState,
    set: &KanbanSet,
    kanban: KanbanId,
    cx: &Ctx<'_>,
    p: PointerAt,
) -> Reply {
    let Some(k) = set.get(kanban) else {
        return Reply::default();
    };
    if st.pop.is_some() {
        if let Some(r) = pointer_pop(st, k, cx, p) {
            return r;
        }
        st.pop = None;
    }
    let g = st.painted.grid.clone();
    match p.kind {
        PointerKind::Move => {
            if let Some((y0, s0)) = st.thumb_drag {
                let d = scrollbar_delta_for_drag(p.y - y0, track(&g).h, g.content_h(), g.body.h);
                st.scroll.1 = (s0 + d).clamp(0.0, g.max_scroll_y());
                return Reply::taken();
            }
            let hover = g.row_at(p.y).zip(g.col_at(p.x));
            let thumb = scrollbar_thumb_rect(track(&g), g.scroll_y, g.content_h(), g.body.h)
                .contains(p.x, p.y);
            let changed = hover != st.hover || thumb != st.thumb_hover;
            (st.hover, st.thumb_hover) = (hover, thumb);
            Reply {
                redraw: changed,
                ..Reply::default()
            }
        }
        PointerKind::Up => {
            let was = st.thumb_drag.take().is_some();
            Reply {
                taken: was,
                redraw: was,
                ..Reply::default()
            }
        }
        PointerKind::Down => {
            if !g.sheet.contains(p.x, p.y) {
                return Reply::default();
            }
            let tr = track(&g);
            if g.max_scroll_y() > 0.0 && tr.contains(p.x, p.y) {
                let thumb = scrollbar_thumb_rect(tr, g.scroll_y, g.content_h(), g.body.h);
                if !thumb.contains(p.x, p.y) {
                    let page = if p.y < thumb.y { -g.body.h } else { g.body.h };
                    st.scroll.1 = (g.scroll_y + page).clamp(0.0, g.max_scroll_y());
                }
                st.thumb_drag = Some((p.y, st.scroll.1));
                return Reply::taken();
            }
            if g.header.contains(p.x, p.y) {
                if let Some(col) = g.col_at(p.x) {
                    st.toggle_sort(col);
                }
                return Reply::taken();
            }
            match (g.row_at(p.y), g.col_at(p.x)) {
                (Some(i), Some(col)) => press_cell(st, k, cx, i, col),
                _ => Reply::taken(),
            }
        }
    }
}

/// A roda: `dy` do hero (positivo = para cima); `shift` rola na horizontal.
pub fn wheel(st: &mut TableState, at: (f32, f32), dy: f32, shift: bool) -> Reply {
    let g = &st.painted.grid;
    if !g.sheet.contains(at.0, at.1) || st.pop.is_some() {
        return Reply::default();
    }
    let px = -dy / m::WHEEL_LINE_PX * m::WHEEL_STEP;
    if shift {
        st.scroll.0 = (st.scroll.0 + px).max(0.0);
    } else {
        st.scroll.1 = (st.scroll.1 + px).clamp(0.0, g.max_scroll_y());
    }
    Reply::taken()
}

/// O teclado: o popover primeiro (o seletor de datas escreve); `Esc` fecha; páginas e setas rolam.
pub fn key(
    st: &mut TableState,
    set: &KanbanSet,
    kanban: KanbanId,
    cx: &Ctx<'_>,
    input: &KeyInput<'_>,
) -> Reply {
    let Some(k) = set.get(kanban) else {
        return Reply::default();
    };
    if let Some(Pop::Dates { picker, card, .. }) = st.pop.as_mut() {
        let card = *card;
        let (taken, ev) = picker.widget.key(input);
        let mut reply = Reply {
            taken,
            redraw: taken,
            ..Reply::default()
        };
        if let Some(ev) = ev {
            reply.plan = picker_event(st, k, cx, card, ev);
            reply.taken = true;
        }
        return reply;
    }
    if !input.pressed {
        return Reply::default();
    }
    let g = &st.painted.grid;
    let step = match input.key {
        Key::Escape if st.pop.is_some() => {
            st.pop = None;
            return Reply::taken();
        }
        Key::Up => -m::ROW_H,
        Key::Down => m::ROW_H,
        Key::PageUp => -g.body.h,
        Key::PageDown => g.body.h,
        _ => return Reply::default(),
    };
    st.scroll.1 = (st.scroll.1 + step).clamp(0.0, g.max_scroll_y());
    Reply::taken()
}
