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

// The BlenderColorPicker's own hit vocabulary (`PaletteIoKind`, `BlenderHitKind`)
// lives in the sibling `types_blender` (HR-18 LOC cap) and is re-exported here so
// `types::BlenderHitKind` and `interaction::BlenderHitKind` keep resolving.
pub use super::types_blender::{BlenderHitKind, PaletteIoKind};

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
    /// Right-clicked an on-canvas **motion-path anchor** (ADR-0141): Corner / Smooth /
    /// Symmetric, mirroring the vector Node `VertexKind`. The anchor has no persistent
    /// selection, so its identity rides in the menu (`target` bits + index `i`, like
    /// `TimelineTrackPath`); the chrome handler parks `(target, i, kind)` (wire u8 `0/1/2`)
    /// in `HeroScreen.pending_motion_path_handle` for the shell → `set_path_tangent_kind`.
    MotionPathAnchor { target: u64, i: u32 },
    /// Right-clicked a timeline key (its dope-sheet diamond or its graph anchor),
    /// or a Summary column. Menu offers the presets for the interpolation LEAVING
    /// the keys in `scope` (W3.E4): Hold / Linear / three easing cascades /
    /// Custom.
    TimelineSegment { scope: TimelineInterpScope },
    /// The easing-family submenu of [`ContextMenuKind::TimelineSegment`], opened
    /// by one of its three cascade rows. `mode` is the wire encoding of the mode
    /// that row stands for (`ids::TL_EASE_MODE_*`); the shell pairs it with the
    /// clicked family id. editor-core never names an easing.
    TimelineSegmentEase {
        scope: TimelineInterpScope,
        mode: u8,
    },
    /// Right-clicked a timeline track row's LABEL (the left name column). Menu
    /// offers whole-track actions (`ids::TIMELINE_TRACK_MENU`, currently Delete
    /// Track). `target` is the row's raw `AnimTarget` — opaque here; the
    /// timeline panel resolves it against its snapshot and raises the intent.
    TimelineTrack { target: u64 },
    /// O menu de uma track de **EIXO** (`TranslationX`/`Y`): tem *Convert to Motion
    /// Path* (ADR-0141 §5), que canal nenhum dos outros tem.
    TimelineTrackAxis { target: u64 },
    /// O mesmo, para uma track de **TRAJETÓRIA** (`PropKind::Position`): o menu dela
    /// tem duas linhas a mais — o **Auto-Orient** e o *Convert to Separate Axes*
    /// (ADR-0141) —, que não existem em canal nenhum dos outros. Variante própria e não
    /// um campo, porque o overlay dispacha a TABELA por variante, e é a tabela que
    /// difere.
    TimelineTrackPath { target: u64 },
    /// Right-clicked a stack lane's LABEL (`ids::TIMELINE_LANE_MENU`): how the
    /// lane enters the blend, and whether it stays at all.
    TimelineLane {
        /// Index into the document's stack.
        lane: usize,
    },
    /// Right-clicked a clip strip in a stack lane (`ids::TIMELINE_STRIP_MENU`).
    /// Both fields are opaque here — the timeline panel resolves them against its
    /// snapshot, exactly as it does for [`ContextMenuKind::TimelineTrack`].
    TimelineStrip {
        /// Which lane the strip is on.
        lane: usize,
        /// The strip's stable id (`ph2d_timeline::StripId`), NOT its index: a lane
        /// re-sorts on every move, so an index parked in a menu request would name
        /// a different strip by the time the click arrives.
        strip: u64,
    },
    /// Right-clicked a marker pennant on the ruler (`ids::TIMELINE_MARKER_MENU`,
    /// ADR-0143). Menu offers a marker's whole EDIT surface — Rename / Set Signal /
    /// Delete — the three verbs that used to hide behind double-click,
    /// Shift+double-click and Alt+click (Enio, 2026-07-25: *"todas as opções de
    /// marker no menu do botão direito"*). A plain click still seeks and a drag
    /// still moves, so those stay off the menu. `index` is the marker's storage
    /// index, opaque here — the timeline panel keys its document by it (like
    /// [`Self::TimelineLane`] carries a lane index).
    TimelineMarker { index: usize },
}

