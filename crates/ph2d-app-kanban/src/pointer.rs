//! ⭐ **O rato na aba** — o que está sob ele ([`hit`], sobre a disposição VISTA), e o gesto: carregar,
//! decidir entre clique e arrasto (a folga da casa, `PRESS_SLOP_PX`), arrastar um cartão, uma lista
//! ou o fundo (rola), largar. Cada clique é um verbo do Trello.

use crate::docs::KanbanDocs;
use crate::edit::{Field, Target};
use crate::front;
use crate::menu::{Menu, Row as MenuRow};
use crate::metrics as m;
use crate::paint_card::edit_button;
use crate::tab::{Drag, Hit, Lift, Press, Tab};
use ph2d_editor_core::interaction::PRESS_SLOP_PX;
use ph2d_host::PointerButton;
use ph2d_kanban_model::{Kanban, RecordId};
use ph2d_text::TextSystem;

/// ⭐ O que está em `(x, y)`: o menu e o painel por cima, depois os campos, o cabeçalho e as listas.
pub(crate) fn hit(tab: &Tab, k: &Kanban, ts: &mut TextSystem, x: f32, y: f32) -> Option<Hit> {
    let l = &tab.layout;
    if !l.area.contains(x, y) {
        return None;
    }
    if let Some(p) = crate::overlay::menu_placed(tab)
        && let Some(row) = crate::menu::hit(&p, x, y)
    {
        return Some(row.map_or(Hit::Field, Hit::Menu));
    }
    if let Some(p) = crate::overlay::panel_placed(tab, k, ts)
        && let Some(row) = crate::archive::hit(&p, x, y)
    {
        return Some(Hit::Panel(row));
    }
    if l.header.contains(x, y) {
        return Some(if l.title.contains(x, y) {
            Hit::KanbanTitle
        } else if l.star.contains(x, y) {
            Hit::Star
        } else if l.board_menu.contains(x, y) {
            Hit::BoardMenu
        } else {
            Hit::Board
        });
    }
    if !l.board.contains(x, y) {
        return None;
    }
    if l.add_list_close.is_some_and(|r| r.contains(x, y)) {
        return Some(Hit::FieldClose);
    }
    if tab
        .field
        .as_ref()
        .is_some_and(|f| f.target == Target::NewList)
    {
        if l.add_list.contains(x, y) {
            return Some(Hit::FieldAdd);
        }
        if l.add_list_field.is_some_and(|r| r.contains(x, y)) {
            return Some(Hit::Field);
        }
    } else if l.add_list.contains(x, y) {
        return Some(Hit::AddList);
    }
    if let Some(lb) = l.lists.iter().find(|lb| lb.rect.contains(x, y)) {
        if lb.collapsed {
            return Some(Hit::ListHead(lb.id));
        }
        if lb.collapse.contains(x, y) {
            return Some(Hit::Collapse(lb.id));
        }
        if lb.menu.contains(x, y) {
            return Some(Hit::ListMenu(lb.id));
        }
        if lb.title.contains(x, y) {
            return Some(Hit::ListTitle(lb.id));
        }
        if let Some(close) = lb.close {
            if close.contains(x, y) {
                return Some(Hit::FieldClose);
            }
            if lb.add.contains(x, y) {
                return Some(Hit::FieldAdd);
            }
        } else if lb.add.contains(x, y) {
            return Some(Hit::AddCard(lb.id));
        }
        if y < lb.body.y {
            return Some(Hit::ListHead(lb.id));
        }
        if !lb.body.contains(x, y) {
            return Some(Hit::ListHead(lb.id));
        }
        if lb.composer.is_some_and(|r| r.contains(x, y)) {
            return Some(Hit::Field);
        }
        let Some(cb) = lb.cards.iter().find(|c| c.rect.contains(x, y)) else {
            return Some(Hit::Board);
        };
        if tab
            .field
            .as_ref()
            .is_some_and(|f| f.target == Target::CardTitle(cb.id))
        {
            return Some(Hit::Field);
        }
        if edit_button(cb.rect).contains(x, y) {
            return Some(Hit::CardEdit(cb.id));
        }
        if let Some(f) = front::front(k, &front::Counts::default(), cb.id)
            && !f.labels.is_empty()
        {
            let top = cb.rect.y + f.cover.map_or(0.0, |_| m::COVER_COLOR_H) + m::CARD_PAD_Y;
            if (top..top + front::labels_h(ts, &f, tab.label_text)).contains(&y) {
                return Some(Hit::CardLabel(cb.id));
            }
        }
        return Some(Hit::Card(cb.id));
    }
    Some(Hit::Board)
}

