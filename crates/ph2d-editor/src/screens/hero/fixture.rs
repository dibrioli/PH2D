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
pub fn topbar_clusters() -> Vec<(ph2d_a11y::NodeId, TopBarCluster)> {
    use crate::screens::hero::ids;
    vec![
        (ids::TOPBAR_THEME, TopBarCluster::theme("Forge SDF")),
        (
            ids::TOPBAR_SAVE,
            TopBarCluster::single("Save", IconId::Save),
        ),
        (ids::TOPBAR_PROJECT, TopBarCluster::project("Level_01")),
        (ids::TOPBAR_PLAY_BUTTON, TopBarCluster::play()),
        (ids::TOPBAR_RIGHT_LAYERS, TopBarCluster::right()),
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
    }]
}

/// `(entities_count, components_count)` shown in the Hierarchy header.
pub fn hierarchy_counts() -> (u32, u32) {
    (1, 0)
}
