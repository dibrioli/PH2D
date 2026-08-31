#![forbid(unsafe_code)]
//! `ph2d-panel-sculpt3d` — **a UI da cena 3D** (ADR-0150, W12).
//!
//! A ferramenta em mãos, os knobs do pincel, o espelho, a resolução da malha, o
//! sombreamento e a lista de peças. Referências: a barra lateral do **SculptGL**
//! (Sculpt / Topology / Scene) e o cabeçalho de escultura do **Blender** (o verbo
//! com Radius+Strength ao lado, a simetria e o dyntopo no painel).
//!
//! ## Por que ele existe
//!
//! O módulo 3D nasceu dirigido por TECLAS, e um ciclo de tecla sobre quatro
//! degraus que eu escolhi responde por outra pessoa a pergunta *"quanto disto?"*
//! — que é dela. Um gate verde prova que o motor faz o que eu disse; ele nunca
//! prova que o artista alcança
//! ([[feedback_ship_the_ui_in_the_same_wave_not_later]]).
//!
//! ## Uma categoria própria
//!
//! Os painéis de hoje são tool-gated (painter, vector, flip) ou docados por
//! seleção (inspector). Este é do mesmo tipo que o de física — **mundo**, não
//! ferramenta —, e é por isso que ele emite [`Sculpt3dIntent`]s para a ponte do
//! shell drenar em vez de encaminhar `ToolPanelEvent`: não há tool a que
//! encaminhar, e inventar uma para o cano existente servir seria uma tool que não
//! é uma tool. É também o que mantém `Tool=12` intacto.
//!
//! ⚠️ **Ele não pinta sem cena.** O `state::current()` devolve `None` até a cena
//! 3D existir, e aí `paint` sai no primeiro `if`: um painel de escultura sem
//! escultura seriam seis seções de controles que não alcançam nada.

pub mod ids;
mod paint;
mod populate;
mod preview;
pub mod rows;
pub mod slots;
pub mod state;
/// **AS ESCOLHAS nomeadas** — os enums que um chip escreve — ver [`state_modes`].
pub mod state_modes;

mod event;

pub use state::{
    RetopoMode, Sculpt3dIntent, Sculpt3dPanelState, Sculpt3dSnapshot, Sculpt3dUi, UiLevel,
    alpha_chip_index, drain_intents, last_content_h, last_visible_h, set_current_sculpt3d,
};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Marcador de tamanho zero que implementa o contrato tipado do painel.
///
/// ⚠️ O nome é load-bearing: o `ph2d-panel-sync` extrai `pub struct <Nome>Panel`
/// deste arquivo e entra em pânico se ele não estiver aqui.
pub struct Sculpt3dPanel;

impl Panel for Sculpt3dPanel {
    type State = Sculpt3dPanelState;

    const ID: &'static str = "sculpt3d";
    const NODE_ID: NodeId = ph2d_editor_core::ids::SCULPT3D_PANEL;
    /// Fechado até ser pedido. Quem o abre é a ponte do shell, no frame em que a
    /// cena 3D nasce — um painel que se abrisse para todo projeto seria chrome a
    /// dispensar em vez de a encontrar, e a cena 3D não existe na maioria deles.
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: &'static str = "Sculpt 3D";
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::RightTop;

    fn paint(state: &mut Sculpt3dPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut Sculpt3dPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
