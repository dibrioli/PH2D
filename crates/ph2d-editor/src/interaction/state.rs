//! [`WidgetStore`] — retained per-widget interactive state.
//!
//! Pre-populated when a screen is constructed (typically inside
//! `Screen::new`); during the hot path callers only read or
//! mutate-in-place via [`WidgetStore::get_mut`]. Inserts only happen
//! at construction time via [`WidgetStore::register`] — see ADR-0024
//! §"Plano de conformidade HR-3".
//!
//! `NodeId` is the AccessKit-canonical identity (re-exported from
//! `ph2d_a11y`) so the store and the `accesskit::Tree` share keys
//! without translation.
//!
//! Note on `BTreeMap` over `HashMap`: workspace clippy bans
//! `HashMap` everywhere (HR-5/ADR-0022). `BTreeMap` allocates per
//! insert, but inserts only happen at construction time via
//! [`WidgetStore::register`]; the hot path uses `get`/`get_mut` on
//! existing entries, which is allocation-free. Lookup is O(log n)
//! instead of O(1), trivial at editor widget counts (~50).

use ph2d_a11y::NodeId;
use std::collections::BTreeMap;

use crate::widget::{
    ButtonState, CheckboxState, CheckboxValue, ColorPickerMode, ComboboxState, DropdownState,
    ListItemState, SliderOrientation, SliderState, TagState, TextInputState, ToggleState,
};
use crate::zones::Rect;

/// One per-widget state slot. Variants mirror the widget kinds in
/// `crate::widget::*`; mappings to the original widget's state enum
/// are 1:1 so `paint_X` keeps reading the same field names.
#[derive(Clone, Debug, PartialEq)]
pub enum InteractiveState {
    Button {
        state: ButtonState,
    },
    Toggle {
        state: ToggleState,
        on: bool,
    },
    Slider {
        state: SliderState,
        value: f32,
        orientation: SliderOrientation,
    },
    Checkbox {
        state: CheckboxState,
        value: CheckboxValue,
    },
    Radio {
        state: ButtonState,
        selected_index: usize,
    },
    Tag {
        state: TagState,
    },
    Tabs {
        selected: usize,
    },
    Dropdown {
        state: DropdownState,
        open: bool,
        selected_index: Option<usize>,
    },
    Combobox {
        state: ComboboxState,
        open: bool,
        query: String,
    },
    TextInput {
        state: TextInputState,
        text: String,
        caret: usize,
    },
    NumberInput {
        state: TextInputState,
        value: f64,
    },
    ListItem {
        state: ListItemState,
        selected: bool,
    },
    TreeView {
        // Tree expand/select state stays on the TreeView struct — the
        // store only carries hot/active flags. Value-bearing reads go
        // through the widget directly.
        last_focused_index: Option<usize>,
    },
    ColorPicker {
        mode: ColorPickerMode,
        rgba: [u8; 4],
    },
    Modal {
        // Open/closed lives on the host; store only tracks ESC->dismiss intent.
        dismissing: bool,
    },
    /// Generic chrome with a focusable hit rect but no interactive
    /// state to carry between frames (e.g., section headers,
    /// hierarchy header add-button).
    Plain,
}

/// One event emitted by [`super::dispatch`]. No `String`/`Vec`
/// payloads — value-bearing variants carry only the `NodeId`; the
/// caller re-reads from the store. Keeps events `Copy` so arena
/// allocation costs a single pointer bump.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WidgetEvent {
    /// Button / Tag remove / ContextMenu item / Modal cancel|confirm.
    Click(NodeId),
    /// Toggle / Checkbox / Switch — caller reads the new state from
    /// `store.get(id)`.
    Toggled(NodeId),
    /// Slider / NumberInput / ColorPicker channel — caller reads
    /// `store.get(id)` for the new numeric value.
    ValueChanged(NodeId),
    /// TextInput / Combobox query — caller reads `store.text(id)`.
    TextChanged(NodeId),
    Focus(NodeId),
    Blur(NodeId),
    /// Tabs / Dropdown / TreeView — selected index changed.
    SelectionChanged(NodeId),
}

#[derive(Debug, Default)]
pub struct WidgetStore {
    states: BTreeMap<NodeId, InteractiveState>,
    /// Insertion order, used for keyboard Tab traversal.
    focus_order: Vec<NodeId>,
    hot_id: Option<NodeId>,
    active_id: Option<NodeId>,
    focus_id: Option<NodeId>,
    /// Rect of the active widget at the moment of Down. Used by
    /// drag dispatch (Slider) to compute new value from pointer
    /// position relative to the original geometry.
    active_rect: Option<Rect>,
}

