//! **Que menu cada coisa abre** — os tipos que o dispatch usa para escolher a TABELA.
//!
//! Split de `types.rs` sob o teto de 700 LOC, e uma unidade por direito próprio: um
//! `TimelineHitKind` diz *o que está sob o cursor*, e isto diz *que menu isso merece*.
//! São perguntas diferentes, e a segunda é a que cresce quando uma família de track
//! ganha uma ação que as outras não têm.

use ph2d_a11y::NodeId;

use super::types::*;

/// **Que família de track esta row é**, e portanto que menu o botão direito abre.
///
/// Vive aqui e não no `ph2d-timeline` porque é uma pergunta de INTERAÇÃO: o dispatch
/// precisa dela para escolher a tabela, e ele não conhece `PropKind` (nem deve).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackMenuKind {
    /// Rotation, Scale, Opacity — as ações comuns MAIS a extrapolação por-track
    /// (o menu comum agora oferece loopOut/cycle/pingpong/continue).
    Plain,
    /// `TranslationX`/`Y`: o modo de eixos SEPARADOS, que pode virar trajetória.
    Axis,
    /// `Position`: a trajetória, que pode virar eixos separados — e que tem
    /// auto-orient.
    Path,
    /// `TimeRemap`: só *Delete Track*. É amostrada por um relógio próprio, então a
    /// extrapolação por-track é inerte nela e o menu NÃO a oferece — daí uma tabela
    /// própria em vez de herdar a do `Plain` (uma linha morta em toda row de Time
    /// Remap é a doença que a forma "uma tabela por menu" existe pra impedir).
    TimeRemap,
}

// ── Context-menu vocabulary (moved from `types.rs` under the LOC cap) ─────────
// `ContextMenuKind` is literally *which menu a right-click opens* — the same
// subject as `TrackMenuKind` above — so it lives here, next to it.

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
    /// Submenu opened when the user picks "Motion" — os DOIS eixos da UI viva
    /// (`crate::motion`): o **carácter** (Expressivo / Discreto, um rádio) e o **reduced motion**
    /// (um toggle que se sobrepõe aos dois). Escolher escreve em `HeroScreen.motion`; a shell
    /// persiste ao notar a diferença (ver `shells/desktop/src/prefs.rs`).
    SettingsMotionSubmenu,
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
    /// **O modal de resolução da folha** (Enio 2026-08-19): abre ao escolher "Pack into Sheet" na
    /// hierarquia, e a folha só nasce no Create. Uma fila de resoluções quadradas
    /// (`CTX_MENU_SHEET_SIZES`) + o botão `CTX_MENU_SHEET_SIZE_CREATE`; clicar fora cancela.
    ///
    /// ⚠️ **O tamanho da folha passa a ser AUTORADO, e isso muda o re-arranjo:** antes ele
    /// redimensionava a folha para caber o encaixe, o que agora apagaria a escolha do artista. Ele
    /// encaixa DENTRO da resolução escolhida, e o que não couber acende a moldura — vide
    /// `sheet_bounds::health`.
    SheetSizeDialog,
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
    /// A **Time Remap** track menu (`ids::TIMELINE_TIMEREMAP_TRACK_MENU`): Delete
    /// Track only. Its own clock (`remap_through`) makes per-track extrapolation
    /// inert, so the menu omits the cascades — a table of its own.
    TimelineTrackTimeRemap { target: u64 },
    /// The extrapolation-mode submenu (plan §6), opened by a track menu's Pre/Post
    /// cascade. `target` is the raw `AnimTarget`; `side` is `0` = Pre, `1` = Post.
    /// The panel resolves it and raises `SetTrackExtrap`.
    TimelineExtrap { target: u64, side: u8 },
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

