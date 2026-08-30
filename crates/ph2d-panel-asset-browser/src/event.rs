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
use ph2d_editor_core::action_bus::{AssetCardAction, CatalogVerb, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::interaction::drag_payload::DragPayload;
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
                host.bus_mut().push(EditorAction::AssetInstantiate {
                    stable_id,
                    // ⚠️ Um duplo-clique não aponta para lado nenhum — a cascata é a resposta.
                    at: None,
                });
                EventOutcome::Consumed
            }
            // ⛔ **Uma textura NÃO se instancia**, e o silêncio aqui é declarado: pôr uma imagem na
            // cena é a queda num alvo (wave B3), porque *qual* objecto a recebe é o que a queda
            // responde e um duplo-clique não. Fazer nascer uma sprite nova seria inventar um
            // gesto que o artista não pediu.
            Some(AssetRef::Texture { .. }) | None => EventOutcome::Ignored,
        },

        // ── ⭐⭐ A COLUNA DE CATÁLOGOS (etapa D) ────────────────────────────────────────────────
        //
        // ⚠️ **A linha escolhida é VISTA, e o catálogo é DOCUMENTO.** Escolher não levanta acção
        // nenhuma — ela vive no estado do painel e morre com a sessão. Criar, renomear, apagar e
        // atribuir são o documento, e esses atravessam o barramento.
        WidgetEvent::Click(id) if id == ids::ASSET_CATALOG_TOGGLE => {
            state.show_catalogs = !state.show_catalogs;
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::ASSET_CATALOG_NEW => {
            // ⭐ O catálogo nasce DENTRO do escolhido — é o que o Blender faz, e é o que torna a
            // hierarquia alcançável sem um campo de caminho.
            let parent = match state.pick {
                crate::state::CatalogPick::One(c) => Some(c.0),
                _ => None,
            };
            host.bus_mut()
                .push(EditorAction::AssetCatalogVerb(CatalogVerb::New { parent }));
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if catalog_row_index(id).is_some() => {
            if let Some(i) = catalog_row_index(id)
                && let Some(pick) = crate::state::painted_row_at(i)
            {
                state.pick = pick;
                reset_scroll(host);
            }
            EventOutcome::Consumed
        }

        // ── ⭐⭐ O MENU DO CARTÃO (etapa C) ─────────────────────────────────────────────────────
        //
        // ⚠️ **Consumir o pedido de menu é destrutivo, e por isso a guarda é o ID primeiro.** O
        // `consume_last_context_menu` TIRA o pedido do store; se este braço corresse sobre um
        // `Click` alheio, ele engoliria o menu da Hierarquia e o item que o artista apertou lá
        // deixaria de achar a linha. Os três ids só nascem deste menu, então quando um deles
        // chega o pedido pendente é, por construção, o `AssetCard`.
        WidgetEvent::Click(id) if card_verb_of(id).is_some() => {
            let Some(verb) = card_verb_of(id) else {
                return EventOutcome::Ignored;
            };
            let Some(req) = host.store_mut().consume_last_context_menu() else {
                return EventOutcome::Ignored;
            };
            let ph2d_editor_core::interaction::ContextMenuKind::AssetCard { cell } = req.kind
            else {
                return EventOutcome::Ignored;
            };
            // ⚠️ **A janela de obsolescência fecha AQUI.** O menu abriu no `Down` e este `Click`
            // é posterior; se a grade mudou de conteúdo no meio, a célula já não desenha asset
            // nenhum, e agir sobre a célula seguinte apagaria o prefab errado.
            //
            // ⛔⛔ **E o `None` é SILENCIOSO — a 1.ª redacção desta nota prometia o contrário** e a
            // auditoria de 2026-08-30 apanhou-a: ela dizia *«o `None` não se trata como nada a
            // fazer, o shell é quem fala»*, e o shell nunca é informado, portanto ninguém fala. O
            // gémeo exacto — uma queda cujo `StableId` desapareceu entre o `Down` e o `Up` — FALA,
            // em `render_loop/hierarchy.rs` (*«That prefab is no longer in the project»*).
            //
            // ⏳ **Fica NOMEADO e não curado:** o `AssetCardVerb` carrega um endereço, e um menu
            // sem sujeito não tem endereço nenhum para carregar. Dizê-lo pede uma acção própria —
            // e a janela em que isto acontece é a de um artista a rolar a grade com o menu aberto.
            match cell_target_of(cell) {
                Some(AssetRef::Component { stable_id }) => {
                    host.bus_mut().push(EditorAction::AssetCardVerb {
                        asset: DragPayload::Prefab { stable_id },
                        verb,
                    });
                    EventOutcome::Consumed
                }
                Some(AssetRef::Texture { asset }) => {
                    host.bus_mut().push(EditorAction::AssetCardVerb {
                        asset: DragPayload::Image { asset },
                        verb,
                    });
                    EventOutcome::Consumed
                }
                None => EventOutcome::Ignored,
            }
        }

        _ => EventOutcome::Ignored,
    }
}

/// O índice da linha de catálogo, se `id` for uma.
///
/// ⚠️ **Varre a tabela de ids como o irmão dos cartões**, e pela mesma razão: o id é posicional na
/// lista VISÍVEL, e é isso que faz a linha debaixo do dedo ser a que o artista vê.
fn catalog_row_index(id: ph2d_a11y::NodeId) -> Option<usize> {
    (0..ids::MAX_CATALOG_ROWS).find(|i| ids::catalog_row_id(*i) == id)
}

/// O verbo que este id de menu representa. ⚠️ **Uma tabela, não três `if`** — ela é o par exacto
/// da tabela que o overlay pinta (`menu_rows`), e é o que faz um verbo novo ter de aparecer nos
/// dois sítios ou em nenhum.
fn card_verb_of(id: ph2d_a11y::NodeId) -> Option<AssetCardAction> {
    use ph2d_editor_core::ids as core_ids;
    match id {
        i if i == core_ids::CTX_MENU_ASSET_INSTANTIATE => Some(AssetCardAction::Instantiate),
        i if i == core_ids::CTX_MENU_ASSET_SELECT_USERS => Some(AssetCardAction::SelectUsers),
        i if i == core_ids::CTX_MENU_ASSET_REMOVE => Some(AssetCardAction::RemoveFromLibrary),
        _ => None,
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
