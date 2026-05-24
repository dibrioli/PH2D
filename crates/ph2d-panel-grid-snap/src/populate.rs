//! Grid Snap panel populate — ADR-0029 Phase C.4 port of the legacy
//! `ph2d_editor_core::grid_snap::panel::populate`.
//!
//! Registers the panel chrome (drag/resize/close), every Kind / Target
//! / Neighborhood / Orientation / Offset / Parity option button, the
//! Snap + ShowOverlay toggles, the opacity Slider, the color picker
//! swatch, the scrollbar thumb, and the universe of NumberInput slots
//! (per-kind config, AABB bounds, probe coords, snap magnetism).
//!
//! Default values come from `GridSnapState::default()` — populate is
//! a one-shot at boot.

use crate::ids;
use crate::state::meters_to_display;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::grid_snap::GridSnapState;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{
    ButtonState, GRID_SETTINGS_SCROLLBAR_ID, SliderOrientation, SliderState, TextInputState,
    ToggleState,
};

pub(crate) fn populate(store: &mut WidgetStore) {
    // Panel chrome — Blender drag/resize handles same as Widget Gallery.
    store.register(
        ids::GS_DRAG_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::GS_PANEL,
            kind: BlenderHitKind::DragHandle,
        },
    );
    store.register(
        ids::GS_RESIZE_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::GS_PANEL,
            kind: BlenderHitKind::ResizeHandle,
        },
    );
    // Scrollbar thumb — must be in the store as `Plain` so dispatch's
    // `is_focusable` lets the Down handler seed the scrollbar drag.
    store.register(GRID_SETTINGS_SCROLLBAR_ID, InteractiveState::Plain);
    // All selectors are segmented Button groups (style parity with
    // Inspector's Strategy switcher per Enio's 2026-05-15 redesign):
    // each option has its own NodeId registered as a Button; the
    // active option is painted with `ButtonState::Pressed` driven
    // from `state` at paint time.
    for id in [
        ids::GS_CLOSE,
        // Kind group (9 options).
        ids::GS_KIND_OPT_SQUARE,
        ids::GS_KIND_OPT_HEX,
        ids::GS_KIND_OPT_ISO,
        ids::GS_KIND_OPT_STAGGERED_SQ,
        ids::GS_KIND_OPT_STAGGERED_HEX,
        ids::GS_KIND_OPT_TRI,
        ids::GS_KIND_OPT_QUADTREE,
        ids::GS_KIND_OPT_VORONOI,
        ids::GS_KIND_OPT_CHUNKS,
        // Target group (5 SnapTarget modes).
        ids::GS_SNAP_CENTER,
        ids::GS_SNAP_INTERSECTION,
        ids::GS_SNAP_TARGET_OPT_CORNER,
        ids::GS_SNAP_TARGET_OPT_CENTER_AND_INTERSECTION,
        ids::GS_SNAP_TARGET_OPT_CENTER_INTERSECTION_AND_CORNERS,
        // Neighborhood groups (Square family Von4/Moore8; Tri Edge3/Vertex12).
        ids::GS_CFG_NEIGHBORHOOD_4,
        ids::GS_CFG_NEIGHBORHOOD_8,
        ids::GS_CFG_TRI_EDGE3,
        ids::GS_CFG_TRI_VERTEX12,
        // Hex Orientation (Pointy / Flat) segmented group.
        ids::GS_CFG_HEX_POINTY,
        ids::GS_CFG_HEX_FLAT,
        // Hex Offset (OddR / EvenR / OddQ / EvenQ) segmented group.
        ids::GS_CFG_HEX_OFFSET_ODDR,
        ids::GS_CFG_HEX_OFFSET_EVENR,
        ids::GS_CFG_HEX_OFFSET_ODDQ,
        ids::GS_CFG_HEX_OFFSET_EVENQ,
        // Stagger parity (Odd / Even) segmented group.
        ids::GS_CFG_STAGGER_PARITY_ODD,
        ids::GS_CFG_STAGGER_PARITY_EVEN,
        ids::GS_CFG_VORONOI_RESEED,
        // Layer-order group (In front / Behind).
        ids::GS_LAYER_IN_FRONT,
        ids::GS_LAYER_BEHIND,
        // Color swatch — clickable; opens BlenderColorPicker.
        ids::GS_COLOR_PICKER,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // Toggles — start values match GridSnapState::default().
    store.register(
        ids::GS_SNAP_ENABLED,
        InteractiveState::Toggle {
            state: ToggleState::Normal,
            on: false,
        },
    );
    store.register(
        ids::GS_SHOW_OVERLAY,
        InteractiveState::Toggle {
            state: ToggleState::Normal,
            on: true,
        },
    );
    // Opacity slider — value matches GridSnapState::default().opacity.
    store.register(
        ids::GS_OPACITY_SLIDER,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.75, // LITERAL-PX-OK: default opacity (normalized 0..1, not a UI metric)
            orientation: SliderOrientation::Horizontal,
        },
    );

    // NumberInputs — register one entry per reserved id, default
    // values pulled from GridSnapState::default() so paint stays
    // in sync on first frame.
    let defaults = GridSnapState::default();
    let number_specs: &[(NodeId, f64)] = &[
        (ids::GS_CFG_CELL_SIZE, defaults.square_cfg.cell_size as f64),
        (ids::GS_CFG_ISO_TILE_W, defaults.iso_cfg.tile_w as f64),
        (ids::GS_CFG_ISO_TILE_H, defaults.iso_cfg.tile_h as f64),
        (
            ids::GS_CFG_QT_MAX_PER_LEAF,
            defaults.quadtree_cfg.max_points_per_leaf as f64,
        ),
        (
            ids::GS_CFG_QT_MAX_DEPTH,
            defaults.quadtree_cfg.max_depth as f64,
        ),
        (
            ids::GS_CFG_VORONOI_SEED_COUNT,
            defaults.voronoi_cfg.seed_count as f64,
        ),
        (
            ids::GS_CFG_VORONOI_RNG_SEED,
            defaults.voronoi_cfg.rng_seed as f64,
        ),
        (
            ids::GS_CFG_VORONOI_LLOYD_ITERS,
            defaults.voronoi_cfg.lloyd_iterations as f64,
        ),
        (
            ids::GS_CFG_CHUNKS_SIZE,
            defaults.chunks_cfg.chunk_size_cells as f64,
        ),
        // Universal extras — values mirror the active kind's *Cfg
        // on each frame (apply_event keeps store in sync).
        (ids::GS_CFG_ORIGIN_X, 0.0),
        (ids::GS_CFG_ORIGIN_Y, 0.0),
        (
            ids::GS_CFG_SPACING_MAJOR,
            defaults.square_cfg.spacing_major as f64,
        ),
        // Grid color RGB — alpha stays implicit (controlled by the
        // opacity slider).
        (ids::GS_CFG_COLOR_R, defaults.color_rgba[0] as f64),
        (ids::GS_CFG_COLOR_G, defaults.color_rgba[1] as f64),
        (ids::GS_CFG_COLOR_B, defaults.color_rgba[2] as f64),
        // Snap subdivisions (sub-grid factor; rendering unaffected).
        (
            ids::GS_CFG_SNAP_SUBDIVISIONS,
            defaults.snap_subdivisions as f64,
        ),
        // Snap magnetism radius (world meters; 0 disables the gate).
        (
            ids::GS_CFG_SNAP_MAGNETISM_RADIUS,
            meters_to_display(defaults.snap_magnetism_radius),
        ),
        // Inspect probes A / B (X, Y) — user-editable from the panel.
        (ids::GS_PROBE_A_X, defaults.probe_a[0] as f64),
        (ids::GS_PROBE_A_Y, defaults.probe_a[1] as f64),
        (ids::GS_PROBE_B_X, defaults.probe_b[0] as f64),
        (ids::GS_PROBE_B_Y, defaults.probe_b[1] as f64),
        // Quadtree bounds + demo seeds.
        (
            ids::GS_CFG_QT_BOUNDS_MIN_X,
            defaults.quadtree_cfg.bounds.min[0] as f64,
        ),
        (
            ids::GS_CFG_QT_BOUNDS_MIN_Y,
            defaults.quadtree_cfg.bounds.min[1] as f64,
        ),
        (
            ids::GS_CFG_QT_BOUNDS_MAX_X,
            defaults.quadtree_cfg.bounds.max[0] as f64,
        ),
        (
            ids::GS_CFG_QT_BOUNDS_MAX_Y,
            defaults.quadtree_cfg.bounds.max[1] as f64,
        ),
        (
            ids::GS_CFG_QT_DEMO_POINTS,
            defaults.quadtree_cfg.demo_point_count as f64,
        ),
        (
            ids::GS_CFG_QT_DEMO_SEED,
            defaults.quadtree_cfg.demo_rng_seed as f64,
        ),
        // Voronoi bounds.
        (
            ids::GS_CFG_VORONOI_BOUNDS_MIN_X,
            defaults.voronoi_cfg.bounds.min[0] as f64,
        ),
        (
            ids::GS_CFG_VORONOI_BOUNDS_MIN_Y,
            defaults.voronoi_cfg.bounds.min[1] as f64,
        ),
        (
            ids::GS_CFG_VORONOI_BOUNDS_MAX_X,
            defaults.voronoi_cfg.bounds.max[0] as f64,
        ),
        (
            ids::GS_CFG_VORONOI_BOUNDS_MAX_Y,
            defaults.voronoi_cfg.bounds.max[1] as f64,
        ),
    ];
    for (id, value) in number_specs {
        store.register(
            *id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: *value,
                buffer: format_value(*value),
                caret: 0,
                last_committed: *value,
                selection_anchor: None,
            },
        );
    }
}

pub(crate) fn format_value(v: f64) -> String {
    // Integers render without a decimal point so step=1 fields read
    // clean ("4" vs "4.0"). Non-integer values keep 2 decimals.
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v:.2}")
    }
}