impl ContextMenuKind {
    /// **UMA AMOSTRA DE CADA VARIANTE** — a porta que torna este enum ENUMERÁVEL.
    ///
    /// # Por que ela existe
    ///
    /// Os gates que perguntam *«isto vale para TODO menu?»* — hoje o
    /// `every_painted_menu_row_is_registered_and_therefore_clickable` — precisavam de uma lista de
    /// tipos, e escreviam-na à mão. ⛔ **A escrita à mão tinha UMA variante de trinta**
    /// (`HierarchyRow`), então `SaveMenu`, `OpenMenu`, `SettingsMenu`, `ThemeSelector` e os dez
    /// menus da timeline eram invisíveis ao gate que dizia cobri-los todos — *um gate que precisa
    /// de ser actualizado para apanhar o caso novo não apanha caso novo nenhum*, que é exactamente
    /// o que o doc daquele gate promete não fazer.
    ///
    /// # A metade que impede ESTA lista de apodrecer também
    ///
    /// ⚠️ Uma lista de amostras é, ela própria, escrita à mão — mover o problema um nível acima não
    /// é curá-lo. Quem a fecha é o gate `the_sample_list_names_every_variant_of_the_enum`
    /// (`tests/every_menu_row_is_registered.rs`): ele lê o **fonte deste arquivo**, extrai os nomes
    /// das variantes do `enum`, e exige que cada um apareça aqui — pelo `Debug` das amostras, não
    /// por um segundo texto escrito à mão. Uma variante nova nasce vermelha no dia em que é
    /// declarada.
    ///
    /// ⚠️ **As três amostras de `TimelineSegment` não são redundância:** o `menu_rows` daquela
    /// variante despacha por `scope.menu_table()`, então `Key`, `Column` e `StripFade` pintam
    /// tabelas DIFERENTES — uma amostra só deixaria duas tabelas por medir. *A unidade que os
    /// gates varrem é a TABELA, e o tipo é apenas como se chega a ela.*
    ///
    /// ⚠️ Os payloads são inertes de propósito (`NodeId(1)`, `0`): nenhum consumidor desta lista
    /// resolve um alvo — eles perguntam pela FORMA do menu, e a forma não depende de em que linha
    /// o clique caiu.
    pub const ALL: &'static [Self] = &[
        Self::CreateNote {
            panel: NodeId(1),
            before_section: None,
        },
        Self::SectionOutline { section: NodeId(1) },
        Self::NoteBackground {
            panel: NodeId(1),
            note_index: 0,
        },
        Self::ThemeSelector,
        Self::SaveMenu,
        Self::OpenMenu,
        Self::SettingsMenu,
        Self::SettingsPpmSubmenu,
        Self::SettingsUnitSubmenu,
        Self::SettingsFilterSubmenu,
        Self::SettingsDisplaySubmenu,
        Self::SettingsTextSubmenu,
        Self::SettingsMotionSubmenu,
        Self::RenamePaletteDialog,
        Self::SceneList,
        Self::HierarchyRow { row: NodeId(1) },
        Self::NewImageDialog,
        Self::SheetSizeDialog,
        Self::FalloffPointHandle,
        Self::CurvePointHandle,
        Self::MotionPathAnchor { target: 0, i: 0 },
        Self::TimelineSegment {
            scope: TimelineInterpScope::Key { target: 0, key: 0 },
        },
        Self::TimelineSegment {
            scope: TimelineInterpScope::Column { t_bits: 0 },
        },
        Self::TimelineSegment {
            scope: TimelineInterpScope::StripFade {
                lane: 0,
                strip: 0,
                edge: 0,
            },
        },
        Self::TimelineSegmentEase {
            scope: TimelineInterpScope::Key { target: 0, key: 0 },
            mode: 0,
        },
        Self::TimelineTrack { target: 0 },
        Self::TimelineTrackAxis { target: 0 },
        Self::TimelineTrackPath { target: 0 },
        Self::TimelineTrackTimeRemap { target: 0 },
        Self::TimelineExtrap { target: 0, side: 0 },
        Self::TimelineLane { lane: 0 },
        Self::TimelineStrip { lane: 0, strip: 0 },
        Self::TimelineMarker { index: 0 },
    ];
}
