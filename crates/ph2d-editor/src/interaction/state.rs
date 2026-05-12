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
    /// Bottom-right resize gripper. Down begins a resize; Move
    /// adjusts the parent's stored `(dw, dh)`; Up ends it.
    ResizeHandle,
    /// M14.6A: eye icon on a hierarchy row — toggles the entity's
    /// `Visibility` component. Parent NodeId on the `BlenderHit` is
    /// the row's id; dispatcher sets `HeroScreen.pending_visibility_toggle`
    /// for the host to drain and apply on `SimWorld`.
    VisibilityToggle,
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
    /// Per-panel manual resize delta (dw, dh) applied on top of the
    /// layout's base width/height. Mutated by dragging the bottom-
    /// right resize gripper.
    panel_resize_delta: BTreeMap<NodeId, (f32, f32)>,
    /// In-progress panel resize: (parent_id, last_cursor_x,
    /// last_cursor_y). Move events apply (cursor − last) to the
    /// stored `panel_resize_delta`, then re-anchor.
    panel_resize_anchor: Option<(NodeId, f32, f32)>,
    /// Clipboard outbox — set by Cmd+C/X handlers; shell drains each
    /// frame via `take_clipboard_copy` and writes to the OS
    /// clipboard. `String` rather than a reference so the data lives
    /// independently of any widget buffer that might mutate next.
    pending_clipboard_copy: Option<String>,
    /// Clipboard paste request — set by Cmd+V on a focused text
    /// widget; shell reads the OS clipboard and calls back into
    /// `apply_clipboard_paste` with the text.
    pending_clipboard_paste: Option<NodeId>,
    /// Currently-loaded scene name shown on the TopBar project chip.
    /// Mutated by `ContextMenuKind::SceneList` row clicks.
    current_scene_name: String,
    /// Coordinate-space toggle for the TOOL_SPACE rail button.
    /// `false` = Global, `true` = Local. Flipped on click.
    tool_space_local: bool,
    /// Camera-framing mode for the TOOL_HOME rail button.
    /// Cycle: 0 = Selected, 1 = Camera, 2 = All. Bumped on click.
    tool_view_mode: u8,
    /// Per-panel Z order — last element paints LAST (= topmost).
    /// Mutated by `bump_panel_z` whenever the user clicks inside a
    /// panel, drags it, or it newly opens (color picker). Painters
    /// walk this in order so the most-recently-touched panel sits
    /// on top of any overlapping siblings.
    panel_z_order: Vec<NodeId>,
    /// Eyedropper pending: when Some(parent), the next pointer Down
    /// (anywhere except on the eyedropper button itself) is intercepted
    /// by the dispatch and emitted as `WidgetEvent::EyedropperPick`,
    /// signaling the host to readback the pixel under the cursor.
    eyedropper_pending: Option<NodeId>,
    /// Vertical scroll offset per panel. Wheel events advance the
    /// offset; painters subtract it from content y. Clamped on each
    /// scroll to `[0, content_h - visible_h]` by the painter (which
    /// knows both heights). See `docs/UI_Bugs/README.md` §1
    /// (hit-testing) — content rendered with offset must compensate
    /// in hit-test too.
    panel_scroll: BTreeMap<NodeId, f32>,
    /// Painter-published rect of each scrollable panel — populated
    /// every frame so the wheel dispatch can find which panel sits
    /// under the cursor. Cleared together with `clear_for_frame` on
    /// the hit_index by the host (or hero) at frame start.
    panel_rects: BTreeMap<NodeId, Rect>,
    /// Painter-published total content height per panel (sum of
    /// every section's height + separators). `dispatch_wheel` reads
    /// this to clamp scroll deltas at the upper bound
    /// (`content_h - visible_h`) — without it, wheeling past the
    /// last element produces a one-frame "jump" as the next paint
    /// clamps the over-scroll back.
    panel_content_h: BTreeMap<NodeId, f32>,
    /// Exact visible body height per panel, also painter-published.
    /// Pairs with `panel_content_h` so `dispatch_wheel` can compute
    /// `max_scroll = content_h - visible_h` precisely (no heuristic).
    panel_visible_h: BTreeMap<NodeId, f32>,
    /// Tooltip text per widget id. Read by `paint_hover_tooltip`
    /// when the user hovers over a registered widget. Populated by
    /// `populate` / paint passes via `set_tooltip`. Replaces the old
    /// hardcoded `tooltip_for(id)` match — every widget can now
    /// participate without per-id boilerplate.
    tooltips: BTreeMap<NodeId, String>,
    /// Collapsed/expanded state per id. `true` = collapsed; missing
    /// entry defaults to "expanded" so newly-registered sections
    /// open by default. Toggled by `apply_event` on Click and
    /// consumed by section painters that early-out when collapsed.
    collapsed: BTreeMap<NodeId, bool>,
    /// Pending right-click context menu. `Some` when a Secondary
    /// Down landed somewhere a menu should appear (e.g. an empty
    /// inspector panel or a section header); `None` when no menu
    /// is open. The hero painter consumes this to render a floating
    /// menu over everything; clicking outside the menu or on a menu
    /// item clears the slot.
    context_menu: Option<ContextMenuRequest>,
    /// Section-header id → highlighter color index (0..4 for the 5
    /// canonical colors; missing entry == "no outline"). Painted by
    /// the inspector as a colored stroke around the section block.
    section_outline_color: BTreeMap<NodeId, u8>,
    /// Per-panel list of user-created notes. Each note carries a
    /// background color index into the highlighter palette. New
    /// notes append; right-click → delete removes by index. The
    /// painter walks this list once per panel each frame.
    notes_per_panel: BTreeMap<NodeId, Vec<NoteData>>,
    /// Sticky source of the most recently completed context-menu
    /// request, captured at apply-event time so the inspector can
    /// route the click → side-table mutation. The dispatch clears
    /// `context_menu` on the same Down event that selects an item;
    /// without this snapshot the inspector loses the request
    /// before it can read it.
    last_context_menu: Option<ContextMenuRequest>,
    /// Currently-active color picker target. `Some(id)` means the
    /// floating BlenderColorPicker is open and editing the color
    /// stored at `widget_colors[id]`. `None` hides the picker. Set
    /// by clicks on color targets (section color circles, color
    /// swatches, …) and cleared by any click outside the picker
    /// and outside another color target.
    picker_target: Option<NodeId>,
    /// Per-widget current color. Keyed by the target widget's id
    /// (section color circles, color swatches). The picker writes
    /// here on every frame while editing; painters read here to
    /// display the widget's current color.
    widget_colors: BTreeMap<NodeId, [u8; 4]>,
    /// In-progress scrollbar drag. Captured on Down inside a
    /// scrollbar thumb's hit rect; consumed by Move events to
    /// translate cursor delta into a `panel_scroll` delta; cleared
    /// on Up. `track_h` and `content_h` are snapshotted so the
    /// drag stays linear even if the painter republishes them
    /// mid-drag.
    scrollbar_drag: Option<ScrollbarDragAnchor>,
    /// Editor-wide corner-radius scale. `1.0` = canonical, `0.0` =
    /// sharp / squared, `1.6` = round. Painters that want to follow
    /// the user's preset multiply their `Radius::*.px()` by this.
    /// Centralized so the topbar theme menu drives the look in one
    /// place.
    radius_scale: f32,
    /// Hierarchy row display order. When non-empty, the hierarchy
    /// painter walks this list instead of the fixture's default
    /// order. Mutated by drag-and-drop (`Down + Move > threshold +
    /// Up`) to reorder rows.
    hierarchy_order: Vec<NodeId>,
    /// Parent map for tree-style hierarchy. `child → parent`; absent
    /// entries are roots. Mutated by drop-inside DnD; consumed by the
    /// painter to indent rows by depth.
    hierarchy_parent: BTreeMap<NodeId, NodeId>,
    /// In-progress hierarchy drag. `Some` when a Primary Down landed
    /// on a hierarchy row and the cursor has moved past the drag
    /// threshold; cleared on Up (with reorder applied) or on Up at
    /// the original position (treated as a regular click).
    hierarchy_drag: Option<HierarchyDragState>,
}

