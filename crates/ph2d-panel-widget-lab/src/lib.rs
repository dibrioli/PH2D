//! `ph2d-panel-widget-lab` — ⭐⭐⭐ **o LABORATÓRIO de widgets**.
//!
//! Pedido do Enio, 2026-09-01, depois de decidir que a caixa única é o alvo:
//!
//! > *"Antes de modificar as coisas seria interessante criar um painel de testes como fizemos com
//! > Widget Gallery. Vamos fazer nossos estudos num painel antes de sair mudando tudo. Precisamos
//! > de um estudo sobre widgets originais, desenhos diferentes, podemos colocar no painel várias
//! > amostras e várias cores e comportamentos para testar."*
//!
//! ⛔⛔ **Ele NÃO é a Widget Gallery, e a diferença é load-bearing.** A galeria é a *fonte única de
//! verdade* do que o editor **é**: agentes periféricos copiam a decoração dela. O laboratório
//! mostra o que o editor **pode vir a ser** — seis desenhos concorrentes de que cinco vão ser
//! deitados fora. *Se as propostas vivessem na galeria, a fonte de verdade passaria a conter
//! candidatos, e a próxima linha copiaria um deles.*
//!
//! # A regra desta crate
//!
//! ⚠️ **Nada aqui é chamado pelo app.** O laboratório desenha as suas próprias amostras com os
//! primitivos (`fill_rounded_rect`, `paint_text`), e **não** reaproveita o `slider_with_chip` —
//! ele é justamente o que está a ser substituído, e herdar a geometria dele importaria a decisão
//! que se quer refazer. Enquanto o desenho não for escolhido, os 162 sítios de chamada do produto
//! ficam **intactos**.
//!
//! O estudo, a medição que o motivou e as recusas: `docs/UI_New_and_Simple/pesquisa/07`.

#![forbid(unsafe_code)]

pub mod design;
mod event;
mod paint;
mod populate;
pub mod state;
mod study;

pub use design::{BoxDesign, BoxState, BoxStyle};
pub use paint::default_rect;
pub use state::WidgetLabState;

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};

/// Marcador de tamanho zero que implementa o contrato tipado do laboratório.
pub struct WidgetLabPanel;

impl Panel for WidgetLabPanel {
    type State = WidgetLabState;

    const ID: &'static str = "widget_lab";
    const NODE_ID: NodeId = ids::LAB_PANEL;
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: &'static str = "Widget Lab";
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::SIDES;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::RightTop;
    /// ⭐ **Flutua** (D1), como a galeria: uma bancada de comparação quer ficar por cima do que se
    /// está a comparar, e não roubar uma coluna ao desenho.
    const CAN_FLOAT: bool = true;

    fn paint(state: &mut WidgetLabState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut WidgetLabState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