/// O campo aberto grava o que tem (clicar fora, a aba que deixa de estar activa) e fecha.
pub(crate) fn commit_field(docs: &mut KanbanDocs, id: u64) {
    let Some(f) = docs.tab_mut(id).and_then(|t| t.field.take()) else {
        return;
    };
    write_field(docs, id, &f);
}

/// Grava o campo. Devolve o cartão criado (o compositor seguinte abre depois dele).
fn write_field(docs: &mut KanbanDocs, id: u64, f: &Field) -> Option<ph2d_kanban_model::CardId> {
    let value = f.value()?;
    docs.with_doc(id, |_, doc| match f.target {
        Target::KanbanName => {
            doc.rename_kanban(value);
            None
        }
        Target::ListTitle(l) => {
            doc.rename_list(l, value);
            None
        }
        Target::CardTitle(c) => {
            doc.rename_card(c, value);
            None
        }
        Target::NewCard { list, after } => doc.add_card(list, after, value),
        Target::NewList => {
            doc.add_list(value);
            None
        }
    })
    .flatten()
}

/// ⭐ `Enter` (ou o botão azul): grava; um compositor fica aberto, vazio, para o seguinte.
pub(crate) fn enter(docs: &mut KanbanDocs, id: u64) {
    let Some(f) = docs.tab_mut(id).and_then(|t| t.field.take()) else {
        return;
    };
    let blank = f.value().is_none();
    let made = write_field(docs, id, &f);
    if !f.target.composes() || blank {
        return;
    }
    let next = match f.target {
        Target::NewCard { list, after } => Target::NewCard {
            list,
            after: after.and(made),
        },
        t => t,
    };
    if let Some(t) = docs.tab_mut(id) {
        t.field = Some(Field::open(next, ""));
    }
}

fn open_field(tab: &mut Tab, target: Target, text: &str) {
    tab.menu = None;
    tab.field = Some(Field::open(target, text));
}

/// Carregar. `true` = tomado.
pub(crate) fn down(
    docs: &mut KanbanDocs,
    id: u64,
    ts: &mut TextSystem,
    button: PointerButton,
    x: f32,
    y: f32,
) -> bool {
    let set = docs.set();
    let set = set.borrow();
    let Some(tab) = docs.tabs.iter_mut().find(|t| t.id() == id) else {
        return false;
    };
    let Some(k) = set.get(tab.view.kanban) else {
        return false;
    };
    let Some(h) = hit(tab, k, ts, x, y) else {
        return false;
    };
    drop(set);
    // Um menu aberto: carregar fora dele só o fecha (o idioma de um popover).
    if tab.menu.is_some() && !matches!(h, Hit::Menu(_) | Hit::Field) {
        tab.menu = None;
        return true;
    }
    let keeps_field = matches!(h, Hit::Field | Hit::FieldAdd | Hit::FieldClose);
    if !keeps_field && tab.field.is_some() {
        commit_field(docs, id);
    }
    let Some(tab) = docs.tab_mut(id) else {
        return true;
    };
    if button == PointerButton::Secondary {
        // O botão direito num cartão é a edição rápida (o Trello).
        if let Hit::Card(c) | Hit::CardEdit(c) | Hit::CardLabel(c) = h {
            let title = docs_title(docs, id, c);
            if let Some(tab) = docs.tab_mut(id) {
                tab.focus = Some(c);
                open_field(tab, Target::CardTitle(c), &title);
            }
        }
        return true;
    }
    tab.press = Some(Press { at: (x, y), hit: h });
    true
}

