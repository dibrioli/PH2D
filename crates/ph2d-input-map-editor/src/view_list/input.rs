//! Os gestos da vista A: clique (campos, Add, chips, «+», lixo, números), arrasto dos números, teclas e roda.

use bumpalo::Bump;
use ph2d_editor_core::doc_surface::{Key as DocKey, KeyInput};
use ph2d_editor_core::interaction::{InteractiveState, dispatch_key, dispatch_text_input};
use ph2d_host::{KeyEvent, KeyKind};
use ph2d_input::Key;

use super::state::{Scrub, Typing, Zone};
use crate::doc::InputMapDoc;
use crate::edit::InputMapEdit;
use crate::ids::{self, ListHit};

fn hit_of(doc: &InputMapDoc, id: ph2d_a11y::NodeId) -> Option<ListHit> {
    doc.snap.map.actions().iter().find_map(|a| {
        let mut all = vec![
            ListHit::Listen(a.id),
            ListHit::Trash(a.id),
            ListHit::Dead(a.id),
            ListHit::Press(a.id),
            ListHit::DeadHandle(a.id),
            ListHit::PressHandle(a.id),
            ListHit::Row(a.id),
        ];
        all.extend((0..a.bindings.len()).map(|n| ListHit::RemoveBinding(a.id, n)));
        all.extend((0..a.bindings.len()).map(|n| ListHit::Chip(a.id, n)));
        all.into_iter().find(|h| h.id() == id)
    })
}

/// A dica (Plain English) de um alvo da lista, se ele tem uma.
#[must_use]
pub fn tip_of(doc: &InputMapDoc, id: ph2d_a11y::NodeId) -> Option<&'static str> {
    let key = match id {
        i if i == ids::NEW_NAME => "input_map_list.new_name_tip",
        i if i == ids::ADD => "input_map_list.add_tip",
        i if i == ids::SEARCH => "input_map_list.search_tip",
        _ => match hit_of(doc, id)? {
            ListHit::Listen(_) => "input_map_list.add_key_tip",
            ListHit::RemoveBinding(..) => "input_map_list.remove_key_tip",
            ListHit::Trash(_) => "input_map_list.remove_action_tip",
            ListHit::Dead(_) | ListHit::DeadHandle(_) => "input_map_list.dead_tip",
            ListHit::Press(_) | ListHit::PressHandle(_) => "input_map_list.press_tip",
            ListHit::Row(_) | ListHit::Chip(..) => return None,
        },
    };
    Some(ph2d_i18n::tr(key))
}

fn focus(doc: &mut InputMapDoc, id: Option<ph2d_a11y::NodeId>) {
    doc.store.set_focus(id);
}

/// O Add: nome vazio ou repetido é recusado com o porquê à vista.
fn commit_add(doc: &mut InputMapDoc) {
    let name = doc
        .store
        .text(ids::NEW_NAME)
        .unwrap_or("")
        .trim()
        .to_owned();
    doc.list.refused = if name.is_empty() {
        Some("input_map_list.empty_name")
    } else if doc.snap.map.id(&name).is_some() {
        Some("input_map_list.duplicate")
    } else {
        None
    };
    if doc.list.refused.is_none() {
        doc.pending.push(InputMapEdit::Add(name));
        if let Some(InteractiveState::TextInput { text, caret, .. }) =
            doc.store.get_mut(ids::NEW_NAME)
        {
            text.clear();
            *caret = 0;
        }
    }
}

