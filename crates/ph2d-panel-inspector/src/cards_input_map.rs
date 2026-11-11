//! ⭐ **O Inspector com a aba Input Map à frente** (onda 4, 7b): a acção aberta da aba em cartões só de leitura.
//! A aba publica-a a cada quadro (`ph2d_input::inspected`); quem edita é a aba (uma porta, um undo).
//!
//! Cada facto é uma linha da casa: rótulo fraco em cima, valor forte em baixo. Uma lista (leitores, teclas) é UM
//! valor «A, B, C» que só desce de linha quando não cabe na largura MEDIDA do cartão (nunca um item por caixa).

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics::VALUE_PX;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Widget};
use ph2d_i18n::tr;
use ph2d_tool_registry::hash_node_id;

pub const CARD_IM_ACTION: NodeId = hash_node_id("cards.insp.input_map.action");
pub const CARD_IM_STICK: NodeId = hash_node_id("cards.insp.input_map.stick");
pub const CARD_IM_PICK: NodeId = hash_node_id("cards.insp.input_map.pick");

pub use ph2d_input::inspected::{
    InputMapAction, InputMapPane, current_input_map, set_current_input_map,
};

/// Um facto: o rótulo e os itens (um só para um valor simples).
type Fact = (&'static str, Vec<String>);

/// Os itens juntos por «, » em linhas que cabem em `w` (um item maior que a linha fica sozinho nela).
fn pack(c: &mut CardCtx<'_>, items: &[String], w: f32) -> Vec<String> {
    let sep = tr("input_map_list.comma");
    let mut lines: Vec<String> = Vec::new();
    for it in items {
        match lines.last_mut() {
            Some(l)
                if c.text_system
                    .prefix_width(&format!("{l}{sep}{it}"), VALUE_PX)
                    <= w =>
            {
                l.push_str(sep);
                l.push_str(it);
            }
            Some(l) => {
                l.push_str(sep.trim_end());
                lines.push(it.clone());
            }
            None => lines.push(it.clone()),
        }
    }
    lines
}

fn card(section: NodeId, title: &'static str, facts: Vec<Fact>) -> CardDef<'static> {
    CardDef {
        section,
        title,
        actions: Vec::new(),
        add: None,
        summary: None,
        body: Some(Box::new(move |c, mut y| {
            for (label, items) in &facts {
                for (i, text) in pack(c, items, c.w).iter().enumerate() {
                    let label = (i == 0).then_some(*label);
                    y = c.full(label, Widget::Readout { text }, y);
                }
            }
            y
        })),
    }
}

fn or_else(items: &[String], none: &'static str) -> Vec<String> {
    if items.is_empty() {
        vec![none.to_owned()]
    } else {
        items.to_vec()
    }
}

/// Os cartões do painel com a aba à frente.
#[must_use]
pub(crate) fn cards(pane: &InputMapPane) -> Vec<CardDef<'static>> {
    let a = match pane {
        InputMapPane::Pick => {
            let frase = tr("panel.inspector.input_map.pick");
            return vec![CardDef {
                section: CARD_IM_PICK,
                title: "",
                actions: Vec::new(),
                add: None,
                summary: None,
                body: Some(Box::new(move |c, y| c.hint(frase, y))),
            }];
        }
        InputMapPane::Action(a) => a,
    };
    let facts = vec![
        (tr("panel.inspector.input_map.name"), vec![a.name.clone()]),
        (tr("panel.inspector.input_map.group"), vec![a.group.clone()]),
        (
            tr("panel.inspector.input_map.read_by"),
            or_else(&a.readers, tr("input_map_keyboard.read_by_nobody")),
        ),
        (
            tr("panel.inspector.input_map.keys"),
            or_else(&a.keys, tr("panel.inspector.input_map.no_keys")),
        ),
    ];
    let mut out = vec![card(
        CARD_IM_ACTION,
        tr("panel.inspector.input_map.action"),
        facts,
    )];
    if let Some((dead, press)) = &a.stick {
        let facts = vec![
            (tr("panel.inspector.input_map.dead"), vec![dead.clone()]),
            (tr("panel.inspector.input_map.press"), vec![press.clone()]),
        ];
        out.push(card(
            CARD_IM_STICK,
            tr("panel.inspector.input_map.stick"),
            facts,
        ));
    }
    out
}
