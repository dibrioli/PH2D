//! ⭐ **A BUSCA QUE RESPONDE** (line/BarraDeMenus, onda 3): além dos comandos, a busca acha as ações
//! do Input Map (com as teclas de cada uma), os objetos da cena (escolher = selecionar) e os campos
//! dos cartões do objeto selecionado (escolher = mostrar onde o campo está e dar-lhe o teclado).
//!
//! Cada fonte é um facto que o app já publica — o `InputMap` do hero, a lista da Hierarquia
//! ([`WidgetStore::scene_objects`]) e o que os cartões pintaram ([`WidgetStore::card_directory`]);
//! nenhuma tabela nova. Os ids das ações saem do `ActionId` estável; os dos objetos e dos campos
//! são os dos próprios widgets (a linha da Hierarquia, o campo do cartão).

use super::super::HeroScreen;
use crate::interaction::WidgetStore;
use crate::widget::command_palette::{PaletteGroup, PaletteItem, PaletteSub, SHORTCUT_SEP};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_i18n::tr;
use ph2d_input::{ActionId, Binding};
use ph2d_tokens::ColorToken;
use ph2d_tool_registry::hash_node_id;

const ACTION_BASE: NodeId = hash_node_id("palette.input_action");

/// O id do item da ação `a`.
#[must_use]
pub fn action_item(a: ActionId) -> NodeId {
    NodeId(ACTION_BASE.0 ^ (u64::from(a.0) + 1))
}

/// A ação do item `id`, se é uma (`map` = o mapa de agora).
fn action_of(map: &ph2d_input::InputMap, id: NodeId) -> Option<ActionId> {
    map.actions()
        .iter()
        .map(|a| a.id)
        .find(|a| action_item(*a) == id)
}

/// O nome de uma ligação: a tecla («Space»), o botão do comando ou a metade do eixo.
fn binding_label(b: &Binding) -> String {
    match b {
        Binding::Key(k) => k.label(),
        Binding::PadButton(p) => tr(p.label_key()).to_owned(),
        Binding::PadAxis { axis, positive } => {
            format!(
                "{}{}",
                tr(axis.label_key()),
                if *positive { "+" } else { "-" }
            )
        }
    }
}

/// As ações do Input Map, com as teclas na coluna da direita.
fn input_actions(hero: &HeroScreen) -> Option<PaletteGroup> {
    let items: Vec<PaletteItem> = hero
        .input_map
        .actions()
        .iter()
        .map(|a| {
            let keys: Vec<String> = a.bindings.iter().map(binding_label).collect();
            let label = if keys.is_empty() {
                a.name.clone()
            } else {
                format!("{}{SHORTCUT_SEP}{}", a.name, keys.join(", "))
            };
            PaletteItem {
                label,
                id: action_item(a.id),
                keywords: "",
                badge: None,
            }
        })
        .collect();
    group(
        tr("chrome.palette.input_map"),
        ColorToken::NodeCatSource,
        None,
        items,
    )
}

/// Os objetos da cena, pela ordem da Hierarquia.
fn scene_objects(store: &WidgetStore) -> Option<PaletteGroup> {
    let items = store
        .scene_objects()
        .iter()
        .map(|(row, name)| PaletteItem {
            label: name.clone(),
            id: *row,
            keywords: "",
            badge: None,
        })
        .collect();
    group(
        tr("chrome.palette.scene"),
        ColorToken::NodeCatTransform,
        None,
        items,
    )
}

/// Os campos dos cartões do objeto selecionado, uma fatia por cartão («Inspector · Transform»); o
/// cartão dobrado (sem campos pintados) entra pelo título.
fn card_fields(store: &WidgetStore) -> Option<PaletteGroup> {
    let dir = store.card_directory();
    let mut subs: Vec<PaletteSub> = Vec::new();
    let mut folded: Vec<PaletteItem> = Vec::new();
    for (card, title) in &dir.cards {
        let items: Vec<PaletteItem> = dir
            .fields
            .iter()
            .filter(|f| f.card == *card)
            .map(|f| PaletteItem {
                label: f.label.clone(),
                id: f.id,
                keywords: "",
                badge: None,
            })
            .collect();
        if items.is_empty() {
            folded.push(PaletteItem {
                label: title.clone(),
                id: *card,
                keywords: "",
                badge: None,
            });
        } else {
            subs.push(PaletteSub {
                title: Some(title.clone()),
                items,
            });
        }
    }
    if !folded.is_empty() {
        subs.insert(
            0,
            PaletteSub {
                title: None,
                items: folded,
            },
        );
    }
    (!subs.is_empty()).then(|| PaletteGroup {
        title: tr("chrome.palette.inspector").to_owned(),
        color: ColorToken::NodeCatUtility,
        subs,
    })
}

fn group(
    title: &str,
    color: ColorToken,
    sub: Option<String>,
    items: Vec<PaletteItem>,
) -> Option<PaletteGroup> {
    (!items.is_empty()).then(|| PaletteGroup {
        title: title.to_owned(),
        color,
        subs: vec![PaletteSub { title: sub, items }],
    })
}

/// Os grupos das três fontes (os que têm o que mostrar), na ordem: cartões, cena, Input Map.
#[must_use]
pub fn groups(hero: &HeroScreen) -> Vec<PaletteGroup> {
    [
        card_fields(&hero.store),
        scene_objects(&hero.store),
        input_actions(hero),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Executa um item destas fontes. `true` se era um deles.
pub fn route(hero: &mut HeroScreen, id: NodeId) -> bool {
    if let Some(a) = action_of(&hero.input_map, id) {
        hero.store.request_input_map_action(a);
        return true;
    }
    if hero.store.scene_objects().iter().any(|(row, _)| *row == id) {
        use crate::action_bus::{EditorAction, HierRequest, SelectModifier};
        hero.bus
            .push(EditorAction::Hierarchy(HierRequest::SelectRow {
                row: id,
                modifier: SelectModifier::Replace,
            }));
        hero.store.flash_anywhere(id);
        return true;
    }
    let dir = hero.store.card_directory();
    if let Some(f) = dir.fields.iter().find(|f| f.id == id) {
        // Fora da vista, a lista do Inspector rola até o campo (rótulo e caixa) ficar ao meio.
        let panel = crate::ids::INSP_PANEL;
        if let Some(view_h) = hero.store.panel_visible_h(panel) {
            let h = m::LABEL_TOP + m::FIELD_H;
            let now = hero.store.panel_scroll_target(panel);
            let fora = f.y < now || f.y + h > now + view_h;
            if fora {
                hero.store
                    .set_panel_scroll(panel, (f.y - (view_h - h) * 0.5).max(0.0));
            }
        }
        hero.store.flash_anywhere(id);
        hero.store.set_focus(Some(id));
        return true;
    }
    if dir.cards.iter().any(|(c, _)| *c == id) {
        hero.store.set_collapsed(id, false);
        hero.store.flash_anywhere(id);
        return true;
    }
    false
}

#[cfg(test)]
#[path = "global_palette_finds_tests.rs"]
mod tests;
