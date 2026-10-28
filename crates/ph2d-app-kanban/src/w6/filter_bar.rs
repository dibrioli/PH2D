//! ⭐ **O filtro** (a maquete `Filter.dc.html`): no cabeçalho, a caixa da pesquisa, o botão
//! «Filters» com o número de escolhas e o «Clear all»; o painel com a palavra, os membros, o
//! estado, as datas, a atividade, as etiquetas e «Any / Exact match». O filtro é da VISTA.

use super::sheet::{self, Item, Sheet};
use super::{Act, Boxed, Over, W6};
use crate::docs::KanbanDocs;
use crate::metrics as m;
use crate::paint::{c, text_top};
use crate::tab::Tab;
use ph2d_editor_core::IconId;
use ph2d_editor_core::doc_surface::{Key, KeyInput, KeyOutcome};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_elided, paint_text_title_elided};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_filter::{ActivityWindow, DueWindow, MatchMode};
use ph2d_kanban_model::Kanban;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Stroke, VectorScene};

/// A caixa da pesquisa no cabeçalho.
const SEARCH_W: f32 = 220.0;

/// O glifo do filtro (`M4 6h16M7 12h10M10 18h4`, a maquete), na grelha `24`.
const FILTER_GLYPH: [[(f64, f64); 2]; 3] = [
    [(4.0, 6.0), (20.0, 6.0)],
    [(7.0, 12.0), (17.0, 12.0)],
    [(10.0, 18.0), (14.0, 18.0)],
];

fn glyph(scene: &mut VectorScene, at: Rect, color: ph2d_vector::Color) {
    let mut p = BezPath::new();
    for l in FILTER_GLYPH {
        p.move_to(l[0]);
        p.line_to(l[1]);
    }
    let s = f64::from(m::ICON_PX) / m::ICON_GRID;
    let t = Affine::translate((
        f64::from(at.x).round(),
        f64::from(at.y + (at.h - m::ICON_PX) / 2.0).round(),
    )) * Affine::scale(s);
    scene.stroke_path(&Stroke::new(m::GLYPH_STROKE), t, &Brush::Solid(color), &p);
}

