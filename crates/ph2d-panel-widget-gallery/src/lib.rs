//! `ph2d-panel-widget-gallery` — ADR-0029 Phase C.3 typed panel crate.
//!
//! Owns the Widget Gallery panel's:
//! - retained state ([`WidgetGalleryState`]) — held by `ErasedPanel`
//!   in the runtime `panel::PANEL_REGISTRY`. Just the persisted
//!   `Option<Rect>` post-Phase-C; visibility lives on the host's
//!   `panel_visibility` map.
//! - paint thunk ([`paint`]), event router ([`event`]),
//!   widget registration ([`populate`]).
//! - typed contract via [`WidgetGalleryPanel`] implementing
//!   [`ph2d_editor_core::panel::Panel`].
//!
//! Design intent: peripheral agents run `./play.command`, click the
//! palette pill in the TopBar (`ids::TOPBAR_WIDGET_GALLERY`), and see
//! every canonical widget rendered the way the editor expects so they
//! can replicate the decoration in their own modules. Single in-app
//! source of UI truth. Reuses the 10-section showcase painters owned
//! by `ph2d-editor-core::widget::showcase`.

#![forbid(unsafe_code)]

mod cards;
mod cards_vocab;
mod event;
pub mod ids;
mod paint;
mod populate;
pub mod state;
mod valores;

pub use cards::{cards_on, force_cards_on_this_thread};

pub use paint::default_rect;
pub use state::WidgetGalleryState;

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal, TextKey};

/// Zero-size marker implementing the typed Widget Gallery panel
/// contract.
pub struct WidgetGalleryPanel;

impl Panel for WidgetGalleryPanel {
    type State = WidgetGalleryState;

    const ID: &'static str = "widget_gallery";
    const NODE_ID: NodeId = ph2d_editor_core::ids::GAL_PANEL;
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: TextKey = TextKey::new("panel.widget_gallery.title");
    const ICON: ph2d_editor_core::icons::IconId = ph2d_editor_core::icons::IconId::Grid;
    /// ⚠️ **Um painel de COLUNA não cabe na faixa de baixo.** Ela tem 240 px de altura e a
    /// largura da área: uma lista de propriedades ali fica com duas linhas visíveis. ⇒ as duas
    /// colunas, e o gesto que o levaria ao fundo não é oferecido (decisão D1).
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::SIDES;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::RightTop;
    /// ⭐ **Mora no encaixe da direita, em ABA com o Inspector** (dono 03/11, doc `CartoesUI/08`
    /// §1: *«abrir no painel lateral como um inspector»*). Um `CAN_FLOAT` não ocupa encaixe
    /// (`slot_tabs::occupants` exclui-o) e a lista de cartões pintaria POR CIMA do Inspector no
    /// mesmo `ctx.slot`; como inquilino, abrir a galeria acende a aba dela (a ordem z reconciliada).
    /// ⚠️ Com `PH2D_CARDS=0` a janela flutuante de hoje continua a pintar-se no rect próprio — a
    /// aba fica no encaixe, mas o corpo é o de sempre.
    const CAN_FLOAT: bool = false;

    fn paint(state: &mut WidgetGalleryState, ctx: &mut PaintCtx) {
        paint::paint(state, ctx);
    }

    fn apply_event(
        state: &mut WidgetGalleryState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        event::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        populate::populate(store);
    }
}
