//! Lightweight value types used by `WidgetStore` and the dispatcher.
//!
//! Extracted from [`super::state`] as part of the post-M14 refactor
//! (Track E2). These types are pure data — no methods that touch
//! widget state; they only describe **what** the interactive layer is
//! looking at (a context-menu request, a sticky-note record, the
//! sub-control of a blender picker that was hit).
//!
//! Re-exported via `interaction::mod` so external call-sites
//! (`crate::interaction::ContextMenuKind` etc.) keep working without
//! edits.

use ph2d_a11y::NodeId;
use ph2d_host::PointerButton;

/// A pending palette file-I/O request the picker dispatch raises and the host (shell) services by
/// opening a file dialog: [`Import`](Self::Import) loads + REPLACES the active palette,
/// [`Export`](Self::Export) saves it. The format is chosen from the picked file's extension.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PaletteIoKind {
    /// Load a palette file and replace the active swatches.
    Import,
    /// Save the active swatches to a palette file.
    Export,
}

/// Which sub-control of a [`super::InteractiveState::BlenderPicker`] a
/// [`super::InteractiveState::BlenderHit`] points at.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlenderHitKind {
    Wheel,
    ValueSlider,
    InterpolationLinear,
    InterpolationPerceptual,
    ChannelRgb,
    ChannelHsv,
    /// Switch the channel sliders to OKLCH (L/C/H/Alpha).
    ChannelOklch,
    /// One of the 4 horizontal channel sliders (R/G/B/A, H/S/V/A, or
    /// L/C/H/A). Index 0..3: 0 = R/H/L, 1 = G/S/C, 2 = B/V/H, 3 = A.
    ChannelSlider(u8),
    /// The hex `#RRGGBBAA` text input field.
    Hex,
    /// One swatch in the active palette. Index into the picker's
    /// store-side palette (see [`super::WidgetStore::blender_palette`]).
    /// Left-click picks the swatch; right-click removes it.
    PaletteSwatch(u8),
    /// "+ swatch" button at the end of the palette grid; clicking
    /// appends the picker's current value to the palette.
    AddSwatch,
    /// A palette TAB in the named-palette strip. Index into the picker's palette set
    /// ([`WidgetStore::blender_palette_set`]); clicking selects it as the active palette.
    PaletteTab(u8),
    /// "+ palette" button — appends a fresh empty "Palette N" and makes it active.
    NewPalette,
    /// "delete palette" button — removes the active palette (keeping at least one).
    DeletePalette,
    /// Palette-select dropdown chip — clicking toggles the popover listing every named
    /// palette (replaces the old cramped one-line tab strip; index = palette position).
    PaletteDropdown,
    /// "R" rename button next to the dropdown — toggles the inline rename field (a
    /// `TextInput` whose Enter commits the new active-palette name).
    RenamePalette,
    /// "×" close button at the top-right of the floating picker — clears the picker
    /// target (`WidgetStore::set_picker_target(None)`), dismissing the popover.
    Close,
    /// "Import" palette button — clicking flags a host file-dialog request
    /// (`WidgetStore::set_palette_io_pending`) to load a `.gpl`/`.hex`/`.ase`/
    /// `.aco` as a NEW named palette.
    ImportPalette,
    /// "Export" palette button — flags a host file-dialog request to save the
    /// active palette's swatches in the format the chosen extension selects.
    ExportPalette,
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
    /// Bottom-LEFT resize gripper. Mirror of [`ResizeHandle`]. Down
    /// begins a BL-mode resize; Move adjusts the parent's stored
    /// `(dw, dh)` AND `(dx, dy)` so the right edge stays put while
    /// the left edge follows the cursor; Up ends it. Lets the user
    /// grab the panel from either bottom corner.
    ResizeHandleBl,
    /// M14.6A: eye icon on a hierarchy row — toggles the entity's
    /// `Visibility` component. Parent NodeId on the `BlenderHit` is
    /// the row's id; dispatcher sets `HeroScreen.pending_visibility_toggle`
    /// for the host to drain and apply on `SimWorld`.
    VisibilityToggle,
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
    /// Clicked the TOPBAR Settings (gear) cluster. Top-level menu
    /// listing project-setting categories (Pixels per meter, etc.).
    /// Clicking a category opens its dedicated submenu (the parent
    /// gets replaced — simpler than a true cascade, same flow as
    /// macOS native preferences).
    SettingsMenu,
    /// Submenu opened when the user picks "Pixels per meter" from
    /// the top-level Settings menu. Shows the 5 canonical presets;
    /// selecting one writes `HeroScreen.project.pixels_per_meter`
    /// and closes the menu.
    SettingsPpmSubmenu,
    /// Submenu opened when the user picks "Display unit" — flips the
    /// formatted readouts in Inspector / Grid Settings / Gizmo
    /// between meters and pixels. Sim storage stays in meters; this
    /// only changes the FORMAT.
    SettingsUnitSubmenu,
    /// Submenu opened when the user picks "Image filter" — flips the
    /// app-wide [`crate::project::ImageFilterMode`] (Pixel Art /
    /// Smooth) applied to EVERY sprite/texture sample and the Vello
    /// preview. Selecting one writes `HeroScreen.project.image_filter`
    /// and raises `EditorAction::SetImageFilter` so the shell rebuilds
    /// the GPU samplers.
    SettingsFilterSubmenu,
    /// Submenu opened when the user picks "Display" — switches the
    /// swap-chain present mode at runtime: VSync (`Fifo`, perfectly
    /// smooth motion) vs Immediate (non-blocking, no mouse-stutter).
    /// Selecting one raises `EditorAction::SetPresentMode` so the shell
    /// reconfigures the surface.
    SettingsDisplaySubmenu,
    /// Submenu opened when the user picks "Text rendering" — switches
    /// the chrome text strategy between `Default` (historic AA-only)
    /// and `Crisp` (snap-X + per-tier FontWeight boost). Selecting one
    /// writes `HeroScreen.text_rendering`; the next frame's
    /// `set_text_rendering` publishes the choice to `paint_text*`.
    SettingsTextSubmenu,
    /// Color-picker palette rename: a centered modal with the shared name `TextInput`
    /// (`BLENDER_PALETTE_NAME`) + a Rename button (`CTX_MENU_PALETTE_RENAME`). Opened by the
    /// picker's "R" button; Rename / Enter commit `blender_rename_active_palette`, outside-click
    /// cancels. Single picker → applies to `INSP_BLENDER_PICKER`.
    RenamePaletteDialog,
    /// Clicked the TOPBAR Project chip. Menu offers a search input
    /// plus a filtered list of scene names; selecting a row updates
    /// the chip's label via `super::WidgetStore::current_scene_name`.
    SceneList,
    /// M14.6 F: right-clicked on a hierarchy row. Menu offers per-
    /// entity actions (Duplicate, Delete, Reset Transform, Add Child).
    /// The dispatcher routes each menu item click into a
    /// `HeroScreen.pending_*` slot keyed by `row`; the host drains
    /// those slots each frame and applies the ECS mutation. Rename
    /// is deferred (needs inline TextInput state-machine) and not
    /// surfaced in this menu yet.
    HierarchyRow { row: NodeId },
    /// New-image modal (Cmd/Ctrl+N): a centered dialog with a row of square-size buttons
    /// (`CTX_MENU_NEW_IMAGE_SIZES`) + background choices (`CTX_MENU_NEW_IMAGE_BGS`) + a Create button
    /// (`CTX_MENU_NEW_IMAGE_CREATE`). Create raises a `(size, bg)` request the shell services via
    /// `spawn_blank_canvas`; outside-click cancels. The selected size/bg live on the `WidgetStore`.
    NewImageDialog,
    /// Right-clicked on a Painter brush Falloff curve control point. Menu offers
    /// the two handle types — Vector (sharp corner) / Auto (smooth). No payload:
    /// the secondary-click already selected the point; the chrome handler routes
    /// the click into `HeroScreen.pending_falloff_point_handle` as the
    /// `HandleType` wire u8 (`0` = Auto, `1` = Vector). The shell drains it and
    /// calls `PainterTool::set_brush_falloff_point_handle` on the selected point
    /// (editor-core can't depend on the brush crate, so it crosses as a u8).
    FalloffPointHandle,
    /// Right-clicked on an on-canvas **Curve / Free Hand** editor control point.
    /// Menu offers the four handle continuity kinds — Free / Aligned / Vector /
    /// Auto. The secondary-click already selected the point; the chrome handler
    /// routes the click into `HeroScreen.pending_curve_point_handle` as the wire
    /// u8 (`0 = Free`, `1 = Aligned`, `2 = Vector`, `3 = Auto`). The shell drains
    /// it and calls `PainterTool::set_curve_handle_kind` (crosses as a u8 since
    /// editor-core can't depend on the tool crate).
    CurvePointHandle,
}