fn docs_title(docs: &KanbanDocs, id: u64, c: ph2d_kanban_model::CardId) -> String {
    let Some(kid) = docs.kanban_of(id) else {
        return String::new();
    };
    docs.set
        .borrow()
        .get(kid)
        .and_then(|k| k.cards.get(&c))
        .map(|c| c.title.clone())
        .unwrap_or_default()
}

/// Movimento (com ou sem botão). `true` = tomado.
pub(crate) fn moved(docs: &mut KanbanDocs, id: u64, ts: &mut TextSystem, x: f32, y: f32) -> bool {
    let set = docs.set();
    let set = set.borrow();
    let Some(tab) = docs.tabs.iter_mut().find(|t| t.id() == id) else {
        return false;
    };
    if let Some(d) = tab.drag.as_mut() {
        d.at = (x, y);
        tab.smoke_hold = None;
        if let Lift::Pan { scroll_x } = d.lift
            && let Some(p) = tab.press
        {
            tab.view.scroll_x = (scroll_x - f64::from(x - p.at.0)).max(0.0);
        }
        return true;
    }
    if let Some(p) = tab.press {
        if (x - p.at.0).hypot(y - p.at.1) > PRESS_SLOP_PX {
            start_drag(tab, p, x, y);
        }
        return true;
    }
    let Some(k) = set.get(tab.view.kanban) else {
        return false;
    };
    tab.hover = hit(tab, k, ts, x, y);
    tab.hover.is_some()
}

fn start_drag(tab: &mut Tab, p: Press, x: f32, y: f32) {
    let lift = match p.hit {
        Hit::Card(c) | Hit::CardEdit(c) | Hit::CardLabel(c) => Lift::Card(c),
        Hit::ListTitle(l) | Hit::ListHead(l) => Lift::List(l),
        Hit::Board => Lift::Pan {
            scroll_x: tab.view.scroll_x,
        },
        _ => return,
    };
    let rect = match lift {
        Lift::Card(c) => tab.layout.card(c).map(|(_, cb)| cb.rect),
        Lift::List(l) => tab.layout.list(l).map(|lb| lb.rect),
        Lift::Pan { .. } => Some(tab.layout.board),
    };
    let Some(r) = rect else {
        return;
    };
    if let Lift::Card(c) = lift {
        tab.focus = Some(c);
    }
    // O espaço abre de início onde o que voa estava (nada se mexe no 1.º quadro).
    let to_card = match lift {
        Lift::Card(c) => tab.layout.card(c).and_then(|(lb, _)| {
            let i = lb.cards.iter().position(|cb| cb.id == c)?;
            Some((lb.id, i))
        }),
        _ => None,
    };
    let to_list = match lift {
        Lift::List(l) => tab.layout.lists.iter().position(|lb| lb.id == l),
        _ => None,
    };
    tab.field = None;
    tab.hover = None;
    tab.drag = Some(Drag {
        lift,
        at: (x, y),
        grab: (p.at.0 - r.x, p.at.1 - r.y),
        size: (r.w, r.h),
        to_card,
        to_list: to_list.unwrap_or(0),
    });
}

/// Largar. `true` = tomado.
pub(crate) fn up(
    docs: &mut KanbanDocs,
    id: u64,
    ts: &mut TextSystem,
    now_ns: u128,
    x: f32,
    y: f32,
) -> bool {
    let Some(tab) = docs.tab_mut(id) else {
        return false;
    };
    let press = tab.press.take();
    if let Some(d) = tab.drag.take() {
        land(docs, id, d);
        return true;
    }
    let Some(p) = press else {
        return false;
    };
    let set = docs.set();
    let now = {
        let set = set.borrow();
        let Some(tab) = docs.tabs.iter().find(|t| t.id() == id) else {
            return true;
        };
        set.get(tab.view.kanban).and_then(|k| hit(tab, k, ts, x, y))
    };
    if now == Some(p.hit) {
        click(docs, id, p.hit, now_ns);
    }
    true
}

