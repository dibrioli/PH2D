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

/// Top-bar pill clusters in left-to-right order. Each pill carries
/// 1-3 icon-only chips.
pub fn topbar_clusters() -> Vec<TopBarCluster> {
    vec![
        TopBarCluster::theme("Forge SDF"),
        TopBarCluster::single("Save", IconId::Save),
        TopBarCluster::project("Level_01"),
        TopBarCluster::play(),
        TopBarCluster::right(),
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

/// Inspector "field" row — label + slider value or other body kind.
#[derive(Clone, Debug)]
pub struct InspectorField {
    pub label: String,
    pub kind: InspectorFieldKind,
}

#[derive(Clone, Debug)]
pub enum InspectorFieldKind {
    /// Slider with formatted value display ("160", "0.0010", etc).
    Slider { value: f32, display: String },
    /// Dropdown with pre-selected option text.
    Select { current: String },
    /// Linked-input pill ("Union.distance", etc).
    Linked { source: String },
    /// Empty "+ link" placeholder + slider beneath.
    LinkedSlider { value: f32, display: String },
}

#[derive(Clone, Debug)]
pub struct InspectorSection {
    pub label: String,
    pub count: u32,
    /// `Some(open)` enables collapsibility; `None` = always expanded.
    pub collapsible: Option<bool>,
    pub fields: Vec<InspectorField>,
}

pub fn inspector_sections() -> Vec<InspectorSection> {
    vec![
        InspectorSection {
            label: String::from("Params (compile-time)"),
            count: 12,
            collapsible: None,
            fields: vec![
                InspectorField {
                    label: String::from("Move Speed"),
                    kind: InspectorFieldKind::Slider {
                        value: 0.62,
                        display: String::from("160"),
                    },
                },
                InspectorField {
                    label: String::from("Jump Height"),
                    kind: InspectorFieldKind::Slider {
                        value: 0.30,
                        display: String::from("200"),
                    },
                },
                InspectorField {
                    label: String::from("Friction"),
                    kind: InspectorFieldKind::Slider {
                        value: 0.08,
                        display: String::from("0.0010"),
                    },
                },
                InspectorField {
                    label: String::from("Damping"),
                    kind: InspectorFieldKind::Slider {
                        value: 0.48,
                        display: String::from("0.70"),
                    },
                },
                InspectorField {
                    label: String::from("Debug"),
                    kind: InspectorFieldKind::Select {
                        current: String::from("Shading (PBR-lite)"),
                    },
                },
            ],
        },
        InspectorSection {
            label: String::from("Advanced"),
            count: 7,
            collapsible: Some(false),
            fields: Vec::new(),
        },
        InspectorSection {
            label: String::from("Inputs"),
            count: 24,
            collapsible: None,
            fields: vec![
                InspectorField {
                    label: String::from("Distance"),
                    kind: InspectorFieldKind::Linked {
                        source: String::from("Union.distance"),
                    },
                },
                InspectorField {
                    label: String::from("Material"),
                    kind: InspectorFieldKind::Linked {
                        source: String::from("Materializer 3D.material"),
                    },
                },
                InspectorField {
                    label: String::from("Cam Yaw"),
                    kind: InspectorFieldKind::LinkedSlider {
                        value: 0.57,
                        display: String::from("0.57"),
                    },
                },
                InspectorField {
                    label: String::from("Cam Pitch"),
                    kind: InspectorFieldKind::LinkedSlider {
                        value: 0.0,
                        display: String::new(),
                    },
                },
            ],
        },
    ]
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