impl WidgetStore {
    /// Construct an empty store. The capacity hint pre-sizes the
    /// `focus_order` vec; the BTreeMap grows on demand at register
    /// time. Hot-path operations (`get`/`get_mut`) never allocate.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            states: BTreeMap::new(),
            focus_order: Vec::with_capacity(capacity),
            hot_id: None,
            active_id: None,
            focus_id: None,
            active_rect: None,
        }
    }

    /// Register a widget at construction time. Idempotent — repeat
    /// calls overwrite the state but never grow capacity. Should NOT
    /// be called during the paint/dispatch hot path.
    pub fn register(&mut self, id: NodeId, initial: InteractiveState) {
        if self.states.insert(id, initial).is_none() {
            self.focus_order.push(id);
        }
    }

    pub fn get(&self, id: NodeId) -> Option<&InteractiveState> {
        self.states.get(&id)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut InteractiveState> {
        self.states.get_mut(&id)
    }

    pub fn contains(&self, id: NodeId) -> bool {
        self.states.contains_key(&id)
    }

    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    pub fn hot_id(&self) -> Option<NodeId> {
        self.hot_id
    }

    pub fn set_hot(&mut self, id: Option<NodeId>) {
        self.hot_id = id;
    }

    pub fn active_id(&self) -> Option<NodeId> {
        self.active_id
    }

    pub fn set_active(&mut self, id: Option<NodeId>) {
        self.active_id = id;
    }

    /// Geometry of the active widget at the moment of Down. Used by
    /// drag-handling dispatch (Slider) to compute new value.
    pub fn active_rect(&self) -> Option<Rect> {
        self.active_rect
    }

    pub fn set_active_rect(&mut self, rect: Option<Rect>) {
        self.active_rect = rect;
    }

    pub fn focus_id(&self) -> Option<NodeId> {
        self.focus_id
    }

    pub fn set_focus(&mut self, id: Option<NodeId>) {
        self.focus_id = id;
    }

    /// Iterate registered widgets in registration order. Used by
    /// keyboard Tab traversal (insertion order is the focus order).
    pub fn focus_order(&self) -> &[NodeId] {
        &self.focus_order
    }

    /// Convenience: read button state.
    pub fn button_state(&self, id: NodeId) -> Option<ButtonState> {
        match self.states.get(&id) {
            Some(InteractiveState::Button { state }) => Some(*state),
            _ => None,
        }
    }

    /// Convenience: read toggle on/off + state.
    pub fn toggle(&self, id: NodeId) -> Option<(ToggleState, bool)> {
        match self.states.get(&id) {
            Some(InteractiveState::Toggle { state, on }) => Some((*state, *on)),
            _ => None,
        }
    }

    /// Convenience: read slider value + state.
    pub fn slider(&self, id: NodeId) -> Option<(SliderState, f32)> {
        match self.states.get(&id) {
            Some(InteractiveState::Slider { state, value, .. }) => Some((*state, *value)),
            _ => None,
        }
    }

    /// Convenience: read checkbox value + state.
    pub fn checkbox(&self, id: NodeId) -> Option<(CheckboxState, CheckboxValue)> {
        match self.states.get(&id) {
            Some(InteractiveState::Checkbox { state, value }) => Some((*state, *value)),
            _ => None,
        }
    }

    /// Convenience: read text input contents.
    pub fn text(&self, id: NodeId) -> Option<&str> {
        match self.states.get(&id) {
            Some(InteractiveState::TextInput { text, .. }) => Some(text.as_str()),
            Some(InteractiveState::Combobox { query, .. }) => Some(query.as_str()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_with(states: &[(u64, InteractiveState)]) -> WidgetStore {
        let mut s = WidgetStore::with_capacity(64);
        for (id, st) in states {
            s.register(NodeId(*id), st.clone());
        }
        s
    }

    #[test]
    fn register_grows_focus_order_to_match() {
        let mut store = WidgetStore::with_capacity(16);
        for i in 0..16 {
            store.register(NodeId(i as u64), InteractiveState::Plain);
        }
        assert_eq!(store.focus_order().len(), 16);
        assert_eq!(store.len(), 16);
    }

    #[test]
    fn focus_order_matches_registration_order() {
        let store = store_with(&[
            (1, InteractiveState::Plain),
            (5, InteractiveState::Plain),
            (3, InteractiveState::Plain),
        ]);
        assert_eq!(store.focus_order(), &[NodeId(1), NodeId(5), NodeId(3)]);
    }

    #[test]
    fn re_register_overwrites_without_growing_focus_order() {
        let mut store = WidgetStore::with_capacity(8);
        store.register(NodeId(1), InteractiveState::Plain);
        store.register(
            NodeId(1),
            InteractiveState::Button {
                state: ButtonState::Hovered,
            },
        );
        assert_eq!(store.focus_order().len(), 1);
        assert_eq!(store.button_state(NodeId(1)), Some(ButtonState::Hovered));
    }

    #[test]
    fn convenience_getters_return_none_for_wrong_kind() {
        let store = store_with(&[(1, InteractiveState::Plain)]);
        assert!(store.button_state(NodeId(1)).is_none());
        assert!(store.slider(NodeId(1)).is_none());
    }

    #[test]
    fn hot_active_focus_round_trip() {
        let mut store = WidgetStore::with_capacity(4);
        store.set_hot(Some(NodeId(2)));
        store.set_active(Some(NodeId(3)));
        store.set_focus(Some(NodeId(4)));
        assert_eq!(store.hot_id(), Some(NodeId(2)));
        assert_eq!(store.active_id(), Some(NodeId(3)));
        assert_eq!(store.focus_id(), Some(NodeId(4)));
    }

    #[test]
    fn slider_convenience_round_trip() {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            NodeId(1),
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.42,
                orientation: SliderOrientation::Horizontal,
            },
        );
        let (st, v) = store.slider(NodeId(1)).unwrap();
        assert_eq!(st, SliderState::Normal);
        assert!((v - 0.42).abs() < f32::EPSILON);
    }
}