/// O que voa assenta: escreve a ordem nova e a mola parte de onde a mão o largou.
fn land(docs: &mut KanbanDocs, id: u64, d: Drag) {
    let ghost = (d.at.0 - d.grab.0, d.at.1 - d.grab.1);
    docs.with_doc(id, |tab, doc| match d.lift {
        Lift::Card(c) => {
            if let Some((list, index)) = d.to_card {
                doc.move_card(c, list, index);
            }
            tab.landed = Some((c, ghost.0, ghost.1));
        }
        Lift::List(l) => {
            doc.move_list(l, d.to_list);
            let o = crate::tab::origin(tab, tab.layout.board);
            crate::tab::pin_list(&mut tab.springs, l, ghost, o);
        }
        Lift::Pan { .. } => {}
    });
}

/// ⭐ Um clique.
#[allow(clippy::too_many_lines)]
fn click(docs: &mut KanbanDocs, id: u64, h: Hit, now_ns: u128) {
    let _ = now_ns;
    let names = docs.kanban_of(id).and_then(|kid| {
        let set = docs.set.borrow();
        set.get(kid).map(|k| k.header.name.clone())
    });
    match h {
        Hit::KanbanTitle => {
            if let (Some(t), Some(n)) = (docs.tab_mut(id), names) {
                open_field(t, Target::KanbanName, &n);
            }
        }
        Hit::Star => {
            docs.with_doc(id, |_, doc| {
                if let Some(mut hd) = doc.kanban().map(|k| k.header.clone()) {
                    hd.starred = !hd.starred;
                    doc.apply(vec![ph2d_kanban_model::KanbanOp::Header(hd)]);
                }
            });
        }
        Hit::BoardMenu => toggle_menu(docs, id, Menu::Board),
        Hit::ListMenu(l) => toggle_menu(docs, id, Menu::List(l)),
        Hit::ListTitle(l) => {
            let title = list_title(docs, id, l);
            if let Some(t) = docs.tab_mut(id) {
                open_field(t, Target::ListTitle(l), &title);
            }
        }
        Hit::Collapse(l) | Hit::ListHead(l) => {
            if let Some(t) = docs.tab_mut(id) {
                // A cabeça de uma lista ABERTA só agarra; recolhida, o clique abre-a.
                let collapsed = t.view.collapsed.contains(&l);
                if matches!(h, Hit::Collapse(_)) || collapsed {
                    t.view.toggle_collapsed(l);
                }
            }
        }
        Hit::AddCard(l) => {
            if let Some(t) = docs.tab_mut(id) {
                open_field(
                    t,
                    Target::NewCard {
                        list: l,
                        after: None,
                    },
                    "",
                );
            }
        }
        Hit::Card(c) => {
            if let Some(t) = docs.tab_mut(id) {
                t.focus = Some(c);
            }
        }
        Hit::CardEdit(c) => {
            let title = docs_title(docs, id, c);
            if let Some(t) = docs.tab_mut(id) {
                t.focus = Some(c);
                open_field(t, Target::CardTitle(c), &title);
            }
        }
        Hit::CardLabel(_) => {
            if let Some(t) = docs.tab_mut(id) {
                t.label_text = !t.label_text;
            }
        }
        Hit::FieldAdd => enter(docs, id),
        Hit::FieldClose => {
            if let Some(t) = docs.tab_mut(id) {
                t.field = None;
            }
        }
        Hit::AddList => {
            if let Some(t) = docs.tab_mut(id) {
                open_field(t, Target::NewList, "");
            }
        }
        Hit::Field | Hit::Board => {}
        Hit::Menu(row) => menu_row(docs, id, row),
        Hit::Panel(row) => panel_row(docs, id, row),
    }
}

fn list_title(docs: &KanbanDocs, id: u64, l: ph2d_kanban_model::ListId) -> String {
    let Some(kid) = docs.kanban_of(id) else {
        return String::new();
    };
    docs.set
        .borrow()
        .get(kid)
        .and_then(|k| k.lists.get(&l))
        .map(|l| l.title.clone())
        .unwrap_or_default()
}

fn toggle_menu(docs: &mut KanbanDocs, id: u64, mn: Menu) {
    if let Some(t) = docs.tab_mut(id) {
        t.menu = if t.menu == Some(mn) { None } else { Some(mn) };
    }
}

