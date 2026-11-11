//! ⭐ **O Inspector com a aba Input Map à frente** (onda 4, 7b): a acção aberta da aba em cartões só de leitura.
//! A aba publica-a a cada quadro (`ph2d_input::inspected`); quem edita é a aba (uma porta, um undo).

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::cards::{CardDef, Widget};
use ph2d_i18n::tr;
use ph2d_tool_registry::hash_node_id;

pub const CARD_IM_ACTION: NodeId = hash_node_id("cards.insp.input_map.action");
pub const CARD_IM_READERS: NodeId = hash_node_id("cards.insp.input_map.readers");
pub const CARD_IM_KEYS: NodeId = hash_node_id("cards.insp.input_map.keys");
pub const CARD_IM_STICK: NodeId = hash_node_id("cards.insp.input_map.stick");
pub const CARD_IM_PICK: NodeId = hash_node_id("cards.insp.input_map.pick");

pub use ph2d_input::inspected::{
    InputMapAction, InputMapPane, current_input_map, set_current_input_map,
};

fn card(
    section: NodeId,
    title: &'static str,
    rows: Vec<(Option<&'static str>, String)>,
) -> CardDef<'static> {
    CardDef {
        section,
        title,
        actions: Vec::new(),
        add: None,
        summary: None,
        body: Some(Box::new(move |c, mut y| {
            for (label, text) in &rows {
                y = c.full(*label, Widget::Readout { text }, y);
            }
            y
        })),
    }
}

fn hint(section: NodeId, title: &'static str, frase: &'static str) -> CardDef<'static> {
    CardDef {
        section,
        title,
        actions: Vec::new(),
        add: None,
        summary: None,
        body: Some(Box::new(move |c, y| c.hint(frase, y))),
    }
}

/// Os cartões do painel com a aba à frente.
#[must_use]
pub(crate) fn cards(pane: &InputMapPane) -> Vec<CardDef<'static>> {
    let a = match pane {
        InputMapPane::Pick => {
            return vec![hint(CARD_IM_PICK, "", tr("panel.inspector.input_map.pick"))];
        }
        InputMapPane::Action(a) => a,
    };
    let mut out = vec![card(
        CARD_IM_ACTION,
        tr("panel.inspector.input_map.action"),
        vec![
            (Some(tr("panel.inspector.input_map.name")), a.name.clone()),
            (Some(tr("panel.inspector.input_map.group")), a.group.clone()),
        ],
    )];
    let title = tr("panel.inspector.input_map.read_by");
    out.push(if a.readers.is_empty() {
        hint(
            CARD_IM_READERS,
            title,
            tr("input_map_keyboard.read_by_nobody"),
        )
    } else {
        card(
            CARD_IM_READERS,
            title,
            a.readers.iter().map(|r| (None, r.clone())).collect(),
        )
    });
    let title = tr("panel.inspector.input_map.keys");
    out.push(if a.keys.is_empty() {
        hint(CARD_IM_KEYS, title, tr("panel.inspector.input_map.no_keys"))
    } else {
        card(
            CARD_IM_KEYS,
            title,
            a.keys.iter().map(|k| (None, k.clone())).collect(),
        )
    });
    if let Some((dead, press)) = &a.stick {
        let rows = vec![
            (Some(tr("panel.inspector.input_map.dead")), dead.clone()),
            (Some(tr("panel.inspector.input_map.press")), press.clone()),
        ];
        out.push(card(
            CARD_IM_STICK,
            tr("panel.inspector.input_map.stick"),
            rows,
        ));
    }
    out
}
