//! Widget Gallery panel populate — incondicional, pelas TABELAS que o paint percorre.
//!
//! - A janela flutuante (`PH2D_CARDS=0`): as pegas de arrasto/redimensionar e o fecho (X).
//! - Os cartões (doc `CartoesUI/08` §2): campo = `NumberInput` com o valor de fábrica, segmento,
//!   ícone, opção e caixa = `Button`, os textos = `TextInput`. Registar só o que se pinta faria o
//!   controlo nascer morto sob o dedo.

use crate::controlos::controls;
use crate::ids as g;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, TextInputState};

pub(crate) fn populate(store: &mut WidgetStore) {
    for (id, kind) in [
        (ids::GAL_DRAG_HANDLE, BlenderHitKind::DragHandle),
        (ids::GAL_RESIZE_HANDLE, BlenderHitKind::ResizeHandle),
        (ids::GAL_RESIZE_HANDLE_BL, BlenderHitKind::ResizeHandleBl),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::GAL_PANEL,
                kind,
            },
        );
    }
    button(store, ids::GAL_CLOSE);
    for c in controls() {
        button(store, c.id);
    }
    for (id, factory, _, _) in g::FIELDS {
        number(store, id, factory);
    }
    for (id, key) in g::TEXTS {
        let text = ph2d_i18n::tr(key).to_string();
        store.register(
            id,
            InteractiveState::TextInput {
                state: TextInputState::Normal,
                caret: text.len(),
                text,
                selection_anchor: None,
            },
        );
    }
}

fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

/// Um campo de cartão com o valor de fábrica.
pub(crate) fn number(store: &mut WidgetStore, id: ph2d_a11y::NodeId, value: f64) {
    store.register(
        id,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value,
            buffer: String::new(),
            caret: 0,
            last_committed: value,
            selection_anchor: None,
        },
    );
    store.set_number_value(id, value);
}
