//! O despacho do navegador.
//!
//! ⚠️ **A abertura do painel é a metade que faltava, e ela é sobre um CHIP MORTO.** O pill
//! `TOPBAR_RIGHT_ASSETS` era pintado, registado e hit-indexado, e nenhum `apply_event` do app
//! ramificava nele — a terceira pergunta do §5.0 (*o leitor DECIDE?*) respondia **não**. É este
//! braço que a responde.

use crate::AssetBrowserPanel;
use crate::ids;
use crate::state::AssetBrowserState;
use ph2d_asset_index::{AssetRef, SortBy};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};

pub(crate) fn apply_event(
    state: &mut AssetBrowserState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    match ev {
        // ── Abrir / fechar ─────────────────────────────────────────────────────────────────────
        WidgetEvent::Click(id)
            if id == ph2d_editor_core::ids::TOPBAR_RIGHT_ASSETS || id == ids::ASSET_CLOSE =>
        {
            let next = !host.panel_visible(AssetBrowserPanel::ID);
            host.set_panel_visible(AssetBrowserPanel::ID, next);
            EventOutcome::Consumed
        }

        // ⛔ **Tudo o que vem a seguir só existe com o painel ABERTO.** Sem esta guarda os chips
        // do painel fechado continuariam a responder — eles não estão pintados, mas o
        // `WidgetStore` ainda os conhece, e um `Click` sintético (de um teste, de um atalho, da
        // paleta de comandos) chegaria a eles.
        _ if !host.panel_visible(AssetBrowserPanel::ID) => EventOutcome::Ignored,

        // ── O filtro de família ────────────────────────────────────────────────────────────────
        WidgetEvent::Click(id) if chip_index(&ids::ASSET_KIND, id).is_some() => {
            let i = chip_index(&ids::ASSET_KIND, id).unwrap_or(0);
            state.kind = AssetBrowserState::kind_for_chip(i);
            reset_scroll(host);
            EventOutcome::Consumed
        }

        // ── A ordenação ────────────────────────────────────────────────────────────────────────
        WidgetEvent::Click(id) if chip_index(&ids::ASSET_SORT, id).is_some() => {
            let i = chip_index(&ids::ASSET_SORT, id).unwrap_or(0);
            if let Some(sort) = SortBy::ALL.get(i) {
                state.sort = *sort;
            }
            reset_scroll(host);
            EventOutcome::Consumed
        }

        // ── O tamanho do cartão ────────────────────────────────────────────────────────────────
        WidgetEvent::ValueChanged(id) if id == ids::ASSET_SIZE => {
            if let Some((_, v)) = host.store().slider(ids::ASSET_SIZE) {
                state.set_size_from_slider(v);
            }
            EventOutcome::Consumed
        }

        // ── A busca ────────────────────────────────────────────────────────────────────────────
        //
        // ⚠️ **O texto não é copiado para o estado do painel.** Ele vive no `WidgetStore`, que é
        // quem o campo escreve, e o `paint` lê-o de lá — uma cópia aqui seria a segunda resposta a
        // *«o que está escrito na busca?»*, e a que envelhece é a que o artista vê.
        // O que este braço faz é o que a mudança de texto EXIGE: voltar ao topo, senão a rolagem
        // aponta para uma linha que a nova lista não tem.
        WidgetEvent::TextChanged(id) if id == ids::ASSET_SEARCH => {
            reset_scroll(host);
            EventOutcome::Consumed
        }

        // ── ⭐ O VERBO DE USAR (wave A7) ────────────────────────────────────────────────────────
        //
        // ⚠️ **Duplo-clique instancia; clique simples escolhe.** É a convenção dos três
        // referenciais (Godot, Blender, Unreal) e a razão é a mesma: um clique é como se navega, e
        // um navegador que age ao primeiro toque não deixa ninguém percorrer.
        WidgetEvent::DoubleClick(id) => match cell_target_of(id) {
            Some(AssetRef::Component { stable_id }) => {
                host.bus_mut()
                    .push(EditorAction::AssetInstantiate { stable_id });
                EventOutcome::Consumed
            }
            // ⛔ **Uma textura NÃO se instancia**, e o silêncio aqui é declarado: pôr uma imagem na
            // cena é a queda num alvo (wave B3), porque *qual* objecto a recebe é o que a queda
            // responde e um duplo-clique não. Fazer nascer uma sprite nova seria inventar um
            // gesto que o artista não pediu.
            Some(AssetRef::Texture { .. }) | None => EventOutcome::Ignored,
        },

        _ => EventOutcome::Ignored,
    }
}

/// O índice do chip na fileira, se `id` for um deles.
fn chip_index(table: &[ph2d_a11y::NodeId], id: ph2d_a11y::NodeId) -> Option<usize> {
    table.iter().position(|c| *c == id)
}

/// O asset que este id de célula desenhou **neste quadro**.
fn cell_target_of(id: ph2d_a11y::NodeId) -> Option<AssetRef> {
    (0..ids::MAX_ASSET_CELLS)
        .find(|i| ids::asset_cell_id(*i) == id)
        .and_then(crate::paint::cell_target)
}

/// Mudar o que a lista contém tem de voltar ao topo — senão a rolagem aponta para uma linha que a
/// lista nova não tem, e a grade parece vazia sobre um resultado que existe.
fn reset_scroll(host: &mut dyn PanelHostInternal) {
    host.store_mut()
        .set_panel_scroll(ph2d_editor_core::ids::ASSET_PANEL, 0.0);
}
