//! ⭐ **As ferramentas do PINTOR na coluna da esquerda** (dono 23/10: *«conforme foi feito no módulo
//! vector vamos colocar na barra lateral esquerda, com a mesma decoração»*).
//!
//! A coluna é a MESMA porta do vetor: o chrome publica [`ToolBars`] no store e o
//! [`super::float_bar::tool_bars::paint`] desenha-a — nenhum segundo pintor. A LISTA continua a do
//! trilho ([`super::left_rail::tool_section`]); aqui só se agrupa e se troca a cara (chip → botão). Os
//! ids são os do trilho, e o clique segue pelo despacho dele (`chrome/rail_painter_tools.rs`). A fila
//! de cima fica com o que não é ferramenta ([`super::left_rail::view_entries`]).

use super::HeroScreen;
use super::left_rail::{PAINTER_MASK_SUBS, PAINTER_SHAPES, RailTool, tool_section};
use crate::ids;
use crate::interaction::{BarButton, BarFace, BarFamily, ToolBars, WidgetStore};
use crate::widget::{ButtonState, ToolRailEntry};
use ph2d_a11y::NodeId;

/// Os grupos da coluna, por ofício: pôr tinta · mexer na tinta que lá está · máscara, seleção, cura e formas.
/// ⚠️ A Máscara e a Seleção são DOIS botões (dono 24/10: *«temos espaço para ambos»*) — o grupo com flyout do
/// trilho (`PAINTER_RAIL_MASK_GROUP`) só existe na fila antiga.
const GRUPOS: [&[NodeId]; 3] = [
    &[
        ids::PAINTER_RAIL_BRUSH,
        ids::PAINTER_RAIL_FILL,
        ids::PAINTER_RAIL_EYEDROPPER,
        ids::PAINTER_RAIL_ERASER,
    ],
    &[
        ids::PAINTER_RAIL_WET_BLEND,
        ids::PAINTER_RAIL_WET_SMEAR,
        ids::PAINTER_RAIL_WET_WATER,
        ids::PAINTER_RAIL_WET_DRY,
        ids::PAINTER_RAIL_WET_BLOW,
        ids::PAINTER_RAIL_CLONE,
        ids::PAINTER_RAIL_SMEAR,
        ids::PAINTER_RAIL_BLUR,
        ids::PAINTER_RAIL_LIQUIFY,
        ids::PAINTER_RAIL_TRANSFORM,
    ],
    &[
        ids::PAINTER_RAIL_MASK,
        ids::PAINTER_RAIL_SELECTION,
        ids::PAINTER_RAIL_INPAINT,
        ids::PAINTER_RAIL_SHAPES,
    ],
];

/// A cor do ícone: o vetor pinta por família, e o pintor também.
fn familia(grupo: usize) -> BarFamily {
    match grupo {
        0 => BarFamily::Draw,
        1 => BarFamily::Carve,
        _ => BarFamily::Build,
    }
}

fn botao(e: ToolRailEntry, family: BarFamily) -> Option<BarButton> {
    match e {
        ToolRailEntry::Icon {
            id,
            label,
            icon,
            active,
            ..
        } => Some(BarButton {
            id,
            face: BarFace::Icon(icon),
            label,
            on: active,
            family,
        }),
        ToolRailEntry::Swatch {
            id,
            label,
            color,
            active,
            mixed,
            ..
        } => Some(BarButton {
            id,
            face: BarFace::Swatch {
                rgba: Some(color),
                ring: false,
                mixed,
            },
            label,
            on: active,
            family,
        }),
        _ => None,
    }
}

fn flyout(store: &WidgetStore, itens: &[RailTool], family: BarFamily) -> Vec<BarButton> {
    itens
        .iter()
        .map(|(id, label, icon, _)| BarButton {
            id: *id,
            face: BarFace::Icon(*icon),
            label: label.tr().to_owned(),
            on: matches!(store.button_state(*id), Some(ButtonState::Pressed)),
            family,
        })
        .collect()
}

/// A Máscara e a Seleção, cada uma no seu botão: acesa a que o sub-rádio do grupo tem (a Máscara por omissão)
/// quando o grupo é a ferramenta na mão. O clique segue pelo despacho do sub-botão (`rail_painter_tools`).
fn mascara_e_selecao(store: &WidgetStore) -> Vec<BarButton> {
    let grupo = matches!(
        store.button_state(ids::PAINTER_RAIL_MASK_GROUP),
        Some(ButtonState::Pressed)
    );
    let premido = |id| matches!(store.button_state(id), Some(ButtonState::Pressed));
    let ativo = PAINTER_MASK_SUBS
        .iter()
        .map(|t| t.0)
        .find(|id| premido(*id))
        .unwrap_or(ids::PAINTER_RAIL_MASK);
    let mut b = flyout(store, &PAINTER_MASK_SUBS, familia(2));
    for x in &mut b {
        x.on = grupo && x.id == ativo;
    }
    b
}

/// ⭐ **As barras do pintor neste quadro** — a coluna (pela lista do trilho), o flyout aberto e a fila.
#[must_use]
pub fn painter_tool_bars(store: &WidgetStore) -> ToolBars {
    let mut column: Vec<Vec<BarButton>> = vec![Vec::new(); GRUPOS.len()];
    for e in tool_section(store, true) {
        let Some(id) = e.node_id() else { continue };
        if id == ids::PAINTER_RAIL_MASK_GROUP {
            column[2].extend(mascara_e_selecao(store));
            continue;
        }
        let Some(g) = GRUPOS.iter().position(|ids| ids.contains(&id)) else {
            continue;
        };
        if let Some(b) = botao(e, familia(g)) {
            column[g].push(b);
        }
    }
    let flyout = if store.painter_shapes_flyout_open() {
        Some((
            ids::PAINTER_RAIL_SHAPES,
            flyout(store, &PAINTER_SHAPES, familia(2)),
        ))
    } else {
        None
    };
    ToolBars {
        column,
        flyout,
        row_lead: super::left_rail::view_entries(store),
        painter_rail: true,
        ..ToolBars::default()
    }
}

/// Publica (ou limpa) a coluna do pintor — em todo quadro, antes de qualquer leitor das barras. Um
/// módulo com barras próprias (o vetor) tem a vez: a coluna do pintor só ocupa um lugar vazio ou o seu.
pub fn publish(hero: &mut HeroScreen) {
    let quer = hero.rail_shows_painter_tools()
        && !hero.view.legacy_chrome
        && hero.documents.scene_active();
    let de_quem = hero.store.tool_bars().map(|b| b.painter_rail);
    match (quer, de_quem) {
        (true, None | Some(true)) => {
            let bars = painter_tool_bars(&hero.store);
            hero.store.set_tool_bars(Some(bars));
        }
        (false, Some(true)) => hero.store.set_tool_bars(None),
        _ => {}
    }
}
