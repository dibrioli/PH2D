//! Os gestos: clicar numa linha pede ir até ela.

use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, Panel};
use ph2d_editor_core::widget::ButtonState;

use crate::canal::{self, HistoryRequest};
use crate::{PixelHistoryPanel, ids};

pub(crate) fn populate(store: &mut WidgetStore) {
    store.register(
        ids::DRAG,
        InteractiveState::BlenderHit {
            parent: PixelHistoryPanel::NODE_ID,
            kind: BlenderHitKind::DragHandle,
        },
    );
}

/// As linhas: a história decide quantas são.
pub(crate) fn regista_linhas(store: &mut WidgetStore, n: usize) {
    for i in 0..n {
        store.register(
            ids::row(i),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

pub(crate) fn apply_event(ev: WidgetEvent) -> EventOutcome {
    let WidgetEvent::Click(id) = ev else {
        return EventOutcome::Ignored;
    };
    let n = canal::view().rows.len();
    match (0..n).find(|&i| ids::row(i) == id) {
        Some(i) => {
            canal::request(HistoryRequest::GoTo(i));
            EventOutcome::Consumed
        }
        None => EventOutcome::Ignored,
    }
}
