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

/// Which sub-control of a [`InteractiveState::BlenderPicker`] a
/// [`InteractiveState::BlenderHit`] points at.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlenderHitKind {
    Wheel,
    ValueSlider,
    InterpolationLinear,
    InterpolationPerceptual,
    ChannelRgb,
    ChannelHsv,
    /// One of the 4 horizontal channel sliders (R/G/B/A or H/S/V/A).
    /// Index 0..3: 0 = R/H, 1 = G/S, 2 = B/V, 3 = A.
    ChannelSlider(u8),
    /// The hex `#RRGGBBAA` text input field.
    Hex,
    /// One swatch in the active palette. Index into the picker's
    /// store-side palette (see [`WidgetStore::blender_palette`]).
    /// Left-click picks the swatch; right-click removes it.
    PaletteSwatch(u8),
    /// "+ swatch" button at the end of the palette grid; clicking
    /// appends the picker's current value to the palette.
    AddSwatch,
    /// Eyedropper button next to the hex field. Clicking enters
    /// pixel-pick mode (the host samples the next click's color from
    /// the rendered scene).
    Eyedropper,
    /// Drag handle bar at the top of the picker — Down begins a
    /// drag, Move updates the picker offset, Up ends it.
    DragHandle,
}

use crate::widget::{
    ButtonState, ChannelMode, CheckboxState, CheckboxValue, ColorPickerMode, ComboboxState,
    DropdownState, InterpolationMode, ListItemState, SliderOrientation, SliderState, TagState,
    TextInputState, ToggleState,
};
use crate::zones::Rect;
use ph2d_tokens::ColorValue;

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
        caret: usize,
        /// Same semantics as `TextInput::selection_anchor`.
        selection_anchor: Option<usize>,
    },
    TextInput {
        state: TextInputState,
        text: String,
        caret: usize,
        /// `None` = collapsed (no selection); `Some(anchor)` = the
        /// selection covers `[min(anchor, caret), max(anchor, caret)]`.
        /// Set by double-click ("select all") and by Shift+Arrow; any
        /// non-shift cursor motion or text mutation collapses it.
        selection_anchor: Option<usize>,
    },
    NumberInput {
        state: TextInputState,
        value: f64,
        /// Mirror of `value` formatted as a string while the input is
        /// not focused; the user's in-progress edit while it is. Pre-
        /// allocated by the caller via [`InteractiveState::number_input`]
        /// so dispatch never grows the String at construction time.
        buffer: String,
        caret: usize,
        /// Snapshot of `value` taken when focus arrives — restored on
        /// Escape or on Blur with an unparsable buffer.
        last_committed: f64,
        /// Same semantics as `TextInput::selection_anchor`.
        selection_anchor: Option<usize>,
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
    /// `BlenderColorPicker` retained state. Painted by
    /// `paint_blender_color_picker_with_store`; mutated by clicks
    /// on registered wheel/value/swatch/segmented sub-rect ids.
    BlenderPicker {
        value: ColorValue,
        channel_mode: ChannelMode,
        interpolation: InterpolationMode,
        active_palette: usize,
        /// Retained HSV hue (0..1). Used by the SV-rect/hue-strip
        /// painters when `value.rgba` collapses to gray (S=0) or
        /// black (V=0) and would otherwise lose the user's chosen
        /// hue. Updated whenever a pick path knows the canonical H.
        ///
        /// **Don't read hue from `rgba_to_hsv(value.rgba)` directly**
        /// in painters / dispatchers — for dark or gray colors it
        /// returns 0 (red) and the SV cursor / hue thumb teleport.
        /// See `docs/UI_Bugs/README.md` §4.1.
        hsv_h: f32,
        /// Retained HSV saturation (0..1). Same role as `hsv_h` —
        /// preserved across V→0 transitions where round-tripping
        /// through RGBA loses the value.
        hsv_s: f32,
    },
    /// Sub-control hit shim: pointing at a sub-rect of a parent
    /// BlenderPicker. The dispatcher uses `kind` to route the click
    /// into the correct widget-side mutation.
    BlenderHit {
        parent: NodeId,
        kind: BlenderHitKind,
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
    /// Eyedropper pick request — emitted when the user clicks
    /// anywhere outside the eyedropper button while eyedropper mode
    /// is pending. The host should sample the rendered pixel at
    /// `(px, py)` (physical pixels) and apply it to the picker at
    /// `parent` via `store.set_blender_value`. Pixel coords are
    /// `u32` so the event keeps `Copy + Eq` (no f32 fields).
    EyedropperPick {
        parent: NodeId,
        px: u32,
        py: u32,
    },
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
    /// Slider id ↔ NumberInput id pairs that should mirror each
    /// other's value. When the slider's value changes via drag, the
    /// number input's `value` (and `buffer`, when not focused) is
    /// updated; when the number input's buffer commits via Enter or
    /// Blur, the slider's value is updated. Pre-populated by the
    /// hosting screen at construction time.
    slider_to_number: BTreeMap<NodeId, NodeId>,
    number_to_slider: BTreeMap<NodeId, NodeId>,
    /// Hex `TextInput` id → its parent `BlenderPicker` id, so the
    /// dispatch can parse the typed buffer on Enter / blur and apply
    /// the resulting color to the parent state.
    hex_to_blender_parent: BTreeMap<NodeId, NodeId>,
    /// Channel `NumberInput` chip id → (parent `BlenderPicker`,
    /// channel index 0..=3). Lets dispatch rewrite the parent's
    /// color value when the user commits a new channel value.
    blender_channel_chip: BTreeMap<NodeId, (NodeId, u8)>,
    /// Most recent pointer-Down event, used for double-click
    /// detection. Stores the hit `NodeId` (or `None` if the click
    /// missed every widget) and the event timestamp.
    last_down_id: Option<NodeId>,
    last_down_at_ns: u128,
    /// Mutable color palettes per BlenderPicker — one Vec of swatches
    /// per parent picker id. Initialized at populate time; mutated by
    /// "+ swatch" / right-click-delete dispatch paths.
    blender_palettes: BTreeMap<NodeId, Vec<ColorValue>>,
    /// Per-picker drag offset (dx, dy) applied to the rect chosen by
    /// the host painter. Mutated by drag-handle clicks; defaults to
    /// (0, 0). When the drag handle is `active`, `drag_anchor_px`
    /// stores the (cursor.x − rect.x, cursor.y − rect.y) at Down so
    /// Move events can keep the picker stuck to the cursor.
    blender_picker_offset: BTreeMap<NodeId, (f32, f32)>,
    /// In-progress picker drag: (parent_id, cursor_x_at_down,
    /// cursor_y_at_down, offset_x_at_down, offset_y_at_down). Move
    /// events compute `new_offset = offset_at_down + (cursor − down_cursor)`.
    /// Cleared on pointer Up.
    blender_drag_anchor: Option<(NodeId, f32, f32, f32, f32)>,
    /// Eyedropper pending: when Some(parent), the next pointer Down
    /// (anywhere except on the eyedropper button itself) is intercepted
    /// by the dispatch and emitted as `WidgetEvent::EyedropperPick`,
    /// signaling the host to readback the pixel under the cursor.
    eyedropper_pending: Option<NodeId>,
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
            slider_to_number: BTreeMap::new(),
            number_to_slider: BTreeMap::new(),
            hex_to_blender_parent: BTreeMap::new(),
            blender_channel_chip: BTreeMap::new(),
            last_down_id: None,
            last_down_at_ns: 0,
            blender_palettes: BTreeMap::new(),
            blender_picker_offset: BTreeMap::new(),
            blender_drag_anchor: None,
            eyedropper_pending: None,
        }
    }

    /// Register a bidirectional link: when `slider`'s value changes,
    /// `number`'s value follows; when `number` commits a new value,
    /// `slider` follows. Caller is responsible for both ids being
    /// pre-registered as Slider and NumberInput respectively.
    pub fn link_slider_number(&mut self, slider: NodeId, number: NodeId) {
        self.slider_to_number.insert(slider, number);
        self.number_to_slider.insert(number, slider);
    }

    pub fn linked_number(&self, slider: NodeId) -> Option<NodeId> {
        self.slider_to_number.get(&slider).copied()
    }

    pub fn linked_slider(&self, number: NodeId) -> Option<NodeId> {
        self.number_to_slider.get(&number).copied()
    }

    /// Tag a hex `TextInput` widget as belonging to a `BlenderPicker`.
    /// Caller is responsible for both ids being pre-registered.
    pub fn link_blender_hex(&mut self, parent: NodeId, hex: NodeId) {
        self.hex_to_blender_parent.insert(hex, parent);
    }

    pub fn blender_hex_parent(&self, hex: NodeId) -> Option<NodeId> {
        self.hex_to_blender_parent.get(&hex).copied()
    }

    /// Tag a channel `NumberInput` chip as belonging to a
    /// `BlenderPicker` at channel index `idx` (0..=3). On commit,
    /// dispatch reads `idx` to know which RGBA / HSVA dimension to
    /// rewrite.
    pub fn link_blender_channel(&mut self, parent: NodeId, chip: NodeId, idx: u8) {
        self.blender_channel_chip.insert(chip, (parent, idx));
    }

    pub fn blender_channel_chip(&self, chip: NodeId) -> Option<(NodeId, u8)> {
        self.blender_channel_chip.get(&chip).copied()
    }

    /// Record the latest pointer-Down for double-click detection.
    /// Returns true iff this Down should be treated as a double-click
    /// (same id as the previous Down + within `DOUBLE_CLICK_WINDOW_NS`
    /// of it).
    pub fn record_pointer_down(&mut self, id: Option<NodeId>, timestamp_ns: u128) -> bool {
        const DOUBLE_CLICK_WINDOW_NS: u128 = 350_000_000; // 350 ms
        let is_double = id.is_some()
            && id == self.last_down_id
            && timestamp_ns.saturating_sub(self.last_down_at_ns) < DOUBLE_CLICK_WINDOW_NS;
        // Reset the counter on a confirmed double-click so a third
        // rapid click doesn't register as another double.
        self.last_down_id = if is_double { None } else { id };
        self.last_down_at_ns = timestamp_ns;
        is_double
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
            Some(InteractiveState::NumberInput { buffer, .. }) => Some(buffer.as_str()),
            _ => None,
        }
    }

    /// Convenience: read number-input full state (state + value +
    /// editing buffer + caret + selection anchor). Returns `None`
    /// for non-number widgets.
    #[allow(clippy::type_complexity)]
    pub fn number_input(
        &self,
        id: NodeId,
    ) -> Option<(TextInputState, f64, &str, usize, Option<usize>)> {
        match self.states.get(&id) {
            Some(InteractiveState::NumberInput {
                state,
                value,
                buffer,
                caret,
                selection_anchor,
                ..
            }) => Some((*state, *value, buffer.as_str(), *caret, *selection_anchor)),
            _ => None,
        }
    }

    /// Read the BlenderPicker state at `id`. Returns `None` for
    /// non-picker widgets.
    pub fn blender_picker(
        &self,
        id: NodeId,
    ) -> Option<(ColorValue, ChannelMode, InterpolationMode, usize)> {
        match self.states.get(&id) {
            Some(InteractiveState::BlenderPicker {
                value,
                channel_mode,
                interpolation,
                active_palette,
                ..
            }) => Some((*value, *channel_mode, *interpolation, *active_palette)),
            _ => None,
        }
    }

    /// Initialize the BlenderPicker's palette swatches. Caller passes
    /// the seed colors (typically `default_palette()`).
    pub fn init_blender_palette(&mut self, parent: NodeId, swatches: Vec<ColorValue>) {
        self.blender_palettes.insert(parent, swatches);
    }

    /// Read the BlenderPicker's current palette swatches. Returns
    /// `None` if `init_blender_palette` was never called for `parent`.
    pub fn blender_palette(&self, parent: NodeId) -> Option<&[ColorValue]> {
        self.blender_palettes.get(&parent).map(|v| v.as_slice())
    }

    /// Read the BlenderPicker's drag offset (dx, dy). Defaults to
    /// (0, 0) if no drag has happened yet.
    pub fn blender_picker_offset(&self, parent: NodeId) -> (f32, f32) {
        self.blender_picker_offset
            .get(&parent)
            .copied()
            .unwrap_or((0.0, 0.0))
    }

    pub fn set_blender_picker_offset(&mut self, parent: NodeId, dx: f32, dy: f32) {
        self.blender_picker_offset.insert(parent, (dx, dy));
    }

    /// Begin a picker drag at cursor `(px, py)`. Snapshots the
    /// current offset so Move events can compute new_offset =
    /// offset_at_down + (cursor − down_cursor).
    pub fn begin_blender_drag(&mut self, parent: NodeId, cursor_x: f32, cursor_y: f32) {
        let (off_x, off_y) = self.blender_picker_offset(parent);
        self.blender_drag_anchor = Some((parent, cursor_x, cursor_y, off_x, off_y));
    }

    pub fn blender_drag_anchor(&self) -> Option<(NodeId, f32, f32, f32, f32)> {
        self.blender_drag_anchor
    }

    pub fn end_blender_drag(&mut self) {
        self.blender_drag_anchor = None;
    }

    pub fn eyedropper_pending(&self) -> Option<NodeId> {
        self.eyedropper_pending
    }

    pub fn set_eyedropper_pending(&mut self, parent: Option<NodeId>) {
        self.eyedropper_pending = parent;
    }

    /// Append `color` to the BlenderPicker's palette. No-op if the
    /// palette wasn't initialized OR is already at the static cap
    /// (24 entries — matches the pre-registered swatch hit slots so
    /// every visible swatch has a clickable hit rect).
    pub fn blender_palette_push(&mut self, parent: NodeId, color: ColorValue) {
        const PALETTE_CAP: usize = 27;
        if let Some(palette) = self.blender_palettes.get_mut(&parent)
            && palette.len() < PALETTE_CAP
        {
            palette.push(color);
        }
    }

    /// Remove the swatch at `idx` from the BlenderPicker's palette.
    /// Returns true if a swatch was actually removed.
    pub fn blender_palette_remove(&mut self, parent: NodeId, idx: usize) -> bool {
        if let Some(palette) = self.blender_palettes.get_mut(&parent)
            && idx < palette.len()
        {
            palette.remove(idx);
            return true;
        }
        false
    }

    /// Read the retained HSV anchor (h, s) the picker uses to
    /// preserve hue + saturation across V→0 transitions where the
    /// RGBA representation would otherwise lose them. Both in 0..1.
    pub fn blender_hsv_anchor(&self, id: NodeId) -> Option<(f32, f32)> {
        match self.states.get(&id) {
            Some(InteractiveState::BlenderPicker { hsv_h, hsv_s, .. }) => Some((*hsv_h, *hsv_s)),
            _ => None,
        }
    }

    /// Mutate the BlenderPicker's value. Auto-updates the retained
    /// (h, s) anchor when the new color is chromatic (S>0, V>0); for
    /// gray/black inputs the anchor is preserved so the user's chosen
    /// hue doesn't reset to red on a V=0 click.
    pub fn set_blender_value(&mut self, id: NodeId, new_value: ColorValue) {
        if let Some(InteractiveState::BlenderPicker {
            value,
            hsv_h,
            hsv_s,
            ..
        }) = self.states.get_mut(&id)
        {
            *value = new_value;
            let (h, s, v, _) = crate::widget::rgba_to_hsv(new_value.rgba);
            if s > 1e-3 && v > 1e-3 {
                *hsv_h = h;
                *hsv_s = s;
            }
        }
    }

    /// Mutate the BlenderPicker's value AND override the retained
    /// (h, s) anchor explicitly. Used by the SV-rect / hue-strip
    /// dispatchers, which know the canonical H or S even when the
    /// resulting RGBA collapses (e.g. picking V=0 → all-zero RGBA).
    pub fn set_blender_value_with_hsv(
        &mut self,
        id: NodeId,
        new_value: ColorValue,
        h: f32,
        s: f32,
    ) {
        if let Some(InteractiveState::BlenderPicker {
            value,
            hsv_h,
            hsv_s,
            ..
        }) = self.states.get_mut(&id)
        {
            *value = new_value;
            // Clamp instead of `rem_euclid`: the user-picked H from
            // a hue-strip click may equal 1.0 at the right edge; we
            // want the thumb to stay at the right rather than
            // wrapping to 0.0 (left edge).
            *hsv_h = h.clamp(0.0, 1.0);
            *hsv_s = s.clamp(0.0, 1.0);
        }
    }

    /// Mutate the BlenderPicker's channel mode (RGB↔HSV).
    pub fn set_blender_channel_mode(&mut self, id: NodeId, mode: ChannelMode) {
        if let Some(InteractiveState::BlenderPicker { channel_mode, .. }) = self.states.get_mut(&id)
        {
            *channel_mode = mode;
        }
    }

    /// Mutate the BlenderPicker's interpolation (Linear↔Perceptual).
    pub fn set_blender_interpolation(&mut self, id: NodeId, mode: InterpolationMode) {
        if let Some(InteractiveState::BlenderPicker { interpolation, .. }) =
            self.states.get_mut(&id)
        {
            *interpolation = mode;
        }
    }

    /// Mutate a single channel of the BlenderPicker's RGBA value.
    /// `channel_idx` 0..=3 = R/G/B/A (or H/S/V/A in HSV mode — caller
    /// is responsible for converting before calling). `norm` must be in
    /// [0.0, 1.0].
    pub fn set_blender_channel(&mut self, id: NodeId, channel_idx: u8, norm: f32) {
        if let Some(InteractiveState::BlenderPicker {
            value,
            channel_mode,
            hsv_h,
            hsv_s,
            ..
        }) = self.states.get_mut(&id)
        {
            let byte = (norm.clamp(0.0, 1.0) * 255.0).round() as u8;
            match *channel_mode {
                ChannelMode::Rgb => {
                    if let Some(slot) = value.rgba.get_mut(channel_idx as usize) {
                        *slot = byte;
                    }
                    let [r, g, b, a] = value.rgba;
                    *value = ColorValue::from_rgba8(r, g, b, a);
                    // Refresh retained anchor when the new RGB is
                    // chromatic (else keep what we had so the H chip
                    // doesn't spuriously reset on RGB-mode edits).
                    let (h, s, v, _) = crate::widget::rgba_to_hsv(value.rgba);
                    if s > 1e-3 && v > 1e-3 {
                        *hsv_h = h;
                        *hsv_s = s;
                    }
                }
                ChannelMode::Hsv => {
                    // Use retained (h, s) as the canonical HSV basis
                    // — see `apply_blender_channel_value` for the
                    // why. V + A from RGBA are recoverable.
                    let (_, _, v_rgba, a_rgba) = crate::widget::rgba_to_hsv(value.rgba);
                    let mut h = *hsv_h;
                    let mut s = *hsv_s;
                    let mut v = v_rgba;
                    let mut a = a_rgba;
                    match channel_idx {
                        0 => h = norm.clamp(0.0, 1.0),
                        1 => s = norm.clamp(0.0, 1.0),
                        2 => v = norm.clamp(0.0, 1.0),
                        3 => a = norm.clamp(0.0, 1.0),
                        _ => {}
                    }
                    *value = hsv_to_color_value(h, s, v, a);
                    *hsv_h = h;
                    *hsv_s = s;
                }
            }
        }
    }

    /// Read just the current numeric value (committed). Useful for
    /// linked sliders that don't care about the in-progress buffer.
    pub fn number_value(&self, id: NodeId) -> Option<f64> {
        match self.states.get(&id) {
            Some(InteractiveState::NumberInput { value, .. }) => Some(*value),
            _ => None,
        }
    }

    /// Mutate a NumberInput's committed value programmatically (e.g.
    /// from a linked Slider drag). Re-syncs the buffer to the new
    /// formatted value when the input is **not** focused; if it is
    /// focused, the user's edit is preserved.
    pub fn set_number_value(&mut self, id: NodeId, new_value: f64) {
        let focused = self.focus_id == Some(id);
        if let Some(InteractiveState::NumberInput {
            value,
            buffer,
            last_committed,
            ..
        }) = self.states.get_mut(&id)
        {
            *value = new_value;
            *last_committed = new_value;
            if !focused {
                buffer.clear();
                use std::fmt::Write;
                let _ = write!(buffer, "{}", format_number(new_value));
            }
        }
    }
}

/// Convert HSV (all in [0..1]) + alpha to [`ColorValue`].
/// Inverse of [`crate::widget::blender_color_picker::channels::rgba_to_hsv`].
pub fn hsv_to_color_value(h: f32, s: f32, v: f32, a: f32) -> ColorValue {
    let h6 = h * 6.0;
    let i = h6.floor() as u32 % 6;
    let f = h6 - h6.floor();
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    ColorValue::from_rgba8(
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
        (a * 255.0).round() as u8,
    )
}

/// Pretty-print a `f64` for NumberInput buffer initialisation:
/// integers without trailing `.0`, fractions with up to 3 decimals.
/// Mirrors `widget::number_input::format_number` to keep both reps
/// in sync without crossing the module boundary.
pub fn format_number(v: f64) -> String {
    if (v - v.round()).abs() < 1e-6 {
        format!("{}", v as i64)
    } else {
        format!("{v:.3}")
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