/// Internal state of an in-progress hierarchy drag.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HierarchyDragState {
    /// Row being dragged.
    pub dragged: NodeId,
    /// Cursor x/y at Down — used to detect "drag started" via the
    /// distance threshold.
    pub down_x: f32,
    pub down_y: f32,
    /// Latest cursor x/y (updated on Move) so the painter can render
    /// a drop-indicator that matches what the dispatch will resolve
    /// on Up (x-aware to distinguish "inside indented row" from
    /// "sibling at root level").
    pub cursor_x: f32,
    pub cursor_y: f32,
    /// `true` once the cursor has moved past the threshold; until
    /// then the gesture is "maybe-click, maybe-drag".
    pub active: bool,
}

/// State of an in-progress drag on a scrollbar thumb.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ScrollbarDragAnchor {
    /// Panel whose `panel_scroll` the drag updates.
    pub panel: NodeId,
    /// Cursor y at the moment of Down.
    pub cursor_y_at_down: f32,
    /// `panel_scroll(panel)` at the moment of Down.
    pub scroll_at_down: f32,
    /// Track height used to convert cursor delta → scroll delta.
    pub track_h: f32,
    /// Total content height (= `panel_content_h(panel)`).
    pub content_h: f32,
    /// Visible body height (= `panel_visible_h(panel)`).
    pub visible_h: f32,
}

