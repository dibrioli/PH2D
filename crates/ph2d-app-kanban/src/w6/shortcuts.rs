//! ⭐ **Os atalhos do Trello que a W6 traz** (plano §1.7): `E` edição rápida · `F` filtro · `Q` os
//! meus cartões · `X` limpar · `W` o menu do Kanban · `/` pesquisa · `?` ajuda · `D` `L` `M` datas,
//! etiquetas, membros · `Space` atribuir-me · `S` seguir · `V` votar · `1`–`9` etiqueta n ·
//! `Ctrl+C`/`Ctrl+V` copiar/colar o cartão · `Z` desfazer. Os de andar, criar, arquivar e mudar de
//! lista moram em [`crate::keys`].

use super::Over;
use crate::back::Side;
use crate::docs::KanbanDocs;
use ph2d_editor_core::doc_surface::{Key, KeyInput, KeyOutcome};
use ph2d_kanban_model::{CardId, CardRef, CopyKeep, Entry};

/// A linha com o foco e o cartão que ela mostra.
fn focused(docs: &mut KanbanDocs, id: u64) -> Option<(CardId, CardRef)> {
    let row = docs.tab_mut(id)?.focus?;
    Some((row, super::gesture::shows(docs, id, row)?))
}

/// Escreve no cartão do foco quando ele é deste Kanban (o original de um espelho alheio escreve-se
/// no verso dele).
fn on_card(
    docs: &mut KanbanDocs,
    id: u64,
    f: impl FnOnce(&mut KanbanDocs, CardId) -> bool,
) -> bool {
    let Some((_, r)) = focused(docs, id) else {
        return false;
    };
    if docs.kanban_of(id) != Some(r.kanban) {
        return false;
    }
    f(docs, r.card)
}

/// Abre o verso do cartão do foco com o popover `s` (o `D`, o `L`, o `M`).
fn side(docs: &mut KanbanDocs, id: u64, s: Side) -> bool {
    let Some((row, r)) = focused(docs, id) else {
        return false;
    };
    super::gesture::open_back(docs, id, row);
    let at = if docs.kanban_of(id) == Some(r.kanban) {
        id
    } else {
        r.kanban.0
    };
    crate::back::press_side(docs, at, s);
    true
}

/// `Ctrl+C`: o cartão do foco fica na prancheta da família; o link dele vai para a do sistema.
fn copy(docs: &mut KanbanDocs, id: u64) -> KeyOutcome {
    let Some((_, r)) = focused(docs, id) else {
        return KeyOutcome::default();
    };
    docs.clip = Some(r);
    let link = {
        let set = docs.set.borrow();
        set.get(r.kanban)
            .zip(set.card(r))
            .map(|(k, c)| crate::back::card_link(k, c))
    };
    KeyOutcome {
        consumed: true,
        copy: link,
    }
}

/// `Ctrl+V`: uma cópia do cartão da prancheta logo abaixo do foco (ou no fim da 1.ª lista).
fn paste(docs: &mut KanbanDocs, id: u64) -> bool {
    let (Some(src), Some(kid)) = (docs.clip, docs.kanban_of(id)) else {
        return false;
    };
    let Some(t) = docs.tabs.iter().find(|t| t.id() == id) else {
        return false;
    };
    let at = t.focus.and_then(|row| {
        let (list, _) = t.rows.find(row)?;
        let e = match t.rows.mirrors.get(&row) {
            Some((m, of)) => Entry::Mirror { id: *m, of: *of },
            None => Entry::Card(row),
        };
        Some((list, e))
    });
    let first = t.layout.lists.iter().find(|l| !l.collapsed).map(|l| l.id);
    let plan = {
        let set = docs.set.borrow();
        let place = match at {
            Some((list, e)) => {
                let i = set.entries_in(kid, list).iter().position(|x| *x == e);
                set.place_at(kid, list, i.map_or(usize::MAX, |i| i + 1), None)
            }
            None => first.and_then(|l| set.place_at(kid, l, usize::MAX, None)),
        };
        let title = set.card(src).map(|c| c.title.clone());
        place.zip(title).and_then(|(p, title)| {
            ph2d_kanban_model::copy_card(
                &set,
                src,
                &p,
                CopyKeep::default(),
                &title,
                crate::actions::now(),
            )
        })
    };
    super::gesture::apply(docs, plan)
}