fn menu_row(docs: &mut KanbanDocs, id: u64, row: MenuRow) {
    let Some(mn) = docs.tab_mut(id).and_then(|t| t.menu.take()) else {
        return;
    };
    match (mn, row) {
        (Menu::List(l), MenuRow::AddCard) => {
            if let Some(t) = docs.tab_mut(id) {
                open_field(
                    t,
                    Target::NewCard {
                        list: l,
                        after: None,
                    },
                    "",
                );
            }
        }
        (Menu::List(l), MenuRow::Collapse) => {
            if let Some(t) = docs.tab_mut(id) {
                t.view.collapsed.insert(l);
            }
        }
        (Menu::List(l), MenuRow::ArchiveList) => {
            docs.with_doc(id, |_, doc| doc.archive_list(l, true));
        }
        (Menu::List(l), MenuRow::ArchiveCards) => {
            docs.with_doc(id, |_, doc| doc.archive_all_cards(l));
        }
        (Menu::List(l), MenuRow::Color(c)) => {
            docs.with_doc(id, |_, doc| doc.set_list_color(l, c));
        }
        (Menu::Board, MenuRow::Archived) => {
            if let Some(t) = docs.tab_mut(id) {
                t.panel = Some(crate::archive::Panel::default());
            }
        }
        (Menu::Board, MenuRow::CollapseAll | MenuRow::ExpandAll) => {
            let all: Vec<_> = docs
                .kanban_of(id)
                .and_then(|kid| {
                    docs.set
                        .borrow()
                        .get(kid)
                        .map(|k| k.lists_in_order().iter().map(|l| l.id).collect())
                })
                .unwrap_or_default();
            if let Some(t) = docs.tab_mut(id) {
                if row == MenuRow::CollapseAll {
                    t.view.collapsed.extend(all);
                } else {
                    t.view.collapsed.clear();
                }
            }
        }
        _ => {}
    }
}

fn panel_row(docs: &mut KanbanDocs, id: u64, row: crate::archive::Row) {
    use crate::archive::Row as P;
    match row {
        P::Close => {
            if let Some(t) = docs.tab_mut(id) {
                t.panel = None;
            }
        }
        P::ShowCards | P::ShowLists => {
            if let Some(p) = docs.tab_mut(id).and_then(|t| t.panel.as_mut()) {
                *p = crate::archive::Panel {
                    lists: row == P::ShowLists,
                    scroll: 0.0,
                };
            }
        }
        P::Restore(r) => {
            docs.with_doc(id, |_, doc| match r {
                RecordId::Card(c) => doc.archive_card(c, false),
                RecordId::List(l) => doc.archive_list(l, false),
                _ => false,
            });
        }
        P::Delete(r) => {
            docs.with_doc(id, |_, doc| match r {
                RecordId::Card(c) => doc.delete_card(c),
                RecordId::List(l) => doc.delete_list(l),
                _ => false,
            });
        }
        P::Inside => {}
    }
}

/// A roda: sobre o corpo de uma lista rola-a; no resto, o quadro na horizontal. `true` = tomada.
pub(crate) fn wheel(docs: &mut KanbanDocs, id: u64, x: f32, y: f32, dy: f32, shift: bool) -> bool {
    let Some(tab) = docs.tab_mut(id) else {
        return false;
    };
    if !tab.layout.area.contains(x, y) {
        return false;
    }
    let step = f64::from(dy * m::WHEEL_STEP);
    if let Some(p) = tab.panel.as_mut()
        && x > tab.layout.board.x + tab.layout.board.w - m::PANEL_W
    {
        p.scroll = (p.scroll + dy * m::WHEEL_STEP).max(0.0);
        return true;
    }
    let over = tab
        .layout
        .lists
        .iter()
        .find(|l| !l.collapsed && l.body.contains(x, y))
        .map(|l| (l.id, crate::layout::max_scroll_y(l)));
    match over {
        Some((list, max)) if !shift => {
            let v = tab.view.scroll_y.entry(list).or_default();
            *v = (*v + step).clamp(0.0, f64::from(max));
        }
        _ => {
            let max = f64::from(crate::layout::max_scroll_x(&tab.layout));
            tab.view.scroll_x = (tab.view.scroll_x + step).clamp(0.0, max);
        }
    }
    true
}