/// ⭐ Os botões do cabeçalho, da direita para a esquerda a partir de `right` (à esquerda do «Share» e
/// dos avatares): «Clear all» (com escolhas), «Filters», a pesquisa. Devolve o filtro e a caixa da
/// pesquisa, se coube (sem ela, a pesquisa ancora-se no filtro e o painel traz a caixa).
pub(crate) fn header(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    tab: &Tab,
    (left, right): (f32, f32),
) -> (Rect, Option<Rect>) {
    let l = &tab.layout;
    let w6 = &tab.w6;
    let n = w6.filter.selected();
    let fg = c(theme, T::KanbanHeaderFg);
    let y = l.board_menu.y;
    let mut x = right;
    let mut hits = Vec::new();
    let hover = |a: Act| w6.hover == Some(a);
    if n > 0 {
        let label = tr("kanban.w6.clear_all");
        let w = crate::docs::title_width(ts, label, m::BUTTON_TEXT_PX) + 2.0 * m::BUTTON_PAD_X;
        x -= w;
        let r = Rect::new(x, y, w, m::BUTTON_H);
        crate::paint::hover_fill(scene, r, hover(Act::ClearAll), theme, m::BUTTON_RADIUS);
        let ty = text_top(ts, r.y, r.h, m::BUTTON_TEXT_PX);
        paint_text_title_elided(
            ts,
            scene,
            label,
            r.x + m::BUTTON_PAD_X,
            ty,
            m::BUTTON_TEXT_PX,
            w,
            fg,
        );
        hits.push((r, Act::ClearAll));
        x -= m::HEADER_GAP;
    }
    // «Filters» e a pílula do número.
    let label = tr("kanban.w6.filters");
    let tw = crate::docs::title_width(ts, label, m::BUTTON_TEXT_PX);
    let count = n.to_string();
    let pw = if n > 0 {
        crate::docs::title_width(ts, &count, m::SMALL_PX) + 2.0 * m::COUNT_PAD_X
    } else {
        0.0
    };
    let w = m::BUTTON_PAD_X
        + m::ICON_PX
        + m::LIST_ICON_GAP
        + tw
        + m::BUTTON_PAD_X
        + if n > 0 { pw + m::LIST_ICON_GAP } else { 0.0 };
    x -= w;
    let filter = Rect::new(x, y, w, m::BUTTON_H);
    let on = n > 0 || w6.over == Some(Over::Filter);
    let ink = if on { c(theme, T::KanbanBtnOnFg) } else { fg };
    if on {
        fill_rounded_rect(
            scene,
            filter,
            m::BUTTON_RADIUS,
            c(theme, T::KanbanHeaderBtnOn),
        );
    } else {
        crate::paint::hover_fill(
            scene,
            filter,
            hover(Act::FilterButton),
            theme,
            m::BUTTON_RADIUS,
        );
    }
    let gx = filter.x + m::BUTTON_PAD_X;
    glyph(scene, Rect::new(gx, filter.y, m::ICON_PX, filter.h), ink);
    let tx = gx + m::ICON_PX + m::LIST_ICON_GAP;
    let ty = text_top(ts, filter.y, filter.h, m::BUTTON_TEXT_PX);
    paint_text_title_elided(ts, scene, label, tx, ty, m::BUTTON_TEXT_PX, tw + 1.0, ink);
    if n > 0 {
        let p = Rect::new(
            tx + tw + m::LIST_ICON_GAP,
            filter.y + (filter.h - m::COUNT_H) / 2.0,
            pw,
            m::COUNT_H,
        );
        fill_rounded_rect(scene, p, m::COUNT_RADIUS, c(theme, T::Accent));
        let py = text_top(ts, p.y, p.h, m::SMALL_PX);
        paint_text_title_elided(
            ts,
            scene,
            &count,
            p.x + m::COUNT_PAD_X,
            py,
            m::SMALL_PX,
            pw,
            c(theme, T::AccentFg),
        );
    }
    hits.push((filter, Act::FilterButton));
    // A pesquisa: cabe entre a estrela e o filtro, ou fica só no `/`.
    x -= m::HEADER_GAP + SEARCH_W;
    let search = Rect::new(x, y, SEARCH_W, m::BUTTON_H);
    // A pesquisa cabe entre o seletor de vistas (`left`) e o filtro.
    if search.x > left {
        match w6.input.as_ref().filter(|(b, _)| *b == Boxed::Query) {
            Some((_, f)) => crate::paint::field_box(scene, ts, theme, f, search, m::BUTTON_TEXT_PX),
            None => {
                let t = if hover(Act::SearchBox) {
                    T::KanbanHeaderBtnOn
                } else {
                    T::KanbanHeaderBtn
                };
                fill_rounded_rect(scene, search, m::BUTTON_RADIUS, c(theme, t));
                let ink = if hover(Act::SearchBox) {
                    c(theme, T::KanbanBtnOnFg)
                } else {
                    fg
                };
                let ir = Rect::new(search.x + m::BUTTON_PAD_X, search.y, m::ICON_PX, search.h);
                crate::paint::icon(scene, IconId::Search, ir, ink);
                let sx = ir.x + m::ICON_PX + m::LIST_ICON_GAP;
                let q = if w6.query.is_empty() {
                    tr("kanban.w6.search")
                } else {
                    w6.query.as_str()
                };
                paint_text_elided(
                    ts,
                    scene,
                    q,
                    sx,
                    ty,
                    m::BUTTON_TEXT_PX,
                    search.x + search.w - sx,
                    ink,
                );
            }
        }
        hits.push((search, Act::SearchBox));
    }
    tab.w6.hits.borrow_mut().extend(hits);
    (filter, (search.x > left).then_some(search))
}

const DUES: [(DueWindow, &str, T, T); 5] = [
    (
        DueWindow::NoDates,
        "kanban.w6.due.none",
        T::KanbanSlot,
        T::KanbanCardMeta,
    ),
    (
        DueWindow::Overdue,
        "kanban.w6.due.over",
        T::KanbanDueOver,
        T::KanbanDueOverFg,
    ),
    (
        DueWindow::NextDay,
        "kanban.w6.due.day",
        T::KanbanDueSoon,
        T::KanbanDueSoonFg,
    ),
    (
        DueWindow::NextWeek,
        "kanban.w6.due.week",
        T::KanbanSlot,
        T::KanbanCardMeta,
    ),
    (
        DueWindow::NextMonth,
        "kanban.w6.due.month",
        T::KanbanSlot,
        T::KanbanCardMeta,
    ),
];

const ACTIVITY: [(ActivityWindow, &str); 4] = [
    (ActivityWindow::LastWeek, "kanban.w6.act.week"),
    (ActivityWindow::LastTwoWeeks, "kanban.w6.act.two"),
    (ActivityWindow::LastFourWeeks, "kanban.w6.act.four"),
    (ActivityWindow::IdleFourWeeks, "kanban.w6.act.idle"),
];