/// What a timeline preset pick applies to. Both variants are opaque here —
/// editor-core carries the ids and never dereferences them (same contract as
/// [`TimelineHitKind`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TimelineInterpScope {
    /// One key, named by its track's raw `AnimTarget` and its raw `KeyId`. The
    /// shell widens this to the whole selection when the key is part of it.
    Key { target: u64, key: u64 },
    /// Every key at one time, across every track — the Summary channel's column.
    /// `t_bits` is `f64::to_bits` of the time in seconds.
    Column { t_bits: u64 },
}

/// A preset the user picked from the timeline segment menu, parked on
/// [`crate::HeroScreen`] for the shell to drain (mirror of
/// `pending_curve_point_handle`).
///
/// `item` is the clicked menu row's id and `mode` the easing mode it inherited
/// from its cascade (`u8::MAX` for the rows that carry no mode). editor-core
/// deliberately stops here: turning `(item, mode)` into an `Interp` needs the
/// `ph2d-anim` vocabulary, which lives shell-side with the document.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TimelineInterpPick {
    /// The keys this pick retunes.
    pub scope: TimelineInterpScope,
    /// The clicked row (`ids::CTX_MENU_TL_*`).
    pub item: NodeId,
    /// `ids::TL_EASE_MODE_*`, or `u8::MAX` when the row is not a family.
    pub mode: u8,
}

/// `mode` value for a pick that carries no easing mode (Hold / Linear / Custom).
pub const TL_NO_EASE_MODE: u8 = u8::MAX;

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
    /// Space — toggle transport play/pause (so time-driven behaviours animate).
    TogglePlay,
    /// Ctrl/Cmd+G — collapse the selection into a subgraph. Blender's and Nuke's
    /// chord for this verb, in both cases.
    Group,
    /// Ctrl/Cmd+Alt+G — dissolve the selected subgraph (or the one being edited).
    /// Blender's *Ungroup* and Nuke's *Expand Group* are BOTH this chord.
    Ungroup,
    /// **F2 — rename the selected card, group or backdrop** (doc 61). The one key every
    /// editor that has the verb at all binds it to (Windows, Blender, Houdini, Nuke).
    Rename,
}

