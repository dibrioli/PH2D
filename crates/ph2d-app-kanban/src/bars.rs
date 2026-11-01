//! ⭐ **As barras de rolagem** (smoke do dono 30/10): a do quadro, por baixo das listas, e a de cada
//! lista, numa margem própria à direita (D16: a lista que rola estreita os cartões, como o Trello).
//! A calha sai da disposição; as contas são as da [`AxisBar`]; as setas rolam um passo; carregar na
//! calha leva o polegar lá e agarra-o, e o arrasto segue até largar.

use crate::layout::Layout;
use crate::metrics as m;
use crate::paint::c;
use crate::tab::{Hit, Tab};
use ph2d_editor_core::widget::{AxisBar, BarGrab};
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::ListId;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::VectorScene;

/// Uma barra agarrada: a de uma lista (`None` = a do quadro) e o agarrar dela ([`BarGrab`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Grab {
    pub list: Option<ListId>,
    pub g: BarGrab,
}

/// A margem do quadro (D17): a da lista deitada — a mesma margem e espessura, em baixo de `area`
/// (por dentro da folga dos lados, longe dos cantos das listas); as listas acabam no topo dela.
pub(crate) fn board_band(area: Rect) -> Rect {
    AxisBar::gutter(area, true, m::LIST_GUTTER, m::LIST_BAR).0
}

/// A margem de uma barra e a barra: a do quadro (`None`) e a de cada lista ([`AxisBar::in_gutter`]).
fn lane(l: &Layout, list: Option<ListId>) -> Option<(Rect, AxisBar)> {
    let (area, horizontal, content, visible) = match list {
        None => (l.hbar, true, l.content_w, l.board.w),
        Some(id) => {
            let b = l.list(id).filter(|lb| !lb.collapsed)?;
            let h = (b.body.h - m::LIST_BODY_TOP - m::LIST_BODY_BOTTOM).max(0.0);
            let area = Rect::new(b.body.x, b.body.y + m::LIST_BODY_TOP, b.body.w, h);
            (area, false, b.content_h, h)
        }
    };
    let g = (m::LIST_GUTTER, m::LIST_BAR);
    Some(AxisBar::in_gutter(
        area,
        horizontal,
        g,
        (content, visible, m::SCROLL_STEP),
    ))
}

fn bar(l: &Layout, list: Option<ListId>) -> Option<AxisBar> {
    lane(l, list).map(|(_, b)| b).filter(AxisBar::needed)
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
    let ids = std::iter::once(None).chain(l.lists.iter().map(|lb| Some(lb.id)));
    ids.into_iter().find_map(|id| {
        let (band, b) = lane(l, id)?;
        (b.needed() && band.contains(x, y)).then_some(Hit::Bar(id))
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
    let (s, g) = b.grab(p, scroll(tab, list), now_ns);
    set(tab, list, s);
    tab.bar = Some(Grab { list, g });
    true
}

/// Cada quadro, as setas seguras repetem ([`AxisBar::repeat`]): nas abas, no canvas do Quadro e
/// na lista suspensa do verso; enquanto houver uma, a família pede quadros.
pub(crate) fn tick_all(docs: &mut crate::docs::KanbanDocs) {
    let (mut held, now) = (false, docs.clock.0);
    let mut canvas = docs.canvas.borrow_mut();
    for t in docs.tabs.iter_mut().chain(canvas.tabs.values_mut()) {
        if let Some(Grab { list, mut g }) = t.bar
            && let Some(s) = bar(&t.layout, list).and_then(|b| b.tick(&mut g, scroll(t, list), now))
        {
            (t.bar, held) = (Some(Grab { list, g }), true);
            set(t, list, s);
        }
        held |= crate::w6::tick(t, now);
        if let Some(crate::back::Pop::Choice(c)) = t.back.as_mut().and_then(|b| b.pop.as_mut()) {
            held |= c.s.tick(now);
        }
    }
    drop(canvas);
    docs.held = held | crate::io::tick(docs, now);
}

/// O arrasto da barra agarrada. `false` = nenhuma agarrada.
pub(crate) fn drag(tab: &mut Tab, p: (f32, f32)) -> bool {
    let Some(Grab { list, mut g }) = tab.bar else {
        return false;
    };
    if let Some(b) = bar(&tab.layout, list) {
        let s = b.follow(&mut g, p, scroll(tab, list));
        set(tab, list, s);
        tab.bar = Some(Grab { list, g });
    }
    true
}

/// Pinta as barras: a do quadro com as tintas da lista (D17: a mesma barra, deitada).
pub(crate) fn paint(scene: &mut VectorScene, theme: Theme, tab: &Tab) {
    let on = |list| tab.hover == Some(Hit::Bar(list)) || tab.bar.is_some_and(|g| g.list == list);
    let ids = std::iter::once(None).chain(tab.layout.lists.iter().map(|lb| Some(lb.id)));
    for id in ids {
        let Some((_, b)) = lane(&tab.layout, id) else {
            continue;
        };
        let thumb = if on(id) {
            T::KanbanListFg
        } else {
            T::KanbanListMeta
        };
        b.paint(
            scroll(tab, id),
            (c(theme, T::KanbanSlot), c(theme, thumb)),
            scene,
        );
    }
}