/// ⭐ O painel do filtro.
pub(crate) fn sheet(k: &Kanban, tab: &Tab, team: &[String]) -> Sheet {
    let f = &tab.w6.filter;
    let mut s = Sheet::new(sheet::W);
    s.title("kanban.w6.filter");
    s.caption("kanban.w6.keyword");
    let field = match &tab.w6.input {
        Some((Boxed::Keyword, fl)) => Some(fl.clone()),
        _ => None,
    };
    s.push(
        Item::Input {
            field,
            text: f.keyword.clone(),
            placeholder: tr("kanban.w6.keyword_hint").to_owned(),
        },
        Act::Keyword,
    );
    s.push(
        Item::Note(tr("kanban.w6.keyword_note").to_owned()),
        Act::Inside,
    );
    // A palavra-chave fica sempre à vista (o cabeçalho fixo da maquete); o resto rola.
    s.pin();
    s.caption("kanban.w6.members");
    s.check(f.no_members, tr("kanban.w6.no_members"), Act::NoMembers);
    s.check(f.mine, tr("kanban.w6.mine"), Act::Mine);
    for (i, p) in super::people(k, team).into_iter().enumerate() {
        let on = f.members.contains(&p);
        s.push(
            Item::Check {
                on,
                text: p.0.clone(),
                swatch: None,
                person: Some(p),
                icon: None,
            },
            Act::Member(u16::try_from(i).unwrap_or(u16::MAX)),
        );
    }
    s.caption("kanban.w6.status");
    s.check(f.complete, tr("kanban.w6.complete"), Act::Complete);
    s.check(f.incomplete, tr("kanban.w6.incomplete"), Act::Incomplete);
    s.caption("kanban.w6.due");
    for (w, key, bg, ink) in DUES {
        s.push(
            Item::Check {
                on: f.due.contains(&w),
                text: tr(key).to_owned(),
                swatch: None,
                person: None,
                icon: Some((IconId::Timer, bg, ink)),
            },
            Act::Due(w),
        );
    }
    s.caption("kanban.w6.activity");
    for (w, key) in ACTIVITY {
        s.check(f.activity.contains(&w), tr(key), Act::Activity(w));
    }
    s.caption("kanban.w6.labels");
    s.check(f.no_labels, tr("kanban.w6.no_labels"), Act::NoLabels);
    for l in k.labels.values().filter(|l| !l.meta.deleted) {
        s.push(
            Item::Check {
                on: f.labels.contains(&l.id),
                text: l.name.clone(),
                swatch: l.color,
                person: None,
                icon: None,
            },
            Act::Label(l.id),
        );
    }
    s.push(
        Item::Segment {
            label: tr("kanban.w6.match").to_owned(),
            options: vec![
                (
                    tr("kanban.w6.match_any").to_owned(),
                    f.mode == MatchMode::Any,
                    Act::Mode(MatchMode::Any),
                ),
                (
                    tr("kanban.w6.match_exact").to_owned(),
                    f.mode == MatchMode::Exact,
                    Act::Mode(MatchMode::Exact),
                ),
            ],
        },
        Act::Inside,
    );
    s
}

fn flip<T: Ord>(set: &mut std::collections::BTreeSet<T>, x: T) {
    if !set.remove(&x) {
        set.insert(x);
    }
}

/// Limpa o filtro (o «Clear all» e o `X`).
pub(crate) fn clear(w: &mut W6) {
    w.filter = ph2d_kanban_filter::Filter::default();
    if let Some((Boxed::Keyword, f)) = w.input.as_mut() {
        *f = crate::edit::Field::open(crate::edit::Target::Scratch, "");
    }
}

/// Um clique no painel do filtro (ou no «Clear all»).
pub(crate) fn click(docs: &mut KanbanDocs, id: u64, a: Act) {
    let team = docs.team.clone();
    let people = docs
        .kanban_of(id)
        .and_then(|kid| docs.set.borrow().get(kid).map(|k| super::people(k, &team)))
        .unwrap_or_default();
    let Some(t) = docs.tab_mut(id) else {
        return;
    };
    let w = &mut t.w6;
    let f = &mut w.filter;
    match a {
        Act::ClearAll => clear(w),
        Act::Keyword => {
            let kw = f.keyword.clone();
            w.focus(Boxed::Keyword, &kw);
        }
        Act::Member(i) => {
            if let Some(p) = people.get(usize::from(i)) {
                flip(&mut f.members, p.clone());
            }
        }
        Act::NoMembers => f.no_members = !f.no_members,
        Act::Mine => f.mine = !f.mine,
        Act::Complete => f.complete = !f.complete,
        Act::Incomplete => f.incomplete = !f.incomplete,
        Act::Due(d) => flip(&mut f.due, d),
        Act::Activity(d) => flip(&mut f.activity, d),
        Act::Label(l) => flip(&mut f.labels, l),
        Act::NoLabels => f.no_labels = !f.no_labels,
        Act::Mode(md) => f.mode = md,
        _ => {}
    }
}

/// O teclado com o painel aberto: a palavra-chave escreve; `Esc` fecha.
pub(crate) fn key(docs: &mut KanbanDocs, id: u64, input: &KeyInput<'_>) -> KeyOutcome {
    if !input.pressed {
        return super::taken();
    }
    if input.key == Key::Escape {
        super::close_top(docs, id);
        return super::taken();
    }
    let has_box = docs.tab_mut(id).is_some_and(|t| t.w6.input.is_some());
    if !has_box {
        return KeyOutcome::default();
    }
    if input.key == Key::Enter {
        super::close_top(docs, id);
        return super::taken();
    }
    super::outcome(super::type_into(docs, id, input))
}
