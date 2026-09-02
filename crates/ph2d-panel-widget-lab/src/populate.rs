//! Registo dos alvos do laboratório.
//!
//! ⚠️ **A caixa viva é um `Slider` a sério.** Não é uma imitação: registá-la como
//! [`InteractiveState::Slider`] põe-na no MESMO despacho de ponteiro que conduz todos os sliders
//! do app, e é isso que faz o gesto medido na bancada ser o gesto do produto. *Um laboratório com
//! arrasto próprio mediria o arrasto do laboratório.*

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState};

pub(crate) fn populate(store: &mut WidgetStore) {
    store.register(
        ids::LAB_DRAG_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::LAB_PANEL,
            kind: BlenderHitKind::DragHandle,
        },
    );
    store.register(
        ids::LAB_RESIZE_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::LAB_PANEL,
            kind: BlenderHitKind::ResizeHandle,
        },
    );
    store.register(
        ids::LAB_RESIZE_HANDLE_BL,
        InteractiveState::BlenderHit {
            parent: ids::LAB_PANEL,
            kind: BlenderHitKind::ResizeHandleBl,
        },
    );
    for id in [
        ids::LAB_CLOSE,
        ids::LAB_VARIANT_NEXT,
        ids::LAB_VARIANT_PREV,
        ids::LAB_ACCENT_CYCLE,
        ids::LAB_DENSITY_CYCLE,
        ids::LAB_DECORATOR_TOGGLE,
        ids::LAB_COMPARE_TOGGLE,
        ids::LAB_RADIUS_CYCLE,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    store.register(
        ids::LAB_LIVE_BOX,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: crate::study::SAMPLE_T,
            orientation: SliderOrientation::Horizontal,
        },
    );
}