pub(crate) fn press(doc: &mut InputMapDoc, x: f32, y: f32) -> bool {
    commit_typing(doc);
    if doc.list.bar.is_some_and(|b| b.thumb.contains(x, y)) {
        doc.list.thumb_drag = Some((y, doc.list.scroll));
        return true;
    }
    let hit = doc.hits.hit(x, y);
    let listen = hit.and_then(|id| hit_of(doc, id));
    if doc.snap.listening.is_some() && !matches!(listen, Some(ListHit::Listen(_))) {
        doc.list.listen_request = Some(None);
    }
    match hit {
        Some(id) if id == ids::ADD => commit_add(doc),
        Some(id) if id == ids::NEW_NAME || id == ids::SEARCH => focus(doc, Some(id)),
        Some(id) if id == ids::SEARCH_CLEAR => {
            doc.list.search_key = None;
            focus(doc, Some(ids::SEARCH));
        }
        _ => {
            focus(doc, None);
            match listen {
                Some(ListHit::RemoveBinding(a, n)) => {
                    doc.pending.push(InputMapEdit::RemoveBinding(a, n))
                }
                Some(ListHit::Trash(a)) => doc.pending.push(InputMapEdit::Remove(a)),
                Some(ListHit::Listen(a)) => {
                    let armed = doc.snap.listening == Some(a);
                    doc.list.listen_request = Some((!armed).then_some(a));
                }
                Some(h @ (ListHit::DeadHandle(a) | ListHit::PressHandle(a))) => {
                    let zone = if matches!(h, ListHit::DeadHandle(_)) {
                        Zone::Dead
                    } else {
                        Zone::Press
                    };
                    let act = doc.snap.map.get(a);
                    let start = act.map_or(0.0, |v| {
                        if zone == Zone::Dead {
                            v.dead_zone
                        } else {
                            v.press_point
                        }
                    });
                    let field = if zone == Zone::Dead {
                        ListHit::Dead(a)
                    } else {
                        ListHit::Press(a)
                    };
                    let width = doc.hits.rect_for(field.id()).map_or(1.0, |r| r.w.max(1.0));
                    doc.list.scrub = Some(Scrub {
                        action: a,
                        zone,
                        start_x: x,
                        start,
                        width,
                        moved: false,
                    });
                }
                _ => {}
            }
        }
    }
    true
}

/// O ponteiro anda: com um arrasto de número em curso, o valor segue-o (a lei do `set_zone` na família).
pub(crate) fn drag(doc: &mut InputMapDoc, x: f32) -> bool {
    let Some(s) = doc.list.scrub.as_mut() else {
        return false;
    };
    s.moved |= (x - s.start_x).abs() > f32::EPSILON;
    let s = *s;
    set_zone(doc, s.action, s.zone, s.start + (x - s.start_x) / s.width)
}

/// Um dos dois números passa a `v`, na lei do `set_zone` (Press at nunca abaixo da zona morta).
fn set_zone(doc: &mut InputMapDoc, action: ph2d_input::ActionId, zone: Zone, v: f32) -> bool {
    let Some(a) = doc.snap.map.get(action) else {
        return false;
    };
    let v = v.clamp(0.0, 1.0);
    let (dead, press) = match zone {
        Zone::Dead => (v, a.press_point.max(v)),
        Zone::Press => (a.dead_zone.min(v), v),
    };
    doc.pending.push(InputMapEdit::SetZone(action, dead, press));
    true
}

/// Soltar: acaba o arrasto; um clique sem arrasto abre o número para escrever.
/// O polegar agarrado segue o ponteiro (a conversão da barra do cartão, `scrollbar_delta_for_drag`).
pub(crate) fn drag_thumb(doc: &mut InputMapDoc, y: f32) -> bool {
    let (Some((y0, s0)), Some(b)) = (doc.list.thumb_drag, doc.list.bar) else {
        return false;
    };
    let d = ph2d_editor_core::widget::scrollbar_delta_for_drag(
        y - y0,
        b.track_h,
        b.content_h,
        b.visible_h,
    );
    let s = (s0 + d).clamp(0.0, doc.list.scroll_max);
    let changed = (s - doc.list.scroll).abs() > f32::EPSILON;
    doc.list.scroll = s;
    changed
}

pub(crate) fn release(doc: &mut InputMapDoc) -> bool {
    if doc.list.thumb_drag.take().is_some() {
        return true;
    }
    let Some(s) = doc.list.scrub.take() else {
        return false;
    };
    if !s.moved {
        open_zone(doc, s.action, s.zone);
    }
    true
}

/// Abre o número `zone` de `action` (`field.rs`): o texto do valor, todo seleccionado.
fn open_zone(doc: &mut InputMapDoc, action: ph2d_input::ActionId, zone: Zone) {
    let Some(a) = doc.snap.map.get(action) else {
        return;
    };
    let v = if zone == Zone::Dead {
        a.dead_zone
    } else {
        a.press_point
    };
    let (buffer, caret) = crate::field::Caret::open(v);
    doc.list.caret = caret;
    doc.list.typing = Some(Typing {
        action,
        zone,
        buffer,
    });
}

