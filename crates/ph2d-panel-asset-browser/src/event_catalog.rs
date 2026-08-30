//! ⭐⭐ **Os gestos da TAXONOMIA** (plano 07, etapa D) — cortados do [`crate::event`] por
//! responsabilidade quando ele passou o tecto de 200 LOC por função.
//!
//! ⚠️ **A guarda do painel fechado vive no chamador**, e é por isso que ela não se repete aqui:
//! este módulo só é alcançado depois de o `apply_event` a ter passado. Um segundo `panel_visible`
//! aqui seria a segunda resposta à mesma pergunta.

use crate::ids;
use crate::state::AssetBrowserState;
use ph2d_editor_core::action_bus::{CatalogVerb, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};

/// Despacha um evento de catálogo. `Ignored` = não era um.
pub(crate) fn apply(
    state: &mut AssetBrowserState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    match ev {
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
        WidgetEvent::Click(id) if ids::catalog_row_index(id).is_some() => {
            if let Some(i) = ids::catalog_row_index(id)
                && let Some(pick) = crate::state::painted_row_at(i)
            {
                state.pick = pick;
                crate::event::reset_scroll(host);
            }
            EventOutcome::Consumed
        }

        // ── ⭐⭐ O MENU DA LINHA DE CATÁLOGO (etapa D) ──────────────────────────────────────────
        //
        // ⚠️ **A mesma guarda destrutiva do menu do cartão**: o id primeiro, o
        // `consume_last_context_menu` depois. Os dois ids só nascem deste menu, então quando um
        // chega o pedido pendente é, por construção, o `CatalogRow`.
        //
        // ⚠️ **O sujeito resolve-se pelo CENSO DO QUADRO** (`catalog_row_pick`), não pela escada:
        // o id é posicional, e a lista pode ter mudado entre o `Down` que abriu o menu e este
        // `Click`. Se ele já não nomeia catálogo nenhum, o braço desiste — apagar a gaveta
        // seguinte seria pior do que não fazer nada.
        WidgetEvent::Click(id)
            if id == ph2d_editor_core::ids::CTX_MENU_CATALOG_RENAME
                || id == ph2d_editor_core::ids::CTX_MENU_CATALOG_DELETE =>
        {
            let Some(req) = host.store_mut().consume_last_context_menu() else {
                return EventOutcome::Ignored;
            };
            let ph2d_editor_core::interaction::ContextMenuKind::CatalogRow { row } = req.kind
            else {
                return EventOutcome::Ignored;
            };
            // ⛔ Só um catálogo de verdade tem nome para mudar ou gaveta para apagar — o `All` e
            // o `Unassigned` são linhas fixas, e o menu sobre elas não faz nada.
            let Some(crate::state::CatalogPick::One(cid)) = crate::state::catalog_row_pick(row)
            else {
                return EventOutcome::Ignored;
            };
            if id == ph2d_editor_core::ids::CTX_MENU_CATALOG_RENAME {
                crate::catalog_rename::open(state, cid);
            } else {
                // ⚠️ **A linha escolhida volta a `All` se era ESTA** — senão a grade ficaria a
                // filtrar por um catálogo que já não existe, e o artista veria zero cartões sem
                // nada seleccionado que o explicasse.
                if state.pick == crate::state::CatalogPick::One(cid) {
                    state.pick = crate::state::CatalogPick::All;
                }
                host.bus_mut()
                    .push(EditorAction::AssetCatalogVerb(CatalogVerb::Delete {
                        id: cid.0,
                    }));
            }
            EventOutcome::Consumed
        }

        // ── ⭐ O CAMPO DE RENOMEAR ─────────────────────────────────────────────────────────────
        //
        // Enter (`Submit`) e o clique-fora (`Blur`) gravam — o `take` lá dentro faz do segundo do
        // par um no-op; o Esc (`Cancel`) abandona.
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id) if id == ids::ASSET_CATALOG_RENAME => {
            if let Some((cid, name)) = crate::catalog_rename::commit(state, host.store()) {
                host.bus_mut()
                    .push(EditorAction::AssetCatalogVerb(CatalogVerb::Rename {
                        id: cid,
                        name,
                    }));
            }
            EventOutcome::Consumed
        }
        WidgetEvent::Cancel(id) if id == ids::ASSET_CATALOG_RENAME => {
            crate::catalog_rename::cancel(state);
            EventOutcome::Consumed
        }
        _ => EventOutcome::Ignored,
    }
}
