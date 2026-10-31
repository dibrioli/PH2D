//! ⭐ **As barras de rolagem** (smoke do dono 30/10): a do quadro, por baixo das listas, e a de cada
//! lista, numa margem própria à direita (D16: a lista que rola estreita os cartões, como o Trello).
//! A calha sai da disposição; as contas são as da [`AxisBar`]; as setas rolam um passo; carregar na
//! calha leva o polegar lá e agarra-o, e o arrasto segue até largar.

use crate::layout::{Layout, ListBox};
use crate::metrics as m;
use crate::paint::c;
use crate::tab::{Hit, Tab};
use ph2d_editor_core::widget::{ArrowHold, AxisBar};
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::ListId;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::VectorScene;

/// Uma barra agarrada: a de uma lista (`None` = a do quadro), onde agarrou e a rolagem de então;
/// `hold` = a seta segura (o arrasto só move o rato; segurar repete, D17).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Grab {
    pub list: Option<ListId>,
    pub from: (f32, f32),
    pub s0: f32,
    pub hold: Option<ArrowHold>,
}

/// A calha do quadro: na margem de baixo, por baixo das listas.
pub(crate) fn board_track(board: Rect, bottom: f32) -> Rect {
    let w = (board.w - 2.0 * m::BOARD_PAD).max(0.0);
    Rect::new(
        board.x + m::BOARD_PAD,
        bottom + (m::BOARD_PAD - m::SCROLL_BAR) / 2.0,
        w,
        m::SCROLL_BAR,
    )
}

fn of_board(l: &Layout) -> AxisBar {
    AxisBar {
        track: l.hbar,
        horizontal: true,
        content: l.content_w,
        visible: l.board.w,
        arrows: true,
        step: m::SCROLL_STEP,
    }
}

fn of_list(lb: &ListBox) -> AxisBar {
    let b = lb.body;
    let h = (b.h - m::LIST_BODY_TOP - m::LIST_BODY_BOTTOM).max(0.0);
    let x = b.x + b.w - m::LIST_GUTTER + (m::LIST_GUTTER - m::LIST_BAR) / 2.0;
    AxisBar {
        track: Rect::new(x, b.y + m::LIST_BODY_TOP, m::LIST_BAR, h),
        horizontal: false,
        content: lb.content_h,
        visible: h,
        arrows: true,
        step: m::SCROLL_STEP,
    }
}

fn bar(l: &Layout, list: Option<ListId>) -> Option<AxisBar> {
    let b = match list {
        None => of_board(l),
        Some(id) => of_list(l.list(id).filter(|lb| !lb.collapsed)?),
    };
    b.needed().then_some(b)
}

#[allow(clippy::cast_possible_truncation)]
fn scroll(tab: &Tab, list: Option<ListId>) -> f32 {
    match list {
        None => tab.view.scroll_x as f32,
        Some(id) => tab.scroll_y(id),
    }
}

fn set(tab: &mut Tab, list: Option<ListId>, s: f32) {
    match list {
        None => tab.view.scroll_x = f64::from(s),
        Some(id) => {
            tab.view.scroll_y.insert(id, f64::from(s));
        }
    }
}

pub(crate) fn probe(tab: &Tab, list: Option<ListId>) -> Option<(Rect, Rect)> {
    let b = bar(&tab.layout, list)?;
    Some((b.track, b.thumb(scroll(tab, list))))
}

/// A barra em `(x, y)` — o alvo é a margem inteira, não só o polegar.
pub(crate) fn hit(l: &Layout, x: f32, y: f32) -> Option<Hit> {
    if let Some(b) = bar(l, None) {
        let pad = (m::BOARD_PAD - m::SCROLL_BAR) / 2.0;
        let t = b.track;
        if Rect::new(t.x, t.y - pad, t.w, m::BOARD_PAD).contains(x, y) {
            return Some(Hit::Bar(None));
        }
    }
    l.lists.iter().find_map(|lb| {
        let t = bar(l, Some(lb.id))?.track;
        let edge = lb.body.x + lb.body.w - m::LIST_GUTTER;
        Rect::new(edge, t.y, m::LIST_GUTTER, t.h)
            .contains(x, y)
            .then_some(Hit::Bar(Some(lb.id)))
    })
}

/// Carregar numa barra: na seta rola um passo; senão agarra-a (na calha, o polegar salta lá
/// primeiro). `false` = não era barra.
pub(crate) fn press(tab: &mut Tab, h: Hit, p: (f32, f32), now_ns: u128) -> bool {
    let Hit::Bar(list) = h else {
        return false;
    };
    let Some(b) = bar(&tab.layout, list) else {
        return false;
    };
    let (s0, hold) = b.press_held(p, scroll(tab, list), now_ns);
    set(tab, list, s0);
    tab.bar = Some(Grab {
        list,
        from: p,
        s0,
        hold,
    });
    true
}

/// Cada quadro, as setas seguras repetem ([`AxisBar::repeat`]): nas abas, no canvas do Quadro e
/// na lista suspensa do verso; enquanto houver uma, a família pede quadros.
pub(crate) fn tick_all(docs: &mut crate::docs::KanbanDocs) {
    let (mut held, now) = (false, docs.clock.0);
    let mut canvas = docs.canvas.borrow_mut();
    for t in docs.tabs.iter_mut().chain(canvas.tabs.values_mut()) {
        if let Some(
            g @ Grab {
                hold: Some(mut h), ..
            },
        ) = t.bar
        {
            if let Some(b) = bar(&t.layout, g.list) {
                set(t, g.list, b.repeat(&mut h, scroll(t, g.list), now));
            }
            (t.bar, held) = (Some(Grab { hold: Some(h), ..g }), true);
        }
        if let Some(crate::back::Pop::Choice(c)) = t.back.as_mut().and_then(|b| b.pop.as_mut()) {
            held |= c.s.tick(now);
        }
    }
    drop(canvas);
    docs.held = held;
}

/// O arrasto da barra agarrada. `false` = nenhuma agarrada.
pub(crate) fn drag(tab: &mut Tab, p: (f32, f32)) -> bool {
    let Some(g) = tab.bar.as_mut() else {
        return false;
    };
    if let Some(h) = g.hold.as_mut() {
        h.at = p;
        return true;
    }
    let g = *g;
    if let Some(b) = bar(&tab.layout, g.list) {
        set(tab, g.list, b.drag(g.from, g.s0, p));
    }
    true
}

/// Pinta as barras; `light` = o chão do quadro é claro (a barra do quadro leva as cores da lista).
pub(crate) fn paint(scene: &mut VectorScene, theme: Theme, tab: &Tab, light: bool) {
    let on = |list| tab.hover == Some(Hit::Bar(list)) || tab.bar.is_some_and(|g| g.list == list);
    for lb in tab.layout.lists.iter().filter(|lb| !lb.collapsed) {
        let thumb = if on(Some(lb.id)) {
            T::KanbanListFg
        } else {
            T::KanbanListMeta
        };
        let colors = (c(theme, T::KanbanSlot), c(theme, thumb));
        of_list(lb).paint(tab.scroll_y(lb.id), colors, scene);
    }
    let (track, thumb, strong) = if light {
        (T::KanbanList, T::KanbanListMeta, T::KanbanListFg)
    } else {
        (T::KanbanHeaderBtn, T::KanbanHeaderBtnOn, T::KanbanHeaderFg)
    };
    let thumb = if on(None) { strong } else { thumb };
    let colors = (c(theme, track), c(theme, thumb));
    of_board(&tab.layout).paint(scroll(tab, None), colors, scene);
}
