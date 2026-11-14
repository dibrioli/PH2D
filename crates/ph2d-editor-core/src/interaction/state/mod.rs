//! [`WidgetStore`] — retained per-widget interactive state.
//!
//! Pre-populated when a screen is constructed (typically inside
//! `Screen::new`); during the hot path callers only read or
//! mutate-in-place via [`WidgetStore::get_mut`]. Inserts only happen
//! at construction time via [`WidgetStore::register`] — see ADR-0024
//! §"Plano de conformidade HR-3".
//!
//! `NodeId` is the AccessKit-canonical identity (re-exported from
//! `ph2d_a11y`) so the store and the `accesskit::TreeInfo` share keys
//! without translation.
//!
//! `BTreeMap`, never `HashMap` (HR-5/ADR-0022): inserts only happen at construction
//! ([`WidgetStore::register`]); the hot path's `get`/`get_mut` is allocation-free.

mod blender_ops;
mod estados_dos_widgets;
mod chrome_ops;
mod collapse_ops;
mod input_map_ops;
mod joint_editor_ops;
pub use input_map_ops::capture_if_listening;
/// **O que um estado de widget É** (o `InteractiveState` + a paleta nomeada).
mod kinds;
mod slider_curve;
pub use kinds::{InteractiveState, NamedPalette};

mod accessory_width_ops;
pub use accessory_width_ops::AccessoryWidths;
pub use key_card_ops::{BezAlvo, BezArea};
mod asset_drag_ops;
mod dock_width_ops;
mod floating_card_ops;
mod graph_ops;
pub use graph_ops::WordFinder;
mod hier_rows_ops;
pub use hier_rows_ops::InspectorReveal;
mod notes_ops;
pub use notes_ops::NoteDrag;
mod faixa_ops;
mod menu_chain_ops;
pub mod window_frame;
pub use faixa_ops::{CardDirectory, CardFieldSeen, FaixaState, RecentRow, TeachChip};
mod number_scrub;
mod panel_ops;
pub use menu_chain_ops::{MenuAim, MenuChain};
mod picker_pin_ops;
mod scroll_state;
mod section_prefs_ops;
pub use section_prefs_ops::{
    SECCOES_FIXAS, SectionDrag, alvo_da_queda, ordena_seccoes, reordena_seccoes,
    restore_section_prefs_text, section_prefs_text,
};
mod slot_ops;
pub use slider_curve::{fracao_para_pista, pista_para_fracao};
pub use slot_ops::{TAB_DRAG_THRESHOLD_PX, TabDragAnchor};
mod card_ensaio_ops;
mod card_field_ops;
mod card_guest_ops;
mod card_list_ops;
mod card_loose_ops;
mod key_card_ops;
mod store_census;
mod store_core;
mod store_fields;
pub mod store_hierarchy;
mod store_numbers;
mod timeline_ops;
mod tool_bars_ops;
mod widget_accessors;
pub use card_ensaio_ops::{Ensaio, EstadosPresos, Trocados};
pub use card_list_ops::{CardDrag, CardDrop, CardList};
pub use card_loose_ops::{
    LooseCard, LooseEdge, loose_edge_of, loose_face_of, loose_ghost_of, loose_instance_of,
    loose_pin_of, loose_rect_of, loose_return_of,
};
pub use store_fields::WidgetStore;

use ph2d_a11y::NodeId;
use std::collections::BTreeMap;

use super::ContextMenuRequest;
use super::drag::{
    HierarchyDragState, NumberInputDragState, NumberStepperHoldState, ScrollbarDragAnchor,
};
use super::types::{GraphGesture, GraphKey, GraphZoom, NoteData, TimelineGesture, TimelineWheel};
use super::util::format_number;

use crate::widget::{
    ButtonState, CheckboxState, DropdownState, TagState, TextInputState, ToggleState,
};
use crate::zones::Rect;

#[cfg(test)]
mod tests;
