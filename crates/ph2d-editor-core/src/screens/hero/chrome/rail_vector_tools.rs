// ph2d-chrome-sync:z=61 (dispatch priority, ADR-0107; lower = earlier)
//! ⭐ **As ferramentas de CRIAR forma vetorial no trilho** — Caneta · Lápis · Formas · Texto, na
//! face Object (spec/06 F3 ▸ Vector; escolha do dono, 03/10: o pill VECTOR saiu, e criar uma forma
//! é criar um OBJECTO — é do modo Object, como as ferramentas de *Add* da barra do Blender).
//!
//! O clique pede a ferramenta `vector` e deixa o id no canal de pick: a fundação não conhece o
//! `DrawMode`, e quem o põe na mão é `ph2d_app_vec::rail` — que também deriva, em todo quadro, que
//! botão está aceso ([`sync`]).

use crate::action_bus::EditorAction;
use crate::icons::IconId;
use crate::ids;
use crate::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use crate::screens::hero::HeroScreen;
use crate::widget::ButtonState;
use ph2d_a11y::NodeId;
use ph2d_i18n::TextKey;

/// Os quatro botões, na ordem do trilho: `(id, NOME, ícone, SUB-RÓTULO)` — a forma do `RailTool`.
pub(crate) const TOOLS: [(NodeId, TextKey, IconId, TextKey); 4] = [
    (
        ids::VECTOR_RAIL_PEN,
        TextKey::new("chrome.rail.pen"),
        IconId::VectorPen,
        TextKey::new("chrome.rail.sub.pen"),
    ),
    (
        ids::VECTOR_RAIL_PENCIL,
        TextKey::new("chrome.rail.pencil"),
        IconId::VectorPencil,
        TextKey::new("chrome.rail.sub.pencil"),
    ),
    (
        ids::VECTOR_RAIL_SHAPE,
        TextKey::new("chrome.rail.shapes"),
        IconId::VectorShape,
        TextKey::new("chrome.rail.sub.shape"),
    ),
    (
        ids::VECTOR_RAIL_TEXT,
        TextKey::new("chrome.rail.text"),
        IconId::Text,
        TextKey::new("chrome.rail.sub.text"),
    ),
];

/// ⭐ **O botão aceso é DERIVADO** da ferramenta na mão (`None` = nenhum): o painel do vetor, um
/// atalho ou o modo Edit também trocam de ferramenta, e o trilho não pode dar outra resposta.
pub fn sync(store: &mut WidgetStore, pressed: Option<NodeId>) {
    for id in ids::VECTOR_RAIL_TOOL_IDS {
        let want = if Some(id) == pressed {
            ButtonState::Pressed
        } else {
            ButtonState::Normal
        };
        if let Some(InteractiveState::Button { state }) = store.get_mut(id)
            && *state != want
        {
            *state = want;
        }
    }
}

/// `true` = um destes botões está aceso (a ferramenta de criar está na mão).
#[must_use]
pub fn any_pressed(store: &WidgetStore) -> bool {
    ids::VECTOR_RAIL_TOOL_IDS
        .into_iter()
        .any(|id| matches!(store.button_state(id), Some(ButtonState::Pressed)))
}

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if !ids::VECTOR_RAIL_TOOL_IDS.contains(&id) {
        return false;
    }
    hero.bus
        .push(EditorAction::ActivateTool { tool_id: "vector" });
    hero.store.set_command_pick(id);
    true
}

// ⚠️ Os gates moram em `rail_vector_tools/tests.rs`, num DIRETÓRIO: o `ph2d-chrome-sync` trata todo
// `chrome/*.rs` do topo como um handler (ver `rail_painter_tools.rs`).
#[cfg(test)]
mod tests;