// ───────────────────────────── Motion Nodes M0.T2 ─────────────────────────────
// Foundational graph-surface dispatch types. Editor-core carries these through
// the pointer/key dispatch WITHOUT interpreting them: the motion-graph panel
// registers each hit target with a `GraphHitKind` (Motion Nodes plan §2.2) and
// reads the drained `GraphGesture`s back. Every element handle is an OPAQUE
// integer — editor-core knows no graph semantics (same "crosses as an integer"
// rule as the tool-side context-menu payloads above).

/// What a graph pointer gesture is aimed at. `node`/`edge`/`id` are the panel's
/// own opaque handles (it maps them back to its `MotionDoc`); `port`/`index` are
/// element ordinals. Editor-core never dereferences them — it only routes them
/// from the hit target back to the panel on the drained gesture.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GraphHitKind {
    /// Empty canvas. Left-drag pans (the documented middle-button fallback),
    /// Shift+drag box-selects, right-click opens the add menu.
    Background,
    /// A node body — drag moves it, click selects it.
    Node { node: u64 },
    /// An input socket — a wire dropped here connects to it.
    SocketIn { node: u64, port: u16 },
    /// An output socket — dragging from here begins a wire.
    SocketOut { node: u64, port: u16 },
    /// A wire/edge — click selects it, drag adds a waypoint (panel-side).
    Wire { edge: u64 },
    /// A wire waypoint handle.
    Waypoint { edge: u64, index: u16 },
    /// A backdrop rectangle — drag moves it (and the nodes it frames).
    Backdrop { id: u64 },
    /// A backdrop's resize gripper.
    BackdropResize { id: u64 },
    /// A node's preview/bypass toggle.
    PreviewToggle { node: u64 },
    /// The viewport⟂graph split divider (drag re-splits).
    SplitDivider,
    /// A graph chrome control (a toolbar chip) — click activates it. `id` is the
    /// panel's own opaque button ordinal (editor-core never interprets it), same
    /// "crosses as an integer" rule as the node/edge handles.
    Chrome { id: u16 },
}