/// A hit target inside the general timeline's dope-sheet surface. Mirror of
/// [`GraphHitKind`] scoped to the timeline panel: dispatch captures one on Down
/// and streams [`TimelineGesture`]s to the panel, never interpreting it. The
/// handles are opaque — the panel maps `target`/`key` back to its
/// `ph2d-timeline` document.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TimelineHitKind {
    /// Empty lane / dope-sheet background — a click clears the selection.
    Lane,
    /// A key diamond on some track — click selects it, drag moves the selection
    /// in time. `target`/`key` are the track's `AnimTarget` and the key's
    /// `KeyId`, carried as raw ids (editor-core never dereferences them).
    Key {
        target: u64,
        key: u64,
    },
    /// A **time-scale grip** at one edge of the current key selection's time
    /// extent (the loved retiming box — §4 of the crown-jewels plan). `right`
    /// picks the edge: `false` is the LEFT/start grip (pivot is the RIGHT edge),
    /// `true` is the RIGHT/end grip (pivot is the LEFT edge). Dragging scales the
    /// selected keys' TIME about the opposite edge — a KEY edit
    /// (`ScaleSelectedKeys`), NEVER a strip stretch (`StretchStrip` lives on the
    /// stack lane, a different surface). Painted just OUTSIDE the extent so the
    /// grip never fights the edge diamond underneath it.
    SelectionTimeHandle {
        right: bool,
    },
    /// A diamond on the **Summary** channel: the one row that aggregates every
    /// track's keys by time. Clicking it selects the whole column; dragging moves
    /// it. `t_bits` is the column's time in seconds, as `f64::to_bits` — an
    /// opaque, exact, hashable handle (editor-core never reads it as a number).
    SummaryKey {
        t_bits: u64,
    },
    /// A track row's expand/collapse twirl — a click opens that track's graph
    /// editor (`target` is its raw `AnimTarget`).
    Twirl {
        target: u64,
    },
    /// A track row's LABEL (the name column after the twirl). Inert to primary
    /// gestures — it exists so a right-click can open the track-row context
    /// menu ([`ContextMenuKind::TimelineTrack`], Delete Track).
    Row {
        target: u64,
        /// **Qual menu esta row abre** — cada família de track tem ações que as outras
        /// não têm.
        ///
        /// ⚠️ O fato viaja no HIT porque só o painel o sabe (o dispatch não conhece
        /// `PropKind`). A alternativa — um menu só, com as linhas de todas as famílias —
        /// seria um item morto na maioria das rows, que é precisamente o que a forma
        /// "uma tabela, N consumidores" destes menus existe para impedir.
        menu: super::TrackMenuKind,
    },
    /// The padlock on the Summary channel — a click toggles the column lock. When
    /// locked (default), grabbing any single key grabs its whole time column;
    /// unlocked, keys move independently. Panel-local view state, no payload.
    SummaryLock,
    /// The vertical splitter between the track-name column and the time area;
    /// dragging it horizontally widens or narrows the names.
    LabelSplitter,
    /// A key's anchor dot in an expanded track's graph. Dragging it edits the key
    /// in the `(time, value)` plane: sideways moves the selection in time,
    /// up/down retunes the value. `target`/`key` are the raw `AnimTarget`/`KeyId`.
    CurveAnchor {
        target: u64,
        key: u64,
    },
    /// A bézier handle in an expanded track's graph. `target`/`key` name the
    /// segment's **outgoing** key (the one whose `Interp` the handle edits);
    /// `which` is `0` for the out handle (`P1`) and `1` for the in handle (`P2`).
    CurveHandle {
        target: u64,
        key: u64,
        which: u8,
    },
    /// The grip along the bottom of an expanded track's graph band — dragging it
    /// vertically sets the height of every graph band.
    GraphResize,
    /// A loop-range brace on the ruler. `edge` is `0` = start handle, `1` = end
    /// handle, `2` = the band between them (drag to move the whole range).
    /// Dragging sets the playback loop range; the panel maps pointer x to time.
    LoopBrace {
        edge: u8,
    },
    /// A marker pennant on the ruler, keyed by its storage `index`. A plain click
    /// seeks the playhead to it, Alt+click removes it, a drag moves it.
    Marker {
        index: usize,
    },
    /// A resize gripper on the panel's border. `edges` is a bitmask of
    /// [`TIMELINE_EDGE_L`] .. [`TIMELINE_EDGE_B`] (a corner sets two). The panel
    /// owns the resize itself; the bits are named here only so the shell can pick
    /// the matching double-arrow cursor.
    /// One clip strip on a stack lane (ADR-0115). `edge`: `0` = the start edge,
    /// `1` = the end edge, `2` = the body — the same encoding [`Self::LoopBrace`]
    /// uses, because it is the same gesture (a span with two grabbable ends).
    ///
    /// `strip` is an opaque `StripId`, carried through without interpretation —
    /// the same way [`Self::SummaryKey`] carries a time. Editor-core does not know
    /// what a clip is, and should not.
    Strip {
        /// Which lane it sits on.
        lane: usize,
        /// The strip's stable id.
        strip: u64,
        /// `0` start, `1` end, `2` body.
        edge: u8,
    },
    /// A stack lane's LABEL strip — the surface a right-click turns into the lane
    /// menu (mode, Delete Lane). It carries no left-button gesture: a lane header
    /// is a place to point at, not a thing to drag.
    ///
    /// NOT [`Self::Lane`], which is the dope sheet's empty row (where a marquee
    /// starts). Two different things called "lane" is the timeline's own doing —
    /// a track row and a stack lane — and the names must keep them apart.
    LaneHeader {
        /// Which stack lane.
        lane: usize,
    },
    ResizeEdge {
        edges: u8,
    },
    /// **The grip at the veil's left edge on the ruler** — the authored-duration
    /// end. Dragging it left/right sets the view's explicit duration (the AE comp
    /// end), snapped like the playhead; the panel maps pointer x to time and picks
    /// the scope (clip / scene / container) the same way the Dur(s) chip does. No
    /// payload — a duration has one value per view, resolved panel-side. The shell
    /// gives it a ↔ resize cursor (`timeline_resize_cursor`).
    DurationHandle,
    /// **A container's bar in the Containers list** (ADR-0133, amended 2026-07-21) — one rect,
    /// and a double-click on it enters that container.
    ///
    /// Not a [`Self::Strip`]: a strip is a SPAN (two grabbable ends, a lane, a position in
    /// time) and this is a LABEL for a document asset. Giving it its own kind rather than a
    /// strip with the edges left out is what keeps "it cannot be resized, moved or trimmed"
    /// (Enio, 2026-07-21) a property of the type instead of a rule four call sites have to
    /// remember.
    ContainerRow {
        /// Index into `TimelineDoc::containers`.
        index: usize,
    },
}

