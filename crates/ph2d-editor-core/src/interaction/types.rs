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
    /// Settings → Anthropic API key (P4, ADR-0061). A dedicated popover with a
    /// `TextInput` (`CTX_MENU_API_KEY_INPUT`) + a Save row
    /// (`CTX_MENU_API_KEY_SAVE`); painted by its own branch, like [`Self::SceneList`].
    SettingsApiKeySubmenu,
    /// LLM vector authoring (P4, ADR-0061): a centered prompt dialog with a
    /// `TextInput` (`CTX_MENU_VECTOR_PROMPT_INPUT`) + a Generate button
    /// (`CTX_MENU_VECTOR_PROMPT_GENERATE`). Opened by the shell (Cmd/Ctrl+Shift+G);
    /// Generate raises `EditorAction::GenerateVectorFromPrompt`.
    VectorPromptDialog,
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
    /// Right-clicked on a Direct-Select vertex (vector tool). Menu offers the
    /// 4 frozen vertex continuity kinds — Corner / Smooth / Asymmetric / Auto
    /// (`ph2d_vector_doc::VertexKind` Free/Mirror/Aligned/Auto). No payload: it
    /// applies to the live `VectorSelection`, which the secondary-click selected.
    /// The chrome handler routes the click into `HeroScreen.pending_vector_point_type`
    /// as a 0..=3 index; the shell drains it and calls
    /// `VectorDirectTool::set_selected_vertex_kind` (editor-core can't depend on
    /// the vector-doc crate, so the kind crosses the boundary as an index).
    VectorPointType,
}
