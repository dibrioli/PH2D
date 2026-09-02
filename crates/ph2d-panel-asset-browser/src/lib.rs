//! `ph2d-panel-asset-browser` — **o navegador de assets** (plano
//! [`docs/Components/07`](../../docs/Components/07_plano_do_navegador_de_assets.md), etapa A).
//!
//! # O que ele é
//!
//! Um painel flutuante que mostra **as duas famílias de asset numa grade só** — os componentes que
//! o artista marcou e as imagens que ele importou —, com busca, ordenação, tamanho de cartão, e o
//! verbo de **usar** (duplo-clique põe o componente na cena).
//!
//! # ⭐ O que ele NÃO é, e por quê
//!
//! ⛔ **Não é um navegador de ficheiros.** O `res://` do Godot e o sistema de ficheiros do Blender
//! não existem aqui: um componente é uma sub-árvore marcada no mundo, não um ficheiro. A taxonomia
//! é de **catálogos** (o Blender Asset Browser), e ela chega na wave A3.
//!
//! ⛔ **Ele não sabe o que existe.** O índice é publicado pelo shell
//! ([`set_current_index`]) — é lá que as duas fontes se juntam. *Um painel que descobre assets
//! sozinho é a segunda fonte de verdade sobre «o que existe».*
//!
//! # A porta de entrada
//!
//! O pill **Assets** da barra de cima (`TOPBAR_RIGHT_ASSETS`) — que existe e é pintado **desde
//! sempre** e que, até 2026-08-30, **não tinha despacho nenhum**. Ele era um dos três chips mortos
//! do grupo (Layers · Assets · Script): pintado, registado, hit-indexado, e nenhum leitor decidia
//! nada com ele.

#![forbid(unsafe_code)]

/// ⭐⭐ O QUE FICA POR BAIXO DA MINIATURA — a lei do fundo de um cartão, ver o cabeçalho de lá.
pub mod card_backdrop;
mod event;
mod event_catalog;
pub mod ids;
mod paint;
/// ⭐⭐ A COLUNA DE CATÁLOGOS (wave A3) — irmão por assunto do [`paint`].
mod paint_catalog;
mod paint_catalog_rename;
/// ⭐⭐ A miniatura de um cartão e o memo de textura que ela obriga — irmão por assunto do
/// [`paint`], ver o cabeçalho de lá.
mod paint_thumb;
mod populate;
pub mod state;

/// O id estável do painel — o mesmo `Panel::ID`, exposto como `const` para quem só quer perguntar
/// se ele está aberto sem importar o trait (é o idioma do `ph2d_panel_model3d::PANEL_ID`).
pub const PANEL_ID: &str = "asset_browser";

pub use paint::{default_rect, probe_query};
pub use state::{
    AssetBrowserState, CELL_DEFAULT_PX, CELL_MAX_PX, CELL_MIN_PX, CatalogPick, CatalogRename,
    catalog_row_pick, last_content_h, last_visible_h, payload_at, probe_catalogs,
    probe_first_texture, probe_index_summary, set_current_catalogs, set_current_index,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Marcador de tamanho zero que implementa o contrato tipado do painel.
pub struct AssetBrowserPanel;

impl Panel for AssetBrowserPanel {
    type State = AssetBrowserState;

    const ID: &'static str = PANEL_ID;
    const NODE_ID: NodeId = ph2d_editor_core::ids::ASSET_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut AssetBrowserState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut AssetBrowserState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
