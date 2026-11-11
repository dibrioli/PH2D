//! ⭐ **A acção aberta para o Inspector** (onda 4, 7b): lida do MESMO snapshot que a aba pinta, nunca uma cópia.
//! Nas três vistas é a acção aberta da aba (`view_keyboard::open`), a linha que a List e a Keyboard marcam.

use ph2d_i18n::tr;
use ph2d_input::inspected::{InputMapAction, InputMapPane};

use crate::doc::InputMapDoc;
use crate::view_keyboard::Group;
use crate::view_list::footer::{binding_label, readers_of};

/// O que o Inspector mostra com a aba à frente.
#[must_use]
pub fn pane(doc: &InputMapDoc) -> InputMapPane {
    let map = &doc.snap.map;
    // UMA fonte nas três vistas: a acção aberta da aba (a escolhida, senão a primeira), a que a List e a Keyboard marcam.
    let id = crate::view_keyboard::open(doc);
    let Some(a) = id.and_then(|id| map.get(id)) else {
        return InputMapPane::Pick;
    };
    let group = match crate::view_keyboard::group_of(doc, &a.name) {
        Group::PlatformPlayer => tr("input_map_tab.reader.platform_player"),
        Group::TopDownPlayer => tr("input_map_tab.reader.top_down_player"),
        Group::Yours => tr("panel.inspector.input_map.yours"),
    };
    let readers = readers_of(&doc.snap, &a.name)
        .iter()
        .map(|r| crate::view_list::footer::reader_names(std::slice::from_ref(r)))
        .collect();
    InputMapPane::Action(InputMapAction {
        name: a.name.clone(),
        group: group.to_owned(),
        readers,
        keys: a.bindings.iter().map(|b| binding_label(*b)).collect(),
        stick: a.is_analog().then(|| {
            (
                format!("{:.2}", a.dead_zone),
                format!("{:.2}", a.press_point),
            )
        }),
    })
}

/// ⭐ O snapshot do quadro e, DEPOIS dele, a publicação: o Inspector e a aba mostram a mesma acção no MESMO quadro.
pub fn set_snapshot(doc: &mut InputMapDoc, snap: crate::Snapshot, in_front: bool) {
    doc.set_snapshot(snap);
    publish(doc, in_front);
}

/// Publica (aba à frente) ou apaga (outra à frente) o que o Inspector mostra.
pub fn publish(doc: &InputMapDoc, in_front: bool) {
    ph2d_input::inspected::set_current_input_map(in_front.then(|| pane(doc)));
}
