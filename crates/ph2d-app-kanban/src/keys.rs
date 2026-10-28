//! ⭐ **O teclado na aba** — com um campo aberto, o campo; senão os atalhos do Trello (plano §1.7):
//! setas e `j`/`k`/`h`/`l` andam pelos cartões, `Enter`/`E`/`T` editam o título, `N` cria um cartão
//! por baixo, `C` arquiva, `,` `.` levam o cartão à lista vizinha (`<` `>` ao fundo dela), `\`
//! recolhe a lista, `Esc` larga o foco; `Enter` abre o verso do cartão (que toma o teclado).

use crate::docs::KanbanDocs;
use crate::edit::{Outcome, Target};
use crate::layout::Layout;
use crate::tab::Hit;
use ph2d_editor_core::doc_surface::{Compose, Key, KeyInput, KeyOutcome};
use ph2d_kanban_model::{CardId, ListId};

fn taken() -> KeyOutcome {
    KeyOutcome {
        consumed: true,
        copy: None,
    }
}

/// Onde está o cartão `c` na disposição: `(índice da lista, índice do cartão)`.
fn find(l: &Layout, c: CardId) -> Option<(usize, usize)> {
    l.lists
        .iter()
        .enumerate()
        .find_map(|(i, lb)| lb.cards.iter().position(|cb| cb.id == c).map(|j| (i, j)))
}

/// O cartão `(lista, índice)` mais próximo nas listas abertas, a andar `dl` listas.
fn neighbour(l: &Layout, from: (usize, usize), dl: isize, dc: isize) -> Option<CardId> {
    let open: Vec<usize> = (0..l.lists.len())
        .filter(|i| !l.lists[*i].cards.is_empty())
        .collect();
    let at = open.iter().position(|i| *i == from.0)?;
    let li = open.get(at.checked_add_signed(dl)?)?;
    let cards = &l.lists[*li].cards;
    let ci = if dl == 0 {
        from.1.checked_add_signed(dc)?
    } else {
        from.1.min(cards.len() - 1)
    };
    cards.get(ci).map(|c| c.id)
}

/// A composição (IME, tecla morta, carácter confirmado) para o verso aberto ou o campo no lugar da
/// vista `id` (a aba e o elemento do Quadro). `true` = tomada.
pub(crate) fn compose(docs: &mut KanbanDocs, id: u64, c: Compose<'_>) -> bool {
    if crate::w6::compose(docs, id, &c) {
        return true;
    }
    if crate::back::is_open(docs, id) {
        return crate::back::compose(docs, id, c);
    }
    let Some(f) = docs.tab_mut(id).and_then(|t| t.field.as_mut()) else {
        return false;
    };
    match c {
        Compose::Commit(ch) => {
            f.insert(ch.encode_utf8(&mut [0; 4]));
            true
        }
        Compose::Preedit(..) | Compose::DeadKey(_) => false,
    }
}