/// State of a user-created sticky note inside a panel.
#[derive(Clone, Debug)]
pub struct NoteData {
    /// Highlighter color index (0..4) into the 5-color palette.
    pub color_idx: u8,
    /// Note title (single line).
    pub title: String,
    /// Note body (multi-line).
    pub body: String,
    /// Inspector-section index this note should appear ABOVE.
    /// `Some(i)` means "paint this note just before
    /// `SECTION_IDS[i]`"; `None` appends at the bottom (the legacy
    /// fallback for notes created via context menu hitting the
    /// empty area below all sections).
    pub before_section: Option<u8>,
}

/// Where + why a right-click opened a context menu. Painted as a
/// floating overlay by `paint_inspector` (or any host); items are
/// hit-registered with the same `NodeId`s the dispatch checks for in
/// the next click cycle.
// `f32` is `PartialEq` but not `Eq`, so the request can only be
// `PartialEq`. That's fine — context menu state never goes into a
// hash set, only Option<...> comparisons.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ContextMenuRequest {
    pub x: f32,
    pub y: f32,
    pub kind: ContextMenuKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ContextMenuKind {
    /// Right-clicked inside a panel. Menu offers "Create note" —
    /// the new note is parented to `panel`. `before_section`, when
    /// `Some(i)`, anchors the new note above `SECTION_IDS[i]`
    /// (computed at right-click time from the cursor y).
    CreateNote {
        panel: NodeId,
        before_section: Option<u8>,
    },
    /// Right-clicked on a section header. Menu offers 5 highlight
    /// outline colors for the section.
    SectionOutline { section: NodeId },
    /// Right-clicked on an existing note. Menu offers 5 highlight
    /// background colors. `panel` is the note's host; `note_index`
    /// is the index into `notes_per_panel[panel]`.
    NoteBackground { panel: NodeId, note_index: u8 },
    /// Clicked the TOPBAR theme cluster. Menu offers the 4 theme
    /// options plus 3 corner-radius scale presets (Sharp / Default
    /// / Round) — the standardized way to switch chrome look.
    ThemeSelector,
    /// Clicked the TOPBAR Save chip. Menu offers Save + Save As.
    SaveMenu,
    /// Clicked the TOPBAR Open chip. Menu offers Open Project +
    /// Import (and more later).
    OpenMenu,
    /// Clicked the TOPBAR Settings (gear) cluster. Menu offers
    /// project-level toggles — currently `pixels_per_meter` presets
    /// (16 / 32 / 100 / 256 / 1024). Selected entry writes
    /// `HeroScreen.project.pixels_per_meter`.
    SettingsMenu,
    /// Clicked the TOPBAR Project chip. Menu offers a search input
    /// plus a filtered list of scene names; selecting a row updates
    /// the chip's label via `WidgetStore::current_scene_name`.
    SceneList,
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
            panel_resize_delta: BTreeMap::new(),
            panel_resize_anchor: None,
            pending_clipboard_copy: None,
            pending_clipboard_paste: None,
            current_scene_name: String::from("Level_01"),
            tool_space_local: false,
            tool_view_mode: 0,
            panel_z_order: Vec::new(),
            eyedropper_pending: None,
            panel_scroll: BTreeMap::new(),
            panel_rects: BTreeMap::new(),
            panel_content_h: BTreeMap::new(),
            panel_visible_h: BTreeMap::new(),
            tooltips: BTreeMap::new(),
            collapsed: BTreeMap::new(),
            context_menu: None,
            section_outline_color: BTreeMap::new(),
            notes_per_panel: BTreeMap::new(),
            last_context_menu: None,
            picker_target: None,
            widget_colors: BTreeMap::new(),
            scrollbar_drag: None,
            radius_scale: 1.0,
            hierarchy_order: Vec::new(),
            hierarchy_parent: BTreeMap::new(),
            hierarchy_drag: None,
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
        // Dragged panel → topmost in z-order.
        self.bump_panel_z(parent);
    }

    pub fn blender_drag_anchor(&self) -> Option<(NodeId, f32, f32, f32, f32)> {
        self.blender_drag_anchor
    }

    /// Update only the cursor coordinates in the drag anchor (used by
    /// the incremental drag model — each move re-anchors so the next
    /// move applies a fresh delta to the post-clamp offset).
    pub fn update_blender_drag_cursor(&mut self, cursor_x: f32, cursor_y: f32) {
        if let Some((parent, _, _, off_x, off_y)) = self.blender_drag_anchor {
            self.blender_drag_anchor = Some((parent, cursor_x, cursor_y, off_x, off_y));
        }
    }

    pub fn end_blender_drag(&mut self) {
        self.blender_drag_anchor = None;
    }

    /// Read the manual-resize delta `(dw, dh)` for a panel. Defaults
    /// to `(0, 0)` if no resize has happened.
    pub fn panel_resize_delta(&self, panel: NodeId) -> (f32, f32) {
        self.panel_resize_delta
            .get(&panel)
            .copied()
            .unwrap_or((0.0, 0.0))
    }

    pub fn set_panel_resize_delta(&mut self, panel: NodeId, dw: f32, dh: f32) {
        self.panel_resize_delta.insert(panel, (dw, dh));
    }

    pub fn begin_panel_resize(&mut self, panel: NodeId, cursor_x: f32, cursor_y: f32) {
        self.panel_resize_anchor = Some((panel, cursor_x, cursor_y));
    }

    pub fn panel_resize_anchor(&self) -> Option<(NodeId, f32, f32)> {
        self.panel_resize_anchor
    }

    pub fn update_panel_resize_cursor(&mut self, cursor_x: f32, cursor_y: f32) {
        if let Some((panel, _, _)) = self.panel_resize_anchor {
            self.panel_resize_anchor = Some((panel, cursor_x, cursor_y));
        }
    }

    pub fn end_panel_resize(&mut self) {
        self.panel_resize_anchor = None;
    }

    /// Drain the pending clipboard-copy text (set by a Cmd+C / Cmd+X
    /// dispatch); shell should call once per frame and write to OS
    /// clipboard when non-None.
    pub fn take_clipboard_copy(&mut self) -> Option<String> {
        self.pending_clipboard_copy.take()
    }

    pub fn set_clipboard_copy(&mut self, text: String) {
        self.pending_clipboard_copy = Some(text);
    }

    /// Drain the pending clipboard-paste request; shell should read
    /// the OS clipboard and call `apply_clipboard_paste` with the
    /// result when non-None.
    pub fn take_clipboard_paste_request(&mut self) -> Option<NodeId> {
        self.pending_clipboard_paste.take()
    }

    pub fn set_clipboard_paste_request(&mut self, id: NodeId) {
        self.pending_clipboard_paste = Some(id);
    }

    pub fn current_scene_name(&self) -> &str {
        &self.current_scene_name
    }

    pub fn set_current_scene_name(&mut self, name: impl Into<String>) {
        self.current_scene_name = name.into();
    }

    pub fn tool_space_local(&self) -> bool {
        self.tool_space_local
    }

    pub fn set_tool_space_local(&mut self, local: bool) {
        self.tool_space_local = local;
    }

    pub fn tool_view_mode(&self) -> u8 {
        self.tool_view_mode
    }

    pub fn set_tool_view_mode(&mut self, mode: u8) {
        self.tool_view_mode = mode % 3;
    }

    /// Move `panel_id` to the end of the z-order (= topmost). If
    /// the id isn't in the list yet, append it. Called by dispatch
    /// whenever a panel is clicked, dragged, or first opened.
    pub fn bump_panel_z(&mut self, panel_id: NodeId) {
        self.panel_z_order.retain(|id| *id != panel_id);
        self.panel_z_order.push(panel_id);
    }

    /// Read the current z-order. Bottom-first iteration (= paint
    /// order); the last element is the topmost panel.
    pub fn panel_z_order(&self) -> &[NodeId] {
        &self.panel_z_order
    }

    pub fn eyedropper_pending(&self) -> Option<NodeId> {
        self.eyedropper_pending
    }

    pub fn set_eyedropper_pending(&mut self, parent: Option<NodeId>) {
        self.eyedropper_pending = parent;
    }

    /// Read the vertical scroll offset for a panel (defaults to 0).
    pub fn panel_scroll(&self, panel: NodeId) -> f32 {
        self.panel_scroll.get(&panel).copied().unwrap_or(0.0)
    }

    /// Set the vertical scroll offset for a panel. Caller is
    /// responsible for clamping to `[0, content_h - visible_h]`.
    pub fn set_panel_scroll(&mut self, panel: NodeId, y: f32) {
        self.panel_scroll.insert(panel, y);
    }

    /// Painter publishes its panel rect each frame so wheel
    /// dispatch can find which panel is under the cursor.
    pub fn set_panel_rect(&mut self, panel: NodeId, rect: Rect) {
        self.panel_rects.insert(panel, rect);
    }

    /// Read the published rect of a panel. Returns `None` when no
    /// painter has registered it this frame.
    pub fn panel_rect(&self, panel: NodeId) -> Option<Rect> {
        self.panel_rects.get(&panel).copied()
    }

    /// Drop the published rect for a panel. Used by transient
    /// panels (e.g. the floating BlenderColorPicker) when they're
    /// not currently visible — so dispatch's "is the click inside
    /// this panel?" tests aren't fooled by a stale rect from a
    /// previous frame.
    pub fn clear_panel_rect(&mut self, panel: NodeId) {
        self.panel_rects.remove(&panel);
    }

    /// Total height of all painted content in a panel (sum of
    /// section heights + separators). Set by the painter each
    /// frame; read by `dispatch_wheel` to clamp at the upper
    /// bound. Missing entry = unknown content height = no upper
    /// clamp (treated as infinite).
    pub fn set_panel_content_h(&mut self, panel: NodeId, content_h: f32) {
        self.panel_content_h.insert(panel, content_h);
    }

    /// Read the total content height for a panel. Returns `None`
    /// when the painter hasn't published one yet (e.g. on the
    /// first frame).
    pub fn panel_content_h(&self, panel: NodeId) -> Option<f32> {
        self.panel_content_h.get(&panel).copied()
    }

    /// Set the exact visible body height for a panel. Painters
    /// publish this each frame; `dispatch_wheel` uses it (instead
    /// of a `panel.h - 60` heuristic) to compute `max_scroll`.
    pub fn set_panel_visible_h(&mut self, panel: NodeId, visible_h: f32) {
        self.panel_visible_h.insert(panel, visible_h);
    }

    /// Read the visible body height for a panel.
    pub fn panel_visible_h(&self, panel: NodeId) -> Option<f32> {
        self.panel_visible_h.get(&panel).copied()
    }

    /// Find the panel whose rect contains `(x, y)`. Walks all
    /// registered panels and returns the first match. Acceptable
    /// because there are only a handful of panels (~3-5); for
    /// dozens, switch to the same back-to-front Vec approach as
    /// [`crate::interaction::HitIndex`].
    pub fn panel_at(&self, x: f32, y: f32) -> Option<NodeId> {
        for (id, rect) in &self.panel_rects {
            if rect.contains(x, y) {
                return Some(*id);
            }
        }
        None
    }

    /// Register a tooltip string for `id`. Called by `populate`
    /// passes or directly inside painters; read by the hover-tooltip
    /// pass via [`Self::tooltip_for`]. Empty strings are treated as
    /// no-tooltip and removed from the table.
    pub fn set_tooltip(&mut self, id: NodeId, text: impl Into<String>) {
        let s = text.into();
        if s.is_empty() {
            self.tooltips.remove(&id);
        } else {
            self.tooltips.insert(id, s);
        }
    }

    /// Lookup the tooltip for a widget id. Returns `None` if the id
    /// has no registered tooltip.
    pub fn tooltip_for(&self, id: NodeId) -> Option<&str> {
        self.tooltips.get(&id).map(|s| s.as_str())
    }

    /// `true` iff the section/panel at `id` is currently collapsed.
    /// Missing entries default to expanded — newly-registered
    /// sections start open without any setup.
    pub fn is_collapsed(&self, id: NodeId) -> bool {
        self.collapsed.get(&id).copied().unwrap_or(false)
    }

    /// Set the collapsed state for a section/panel. `true` collapses,
    /// `false` expands.
    pub fn set_collapsed(&mut self, id: NodeId, collapsed: bool) {
        self.collapsed.insert(id, collapsed);
    }

    /// Flip the collapsed state for `id`. Convenience for click
    /// handlers — equivalent to
    /// `set_collapsed(id, !is_collapsed(id))`.
    pub fn toggle_collapsed(&mut self, id: NodeId) {
        let was = self.is_collapsed(id);
        self.collapsed.insert(id, !was);
    }

    /// Open a right-click context menu at `(x, y)` with the given
    /// `kind`. Replaces any previously-open menu (only one menu
    /// can be visible at a time).
    pub fn open_context_menu(&mut self, request: ContextMenuRequest) {
        self.context_menu = Some(request);
    }

    /// Close any currently-open context menu. Snapshots the request
    /// into `last_context_menu` so `apply_event` can still read the
    /// menu's `kind` when handling the item-click that triggered
    /// the close (the click → Click event arrives AFTER the close).
    pub fn close_context_menu(&mut self) {
        if let Some(req) = self.context_menu.take() {
            self.last_context_menu = Some(req);
        }
    }

    /// Read the currently-open context menu request, if any.
    pub fn context_menu(&self) -> Option<ContextMenuRequest> {
        self.context_menu
    }

    /// Read the most recently closed context-menu request — used by
    /// `apply_event` to recover the original `kind` when routing an
    /// item Click into a side-table mutation. Cleared by
    /// `consume_last_context_menu` once the click has been applied.
    pub fn last_context_menu(&self) -> Option<ContextMenuRequest> {
        self.last_context_menu
    }

    /// Take + clear the last-context-menu snapshot. Called by
    /// `apply_event` after it has routed the click.
    pub fn consume_last_context_menu(&mut self) -> Option<ContextMenuRequest> {
        self.last_context_menu.take()
    }

    /// Read the outline-color index for a section header id (0..4
    /// referencing the highlighter palette). `None` = no outline.
    pub fn section_outline_color(&self, section: NodeId) -> Option<u8> {
        self.section_outline_color.get(&section).copied()
    }

    /// Set the outline-color index for a section. Pass `None` to
    /// clear (the "No outline" menu item).
    pub fn set_section_outline_color(&mut self, section: NodeId, color: Option<u8>) {
        match color {
            Some(c) => {
                self.section_outline_color.insert(section, c);
            }
            None => {
                self.section_outline_color.remove(&section);
            }
        }
    }

    /// Read the per-panel note list. Returns an empty slice when no
    /// notes have been created for the panel.
    pub fn notes_for_panel(&self, panel: NodeId) -> &[NoteData] {
        self.notes_per_panel
            .get(&panel)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Currently-active color picker target. The floating
    /// BlenderColorPicker is hidden when this is `None`.
    pub fn picker_target(&self) -> Option<NodeId> {
        self.picker_target
    }

    /// Open the picker editing the color at `target`. Pass `None`
    /// to hide the picker. The caller is responsible for seeding
    /// `widget_colors[target]` if it doesn't yet have a value.
    pub fn set_picker_target(&mut self, target: Option<NodeId>) {
        self.picker_target = target;
        // Opening the picker → it's the most recently summoned panel.
        if target.is_some() {
            // Picker is keyed by INSP_BLENDER_PICKER in the z-order
            // (canonical floating-panel id). Hard-coded here rather
            // than re-exporting the screens::hero::ids const into the
            // interaction crate, since the picker is the single
            // floating panel for the whole editor.
            const INSP_BLENDER_PICKER: NodeId = NodeId(380);
            self.bump_panel_z(INSP_BLENDER_PICKER);
        }
    }

    /// Read the current color of a color-target widget. Returns
    /// `None` when no color has been stored for `id`. Default
    /// colors are seeded lazily by callers (e.g. `apply_event`
    /// inserts a neutral gray when first opening the picker).
    pub fn widget_color(&self, id: NodeId) -> Option<[u8; 4]> {
        self.widget_colors.get(&id).copied()
    }

    /// Set the current color of a color-target widget. Called by
    /// the picker each frame to mirror its edits into the
    /// target's stored color, and by `apply_event` to seed the
    /// default when the picker first opens for that target.
    pub fn set_widget_color(&mut self, id: NodeId, rgba: [u8; 4]) {
        self.widget_colors.insert(id, rgba);
    }

    /// Begin a scrollbar drag at the given anchor. Stores the
    /// cursor / scroll / metrics snapshot until `end_scrollbar_drag`
    /// is called. Move events while this is set update
    /// `panel_scroll` proportionally.
    pub fn begin_scrollbar_drag(&mut self, anchor: ScrollbarDragAnchor) {
        self.scrollbar_drag = Some(anchor);
    }

    pub fn scrollbar_drag(&self) -> Option<ScrollbarDragAnchor> {
        self.scrollbar_drag
    }

    pub fn end_scrollbar_drag(&mut self) {
        self.scrollbar_drag = None;
    }

    /// Editor-wide corner-radius scale (1.0 = canonical, 0.0 =
    /// sharp). Painters that want to honor the user's theme preset
    /// multiply their `Radius::*.px()` by this factor.
    pub fn radius_scale(&self) -> f32 {
        self.radius_scale
    }

    pub fn set_radius_scale(&mut self, scale: f32) {
        self.radius_scale = scale.max(0.0);
    }

    /// Read the hierarchy display order (empty = use fixture's
    /// default order).
    pub fn hierarchy_order(&self) -> &[NodeId] {
        &self.hierarchy_order
    }

    /// Seed the hierarchy order with the fixture's default. Called
    /// at populate time if not already populated.
    pub fn init_hierarchy_order(&mut self, ids: Vec<NodeId>) {
        if self.hierarchy_order.is_empty() {
            self.hierarchy_order = ids;
        }
    }

    /// Forcibly overwrite the hierarchy order. Used by the live-data
    /// bridge (ADR-0025 M14.4a, [`crate::screens::hero::HeroScreen::sync_from_hierarchy`])
    /// to swap the fixture's `[HIER_PLAYER]` placeholder for the
    /// host's per-frame entity list. Also clears any drag-induced
    /// `hierarchy_parent` re-parents — the host owns the tree shape
    /// when live mode is active.
    pub fn set_hierarchy_order(&mut self, ids: Vec<NodeId>) {
        self.hierarchy_order = ids;
        self.hierarchy_parent.clear();
    }

    /// Move `dragged` to land just before `target` (or at the end
    /// when `target == None`). No-op when `dragged` isn't in the
    /// order list.
    pub fn hierarchy_move(&mut self, dragged: NodeId, target: Option<NodeId>) {
        let Some(from) = self.hierarchy_order.iter().position(|i| *i == dragged) else {
            return;
        };
        let item = self.hierarchy_order.remove(from);
        let to = match target {
            Some(t) => self
                .hierarchy_order
                .iter()
                .position(|i| *i == t)
                .unwrap_or(self.hierarchy_order.len()),
            None => self.hierarchy_order.len(),
        };
        self.hierarchy_order
            .insert(to.min(self.hierarchy_order.len()), item);
    }

    pub fn hierarchy_drag(&self) -> Option<HierarchyDragState> {
        self.hierarchy_drag
    }

    pub fn begin_hierarchy_drag(&mut self, dragged: NodeId, down_x: f32, down_y: f32) {
        self.hierarchy_drag = Some(HierarchyDragState {
            dragged,
            down_x,
            down_y,
            cursor_x: down_x,
            cursor_y: down_y,
            active: false,
        });
    }

    pub fn update_hierarchy_drag(&mut self, cursor_x: f32, cursor_y: f32) {
        if let Some(d) = self.hierarchy_drag.as_mut() {
            d.cursor_x = cursor_x;
            d.cursor_y = cursor_y;
            let dx = cursor_x - d.down_x;
            let dy = cursor_y - d.down_y;
            if (dx * dx + dy * dy) > 25.0 {
                // 5 px threshold²
                d.active = true;
            }
        }
    }

    pub fn end_hierarchy_drag(&mut self) -> Option<HierarchyDragState> {
        self.hierarchy_drag.take()
    }

    /// Parent NodeId of `child` if it's been re-parented via DnD;
    /// `None` for root rows.
    pub fn hierarchy_parent_of(&self, child: NodeId) -> Option<NodeId> {
        self.hierarchy_parent.get(&child).copied()
    }

    /// Depth in the parent tree (0 = root). Capped at 32 levels as
    /// a cycle guard — re-parent operations already reject cycles,
    /// so the cap should be unreachable in practice.
    pub fn hierarchy_depth_of(&self, id: NodeId) -> u32 {
        let mut depth = 0u32;
        let mut cur = id;
        for _ in 0..32 {
            match self.hierarchy_parent.get(&cur).copied() {
                Some(p) => {
                    depth += 1;
                    cur = p;
                }
                None => return depth,
            }
        }
        depth
    }

    /// True if `candidate` is a (strict or non-strict) descendant of
    /// `ancestor`. Used to reject DnD operations that would create a
    /// cycle (you can't drop a parent inside its own child).
    pub fn hierarchy_is_descendant_of(&self, candidate: NodeId, ancestor: NodeId) -> bool {
        if candidate == ancestor {
            return true;
        }
        let mut cur = candidate;
        for _ in 0..32 {
            match self.hierarchy_parent.get(&cur).copied() {
                Some(p) if p == ancestor => return true,
                Some(p) => cur = p,
                None => return false,
            }
        }
        false
    }

    /// Re-parent `child` under `parent`. Pass `None` to detach the
    /// child to the root level. Returns `false` (no change) when the
    /// operation would create a cycle (parent is a descendant of
    /// child).
    pub fn hierarchy_set_parent(&mut self, child: NodeId, parent: Option<NodeId>) -> bool {
        match parent {
            None => {
                self.hierarchy_parent.remove(&child);
                true
            }
            Some(p) => {
                if p == child || self.hierarchy_is_descendant_of(p, child) {
                    return false;
                }
                self.hierarchy_parent.insert(child, p);
                true
            }
        }
    }

    /// Append a new note with the given color index + section
    /// anchor to the panel's note list. Cap at 12 notes per panel
    /// to keep paint bounded. `before_section: Some(i)` makes the
    /// painter slot the note immediately above `SECTION_IDS[i]`;
    /// `None` appends at the bottom.
    pub fn notes_push(&mut self, panel: NodeId, color_idx: u8, before_section: Option<u8>) {
        const CAP: usize = 12;
        let list = self.notes_per_panel.entry(panel).or_default();
        if list.len() >= CAP {
            return;
        }
        list.push(NoteData {
            color_idx,
            title: format!("Note {}", list.len() + 1),
            body: String::new(),
            before_section,
        });
    }

    /// Update an existing note's color index.
    pub fn note_set_color(&mut self, panel: NodeId, index: usize, color_idx: u8) {
        if let Some(list) = self.notes_per_panel.get_mut(&panel)
            && let Some(note) = list.get_mut(index)
        {
            note.color_idx = color_idx.min(4);
        }
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
    fn collapsed_defaults_to_false() {
        let store = WidgetStore::with_capacity(4);
        assert!(!store.is_collapsed(NodeId(99)));
    }

    #[test]
    fn collapsed_set_and_toggle() {
        let mut store = WidgetStore::with_capacity(4);
        store.set_collapsed(NodeId(7), true);
        assert!(store.is_collapsed(NodeId(7)));
        store.toggle_collapsed(NodeId(7));
        assert!(!store.is_collapsed(NodeId(7)));
        store.toggle_collapsed(NodeId(8));
        assert!(store.is_collapsed(NodeId(8)));
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

    #[test]
    fn hierarchy_parent_round_trip_and_depth() {
        let mut store = WidgetStore::with_capacity(4);
        assert_eq!(store.hierarchy_depth_of(NodeId(10)), 0);
        assert!(store.hierarchy_set_parent(NodeId(11), Some(NodeId(10))));
        assert!(store.hierarchy_set_parent(NodeId(12), Some(NodeId(11))));
        assert_eq!(store.hierarchy_parent_of(NodeId(11)), Some(NodeId(10)));
        assert_eq!(store.hierarchy_parent_of(NodeId(12)), Some(NodeId(11)));
        assert_eq!(store.hierarchy_depth_of(NodeId(10)), 0);
        assert_eq!(store.hierarchy_depth_of(NodeId(11)), 1);
        assert_eq!(store.hierarchy_depth_of(NodeId(12)), 2);
    }

    #[test]
    fn hierarchy_set_parent_rejects_cycles() {
        let mut store = WidgetStore::with_capacity(4);
        // Build: 12 → 11 → 10 (12 is grandchild of 10)
        store.hierarchy_set_parent(NodeId(11), Some(NodeId(10)));
        store.hierarchy_set_parent(NodeId(12), Some(NodeId(11)));
        // Attempt to parent 10 under 12 (a descendant) → rejected.
        assert!(!store.hierarchy_set_parent(NodeId(10), Some(NodeId(12))));
        assert_eq!(store.hierarchy_parent_of(NodeId(10)), None);
        // Self-parent is also rejected.
        assert!(!store.hierarchy_set_parent(NodeId(11), Some(NodeId(11))));
    }

    #[test]
    fn hierarchy_set_parent_none_detaches() {
        let mut store = WidgetStore::with_capacity(4);
        store.hierarchy_set_parent(NodeId(11), Some(NodeId(10)));
        assert_eq!(store.hierarchy_depth_of(NodeId(11)), 1);
        assert!(store.hierarchy_set_parent(NodeId(11), None));
        assert_eq!(store.hierarchy_parent_of(NodeId(11)), None);
        assert_eq!(store.hierarchy_depth_of(NodeId(11)), 0);
    }
}
