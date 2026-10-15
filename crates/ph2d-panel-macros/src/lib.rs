//! **`ph2d-panel-macros`** — o painel MACROS (ADR-0186, W5d; `docs/Macros/02_plano_W5.md` §5),
//! `Panel<State>` tipado (ADR-0029).
//!
//! Gravar ▸ Parar nas duas famílias (o Painter em Wet Paint, o objecto Motion em edição), a lista
//! das ETAPAS da macro do objecto escolhido — cada gesto nomeado —, as propriedades da escolhida
//! (renomear, a espera antes dela, a cor e o tamanho de uma pincelada, o valor de um param) e o
//! erro que a macro tem. Clicar numa etapa leva a cabeça de leitura ao fim dela.
//!
//! O painel não vê a `ph2d-macro` nem as famílias: o instantâneo chega pelo vocabulário da
//! `ph2d_editor_core::macro_edits`, publicado pela shell ([`set_current_macros`]), e cada gesto sai
//! como [`EditorAction::Macro`](ph2d_editor_core::action_bus::EditorAction::Macro).

#![forbid(unsafe_code)]

mod corpo;
pub mod ids;
pub mod nomes;
mod paint;
mod seam;
pub mod state;

pub use state::{MacrosPanelState, current_macros, last_content_h, set_current_macros};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal, TextKey};

/// ⚠️ O nome é load-bearing: o `ph2d-panel-sync` extrai `pub struct <Nome>Panel` deste ficheiro.
pub struct MacrosPanel;

impl Panel for MacrosPanel {
    type State = MacrosPanelState;

    const ID: &'static str = "macros";
    const NODE_ID: NodeId = ph2d_editor_core::ids::MACROS_PANEL;
    /// Nasce FECHADO: a porta é o *Window → Macros*.
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: TextKey = TextKey::new("panel.macros.title");
    const ICON: ph2d_editor_core::icons::IconId = ph2d_editor_core::icons::IconId::Macro;
    /// Uma lista de etapas não cabe na faixa de baixo: as duas colunas.
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::SIDES;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::RightTop;

    fn paint(state: &mut MacrosPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut MacrosPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        seam::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        seam::populate(store);
    }
}