fn toggle_menu(docs: &mut KanbanDocs, id: u64) -> bool {
    let Some(t) = docs.tab_mut(id) else {
        return false;
    };
    t.w6.close_top();
    t.menu = match t.menu {
        Some(crate::menu::Menu::Board) => None,
        _ => Some(crate::menu::Menu::Board),
    };
    true
}

/// ⭐ Um atalho da W6 no quadro (sem campo aberto). `None` = não é daqui.
pub(crate) fn key(docs: &mut KanbanDocs, id: u64, input: &KeyInput<'_>) -> Option<KeyOutcome> {
    if !input.pressed || input.mods.alt {
        return None;
    }
    let ch = match input.key {
        Key::Char(c) => Some(c),
        _ => input.text.and_then(|t| t.chars().next()),
    };
    let chord = input.mods.ctrl || input.mods.meta;
    if chord {
        return match ch.map(|c| c.to_ascii_lowercase()) {
            Some('c') => Some(copy(docs, id)),
            Some('v') => Some(done(paste(docs, id))),
            _ => None,
        };
    }
    let did = match (input.key, ch) {
        (Key::Space, _) | (_, Some(' ')) => {
            let me = docs.me.clone();
            on_card(docs, id, |d, c| {
                crate::back::write::toggle_member(d, id, c, &me)
            })
        }
        (_, Some('e' | 'E')) => {
            let row = docs.tab_mut(id)?.focus?;
            super::quick_edit::open(docs, id, row);
            true
        }
        (_, Some('f' | 'F')) => {
            super::toggle_over(docs, id, Over::Filter);
            true
        }
        (_, Some('/')) => {
            super::toggle_over(docs, id, Over::Search);
            true
        }
        (_, Some('?')) => {
            super::toggle_over(docs, id, Over::Help);
            true
        }
        (_, Some('q' | 'Q')) => {
            let t = docs.tab_mut(id)?;
            t.w6.filter.mine = !t.w6.filter.mine;
            true
        }
        (_, Some('x' | 'X')) => {
            let t = docs.tab_mut(id)?;
            super::filter_bar::clear(&mut t.w6);
            true
        }
        (_, Some('w' | 'W')) => toggle_menu(docs, id),
        (_, Some('d' | 'D')) => side(docs, id, Side::Dates),
        (_, Some('l' | 'L')) => side(docs, id, Side::Labels),
        (_, Some('m' | 'M')) => side(docs, id, Side::Members),
        (_, Some('s' | 'S')) => {
            on_card(docs, id, |d, c| crate::back::write::toggle_watch(d, id, c))
        }
        (_, Some('v' | 'V')) => on_card(docs, id, |d, c| crate::back::write::toggle_vote(d, id, c)),
        (_, Some(n @ '1'..='9')) => {
            let i = usize::from(n as u8 - b'1');
            let label = docs.kanban_of(id).and_then(|kid| {
                let set = docs.set.borrow();
                set.get(kid)?
                    .labels
                    .values()
                    .filter(|l| !l.meta.deleted)
                    .nth(i)
                    .map(|l| l.id)
            });
            let Some(l) = label else {
                return Some(KeyOutcome::default());
            };
            on_card(docs, id, |d, c| {
                crate::back::write::toggle_label(d, id, c, l)
            })
        }
        (_, Some('z' | 'Z')) => super::gesture::undo(docs, id, false),
        _ => return None,
    };
    Some(done(did))
}

fn done(did: bool) -> KeyOutcome {
    KeyOutcome {
        consumed: did,
        copy: None,
    }
}