pub(crate) fn key(
    docs: &mut KanbanDocs,
    id: u64,
    ts: &mut ph2d_text::TextSystem,
    input: KeyInput<'_>,
) -> KeyOutcome {
    if let Some(out) = crate::w6::key(docs, id, ts, &input) {
        return out;
    }
    if crate::back::is_open(docs, id) {
        return crate::back::key(docs, id, ts, &input);
    }
    let composing = docs.tab_mut(id).is_some_and(|t| t.field.is_some());
    if composing && let Some(out) = crate::w6::complete::key(docs, id, &input) {
        return out;
    }
    if !composing && let Some(out) = crate::w6::shortcuts::key(docs, id, &input) {
        return out;
    }
    let Some(tab) = docs.tab_mut(id) else {
        return KeyOutcome::default();
    };
    if let Some(f) = tab.field.as_mut() {
        if !input.pressed {
            return taken();
        }
        return match f.key(input.key, input.mods, input.text, input.paste.as_deref()) {
            Outcome::Typed => taken(),
            Outcome::Commit => {
                crate::pointer::enter(docs, id);
                taken()
            }
            Outcome::Cancel => {
                tab.field = None;
                taken()
            }
            Outcome::Copy(s) => KeyOutcome {
                consumed: true,
                copy: Some(s),
            },
            Outcome::Ignored => KeyOutcome::default(),
        };
    }
    if !input.pressed || input.mods.ctrl || input.mods.meta || input.mods.alt {
        return KeyOutcome::default();
    }
    let ch = match input.key {
        Key::Char(c) => Some(c),
        _ => input.text.and_then(|t| t.chars().next()),
    };
    let focus = tab.focus;
    let at = focus.and_then(|c| find(&tab.layout, c));
    let first = tab
        .layout
        .lists
        .iter()
        .find_map(|l| l.cards.first().map(|c| c.id));
    let walk = |dl: isize, dc: isize| match at {
        Some(a) => neighbour(&tab.layout, a, dl, dc).or(focus),
        None => first,
    };
    let go = match (input.key, ch) {
        (Key::Down, _) | (_, Some('j')) => Some(walk(0, 1)),
        (Key::Up, _) | (_, Some('k')) => Some(walk(0, -1)),
        (Key::Right, _) | (_, Some('l')) => Some(walk(1, 0)),
        (Key::Left, _) | (_, Some('h')) => Some(walk(-1, 0)),
        _ => None,
    };
    if let Some(next) = go {
        tab.focus = next;
        reveal(tab, next);
        return taken();
    }
    match (input.key, ch) {
        (Key::Escape, _) => {
            let had = tab.menu.take().is_some()
                || tab.bg.take().is_some()
                || tab.panel.take().is_some()
                || tab.focus.take().is_some();
            return KeyOutcome {
                consumed: had,
                copy: None,
            };
        }
        (Key::Enter, _) => {
            let Some(c) = focus else {
                return KeyOutcome::default();
            };
            crate::back::open(docs, id, c);
        }
        (_, Some('t' | 'T')) => {
            let Some(c) = focus else {
                return KeyOutcome::default();
            };
            let title = card_title(docs, id, c);
            if let Some(t) = docs.tab_mut(id) {
                t.field = Some(crate::edit::Field::open(Target::CardTitle(c), &title));
            }
        }
        (_, Some('n' | 'N')) => {
            let target = match at {
                Some((li, _)) => Target::NewCard {
                    list: tab.layout.lists[li].id,
                    after: focus,
                },
                None => match tab.layout.lists.iter().find(|l| !l.collapsed) {
                    Some(l) => Target::NewCard {
                        list: l.id,
                        after: None,
                    },
                    None => Target::NewList,
                },
            };
            tab.field = Some(crate::edit::Field::open(target, ""));
        }
        (_, Some('c' | 'C')) => {
            let Some(c) = focus else {
                return KeyOutcome::default();
            };
            let next = at.and_then(|a| {
                neighbour(&tab.layout, a, 0, 1).or_else(|| neighbour(&tab.layout, a, 0, -1))
            });
            tab.focus = next;
            crate::w6::gesture::archive_row(docs, id, c);
        }
        (_, Some(k @ (',' | '.' | '<' | '>'))) => {
            let (Some(c), Some((li, ci))) = (focus, at) else {
                return KeyOutcome::default();
            };
            let open: Vec<(ListId, usize)> = tab
                .layout
                .lists
                .iter()
                .filter(|l| !l.collapsed)
                .map(|l| (l.id, l.cards.len()))
                .collect();
            let here = tab.layout.lists[li].id;
            let Some(pos) = open.iter().position(|(l, _)| *l == here) else {
                return taken();
            };
            let to = if matches!(k, ',' | '<') {
                pos.checked_sub(1)
            } else {
                Some(pos + 1)
            };
            let Some((list, len)) = to.and_then(|i| open.get(i)).copied() else {
                return taken();
            };
            let index = if matches!(k, '<' | '>') {
                len
            } else {
                ci.min(len)
            };
            crate::w6::gesture::move_row(docs, id, c, list, index);
        }
        (_, Some('\\')) => {
            let list = match (tab.hover, at) {
                (
                    Some(
                        Hit::ListHead(l)
                        | Hit::ListTitle(l)
                        | Hit::Collapse(l)
                        | Hit::ListMenu(l)
                        | Hit::AddCard(l),
                    ),
                    _,
                ) => Some(l),
                (_, Some((li, _))) => Some(tab.layout.lists[li].id),
                _ => None,
            };
            let Some(l) = list else {
                return KeyOutcome::default();
            };
            tab.view.toggle_collapsed(l);
        }
        _ => return KeyOutcome::default(),
    }
    taken()
}

/// Rola a lista do cartão `c` para que ele se veja inteiro.
fn reveal(tab: &mut crate::tab::Tab, c: Option<CardId>) {
    let Some((lb, cb)) = c.and_then(|c| tab.layout.card(c)) else {
        return;
    };
    let (id, body, r) = (lb.id, lb.body, cb.rect);
    let v = tab.view.scroll_y.entry(id).or_default();
    if r.y < body.y {
        *v -= f64::from(body.y - r.y);
    } else if r.y + r.h > body.y + body.h {
        *v += f64::from(r.y + r.h - body.y - body.h);
    }
    *v = v.max(0.0);
}

fn card_title(docs: &KanbanDocs, id: u64, c: CardId) -> String {
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
