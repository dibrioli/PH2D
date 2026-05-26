//! Hardcoded mockup content for the editor hero screen.
//!
//! Mirrors the labels/numbers shown in `02-editor-main.html`. None
//! of this is wired to a real entity store — the pilot project that
//! drives ECS integration replaces these constructors with live
//! state queries.

use super::HeroSelection;
use crate::icons::IconId;

/// Default selection is empty — the working hero starts without any
/// entity selected. The pilot project wires real scene state and
/// updates this via `HeroScreen::selection`.
pub fn default_selection() -> HeroSelection {
    HeroSelection::default()
}

/// Top-bar pill clusters in left-to-right order, paired with the
/// `NodeId` used in [`crate::screens::hero::ids`] for hit-test +
/// store lookup.
///
/// Layout note: `paint_top_bar` splits this list at index 4 — the
/// first 4 clusters paint left-aligned; the rest paint right-aligned
/// against the viewport edge. New entries should respect that split
/// (8-cluster default: 4 left + 4 right).
pub fn topbar_clusters() -> Vec<(ph2d_a11y::NodeId, TopBarCluster)> {
    use crate::screens::hero::ids;
    vec![
        (ids::TOPBAR_THEME, TopBarCluster::theme("Forge")),
        // Level / scene selector — moved to the LEFT side 2026-05-24
        // (user: "o seletor de level deve ser deslocado para esquerda
        // ao lado do seletor de themes") so the engine identity +
        // current scene sit together as one identity block.
        (ids::TOPBAR_PROJECT, TopBarCluster::project("Level_01")),
        (
            ids::TOPBAR_SAVE,
            TopBarCluster::single("Save", IconId::Save),
        ),
        (
            ids::TOPBAR_OPEN,
            TopBarCluster::single("Open", IconId::Open),
        ),
        // Image Tools — toggle entry-point for the image-editing
        // action row (Trim, BG Removal, Equalize, etc.). Placed
        // immediately to the right of Open per ImageToolsV1 spec.
        // Click is currently no-op (visual only); modal toggle +
        // action row land in a follow-up.
        (
            ids::TOPBAR_IMAGE_TOOLS,
            TopBarCluster::single("IMG", IconId::Image),
        ),
        (ids::TOPBAR_PLAY_BUTTON, TopBarCluster::play()),
        (ids::TOPBAR_RIGHT_LAYERS, TopBarCluster::right()),
        // Widget Gallery (palette) — toggles a floating reference
        // panel that hosts the canonical widget showcase. Peripheral
        // agents run `./play.command`, open this gallery, and see
        // every editor widget in use as the single in-app UI
        // ground-truth. State on `HeroScreen::widget_gallery_visible`.
        (
            ids::TOPBAR_WIDGET_GALLERY,
            TopBarCluster::single("WIDGET", IconId::Palette),
        ),
        // Grid Settings — opens the floating Grid Settings panel
        // (grid-snap subsystem: 9 grid kinds + snap policy + overlay
        // controls). Mirrors the Widget Gallery pattern (panel-only
        // subsystem, NOT a Tool, NOT in the LeftRail).
        (
            ids::TOPBAR_GRID_SETTINGS,
            TopBarCluster::single("GRID", IconId::GridSettings),
        ),
        // Settings (gear) — moved to the end of the bar per
        // ImageToolsV1 spec. Still opens SettingsMenu context with
        // px/m presets and project-level toggles.
        (
            ids::TOPBAR_SETTINGS,
            TopBarCluster::single("SETTINGS", IconId::Settings),
        ),
    ]
}

#[derive(Clone, Debug)]
pub enum TopBarCluster {
    /// Theme dropdown chip (placeholder text only — no real menu v1).
    Theme { label: String },
    /// Single-icon pill.
    Single { label: String, icon: IconId },
    /// Project pill: folder icon + project name.
    Project { name: String },
    /// Theme-mode toggle + play button.
    Play,
    /// Right-cluster: layers, asset library, code.
    Right,
}

impl TopBarCluster {
    fn theme(label: &str) -> Self {
        Self::Theme {
            label: label.into(),
        }
    }
    fn single(label: &str, icon: IconId) -> Self {
        Self::Single {
            label: label.into(),
            icon,
        }
    }
    fn project(name: &str) -> Self {
        Self::Project { name: name.into() }
    }
    fn play() -> Self {
        Self::Play
    }
    fn right() -> Self {
        Self::Right
    }
}

#[derive(Clone, Debug)]
pub struct HierarchyEntity {
    pub name: String,
    pub icon: IconId,
    pub indent: u8,
    pub badge: Option<String>,
    pub swatch: Option<[u8; 4]>,
    pub visible: bool,
    pub selected: bool,
    pub muted: bool,
    /// Mirrors `ph2d_ecs::Locked` — entity's own transform is locked
    /// against gizmo edits. Renders a lock icon next to the eye.
    pub locked: bool,
    /// Mirrors `ph2d_ecs::GroupedChildren` — all descendants are
    /// locked while THIS entity remains editable. Renders a folder
    /// icon in the right-side cluster.
    pub group_locked: bool,
}

/// Placeholder hierarchy until the pilot project wires real entities.
/// Returns a single `Scene Root` row so the panel chrome stays
/// readable. The pilot replaces this with a live ECS query.
pub fn hierarchy() -> Vec<HierarchyEntity> {
    vec![HierarchyEntity {
        name: "Scene Root".into(),
        icon: IconId::Folder,
        indent: 0,
        badge: None,
        swatch: None,
        visible: true,
        selected: false,
        muted: false,
        locked: false,
        group_locked: false,
    }]
}

/// `(entities_count, components_count)` shown in the Hierarchy header.
pub fn hierarchy_counts() -> (u32, u32) {
    (1, 0)
}

/// Placeholder scene list shown in the project-chip Scene List
/// popover. Replaced by the pilot project's real scene index.
pub fn scenes() -> &'static [&'static str] {
    &[
        "Level_01",
        "Level_02",
        "Level_03",
        "Main_Menu",
        "Title_Screen",
        "Boss_Arena",
        "Credits",
        "Test_Sandbox",
    ]
}
