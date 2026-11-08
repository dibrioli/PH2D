//! **`ph2d-panel-pixel-keys`**: a janela KEYBOARD SHORTCUTS do Pixel Art (`docs/PixelArt/P12_plano.md`
//! §5, Aseprite O1–O8): a busca por todas as secções, as teclas de cada acção em fichas, `Add` que
//! captura, o conflito que pergunta, `Reset`/`Reset All`, e Import/Export do delta.
//!
//! A lei vive em [`lei`] (sem pintura); a família fala pelo [`canal`].

#![forbid(unsafe_code)]

pub mod canal;
pub mod ids;
pub mod lei;
mod painel;

pub use canal::{Troca, a_capturar, entrega_troca, oferece, pede_abrir, toma_keymap, toma_troca};
pub use ph2d_pixel_keys::user_file::load_user;

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal, TextKey};

/// O estado: a janela (o mapa carregado ao abrir) e o que veio do último Export/Import.
#[derive(Default)]
pub struct PixelKeysPanelState {
    pub(crate) janela: Option<lei::Janela>,
}

/// ⚠️ O nome é load-bearing: o `ph2d-panel-sync` extrai `pub struct <Nome>Panel` deste ficheiro.
pub struct PixelKeysPanel;

impl Panel for PixelKeysPanel {
    type State = PixelKeysPanelState;

    const ID: &'static str = ph2d_editor_core::screens::hero::PANEL_PIXEL_KEYS;
    const NODE_ID: NodeId = ph2d_editor_core::ids::PIXEL_KEYS_PANEL;
    /// Nasce FECHADA: abre pelo menu Edit ▸ Keyboard Shortcuts ou `Ctrl+Alt+Shift+K`.
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: TextKey = TextKey::new("panel.pixel_keys.title");
    const ICON: ph2d_editor_core::icons::IconId = ph2d_editor_core::icons::IconId::PixelJumble;
    const ALLOWED_SLOTS: ph2d_editor_core::screens::slot::SlotSet =
        ph2d_editor_core::screens::slot::SlotSet::SIDES;
    const DEFAULT_SLOT: ph2d_editor_core::screens::slot::Slot =
        ph2d_editor_core::screens::slot::Slot::RightTop;
    const CAN_FLOAT: bool = true;

    fn paint(state: &mut PixelKeysPanelState, ctx: &mut PaintCtx) {
        painel::paint(state, ctx);
    }

    fn apply_event(
        state: &mut PixelKeysPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        painel::apply_event(state, host, ev)
    }

    fn populate(store: &mut WidgetStore) {
        painel::populate(store);
    }
}
