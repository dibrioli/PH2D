//! Os gestos da vista B: abrir uma acção, + Add (o campo do nome no lugar) e Remove, a sub-vista, clicar numa tecla
//! ou num alvo do comando (acrescenta à aberta, sem repetir), × do ladrilho, a escuta, os números e as marcas da barra.

use ph2d_editor_core::doc_surface::{Key as DocKey, KeyInput};
use ph2d_input::{Binding, Key};

use super::state::KbHit;
use crate::doc::InputMapDoc;
use crate::edit::InputMapEdit;
use crate::ids::{self, ListHit};
use crate::view_list::state::{Scrub, Zone};

/// A dica (Plain English) de um alvo da vista B.
#[must_use]
pub fn tip_of(doc: &InputMapDoc, id: ph2d_a11y::NodeId) -> Option<&'static str> {
    let key = match doc.kb.hit_of(id)? {
        KbHit::Row(_) => "input_map_keyboard.row_tip",
        KbHit::Add => "input_map_keyboard.add_tip",
        KbHit::Remove => "input_map_keyboard.remove_tip",
        KbHit::Sub(super::state::Sub::Keyboard) => "input_map_keyboard.sub.keyboard_tip",
        KbHit::Sub(super::state::Sub::Gamepad) => "input_map_keyboard.sub.gamepad_tip",
        KbHit::Key(_) => "input_map_keyboard.key_tip",
        KbHit::Pad(_) => "input_map_keyboard.pad_tip",
        KbHit::TileX(_) => "input_map_keyboard.tile_x_tip",
        KbHit::Listen => "input_map_keyboard.listen_tip",
        KbHit::BarDead => "input_map_keyboard.bar_dead_tip",
        KbHit::BarPress => "input_map_keyboard.bar_press_tip",
    };
    Some(ph2d_i18n::tr(key))
}

/// Acrescenta `b` à acção aberta (a família recusa a repetida; aqui nem se pede).
fn add(doc: &mut InputMapDoc, b: Binding) {
    let Some(a) = doc.kb.open.and_then(|id| doc.snap.map.get(id)) else {
        return;
    };
    if !a.bindings.contains(&b) {
        doc.pending.push(InputMapEdit::AddBinding(a.id, b));
    }
}

fn scrub(doc: &mut InputMapDoc, zone: Zone, x: f32, width: f32, opens_text: bool) {
    let Some(a) = doc.kb.open.and_then(|id| doc.snap.map.get(id)) else {
        return;
    };
    let start = if zone == Zone::Dead {
        a.dead_zone
    } else {
        a.press_point
    };
    // A marca da barra não tem texto a abrir: nasce «movida», e soltar sem andar não abre o número.
    doc.list.scrub = Some(Scrub {
        action: a.id,
        zone,
        start_x: x,
        start,
        width: width.max(1.0),
        moved: !opens_text,
    });
}

pub(crate) fn press(doc: &mut InputMapDoc, x: f32, y: f32) -> bool {
    crate::view_list::commit_typing(doc);
    let Some(id) = doc.hits.hit(x, y) else {
        doc.store.set_focus(None);
        return doc.layout.body.contains(x, y);
    };
    let hit = doc.kb.hit_of(id);
    if doc.snap.listening.is_some() && hit != Some(KbHit::Listen) {
        doc.list.listen_request = Some(None);
    }
    if id == ids::NEW_NAME {
        doc.store.set_focus(Some(id));
        return true;
    }
    doc.store.set_focus(None);
    let open = doc.kb.open;
    match hit {
        Some(KbHit::Row(a)) => doc.kb.open = Some(a),
        Some(KbHit::Add) => {
            doc.kb.adding = !doc.kb.adding;
            doc.list.refused = None;
            doc.store.set_focus(doc.kb.adding.then_some(ids::NEW_NAME));
        }
        Some(KbHit::Remove) => {
            if let Some(a) = open {
                doc.pending.push(InputMapEdit::Remove(a));
            }
        }
        Some(KbHit::Sub(s)) => doc.kb.sub = s,
        Some(KbHit::Key(k)) => add(doc, Binding::Key(k)),
        Some(KbHit::Pad(b)) => add(doc, b),
        Some(KbHit::TileX(n)) => {
            if let Some(a) = open {
                doc.pending.push(InputMapEdit::RemoveBinding(a, n));
            }
        }
        Some(KbHit::Listen) => {
            if let Some(a) = open {
                let armed = doc.snap.listening == Some(a);
                doc.list.listen_request = Some((!armed).then_some(a));
            }
        }
        Some(h @ (KbHit::BarDead | KbHit::BarPress)) => {
            let zone = if h == KbHit::BarDead {
                Zone::Dead
            } else {
                Zone::Press
            };
            let w = doc.kb.bar.map_or(1.0, |b| b.1);
            scrub(doc, zone, x, w, false);
        }
        None => {
            let field = open.and_then(|a| {
                [
                    (ListHit::Dead(a), Zone::Dead),
                    (ListHit::Press(a), Zone::Press),
                ]
                .into_iter()
                .find(|(h, _)| h.id() == id)
            });
            if let Some((h, zone)) = field {
                let w = doc.hits.rect_for(h.id()).map_or(1.0, |r| r.w);
                scrub(doc, zone, x, w, true);
            }
        }
    }
    true
}

/// O ponteiro anda: o alvo sob ele (o pé, o ×, a dica) e o arrasto de um número ou de uma marca.
pub(crate) fn motion(doc: &mut InputMapDoc, x: f32, y: f32) -> bool {
    let h = doc.hits.hit(x, y).and_then(|id| doc.kb.hit_of(id));
    let dragged = crate::view_list::drag(doc, x);
    let changed = h != doc.kb.hover || dragged;
    let moved = doc.kb.pointer != Some((x, y));
    doc.kb.hover = h;
    doc.kb.pointer = Some((x, y));
    changed || moved
}

/// A tecla para o campo do nome (pela porta da vista A) ou o número aberto; Enter no nome cria e abre a acção.
pub(crate) fn key(doc: &mut InputMapDoc, input: &KeyInput<'_>) -> bool {
    let name = doc
        .store
        .text(ids::NEW_NAME)
        .unwrap_or("")
        .trim()
        .to_owned();
    let naming = doc.store.focus_id() == Some(ids::NEW_NAME);
    if naming && input.pressed && input.key == DocKey::Escape {
        doc.kb.adding = false;
    }
    let took = crate::view_list::key(doc, input);
    if naming
        && doc
            .pending
            .iter()
            .any(|e| *e == InputMapEdit::Add(name.clone()))
    {
        doc.kb.adding = false;
        doc.kb.select_name = Some(name);
        doc.store.set_focus(None);
    }
    took
}

/// A tecla física: com a acção aberta ainda sem ligações e nenhum campo focado, a tecla liga-se
/// («Click a key above, or press one»).
pub(crate) fn raw_key(doc: &mut InputMapDoc, key: Key, pressed: bool, _writes_text: bool) -> bool {
    if !pressed || doc.store.focus_id().is_some() || doc.list.typing.is_some() {
        return false;
    }
    let empty = doc
        .kb
        .open
        .and_then(|a| doc.snap.map.get(a))
        .is_some_and(|a| a.bindings.is_empty());
    let editing = ["escape", "tab", "backspace", "delete", "enter"];
    if !empty
        || editing
            .iter()
            .filter_map(|n| Key::named_in_script(n))
            .flatten()
            .any(|k| k == key)
    {
        return false;
    }
    add(doc, Binding::Key(key));
    true
}
