//! Hardcoded mockup content for the editor hero screen.
//!
//! Mirrors the labels/numbers shown in `02-editor-main.html`. None
//! of this is wired to a real entity store — the pilot project that
//! drives ECS integration replaces these constructors with live
//! state queries.

use super::HeroSelection;
use crate::icons::IconId;

/// "Player · PRF · 124, −48" — the default selected entity.
pub fn default_selection() -> HeroSelection {
    HeroSelection {
        label: String::from("Player"),
        kind: String::from("PRF"),
        world_pos: (124.0, -48.0),
    }
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

/// Mockup hierarchy listing — Player + 4 children + 7 root entities.
pub fn hierarchy() -> Vec<HierarchyEntity> {
    vec![
        HierarchyEntity {
            name: "Player".into(),
            icon: IconId::Cube,
            indent: 0,
            badge: Some("OUT".into()),
            swatch: None,
            visible: true,
            selected: true,
            muted: false,
        },
        HierarchyEntity {
            name: "Sprite_idle".into(),
            icon: IconId::Sprite,
            indent: 1,
            badge: None,
            swatch: Some([220, 90, 200, 255]),
            visible: true,
            selected: false,
            muted: false,
        },
        HierarchyEntity {
            name: "Collider_box".into(),
            icon: IconId::Collider,
            indent: 1,
            badge: Some("UNI".into()),
            swatch: Some([100, 130, 220, 255]),
            visible: true,
            selected: false,
            muted: false,
        },
        HierarchyEntity {
            name: "Script_player".into(),
            icon: IconId::Script,
            indent: 1,
            badge: Some("UNI".into()),
            swatch: Some([220, 200, 80, 255]),
            visible: true,
            selected: false,
            muted: false,
        },
        HierarchyEntity {
            name: "RigidBody".into(),
            icon: IconId::Rigid,
            indent: 1,
            badge: Some("UNI".into()),
            swatch: Some([130, 200, 130, 255]),
            visible: true,
            selected: false,
            muted: false,
        },
        HierarchyEntity {
            name: "Tilemap_ground".into(),
            icon: IconId::Grid,
            indent: 0,
            badge: None,
            swatch: None,
            visible: true,
            selected: false,
            muted: false,
        },
        HierarchyEntity {
            name: "Tilemap_decor".into(),
            icon: IconId::Grid,
            indent: 0,
            badge: None,
            swatch: None,
            visible: true,
            selected: false,
            muted: false,
        },
        HierarchyEntity {
            name: "Slime_01".into(),
            icon: IconId::Cube,
            indent: 0,
            badge: Some("PRF".into()),
            swatch: None,
            visible: true,
            selected: false,
            muted: false,
        },
        HierarchyEntity {
            name: "Slime_02".into(),
            icon: IconId::Cube,
            indent: 0,
            badge: Some("PRF".into()),
            swatch: None,
            visible: false,
            selected: false,
            muted: true,
        },
        HierarchyEntity {
            name: "Trigger_zoneA".into(),
            icon: IconId::Bolt,
            indent: 0,
            badge: None,
            swatch: None,
            visible: true,
            selected: false,
            muted: false,
        },
        HierarchyEntity {
            name: "Ambient_light".into(),
            icon: IconId::Light,
            indent: 0,
            badge: None,
            swatch: None,
            visible: true,
            selected: false,
            muted: false,
        },
        HierarchyEntity {
            name: "Main_Camera".into(),
            icon: IconId::Camera,
            indent: 0,
            badge: Some("CAM".into()),
            swatch: None,
            visible: true,
            selected: false,
            muted: false,
        },
    ]
}

/// `(entities_count, components_count)` shown in the Hierarchy header.
pub fn hierarchy_counts() -> (u32, u32) {
    (12, 8)
}
