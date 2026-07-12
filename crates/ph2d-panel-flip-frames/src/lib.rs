//! `ph2d-panel-flip-frames` — a **tira de frames** do Flip (ADR-0114 W3).
//!
//! A faixa inferior que transforma o Flip de app de desenho em app de ANIMAÇÃO:
//!
//! - **Células** da camada ativa, cada uma com a sua **exposição** (à TVPaint: a
//!   largura da célula é o tempo que ela fica na tela), a chave atual destacada;
//! - **Transporte**: play/pause + `◀`/`▶` que pulam por **DESENHO** (não por
//!   quadro — num hold de 12, o "anterior" é o desenho anterior; é o *flip* do
//!   animador, o inner loop da profissão);
//! - **Ghost Frames**: liga/desliga + quantos antes/depois;
//! - **Autokey** + **Additive**: o que nasce quando se desenha depois do hold;
//! - **Key ops**: Add (em branco) / Duplicate (cópia) / Delete + a **exposição**
//!   da chave selecionada + mover ±1 quadro;
//! - **Tween**: quantos inbetweens gerar entre a chave atual e a seguinte;
//! - **Cycle**: o pre/post behavior da camada (None/Hold/Loop/Ping-Pong).
//!
//! **Escopo deliberado:** UMA camada por vez (a ativa). A visão multi-camada
//! alinhada é um dope-sheet — e esse é o papel da **timeline global** (W6, cuja
//! integração está adiada até ela ficar pronta). Duplicar o dope-sheet aqui seria
//! construir a mesma coisa duas vezes.
//!
//! O painel é **stateless**: o shell publica [`FlipStripSnapshot`] por frame e
//! recebe as edições por `ToolPanelEvent` (mesmo contrato do `ph2d-panel-flip`).

#![forbid(unsafe_code)]

mod event;
pub mod ids;
mod paint;
mod paint_cells;
mod paint_toolbar;
pub mod populate;
pub mod state;

pub use state::{FlipCell, FlipStripSnapshot, current_flip_strip, set_current_flip_strip};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Marcador de tamanho zero que implementa o contrato do painel da tira.
pub struct FlipFramesPanel;

impl Panel for FlipFramesPanel {
    type State = state::FlipStripState;

    const ID: &'static str = "flip_frames";
    const NODE_ID: NodeId = ph2d_editor_core::ids::FLIP_STRIP_PANEL;
    const DEFAULT_VISIBLE: bool = false;

    fn paint(state: &mut state::FlipStripState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut state::FlipStripState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
