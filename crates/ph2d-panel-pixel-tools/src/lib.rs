//! **`ph2d-panel-pixel-tools`** — o painel PIXEL (ADR-0192, `docs/PixelArt/02_plano_modulo.md`),
//! `Panel<State>` tipado (ADR-0029): a barra de cores e de ferramentas do Aseprite.
//!
//! As cores de frente e de fundo com o hex e o trocar (`X`), e a grelha da PALETA do sprite em
//! edição — clique esquerdo = cor de frente, direito = cor de fundo. Sem Pixel Sprite em Draw o
//! painel diz como entrar. As quatro ferramentas (Pencil `B`, Eraser `E`, Eyedropper `I`, Hand `H`)
//! e as duas amostras moram na coluna da esquerda ([`column`]), como no Painter.
//!
//! O painel não vê a família: o instantâneo chega pelo vocabulário de
//! [`ph2d_editor_core::pixel_panel`], publicado pela shell ([`set_current_pixel`]), e cada gesto sai
//! como [`EditorAction::Pixel`](ph2d_editor_core::action_bus::EditorAction::Pixel).

#![forbid(unsafe_code)]

pub mod column;
pub mod ids;
mod paint;
mod seam;
mod seam_formas;
mod seam_selecao;
mod seam_seletor;
pub mod seletor;
mod state;

pub use paint::hex;
pub use seam_formas::{
    SHAPE_FILL, desc as shape_desc, field_id as shape_field_id, option_id as shape_option_id,
    nome as shape_field_name, set_field_names,
};
pub use state::{PixelToolsPanelState, current_pixel, set_current_pixel};

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal, TextKey};

/// ⚠️ O nome é load-bearing: o `ph2d-panel-sync` extrai `pub struct <Nome>Panel` deste ficheiro.
pub struct PixelToolsPanel;

impl Panel for PixelToolsPanel {
    type State = PixelToolsPanelState;

    const ID: &'static str = ph2d_editor_core::screens::hero::PANEL_PIXEL;
    const NODE_ID: NodeId = ph2d_editor_core::ids::PIXEL_PANEL;
    /// Nasce FECHADO: abre ao entrar em Pixel Sprite ▸ Draw (a família escreve a visibilidade) e
    /// pelo *Window → Pixel*.
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: TextKey = TextKey::new("panel.pixel_tools.title");
    const ICON: ph2d_editor_core::icons::IconId = ph2d_editor_core::icons::IconId::LayoutGrid;
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::SIDES;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::RightTop;

    fn paint(state: &mut PixelToolsPanelState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut PixelToolsPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        seam::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        seam::populate(store);
    }
}
