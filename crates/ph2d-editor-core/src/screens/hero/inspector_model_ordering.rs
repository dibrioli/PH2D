//! **O modelo da §7 Ordering / Sorting do Inspector** — snapshot, flags de divergência e edits.
//!
//! ⚠️ **Irmão de [`super::inspector_model`] por CAP de LOC** (700): aquele ficheiro chegou a 718 ao
//! documentar os achados da auditoria de 2026-08-21. *Cortar para o irmão é a cura.* Mesmo padrão
//! de `inspector_model_joint.rs` / `_physics.rs` / `_player.rs`, e o corte é por família.

/// Snapshot of the selected entity's W3 ordering/sorting components
/// published to the Inspector §7 (Ordering / Sorting). Every field is
/// *optional* (the components are presence-overrides, spec §02): `None`
/// / `false` markers mean "component absent → pipeline default". Raw
/// primitives keep editor-core loose-coupled from `ph2d-ecs`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InspectorOrderingInfo {
    pub entity_bits: u64,
    /// `ZIndexOverride` — `None` = absent (DFS counter). `Some(v)` =
    /// forced Z (spec §3.7: "Z Index: —" vs explicit).
    pub z_index: Option<i32>,
    /// `ZAsRelative.0` — only meaningful when `z_index.is_some()`.
    pub z_as_relative: bool,
    /// `ShowBehindParent` marker present.
    pub show_behind_parent: bool,
    /// `SortingLayer.0.0` (LayerId index); default-layer index when absent.
    pub sorting_layer: u8,
    /// `OrderInLayer.0`.
    pub order_in_layer: i32,
    /// `YSort.enabled` (false when the component is absent).
    pub y_sort_enabled: bool,
    /// `YSort.sort_point` as a tag: 0 Center · 1 Pivot · 2 Custom.
    pub y_sort_point: u8,
    /// `YSort.axis` (only meaningful when `y_sort_point == 2`).
    pub y_sort_axis: [f32; 2],
    /// `SortingGroup` present.
    pub sorting_group: bool,
    /// `SortingGroup.sort_at_root` (only meaningful when `sorting_group`).
    pub sort_at_root: bool,
    /// `TopLevel` marker present.
    pub top_level: bool,
    pub selected_count: usize,
    pub mixed: InspectorOrderingMixed,
}

/// BulkSelect (T2.0) divergence flags for the §7 ordering fields.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct InspectorOrderingMixed {
    pub z_index: bool,
    pub z_as_relative: bool,
    pub show_behind_parent: bool,
    pub sorting_layer: bool,
    pub order_in_layer: bool,
    pub y_sort_enabled: bool,
    pub y_sort_point: bool,
    pub y_sort_axis: bool,
    pub sorting_group: bool,
    pub sort_at_root: bool,
    pub top_level: bool,
}

/// A single editable §7 ordering field, dispatched Inspector → shell as
/// [`EditorAction::InspectorOrderingEdit`]. Unlike [`SpriteFieldEdit`]
/// (which mutates the always-present `Sprite`), each variant maps to an
/// *optional* ECS component: the shell reads the component-or-default,
/// applies the edit, and commits via `EditorCommand::SetComponent`
/// (insert/update) or `EditorCommand::RemoveComponent` (detach). The
/// full set is declared up front so the action contract is stable; only
/// wired controls emit today (spec §3.7).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OrderingFieldEdit {
    /// `Some(v)` attaches/updates `ZIndexOverride(v)` (clamped to
    /// ±i32::MAX/2 at commit); `None` detaches it (back to DFS).
    ZIndex(Option<i32>),
    /// `ZAsRelative(b)` (attaches the component if absent).
    ZAsRelative(bool),
    /// Toggle the `ShowBehindParent` marker (insert / remove).
    ShowBehindParent(bool),
    /// `SortingLayer(LayerId(idx))`.
    SortingLayer(u8),
    /// `OrderInLayer(v)`.
    OrderInLayer(i32),
    /// `YSort.enabled` (read-modify-write the YSort component).
    YSortEnabled(bool),
    /// `YSort.sort_point` tag: 0 Center · 1 Pivot · 2 Custom.
    YSortPoint(u8),
    /// `YSort.axis`.
    YSortAxis([f32; 2]),
    /// Toggle `SortingGroup` presence (insert default / remove).
    SortingGroup(bool),
    /// `SortingGroup.sort_at_root` (attaches the component if absent).
    SortAtRoot(bool),
    /// Toggle the `TopLevel` marker (insert / remove).
    TopLevel(bool),
}
