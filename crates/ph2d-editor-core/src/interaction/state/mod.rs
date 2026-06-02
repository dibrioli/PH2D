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

mod blender_ops;
mod chrome_ops;
mod panel_ops;
mod widget_accessors;

use ph2d_a11y::NodeId;
use std::collections::BTreeMap;

use super::drag::{
    HierarchyDragState, NumberInputDragState, NumberStepperHoldState, ScrollbarDragAnchor,
};
use super::types::{BlenderHitKind, ContextMenuRequest, NoteData};
use super::util::format_number;

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

#[derive(Debug, Default)]
pub struct WidgetStore {
    pub(super) states: BTreeMap<NodeId, InteractiveState>,
    /// Insertion order, used for keyboard Tab traversal.
    pub(super) focus_order: Vec<NodeId>,
    pub(super) hot_id: Option<NodeId>,
    pub(super) active_id: Option<NodeId>,
    pub(super) focus_id: Option<NodeId>,
    /// Rect of the active widget at the moment of Down. Used by
    /// drag dispatch (Slider) to compute new value from pointer
    /// position relative to the original geometry.
    pub(super) active_rect: Option<Rect>,
    /// Slider id ↔ NumberInput id pairs that should mirror each
    /// other's value. When the slider's value changes via drag, the
    /// number input's `value` (and `buffer`, when not focused) is
    /// updated; when the number input's buffer commits via Enter or
    /// Blur, the slider's value is updated. Pre-populated by the
    /// hosting screen at construction time.
    pub(super) slider_to_number: BTreeMap<NodeId, NodeId>,
    pub(super) number_to_slider: BTreeMap<NodeId, NodeId>,
    /// Affine projection `(scale, offset)` such that
    /// `chip_display_value = slider_storage * scale + offset`,
    /// keyed by chip id. Default (when missing) = `(1.0, 0.0)` —
    /// identity, matching the legacy `link_slider_number` contract.
    /// Mapped links (`link_slider_number_mapped`) are the canonical
    /// way to wire a slider+chip pair when the chip's painted unit
    /// differs from the slider's 0..1 storage (Grow `±1`, Min Px
    /// integer count, ...). Without this map the chip's keyboard
    /// commit silently writes display-space text into the slider as
    /// if it were storage — the 2026-05-27 "type 0.2 see -0.6" bug.
    pub(super) number_to_slider_mapping: BTreeMap<NodeId, (f32, f32)>,
    /// Chip ids that should `.round()` their typed display value before
    /// inverse-projecting into the slider's `0..1` storage. Used for
    /// integer-domain chips (Min Px / Tile Grid / Posterize Dither
    /// Grain) so the chip's persisted value matches the painter's
    /// rounded `display_override` — without this, typing "50.5" left
    /// the chip stuck at 50.5 while the painter showed "50" (audit
    /// finding #3, 2026-05-28).
    pub(super) number_to_slider_snap_integer: std::collections::BTreeSet<NodeId>,
    /// NumberInput ids that are painted as bare `paint_number_chip`
    /// pills (no up/down arrows). The dispatch's
    /// `apply_number_stepper_if_hit` carves a stepper column out of
    /// the right edge of EVERY NumberInput's hit rect by default —
    /// fine for the boxed `paint_number_input_with_buffer` widget
    /// (Inspector position etc.) which paints arrows visually, but
    /// for pill chips it produces phantom-stepper continuous-hold
    /// (mouse stopped, value still climbing). Membership here makes
    /// the dispatch skip the stepper hit-test for the id.
    pub(super) chips_without_steppers: std::collections::BTreeSet<NodeId>,
    /// NodeIds eligible for collapse-toggle on left-click. Populated by
    /// `pre_populate` / panel `populate` for every `paint_section_header`
    /// site. The dispatch consults this set to decide whether a click
    /// on `id` should flip the section's collapse state (vs. doing
    /// nothing). Separate from `collapsed` (defined in chrome_ops:
    /// `is_collapsed` / `toggle_collapsed`) because the absence of a
    /// key in `collapsed` means "open by default", not "not
    /// collapsible" — without this guard the dispatch couldn't tell a
    /// section header click from any other Plain hit.
    pub(super) collapsible_sections: std::collections::BTreeSet<NodeId>,
    /// Hex `TextInput` id → its parent `BlenderPicker` id, so the
    /// dispatch can parse the typed buffer on Enter / blur and apply
    /// the resulting color to the parent state.
    pub(super) hex_to_blender_parent: BTreeMap<NodeId, NodeId>,
    /// Channel `NumberInput` chip id → (parent `BlenderPicker`,
    /// channel index 0..=3). Lets dispatch rewrite the parent's
    /// color value when the user commits a new channel value.
    pub(super) blender_channel_chip: BTreeMap<NodeId, (NodeId, u8)>,
    /// Most recent pointer-Down event, used for double-click
    /// detection. Stores the hit `NodeId` (or `None` if the click
    /// missed every widget) and the event timestamp.
    pub(super) last_down_id: Option<NodeId>,
    pub(super) last_down_at_ns: u128,
    /// `Some(id)` between a double-click Mouse Down and the matching
    /// Up — `apply_click` consumes this to upgrade `Click(id)` →
    /// `DoubleClick(id)`. Reset on every confirmed take.
    pub(super) pending_double_click: Option<NodeId>,
    /// Mutable color palettes per BlenderPicker — one Vec of swatches
    /// per parent picker id. Initialized at populate time; mutated by
    /// "+ swatch" / right-click-delete dispatch paths.
    pub(super) blender_palettes: BTreeMap<NodeId, Vec<ColorValue>>,
    /// Per-picker drag offset (dx, dy) applied to the rect chosen by
    /// the host painter. Mutated by drag-handle clicks; defaults to
    /// (0, 0). When the drag handle is `active`, `drag_anchor_px`
    /// stores the (cursor.x − rect.x, cursor.y − rect.y) at Down so
    /// Move events can keep the picker stuck to the cursor.
    pub(super) blender_picker_offset: BTreeMap<NodeId, (f32, f32)>,
    /// In-progress picker drag: (parent_id, cursor_x_at_down,
    /// cursor_y_at_down, offset_x_at_down, offset_y_at_down). Move
    /// events compute `new_offset = offset_at_down + (cursor − down_cursor)`.
    /// Cleared on pointer Up.
    pub(super) blender_drag_anchor: Option<(NodeId, f32, f32, f32, f32)>,
    /// Per-panel manual resize delta (dw, dh) applied on top of the
    /// layout's base width/height. Mutated by dragging the bottom-
    /// right resize gripper.
    pub(super) panel_resize_delta: BTreeMap<NodeId, (f32, f32)>,
    /// In-progress panel resize: (parent_id, last_cursor_x,
    /// last_cursor_y). Move events apply (cursor − last) to the
    /// stored `panel_resize_delta`, then re-anchor.
    pub(super) panel_resize_anchor: Option<(NodeId, f32, f32)>,
    /// In-progress panel resize from the bottom-LEFT corner — same
    /// shape as [`panel_resize_anchor`] but the Move handler also
    /// shifts the panel's stored offset (`panel_drag_offset`) so the
    /// right edge stays anchored. Companion field rather than a mode
    /// tag because only one resize is active at a time and the dispatch
    /// can check both Options cheaply.
    pub(super) panel_resize_anchor_bl: Option<(NodeId, f32, f32)>,
    /// Clipboard outbox — set by Cmd+C/X handlers; shell drains each
    /// frame via `take_clipboard_copy` and writes to the OS
    /// clipboard. `String` rather than a reference so the data lives
    /// independently of any widget buffer that might mutate next.
    pub(super) pending_clipboard_copy: Option<String>,
    /// Clipboard paste request — set by Cmd+V on a focused text
    /// widget; shell reads the OS clipboard and calls back into
    /// `apply_clipboard_paste` with the text.
    pub(super) pending_clipboard_paste: Option<NodeId>,
    /// Currently-loaded scene name shown on the TopBar project chip.
    /// Mutated by `ContextMenuKind::SceneList` row clicks.
    pub(super) current_scene_name: String,
    /// Coordinate-space toggle for the TOOL_SPACE rail button.
    /// `false` = Global, `true` = Local. Flipped on click.
    pub(super) tool_space_local: bool,
    /// Camera-framing mode for the TOOL_HOME rail button.
    /// Cycle: 0 = Selected, 1 = Camera, 2 = All. Bumped on click.
    pub(super) tool_view_mode: u8,
    /// Per-panel Z order — last element paints LAST (= topmost).
    /// Mutated by `bump_panel_z` whenever the user clicks inside a
    /// panel, drags it, or it newly opens (color picker). Painters
    /// walk this in order so the most-recently-touched panel sits
    /// on top of any overlapping siblings.
    pub(super) panel_z_order: Vec<NodeId>,
    /// Eyedropper pending: when Some(parent), the next pointer Down
    /// (anywhere except on the eyedropper button itself) is intercepted
    /// by the dispatch and emitted as `WidgetEvent::EyedropperPick`,
    /// signaling the host to readback the pixel under the cursor.
    pub(super) eyedropper_pending: Option<NodeId>,
    /// Vertical scroll offset per panel. Wheel events advance the
    /// offset; painters subtract it from content y. Clamped on each
    /// scroll to `[0, content_h - visible_h]` by the painter (which
    /// knows both heights). See `docs/UI_Bugs/README.md` §1
    /// (hit-testing) — content rendered with offset must compensate
    /// in hit-test too.
    pub(super) panel_scroll: BTreeMap<NodeId, f32>,
    /// Painter-published rect of each scrollable panel — populated
    /// every frame so the wheel dispatch can find which panel sits
    /// under the cursor. Cleared together with `clear_for_frame` on
    /// the hit_index by the host (or hero) at frame start.
    pub(super) panel_rects: BTreeMap<NodeId, Rect>,
    /// Painter-published total content height per panel (sum of
    /// every section's height + separators). `dispatch_wheel` reads
    /// this to clamp scroll deltas at the upper bound
    /// (`content_h - visible_h`) — without it, wheeling past the
    /// last element produces a one-frame "jump" as the next paint
    /// clamps the over-scroll back.
    pub(super) panel_content_h: BTreeMap<NodeId, f32>,
    /// Exact visible body height per panel, also painter-published.
    /// Pairs with `panel_content_h` so `dispatch_wheel` can compute
    /// `max_scroll = content_h - visible_h` precisely (no heuristic).
    pub(super) panel_visible_h: BTreeMap<NodeId, f32>,
    /// Tooltip text per widget id. Read by `paint_hover_tooltip`
    /// when the user hovers over a registered widget. Populated by
    /// `populate` / paint passes via `set_tooltip`. Replaces the old
    /// hardcoded `tooltip_for(id)` match — every widget can now
    /// participate without per-id boilerplate.
    pub(super) tooltips: BTreeMap<NodeId, String>,
    /// Collapsed/expanded state per id. `true` = collapsed; missing
    /// entry defaults to "expanded" so newly-registered sections
    /// open by default. Toggled by `apply_event` on Click and
    /// consumed by section painters that early-out when collapsed.
    pub(super) collapsed: BTreeMap<NodeId, bool>,
    /// Pending right-click context menu. `Some` when a Secondary
    /// Down landed somewhere a menu should appear (e.g. an empty
    /// inspector panel or a section header); `None` when no menu
    /// is open. The hero painter consumes this to render a floating
    /// menu over everything; clicking outside the menu or on a menu
    /// item clears the slot.
    pub(super) context_menu: Option<ContextMenuRequest>,
    /// Section-header id → highlighter color index (0..4 for the 5
    /// canonical colors; missing entry == "no outline"). Painted by
    /// the inspector as a colored stroke around the section block.
    pub(super) section_outline_color: BTreeMap<NodeId, u8>,
    /// Per-panel list of user-created notes. Each note carries a
    /// background color index into the highlighter palette. New
    /// notes append; right-click → delete removes by index. The
    /// painter walks this list once per panel each frame.
    pub(super) notes_per_panel: BTreeMap<NodeId, Vec<NoteData>>,
    /// Sticky source of the most recently completed context-menu
    /// request, captured at apply-event time so the inspector can
    /// route the click → side-table mutation. The dispatch clears
    /// `context_menu` on the same Down event that selects an item;
    /// without this snapshot the inspector loses the request
    /// before it can read it.
    pub(super) last_context_menu: Option<ContextMenuRequest>,
    /// Currently-active color picker target. `Some(id)` means the
    /// floating BlenderColorPicker is open and editing the color
    /// stored at `widget_colors[id]`. `None` hides the picker. Set
    /// by clicks on color targets (section color circles, color
    /// swatches, …) and cleared by any click outside the picker
    /// and outside another color target.
    pub(super) picker_target: Option<NodeId>,
    /// Per-widget current color. Keyed by the target widget's id
    /// (section color circles, color swatches). The picker writes
    /// here on every frame while editing; painters read here to
    /// display the widget's current color.
    pub(super) widget_colors: BTreeMap<NodeId, [u8; 4]>,
    /// In-progress scrollbar drag. Captured on Down inside a
    /// scrollbar thumb's hit rect; consumed by Move events to
    /// translate cursor delta into a `panel_scroll` delta; cleared
    /// on Up. `track_h` and `content_h` are snapshotted so the
    /// drag stays linear even if the painter republishes them
    /// mid-drag.
    pub(super) scrollbar_drag: Option<ScrollbarDragAnchor>,
    /// Editor-wide corner-radius scale. `1.0` = canonical, `0.0` =
    /// sharp / squared, `1.6` = round. Painters that want to follow
    /// the user's preset multiply their `Radius::*.px()` by this.
    /// Centralized so the topbar theme menu drives the look in one
    /// place.
    pub(super) radius_scale: f32,
    /// Rail button size preset (Small / Medium / Large). User-toggled
    /// via the Themes menu (2026-05-24); painters in
    /// [`crate::widget::tool_rail`] and the hero orchestrator
    /// ([`crate::screens::hero`]) read this to pick the chip edge
    /// and rail column width.
    pub(super) rail_button_size: crate::widget::RailButtonSize,
    /// Cached present-mode (VSync ON vs OFF). Source of truth still
    /// lives in the shell (it owns the swap chain), but the core
    /// mirrors the last value the user picked in Settings → Display
    /// so menu paint can show a "selected" bullet next to the active
    /// row. Default `true` matches the shell's `Fifo` baseline.
    pub(super) present_vsync: bool,
    /// Hierarchy row display order. When non-empty, the hierarchy
    /// painter walks this list instead of the fixture's default
    /// order. Mutated by drag-and-drop (`Down + Move > threshold +
    /// Up`) to reorder rows.
    pub(super) hierarchy_order: Vec<NodeId>,
    /// Parent map for tree-style hierarchy. `child → parent`; absent
    /// entries are roots. Mutated by drop-inside DnD; consumed by the
    /// painter to indent rows by depth.
    pub(super) hierarchy_parent: BTreeMap<NodeId, NodeId>,
    /// M14.6C: parents whose subtree is collapsed in the panel.
    /// View-only state (does NOT touch ECS hierarchy); just hides
    /// descendants in the row list. Click on the chevron toggles
    /// membership.
    pub(super) hierarchy_collapsed: std::collections::BTreeSet<NodeId>,
    /// In-progress hierarchy drag. `Some` when a Primary Down landed
    /// on a hierarchy row and the cursor has moved past the drag
    /// threshold; cleared on Up (with reorder applied) or on Up at
    /// the original position (treated as a regular click).
    pub(super) hierarchy_drag: Option<HierarchyDragState>,
    /// M14.6B: every NodeId currently displayed as a hierarchy row.
    /// Painter republishes the set each frame (fixture + live
    /// modes). Dispatch reads this to decide "this Down is on a
    /// draggable hierarchy row" without hardcoding any id range —
    /// the static `is_hierarchy_entity_id(400..=411)` check covers
    /// only the fixture range; live (ECS-bridge) rows start at
    /// `100_000+` and would silently fall through to "click,
    /// no drag" without this set.
    pub(super) hierarchy_row_ids: std::collections::BTreeSet<NodeId>,
    /// TextInput ids that should treat Enter as "insert newline"
    /// instead of the default "Submit + Blur" (single-line form
    /// behavior). Populated by widgets that wrap multi-line content
    /// (TextArea, note bodies). Default-empty so a freshly registered
    /// TextInput is single-line — matches user expectation that Enter
    /// confirms the value rather than wrapping.
    pub(super) multiline_text_ids: std::collections::BTreeSet<NodeId>,
    /// M14.A polish: in-progress drag on a NumberInput body. Captured
    /// on Down inside the box (NOT inside the up/down arrow), held
    /// across Move events to convert cursor delta → value delta
    /// (Blender-style: horizontal fast, vertical slow, Shift = fine).
    /// On Up: a drag that NEVER crossed the threshold becomes a
    /// regular "click → enter edit mode"; one that did becomes a
    /// committed value (no edit mode).
    pub(super) number_input_drag: Option<NumberInputDragState>,
    /// M14.A polish: in-progress continuous-hold on a NumberInput
    /// stepper arrow. The dispatcher fires one tick on Down, then
    /// `dispatch_tick` repeats while held (initial delay + repeat
    /// interval matching macOS Aqua text-field steppers).
    pub(super) number_stepper_hold: Option<NumberStepperHoldState>,
    /// Latest Shift modifier state, pushed by the shell on every
    /// `ModifiersChanged`. Used by `dispatch_pointer` to scale the
    /// NumberInput drag delta (Shift = 0.001× multiplier = fine
    /// adjustment). Pointer events don't carry modifiers natively in
    /// `ph2d-host::PointerEvent`; this is the canonical cache.
    pub(super) shift_held: bool,
    /// Fase 0c: latest Cmd (macOS) / Ctrl (Linux/Windows) modifier
    /// state, mirror of [`Self::shift_held`]. Shell pushes via
    /// [`Self::set_cmd_held`] on every `ModifiersChanged`, OR'ing
    /// `super_key()` and `control_key()` so panel handlers can treat
    /// the two as interchangeable (toggle-select modifier).
    pub(super) cmd_held: bool,
    /// In-progress Painter layers-panel row drag (W3 T3.8 — reorder +
    /// drop-into-group). Reuses the generic [`HierarchyDragState`] anchor
    /// (dragged id + down/cursor pos + active threshold). Unlike the
    /// hierarchy drag, the dispatch never mutates structure here — the
    /// painter tool owns the `LayerStack` and resolves the drop.
    pub(super) painter_layer_drag: Option<HierarchyDragState>,
    /// Every `NodeId` currently displayed as a Painter layer row. The
    /// layers panel republishes the set each frame; dispatch reads it to
    /// decide "this Down is on a draggable layer row" (mirror of
    /// [`Self::hierarchy_row_ids`]).
    pub(super) painter_layer_row_ids: std::collections::BTreeSet<NodeId>,
    /// Every `NodeId` that is a "picker swatch" — a [`crate::widget::ColorSwatch`]
    /// whose Down opens the canonical Blender color picker seeded with the
    /// swatch's current `widget_color`. Panels register their picker swatches
    /// here as they paint (idempotent; the ids are stable). Generalizes the
    /// former per-id `PAINTER_COLOR_THUMB` special-case so any panel swatch
    /// (Painter brush color, Vector fill, …) opens the picker uniformly.
    pub(super) picker_swatch_ids: std::collections::BTreeSet<NodeId>,
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
            number_to_slider_mapping: BTreeMap::new(),
            number_to_slider_snap_integer: std::collections::BTreeSet::new(),
            chips_without_steppers: std::collections::BTreeSet::new(),
            collapsible_sections: std::collections::BTreeSet::new(),
            hex_to_blender_parent: BTreeMap::new(),
            blender_channel_chip: BTreeMap::new(),
            last_down_id: None,
            last_down_at_ns: 0,
            pending_double_click: None,
            blender_palettes: BTreeMap::new(),
            blender_picker_offset: BTreeMap::new(),
            blender_drag_anchor: None,
            panel_resize_delta: BTreeMap::new(),
            panel_resize_anchor: None,
            panel_resize_anchor_bl: None,
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
            rail_button_size: crate::widget::RailButtonSize::default(),
            present_vsync: true,
            hierarchy_order: Vec::new(),
            hierarchy_parent: BTreeMap::new(),
            hierarchy_collapsed: std::collections::BTreeSet::new(),
            hierarchy_drag: None,
            hierarchy_row_ids: std::collections::BTreeSet::new(),
            multiline_text_ids: std::collections::BTreeSet::new(),
            number_input_drag: None,
            number_stepper_hold: None,
            shift_held: false,
            cmd_held: false,
            painter_layer_drag: None,
            painter_layer_row_ids: std::collections::BTreeSet::new(),
            picker_swatch_ids: std::collections::BTreeSet::new(),
        }
    }

    /// Register a bidirectional link: when `slider`'s value changes,
    /// `number`'s value follows; when `number` commits a new value,
    /// `slider` follows. Caller is responsible for both ids being
    /// pre-registered as Slider and NumberInput respectively.
    ///
    /// Post-2026-05-24: this no longer auto-marks the chip as
    /// no-stepper. The canon `paint_number_chip` always paints up/down
    /// arrows now, and the dispatch's stepper hit-test is the desired
    /// behavior for every chip — including chips linked to a slider.
    /// Phantom-stepper-while-still is impossible because there is no
    /// "no-arrow chip" variant anymore. See
    /// [`mark_chip_no_stepper`](Self::mark_chip_no_stepper) for the
    /// (deprecated) opt-out and the gate
    /// `architecture_no_chip_without_steppers` that prevents
    /// re-introducing pills sans-arrows.
    pub fn link_slider_number(&mut self, slider: NodeId, number: NodeId) {
        self.slider_to_number.insert(slider, number);
        self.number_to_slider.insert(number, slider);
    }

    /// Like [`link_slider_number`](Self::link_slider_number) but
    /// registers an affine projection between the slider's `0..1`
    /// storage and the chip's user-visible value:
    ///
    /// ```text
    /// chip_display = slider_storage * scale + offset
    /// slider_storage = (chip_display - offset) / scale
    /// ```
    ///
    /// Use whenever the chip paints a non-identity transform via
    /// `display_override` (Grow's signed `±1`, Min Px integer count,
    /// Upscale "2.00×", padding pixels, etc.). The dispatch then
    /// inverse-projects on every chip mutation (Enter commit, stepper
    /// arrow click, drag scrub, continuous hold) and forward-projects
    /// on every slider mutation (drag, programmatic set), so the
    /// chip's stored value lives in display-space throughout — exactly
    /// what the buffer shows on focus, exactly what the user types.
    ///
    /// `scale` must be non-zero (asserted in debug). Identity is
    /// `scale=1.0, offset=0.0`, equivalent to `link_slider_number`.
    pub fn link_slider_number_mapped(
        &mut self,
        slider: NodeId,
        number: NodeId,
        scale: f32,
        offset: f32,
    ) {
        self.link_slider_number_mapped_inner(slider, number, scale, offset, false);
    }

    /// Like [`link_slider_number_mapped`] but the chip's typed display
    /// value is **rounded to the nearest integer** before being written
    /// to the chip and inverse-projected to the slider. Use for chips
    /// whose painted unit is an integer count (BgRemoval Min Px,
    /// Color-Eq Tile Grid / Posterize Dither Grain, etc.) — without
    /// this, a user typing "50.5" left the chip stuck at fractional 50.5
    /// while the painter's `display_override` showed the rounded "50",
    /// and Tab-away / re-focus revealed the inconsistency (audit
    /// finding #3, 2026-05-28).
    ///
    /// The mapping itself is still the same affine `display = storage *
    /// scale + offset`; the snap is applied on TOP at the chip-write
    /// boundary so the slider's `0..1` storage can stay continuous (the
    /// painter rounds on its own from that continuous track when needed).
    pub fn link_slider_number_mapped_integer(
        &mut self,
        slider: NodeId,
        number: NodeId,
        scale: f32,
        offset: f32,
    ) {
        self.link_slider_number_mapped_inner(slider, number, scale, offset, true);
    }

    fn link_slider_number_mapped_inner(
        &mut self,
        slider: NodeId,
        number: NodeId,
        scale: f32,
        offset: f32,
        snap_integer: bool,
    ) {
        debug_assert!(
            scale.abs() > f32::EPSILON,
            "link_slider_number_mapped: scale must be non-zero"
        );
        self.slider_to_number.insert(slider, number);
        self.number_to_slider.insert(number, slider);
        if (scale - 1.0).abs() > f32::EPSILON || offset.abs() > f32::EPSILON {
            self.number_to_slider_mapping
                .insert(number, (scale, offset));
        } else {
            // Identity — keep the map clean so default-lookup is fast.
            self.number_to_slider_mapping.remove(&number);
        }
        if snap_integer {
            self.number_to_slider_snap_integer.insert(number);
        } else {
            self.number_to_slider_snap_integer.remove(&number);
        }
    }

    pub fn linked_number(&self, slider: NodeId) -> Option<NodeId> {
        self.slider_to_number.get(&slider).copied()
    }

    pub fn linked_slider(&self, number: NodeId) -> Option<NodeId> {
        self.number_to_slider.get(&number).copied()
    }

    /// `true` iff the chip is registered with
    /// [`link_slider_number_mapped_integer`] — the dispatch's
    /// [`apply_chip_value_with_mirror`] will `.round()` the typed
    /// display value before writing chip+slider.
    pub fn linked_slider_snap_integer(&self, number: NodeId) -> bool {
        self.number_to_slider_snap_integer.contains(&number)
    }

    /// Projection `(scale, offset)` for the chip→slider mirror.
    /// Returns identity `(1.0, 0.0)` when no mapping is registered —
    /// callers can always safely forward/inverse-apply without
    /// branching on the link kind.
    pub fn linked_slider_mapping(&self, number: NodeId) -> (f32, f32) {
        self.number_to_slider_mapping
            .get(&number)
            .copied()
            .unwrap_or((1.0, 0.0))
    }

    /// **Deprecated (2026-05-24).** Marking a NumberInput as
    /// no-stepper made sense when `paint_number_chip` painted a bare
    /// pill — clicking the (invisible) right column armed a
    /// continuous-hold that climbed silently. After unification, every
    /// chip paints arrows and the click→step behavior is the desired
    /// affordance everywhere. Kept as a back-compat no-op for one wave
    /// while in-tree callers are removed; CI gate
    /// `architecture_no_chip_without_steppers` prevents reintroducing
    /// a chip variant that needs it. To be deleted in Wave 12.
    #[deprecated(
        since = "0.0.0",
        note = "all chips paint arrows now; the dispatch's stepper hit-test is the canon (Wave 11)"
    )]
    pub fn mark_chip_no_stepper(&mut self, id: NodeId) {
        self.chips_without_steppers.insert(id);
    }

    /// Whether the given NumberInput id is painted without stepper
    /// arrows. Always `false` for new code (post-2026-05-24 chip
    /// canon) — kept for back-compat with any in-flight call to the
    /// deprecated [`mark_chip_no_stepper`](Self::mark_chip_no_stepper).
    pub fn is_chip_no_stepper(&self, id: NodeId) -> bool {
        self.chips_without_steppers.contains(&id)
    }

    /// Mark a section header NodeId as collapse-toggle eligible.
    /// Called from `pre_populate` / panel `populate` for every
    /// `paint_section_header` site so the dispatch knows a left-click
    /// on `id` should flip the collapse state via the existing
    /// [`toggle_collapsed`](Self::toggle_collapsed) API. UI canon
    /// post-2026-05-24: every section is collapsible (vide
    /// `docs/UI_Padrao/components/section_header.md`).
    pub fn mark_collapsible_section(&mut self, id: NodeId) {
        self.collapsible_sections.insert(id);
    }

    /// True iff the section is registered as collapse-toggle eligible.
    /// Dispatch consults this before firing the toggle on a click.
    pub fn is_collapsible_section(&self, id: NodeId) -> bool {
        self.collapsible_sections.contains(&id)
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
        // Stash the upgrade hint so the matching Up emits
        // `WidgetEvent::DoubleClick(id)` in place of the regular
        // `Click(id)`. Cleared by `take_pending_double_click`.
        if is_double {
            self.pending_double_click = id;
        }
        is_double
    }

    /// Take + clear the `pending_double_click` slot, returning the
    /// id stored on the matching Mouse Down. `apply_click` consumes
    /// this to upgrade `Click(id)` → `DoubleClick(id)` when the id
    /// matches the click target.
    pub fn take_pending_double_click(&mut self) -> Option<NodeId> {
        self.pending_double_click.take()
    }

    /// Register a widget at construction time. Idempotent — repeat
    /// calls overwrite the state but never grow capacity. Should NOT
    /// be called during the paint/dispatch hot path.
    pub fn register(&mut self, id: NodeId, initial: InteractiveState) {
        if self.states.insert(id, initial).is_none() {
            self.focus_order.push(id);
        }
    }

    /// Register `id` only when it isn't already in the store. Unlike
    /// [`Self::register`] (which always replaces and is the right call
    /// for one-shot construction-time wiring), this is safe to call
    /// every frame from live-mode `repopulate` paths without
    /// clobbering user-typed text / cursor state. Returns true iff
    /// the entry was freshly inserted.
    pub fn register_if_absent(&mut self, id: NodeId, initial: InteractiveState) -> bool {
        if self.states.contains_key(&id) {
            return false;
        }
        self.states.insert(id, initial);
        self.focus_order.push(id);
        true
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

    /// M14.A: read the in-progress NumberInput drag (Down on the box
    /// body). `None` when no NumberInput is being dragged or the user
    /// is currently editing one (focus → caret mode, not drag mode).
    pub fn number_input_drag(&self) -> Option<NumberInputDragState> {
        self.number_input_drag
    }

    pub fn begin_number_input_drag(&mut self, drag: NumberInputDragState) {
        self.number_input_drag = Some(drag);
    }

    /// Flip the in-flight drag past the threshold (idempotent). Called
    /// by `dispatch_pointer` Move once the cursor has moved >
    /// `NUMBER_INPUT_DRAG_THRESHOLD_PX` from the Down position.
    ///
    /// `axis_horizontal` locks the active scrub axis for the rest of
    /// the drag — true = horizontal, false = vertical. The caller
    /// decides at the moment of promotion based on `|dx| vs |dy|`.
    /// Subsequent calls (the threshold is already crossed) are no-ops
    /// so the axis can't flip mid-drag.
    ///
    /// `cursor_x`/`cursor_y` re-anchor the incremental `last_x`/`last_y`
    /// to the cursor position AT promotion time. Without this, the
    /// SAME Move that crosses the threshold would compute its step
    /// delta from `start_x` (Down position) → applies the entire
    /// threshold-crossing distance (≈5 px × DRAG_RATE) as an instant
    /// JUMP before the user perceives any drag motion. Re-anchoring
    /// makes the promotion frame contribute a zero-delta and
    /// subsequent Moves compute their deltas from "here".
    pub fn promote_number_input_drag_to_slider(
        &mut self,
        axis_horizontal: bool,
        cursor_x: f32,
        cursor_y: f32,
    ) {
        if let Some(drag) = self.number_input_drag.as_mut()
            && !drag.crossed_threshold
        {
            drag.crossed_threshold = true;
            drag.axis_horizontal = axis_horizontal;
            drag.last_x = cursor_x;
            drag.last_y = cursor_y;
        }
    }

    /// Advance the incremental-drag anchor `last_x` / `last_y` to the
    /// cursor's current position. Called by `dispatch_pointer` Move
    /// after each per-Move delta has been applied so the NEXT Move
    /// computes its delta from "here", not from Down. This is the
    /// Blender/AE scrub model — a reversal after a clamp produces a
    /// non-zero step_dx on the very next Move (the absolute-delta
    /// model kept the value pegged at the clamp edge until the cursor
    /// returned all the way to `start_x`).
    pub fn advance_number_input_drag_anchor(&mut self, x: f32, y: f32) {
        if let Some(drag) = self.number_input_drag.as_mut() {
            drag.last_x = x;
            drag.last_y = y;
        }
    }

    pub fn end_number_input_drag(&mut self) -> Option<NumberInputDragState> {
        self.number_input_drag.take()
    }

    /// M14.A: read the in-progress NumberInput stepper continuous-
    /// hold. `None` when no arrow is held.
    pub fn number_stepper_hold(&self) -> Option<NumberStepperHoldState> {
        self.number_stepper_hold
    }

    pub fn begin_number_stepper_hold(&mut self, hold: NumberStepperHoldState) {
        self.number_stepper_hold = Some(hold);
    }

    /// Update the `last_tick_ns` after `dispatch_tick` applied a
    /// repeat. Returns `None` if there's no hold in flight (no-op).
    pub fn record_number_stepper_tick(&mut self, now_ns: u128) {
        if let Some(h) = self.number_stepper_hold.as_mut() {
            h.last_tick_ns = now_ns;
        }
    }

    pub fn end_number_stepper_hold(&mut self) {
        self.number_stepper_hold = None;
    }

    /// M14.A: latest Shift modifier state. Shell pushes via
    /// [`Self::set_shift_held`] on every `WindowEvent::ModifiersChanged`.
    /// `dispatch_pointer` Move reads this to scale the drag delta
    /// (Shift = fine adjustment).
    pub fn shift_held(&self) -> bool {
        self.shift_held
    }

    pub fn set_shift_held(&mut self, held: bool) {
        self.shift_held = held;
    }

    /// Fase 0c: latest Cmd (macOS) / Ctrl (Linux/Windows) modifier
    /// state. Shell pushes via [`Self::set_cmd_held`] on every
    /// `WindowEvent::ModifiersChanged`. Hierarchy / canvas multi-
    /// select handlers read this to map Click → toggle-select.
    pub fn cmd_held(&self) -> bool {
        self.cmd_held
    }

    pub fn set_cmd_held(&mut self, held: bool) {
        self.cmd_held = held;
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

    pub fn begin_hierarchy_drag(
        &mut self,
        dragged: NodeId,
        down_x: f32,
        down_y: f32,
        timestamp_ns: u128,
    ) {
        self.hierarchy_drag = Some(HierarchyDragState {
            dragged,
            down_x,
            down_y,
            cursor_x: down_x,
            cursor_y: down_y,
            active: false,
            down_timestamp_ns: timestamp_ns,
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

    /// M14.6C: true when `id`'s subtree is collapsed in the panel.
    /// The ECS hierarchy is untouched — this is purely a view filter
    /// applied by `paint_hierarchy`.
    pub fn is_hierarchy_collapsed(&self, id: NodeId) -> bool {
        self.hierarchy_collapsed.contains(&id)
    }

    /// Flip the collapsed flag for `id`. Called by `apply_event` when
    /// the chevron companion NodeId is clicked.
    pub fn toggle_hierarchy_collapsed(&mut self, id: NodeId) {
        if !self.hierarchy_collapsed.insert(id) {
            self.hierarchy_collapsed.remove(&id);
        }
    }

    /// M14.6B: republish the set of NodeIds currently displayed as
    /// hierarchy rows. The painter calls this once per frame after
    /// registering its row hit-rects. Cleared and replaced wholesale
    /// — no merge — so stale ids from the previous frame (e.g. an
    /// entity that despawned) drop out automatically.
    pub fn set_hierarchy_row_ids(&mut self, ids: std::collections::BTreeSet<NodeId>) {
        self.hierarchy_row_ids = ids;
    }

    /// True iff `id` is currently displayed as a hierarchy row.
    /// Covers both fixture HIER_* ids and live ECS-bridge ids in
    /// one query — dispatch uses this to decide whether to start a
    /// drag candidate on Primary Down (replaces the static range
    /// check that used to silently reject every live row).
    /// Mark `id` as a multi-line text widget. Enter on it inserts a
    /// literal newline instead of submitting + blurring (the default
    /// behavior for single-line TextInputs). Idempotent; callers
    /// (TextArea widget, note body populators) re-mark every frame
    /// during populate.
    pub fn mark_multiline_text(&mut self, id: NodeId) {
        self.multiline_text_ids.insert(id);
    }

    /// True iff `id` was previously marked via `mark_multiline_text`.
    pub fn is_multiline_text(&self, id: NodeId) -> bool {
        self.multiline_text_ids.contains(&id)
    }

    pub fn is_hierarchy_row(&self, id: NodeId) -> bool {
        self.hierarchy_row_ids.contains(&id)
    }

    // ── Painter layers-panel row drag (W3 T3.8) — mirror of the hierarchy
    //    drag, but the dispatch never mutates structure (the painter tool
    //    owns the LayerStack and resolves the emitted `PainterLayerReparent`).

    /// Snapshot of the in-progress painter layer-row drag, if any.
    pub fn painter_layer_drag(&self) -> Option<HierarchyDragState> {
        self.painter_layer_drag
    }

    /// Begin a painter layer-row drag (Primary Down on a row). `active`
    /// flips once the cursor passes the 5px threshold (see
    /// [`Self::update_painter_layer_drag`]).
    pub fn begin_painter_layer_drag(
        &mut self,
        dragged: NodeId,
        down_x: f32,
        down_y: f32,
        timestamp_ns: u128,
    ) {
        self.painter_layer_drag = Some(HierarchyDragState {
            dragged,
            down_x,
            down_y,
            cursor_x: down_x,
            cursor_y: down_y,
            active: false,
            down_timestamp_ns: timestamp_ns,
        });
    }

    /// Advance the drag cursor; flips `active` once past the 5px threshold.
    pub fn update_painter_layer_drag(&mut self, cursor_x: f32, cursor_y: f32) {
        if let Some(d) = self.painter_layer_drag.as_mut() {
            d.cursor_x = cursor_x;
            d.cursor_y = cursor_y;
            let dx = cursor_x - d.down_x;
            let dy = cursor_y - d.down_y;
            if (dx * dx + dy * dy) > 25.0 {
                d.active = true;
            }
        }
    }

    /// Take the in-progress drag (cleared on Up).
    pub fn end_painter_layer_drag(&mut self) -> Option<HierarchyDragState> {
        self.painter_layer_drag.take()
    }

    /// Republish the set of `NodeId`s that are currently painter layer rows.
    /// The layers panel calls this each frame with `painter_layer_widget_id(
    /// layer, Row)` for every visible row.
    pub fn set_painter_layer_row_ids(&mut self, ids: std::collections::BTreeSet<NodeId>) {
        self.painter_layer_row_ids = ids;
    }

    /// Is `id` a draggable painter layer row?
    pub fn is_painter_layer_row(&self, id: NodeId) -> bool {
        self.painter_layer_row_ids.contains(&id)
    }

    /// Mark `id` as a picker swatch — a [`crate::widget::ColorSwatch`] whose
    /// Down opens the Blender picker seeded with its `widget_color`. Panels
    /// call this as they paint each picker swatch (idempotent; ids are stable).
    pub fn register_picker_swatch(&mut self, id: NodeId) {
        self.picker_swatch_ids.insert(id);
    }

    /// Does a Down on `id` open the canonical color picker?
    pub fn is_picker_swatch(&self, id: NodeId) -> bool {
        self.picker_swatch_ids.contains(&id)
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

    /// Mutate a NumberInput's committed value programmatically (e.g.
    /// from a linked Slider drag). Re-syncs the buffer to the new
    /// formatted value when the input is **not** focused; if it is
    /// focused, the user's edit is preserved.
    pub fn set_number_value(&mut self, id: NodeId, new_value: f64) {
        let focused = self.focus_id == Some(id);
        // Audit re-pass fix: while a drag-slider is actively
        // scrubbing this very NumberInput, do NOT overwrite
        // `last_committed` from external (snapshot) writes. The drag's
        // Up handler commits `last_committed = value` at release — if
        // an in-flight `set_number_value` clobbered the rollback
        // anchor on every frame, Esc mid-drag would revert to the
        // last-applied dragged value (effectively a no-op), defeating
        // audit fix #2.
        let dragging_this = matches!(self.number_input_drag.as_ref(), Some(d) if d.id == id);
        if let Some(InteractiveState::NumberInput {
            value,
            buffer,
            last_committed,
            ..
        }) = self.states.get_mut(&id)
        {
            *value = new_value;
            if !dragging_this {
                *last_committed = new_value;
            }
            if !focused {
                buffer.clear();
                use std::fmt::Write;
                let _ = write!(buffer, "{}", format_number(new_value));
            }
        }
    }

    /// BulkSelect (T2.0): render a NumberInput as "Mixed" by blanking its
    /// displayed buffer (the underlying `value` / `last_committed` stay
    /// the primary's, so a focus-then-blur with no typing reverts cleanly
    /// — `commit_number_buffer` parses the empty buffer, fails, and
    /// restores `last_committed` WITHOUT emitting `ValueChanged`, so the
    /// diverging values are never stomped). No-op while this input is
    /// focused or being drag-scrubbed so it doesn't fight live input.
    pub fn blank_number_input(&mut self, id: NodeId) {
        if self.focus_id == Some(id) {
            return;
        }
        if matches!(self.number_input_drag.as_ref(), Some(d) if d.id == id) {
            return;
        }
        if let Some(InteractiveState::NumberInput { buffer, .. }) = self.states.get_mut(&id) {
            buffer.clear();
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
    fn blank_number_input_clears_buffer_but_keeps_value_and_respects_focus() {
        let ni = |v: f64| InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: v,
            buffer: format!("{v}"),
            caret: 0,
            last_committed: v,
            selection_anchor: None,
        };
        let mut store = store_with(&[(1, ni(42.0)), (2, ni(7.0))]);

        // BulkSelect "Mixed": blank the display, preserve value/committed.
        store.blank_number_input(NodeId(1));
        match store.get(NodeId(1)) {
            Some(InteractiveState::NumberInput {
                buffer,
                value,
                last_committed,
                ..
            }) => {
                assert!(buffer.is_empty(), "buffer not blanked: {buffer:?}");
                assert_eq!(*value, 42.0, "value must survive (clean blur revert)");
                assert_eq!(*last_committed, 42.0);
            }
            _ => panic!("not a NumberInput"),
        }

        // No-op while the field is focused (must not fight live typing).
        store.set_focus(Some(NodeId(2)));
        store.blank_number_input(NodeId(2));
        match store.get(NodeId(2)) {
            Some(InteractiveState::NumberInput { buffer, .. }) => {
                assert_eq!(buffer, "7", "focused field must not be blanked");
            }
            _ => panic!("not a NumberInput"),
        }
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