/// O foco (clique ou Tab) chega ao número `id`.
pub(crate) fn open_number(doc: &mut InputMapDoc, id: ph2d_a11y::NodeId) {
    commit_typing(doc);
    match hit_of(doc, id) {
        Some(ListHit::Dead(a)) => open_zone(doc, a, Zone::Dead),
        Some(ListHit::Press(a)) => open_zone(doc, a, Zone::Press),
        _ => {}
    }
}

/// Aplica `v` (já preso) ao número aberto e guarda o aviso do limite.
fn apply(
    doc: &mut InputMapDoc,
    action: ph2d_input::ActionId,
    zone: Zone,
    v: f32,
    clamp: Option<crate::field::Clamp>,
) {
    doc.list.warn = clamp.map(|c| (action, zone, c));
    set_zone(doc, action, zone, v);
}

/// Aplica o número escrito (o que não for número devolve o valor do foco).
pub(crate) fn commit_typing(doc: &mut InputMapDoc) {
    if let Some(t) = doc.list.typing.take() {
        let (v, clamp) = crate::field::value_of(&t.buffer, &doc.list.caret);
        apply(doc, t.action, t.zone, v, clamp);
    }
}

/// A tecla para o número aberto (`field::key`): Enter aplica, Esc devolve, Tab aplica e segue.
fn key_typing(doc: &mut InputMapDoc, input: &KeyInput<'_>) -> bool {
    use crate::field::Outcome;
    let Some(t) = doc.list.typing.as_mut() else {
        return false;
    };
    let (action, zone) = (t.action, t.zone);
    match crate::field::key(&mut t.buffer, &mut doc.list.caret, input) {
        Outcome::Ignored | Outcome::Edited => {}
        Outcome::Stepped(clamp) => doc.list.warn = clamp.map(|c| (action, zone, c)),
        Outcome::Commit(v, clamp) => {
            doc.list.typing = None;
            apply(doc, action, zone, v, clamp);
        }
        Outcome::Revert(_) => doc.list.typing = None,
        Outcome::Move { v, clamp, back } => {
            doc.list.typing = None;
            apply(doc, action, zone, v, clamp);
            let id = if zone == Zone::Dead {
                ListHit::Dead(action).id()
            } else {
                ListHit::Press(action).id()
            };
            doc.ui.focus = Some(id);
            let k = if back {
                crate::interact::FocusKey::Prev
            } else {
                crate::interact::FocusKey::Next
            };
            doc.focus_key(k);
        }
    }
    true
}

/// Um controlo da lista disparou (o Up dentro, ou Enter/Space no foco): age pelo id, nunca pela posição.
pub(crate) fn act(doc: &mut InputMapDoc, id: ph2d_a11y::NodeId) {
    let hit = hit_of(doc, id);
    if matches!(hit, Some(ListHit::Dead(_) | ListHit::Press(_))) {
        doc.store.set_focus(None);
        open_number(doc, id);
        return;
    }
    commit_typing(doc);
    if doc.snap.listening.is_some() && !matches!(hit, Some(ListHit::Listen(_))) {
        doc.list.listen_request = Some(None);
    }
    match hit {
        _ if id == ids::ADD => commit_add(doc),
        _ if id == ids::NEW_NAME || id == ids::SEARCH => focus(doc, Some(id)),
        _ if id == ids::SEARCH_CLEAR => {
            doc.list.search_key = None;
            clear_text(doc, ids::SEARCH);
            focus(doc, Some(ids::SEARCH));
        }
        Some(ListHit::RemoveBinding(a, n)) => {
            focus(doc, None);
            doc.pending.push(InputMapEdit::RemoveBinding(a, n));
        }
        Some(ListHit::Trash(a)) => {
            focus(doc, None);
            doc.pending.push(InputMapEdit::Remove(a));
        }
        Some(ListHit::Listen(a)) => {
            focus(doc, None);
            let armed = doc.snap.listening == Some(a);
            doc.list.listen_request = Some((!armed).then_some(a));
        }
        _ => focus(doc, None),
    }
}

fn clear_text(doc: &mut InputMapDoc, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::TextInput {
        text,
        caret,
        selection_anchor,
        ..
    }) = doc.store.get_mut(id)
    {
        text.clear();
        *caret = 0;
        *selection_anchor = None;
    }
}

pub(crate) fn wheel(doc: &mut InputMapDoc, dy: f32) -> bool {
    let before = doc.list.scroll;
    doc.list.scroll = (doc.list.scroll + dy).clamp(0.0, doc.list.scroll_max);
    (doc.list.scroll - before).abs() > f32::EPSILON
}