impl TimelineHitKind {
    /// **Whether a second tap on this surface upgrades to [`GesturePhase::DoubleClick`]** —
    /// asked by the dispatcher's Up, answered HERE so the list of double-click consumers
    /// lives beside the variants a new one is added to.
    ///
    /// ⚠️ This was an inline enumeration in `pointer_up.rs` (*"only a MARKER tap upgrades"*),
    /// written when the marker's rename was the timeline's only double-click. The second
    /// consumer — the Containers list, whose bars are ENTERED by double-click — shipped
    /// painted, registered and routed, and was dead under the mouse, because the enumeration
    /// nobody remembered swallowed the upgrade before the panel ever saw it (Enio,
    /// 2026-07-21: *"ainda não consigo entrar no container com duplo clique"*). Same disease
    /// as the Painter's heading warm-up: a reader list far from its variants rots on the
    /// first new reader.
    ///
    /// ⚠️ The MARKER was dropped from this list (2026-07-25): its double-click / Shift+double-click
    /// authoring moved to the right-click menu ([`ContextMenuKind::TimelineMarker`], Rename / Set
    /// Signal / Delete), so a second tap on a pennant is now a plain Click (it re-seeks) — and if it
    /// still upgraded, `marker_drag` no longer arms anything on `DoubleClick`.
    ///
    /// Every other surface keeps treating a second tap as a plain Click ON PURPOSE — a key
    /// diamond re-selects, an empty lane re-clears; upgrading them would change behaviour
    /// nobody asked for.
    #[must_use]
    pub fn wants_double_click(self) -> bool {
        matches!(self, Self::ContainerRow { .. })
    }
}

/// Left edge bit of [`TimelineHitKind::ResizeEdge`].
pub const TIMELINE_EDGE_L: u8 = 1 << 0;
/// Right edge bit.
pub const TIMELINE_EDGE_R: u8 = 1 << 1;
/// Top edge bit.
pub const TIMELINE_EDGE_T: u8 = 1 << 2;
/// Bottom edge bit.
pub const TIMELINE_EDGE_B: u8 = 1 << 3;

/// Accumulated wheel input over a timeline surface, drained by the panel. Three
/// independent axes, split by modifier the way a dope sheet expects (plain =
/// zoom, Ctrl = horizontal pan, Shift = vertical scroll):
///
/// - `zoom_delta` — anchored zoom of the time axis, holding the time under
///   `anchor_x` fixed.
/// - `pan_delta` — slide the time axis.
/// - `scroll_delta` — scroll the track rows.
///
/// All are in the shell's **logical pixels** (it scales line-deltas by ~16 px
/// per notch). Editor-core assigns no direction; the panel applies the house
/// convention (a positive delta scrolls content right/down, i.e. the view moves
/// *earlier*, mirroring `panel_scroll - delta_y`). The counterpart of
/// [`GraphZoom`].
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct TimelineWheel {
    /// Summed wheel px meaning zoom (positive = zoom in).
    pub zoom_delta: f32,
    /// Summed wheel px meaning horizontal pan of the time axis.
    pub pan_delta: f32,
    /// Summed wheel px meaning vertical scroll of the track rows.
    pub scroll_delta: f32,
    /// Cursor x (global px) the anchored zoom keeps fixed.
    pub anchor_x: f32,
}

/// One timeline dope-sheet pointer gesture, stashed by dispatch and drained by
/// the panel each frame ([`super::WidgetStore::drain_timeline_gestures`]).
/// Positions are global pixels; the panel maps them to time with its own
/// view (`view_start`/`px_per_s`). Mirror of [`GraphGesture`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TimelineGesture {
    /// The timeline surface (panel) this gesture belongs to — the `parent` of
    /// the [`super::InteractiveState::TimelineSurface`] that was hit.
    pub surface: NodeId,
    /// What was under the pointer at `Begin`, carried unchanged through
    /// Update/End/Click.
    pub kind: TimelineHitKind,
    pub phase: GesturePhase,
    pub x: f32,
    pub y: f32,
    pub button: PointerButton,
    pub mods: GestureMods,
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod timeline_hit_kind_tests;