/// Lifecycle phase of a graph pointer gesture. The panel drives its own state
/// machine (drag node / draw wire / box-select / pan) off the sequence.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GesturePhase {
    /// Pointer down on the target (a drag may follow).
    Begin,
    /// Pointer moved while captured — fires every frame, even once the pointer
    /// has left the target's rect (a node drag continues past the panel edge).
    Update,
    /// Pointer up after the gesture moved.
    End,
    /// Pointer up with no meaningful movement — a tap.
    Click,
    /// A second `Click` on the same target within the double-click window.
    /// Editor-core dispatch emits only `Begin`/`Update`/`End`/`Click` in M0
    /// (M0.T3); the panel upgrades consecutive `Click`s to `DoubleClick`. Kept
    /// in the vocabulary so the panel's match is total.
    DoubleClick,
}

/// Modifier snapshot for a gesture. Pointer events carry no modifiers, so these
/// come from the store's cached `shift_held`/`cmd_held`/`alt_held` (pushed by
/// the shell on `ModifiersChanged`).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct GestureMods {
    pub shift: bool,
    /// Cmd (macOS) OR Ctrl (Linux/Windows) — the "command" modifier.
    pub cmd: bool,
    pub alt: bool,
}

/// One graph pointer gesture, stashed by dispatch and drained by the panel each
/// frame ([`super::WidgetStore::drain_graph_gestures`]). Positions are global
/// pixels; the panel maps them into graph space with its own pan/zoom.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GraphGesture {
    /// The graph surface (panel) this gesture belongs to — the `parent` of the
    /// [`super::InteractiveState::GraphSurface`] that was hit.
    pub surface: NodeId,
    /// What was under the pointer at `Begin`, carried unchanged through
    /// Update/End/Click so the panel knows what it is dragging.
    pub kind: GraphHitKind,
    pub phase: GesturePhase,
    pub x: f32,
    pub y: f32,
    pub button: PointerButton,
    pub mods: GestureMods,
}

/// Accumulated anchored-zoom request for a graph surface (wheel over its
/// canvas), drained by the panel. `delta` sums the frame's wheel notches;
/// `(anchor_x, anchor_y)` is the cursor position the zoom should keep fixed.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GraphZoom {
    pub delta: f32,
    pub anchor_x: f32,
    pub anchor_y: f32,
}

/// A graph keyboard command, produced by `dispatch_key` when a graph surface
/// holds focus and drained by the panel. Editor-core maps the keycode; the
/// panel decides what each verb does.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GraphKey {
    /// Delete / Backspace — remove the selected nodes/edges.
    Delete,
    /// F — frame/fit the view to the graph (or selection).
    Fit,
    /// A — open the add-node menu.
    Add,
    /// Escape — cancel the in-progress gesture / clear selection.
    Escape,
    /// K — knife (cut wires).
    Knife,
    /// P — probe (inspect a node's output).
    Probe,
    /// Ctrl/Cmd+D — duplicate the selection.
    Duplicate,
}