/// A tecla física com a busca focada e VAZIA: a que não escreve (setas, F, Shift…) e o Space (nenhum nome de
/// acção tem espaço) vira o filtro por tecla. Com texto, o campo é só texto: o Shift de uma maiúscula, o Ctrl
/// e as setas seguem para ele.
pub(crate) fn raw_key(doc: &mut InputMapDoc, key: Key, pressed: bool, writes_text: bool) -> bool {
    if !pressed
        || doc.store.focus_id() != Some(ids::SEARCH)
        || !doc.store.text(ids::SEARCH).unwrap_or("").is_empty()
    {
        return false;
    }
    let space = Key::named_in_script("space").is_some_and(|k| k.contains(&key));
    if writes_text && !space {
        return false;
    }
    let editing = ["backspace", "delete", "enter", "escape", "tab"];
    if editing
        .iter()
        .filter_map(|n| Key::named_in_script(n))
        .flatten()
        .any(|k| k == key)
    {
        return false;
    }
    doc.list.search_key = Some(key);
    if let Some(InteractiveState::TextInput { text, caret, .. }) = doc.store.get_mut(ids::SEARCH) {
        text.clear();
        *caret = 0;
    }
    true
}

fn code(name: &str) -> Option<u32> {
    Key::named_in_script(name)?.first().map(|k| k.0)
}

/// A tecla para o campo focado (nome novo ou busca), pelas portas do `WidgetStore` do cartão.
pub(crate) fn key(doc: &mut InputMapDoc, input: &KeyInput<'_>) -> bool {
    if key_typing(doc, input) {
        return true;
    }
    let Some(f) = doc
        .store
        .focus_id()
        .filter(|f| *f == ids::NEW_NAME || *f == ids::SEARCH)
    else {
        return false;
    };
    if !input.pressed {
        return true;
    }
    let arena = Bump::new();
    let name = match input.key {
        DocKey::Enter if f == ids::NEW_NAME => {
            commit_add(doc);
            return true;
        }
        DocKey::Escape => {
            focus(doc, None);
            return true;
        }
        DocKey::Backspace if f == ids::SEARCH && doc.store.text(f).unwrap_or("").is_empty() => {
            doc.list.search_key = None;
            return true;
        }
        DocKey::Char('a' | 'A') if input.mods.ctrl => {
            if let Some(InteractiveState::TextInput {
                text,
                caret,
                selection_anchor,
                ..
            }) = doc.store.get_mut(f)
            {
                *selection_anchor = Some(0);
                *caret = text.len();
            }
            return true;
        }
        DocKey::Backspace | DocKey::Delete if take_selection(doc, f) => {
            edited(doc, f);
            return true;
        }
        DocKey::Backspace => "backspace",
        DocKey::Delete => "delete",
        DocKey::Left => "left",
        DocKey::Right => "right",
        DocKey::Home => "home",
        DocKey::End => "end",
        _ => {
            let typed = input.text.unwrap_or("");
            if !typed.is_empty() {
                take_selection(doc, f);
            }
            for ch in typed.chars() {
                let _ = dispatch_text_input(&mut doc.store, ch, &arena);
            }
            edited(doc, f);
            return true;
        }
    };
    if let Some(keycode) = code(name) {
        let ev = KeyEvent {
            keycode,
            modifiers: input.mods,
            kind: KeyKind::Down,
            timestamp_ns: 0,
        };
        let _ = dispatch_key(&mut doc.store, ev, &arena);
    }
    true
}

/// Apaga a selecção do campo `f`. `true` = havia uma.
fn take_selection(doc: &mut InputMapDoc, f: ph2d_a11y::NodeId) -> bool {
    let Some(InteractiveState::TextInput {
        text,
        caret,
        selection_anchor,
        ..
    }) = doc.store.get_mut(f)
    else {
        return false;
    };
    let Some(a) = selection_anchor.take().filter(|a| *a != *caret) else {
        return false;
    };
    let (s, e) = (a.min(*caret).min(text.len()), a.max(*caret).min(text.len()));
    text.replace_range(s..e, "");
    *caret = s;
    true
}

fn edited(doc: &mut InputMapDoc, f: ph2d_a11y::NodeId) {
    if f == ids::NEW_NAME {
        doc.list.refused = None;
    } else {
        doc.list.search_key = None;
    }
}
