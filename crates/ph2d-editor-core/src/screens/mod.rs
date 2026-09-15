//! Composed full-viewport editor screens.
//!
//! ADR-0029 Phase B.2 moved `HeroScreen` + chrome + fixture from
//! `ph2d-editor::screens` into `ph2d-editor-core::screens`. The
//! `screens` module name (plural) was preserved from that old
//! `ph2d_editor::screens::*` path. The `ph2d-editor` shim that kept it
//! resolving was DELETED on 2026-09-12 (architecture audit A4).
//!
//! Each module here paints one of the canonical mockups at
//! `docs/design/screens/`. Screens compose the widget primitives
//! into editor chrome (TopBar / LeftRail / Inspector / Hierarchy /
//! BottomHUD) over a canvas background — they DON'T own engine
//! state. Caller wires real entities/scene state when integrating
//! the screen into a project.

pub mod center_split;
pub mod dock_seam;
pub mod dock_sides;
pub mod hero;
pub mod layout;
pub mod slot;
pub mod slot_layout;
pub mod task_layout;

pub use hero::{
    ActionFieldEdit, AnchorFieldEdit, AnimFieldEdit, AudioFieldEdit, BlendFieldEdit,
    BottomHudStats, CameraFieldEdit, EMISSIVE_MAX_UI, HeroScreen, HeroSelection,
    InspectorActionInfo, InspectorActionRow, InspectorAnchorInfo, InspectorAnchorRow,
    InspectorAnimInfo, InspectorAnimRow, InspectorAudioInfo, InspectorAudioSource,
    FactoryFieldEdit, InspectorBlendInfo, InspectorBlendMixed, InspectorCameraFollow,
    InspectorCameraInfo, InspectorFactory, InspectorFactoryInfo, InspectorLifecycle,
    InspectorSpawnWhere,
    InspectorCameraLimits, InspectorGameCamera, InspectorJointInfo, InspectorNameInfo,
    InspectorOrderingInfo, InspectorOrderingMixed, InspectorPhysicsInfo, InspectorPlayerInfo,
    InspectorSamplingInfo, InspectorSamplingMixed, InspectorSliceInfo, InspectorSliceMixed,
    InspectorSpriteInfo, InspectorSpriteMixed, InspectorSpriteSource, InspectorTagRow,
    InspectorTagsInfo, InspectorTimerInfo, InspectorTimerRow, InspectorTransformInfo,
    InspectorVisibilityInfo, InspectorVisibilityMixed, InspectorVisibilitySectionInfo,
    InspectorWheelInfo, JointFieldEdit, OrderingFieldEdit, PhysicsFieldEdit, PlayerFieldEdit,
    RequestedSpriteStrategy, SamplingFieldEdit, SliceFieldEdit, SpriteFieldEdit, TimerFieldEdit,
    ViewFocusKind, VisibilityFieldEdit, WheelFieldEdit, paint_hero_screen,
};
pub use layout::{
    EDGE_PAD, HERO_VIEWPORT_H, HERO_VIEWPORT_W, HIERARCHY_W, HUD_BOTTOM_PAD, HUD_H, HeroLayout,
    INSPECTOR_W, TOPBAR_GAP, TOPBAR_H, rail_w,
};
