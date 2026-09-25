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
    // Panel-level color tag (UI canon post-2026-05-24).
    store.register(ids::GS_TITLE_COLOR, InteractiveState::Plain);

    // Panel chrome — Blender drag/resize handles same as Widget Gallery.
    store.register(
        ph2d_editor_core::grid_snap::ids::GS_DRAG_HANDLE,
        InteractiveState::BlenderHit {
            parent: ph2d_editor_core::ids::GS_PANEL,
            kind: BlenderHitKind::DragHandle,
        },
    );
    store.register(
        ph2d_editor_core::grid_snap::ids::GS_RESIZE_HANDLE,
        InteractiveState::BlenderHit {
            parent: ph2d_editor_core::ids::GS_PANEL,
            kind: BlenderHitKind::ResizeHandle,
        },
    );
    store.register(
        ph2d_editor_core::grid_snap::ids::GS_RESIZE_HANDLE_BL,
        InteractiveState::BlenderHit {
            parent: ph2d_editor_core::ids::GS_PANEL,
            kind: BlenderHitKind::ResizeHandleBl,
        },
    );
    // ⭐ As quatro secções DOBRAM (2026-09-24) — cabeçalhos canónicos da casa, sem estado: o despacho
    //    dobra-os ao clique (`toggle_collapsed`). ⚠️ Uma chamada por id, e não um laço: a régua
    //    `the_painted_control_reaches_a_consumer` reconhece o despacho por KIND pelo texto
    //    `mark_collapsible_section(ids::X)`, e um id dentro de um array de laço lê-se como órfão.
    store.mark_collapsible_section(crate::ids::GS_SEC_KIND);
    store.mark_collapsible_section(crate::ids::GS_SEC_TARGET);
    store.mark_collapsible_section(crate::ids::GS_SEC_DISPLAY);
    store.mark_collapsible_section(ph2d_editor_core::grid_snap::ids::GS_INSPECT_HEADER);
    // Scrollbar thumb — must be in the store as `Plain` so dispatch's
    // `is_focusable` lets the Down handler seed the scrollbar drag.
    store.register(GRID_SETTINGS_SCROLLBAR_ID, InteractiveState::Plain);
    // All selectors are segmented Button groups (style parity with
    // Inspector's Strategy switcher per Enio's 2026-05-15 redesign):
    // each option has its own NodeId registered as a Button; the
    // active option is painted with `ButtonState::Pressed` driven
    // from `state` at paint time.
    for id in [
        ph2d_editor_core::grid_snap::ids::GS_CLOSE,
        // Kind group (9 options).
        ph2d_editor_core::grid_snap::ids::GS_KIND_OPT_SQUARE,
        ph2d_editor_core::grid_snap::ids::GS_KIND_OPT_HEX,
        ph2d_editor_core::grid_snap::ids::GS_KIND_OPT_ISO,
        ph2d_editor_core::grid_snap::ids::GS_KIND_OPT_STAGGERED_SQ,
        ph2d_editor_core::grid_snap::ids::GS_KIND_OPT_STAGGERED_HEX,
        ph2d_editor_core::grid_snap::ids::GS_KIND_OPT_TRI,
        ph2d_editor_core::grid_snap::ids::GS_KIND_OPT_QUADTREE,
        ph2d_editor_core::grid_snap::ids::GS_KIND_OPT_VORONOI,
        ph2d_editor_core::grid_snap::ids::GS_KIND_OPT_CHUNKS,
        // Target group (5 SnapTarget modes).
        ph2d_editor_core::grid_snap::ids::GS_SNAP_CENTER,
        ph2d_editor_core::grid_snap::ids::GS_SNAP_INTERSECTION,
        ph2d_editor_core::grid_snap::ids::GS_SNAP_TARGET_OPT_CORNER,
        ph2d_editor_core::grid_snap::ids::GS_SNAP_TARGET_OPT_CENTER_AND_INTERSECTION,
        ph2d_editor_core::grid_snap::ids::GS_SNAP_TARGET_OPT_CENTER_INTERSECTION_AND_CORNERS,
        // Neighborhood groups (Square family Von4/Moore8; Tri Edge3/Vertex12).
        ph2d_editor_core::grid_snap::ids::GS_CFG_NEIGHBORHOOD_4,
        ph2d_editor_core::grid_snap::ids::GS_CFG_NEIGHBORHOOD_8,
        ph2d_editor_core::grid_snap::ids::GS_CFG_TRI_EDGE3,
        ph2d_editor_core::grid_snap::ids::GS_CFG_TRI_VERTEX12,
        // Hex Orientation (Pointy / Flat) segmented group.
        ph2d_editor_core::grid_snap::ids::GS_CFG_HEX_POINTY,
        ph2d_editor_core::grid_snap::ids::GS_CFG_HEX_FLAT,
        // Hex Offset (OddR / EvenR / OddQ / EvenQ) segmented group.
        ph2d_editor_core::grid_snap::ids::GS_CFG_HEX_OFFSET_ODDR,
        ph2d_editor_core::grid_snap::ids::GS_CFG_HEX_OFFSET_EVENR,
        ph2d_editor_core::grid_snap::ids::GS_CFG_HEX_OFFSET_ODDQ,
        ph2d_editor_core::grid_snap::ids::GS_CFG_HEX_OFFSET_EVENQ,
        // Stagger parity (Odd / Even) segmented group.
        ph2d_editor_core::grid_snap::ids::GS_CFG_STAGGER_PARITY_ODD,
        ph2d_editor_core::grid_snap::ids::GS_CFG_STAGGER_PARITY_EVEN,
        ph2d_editor_core::grid_snap::ids::GS_CFG_VORONOI_RESEED,
        // Layer-order group (In front / Behind).
        ph2d_editor_core::grid_snap::ids::GS_LAYER_IN_FRONT,
        ph2d_editor_core::grid_snap::ids::GS_LAYER_BEHIND,
        // Color swatch — clickable; opens BlenderColorPicker.
        ph2d_editor_core::grid_snap::ids::GS_COLOR_PICKER,
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
        ph2d_editor_core::grid_snap::ids::GS_SNAP_ENABLED,
        InteractiveState::Toggle {
            state: ToggleState::Normal,
            on: false,
        },
    );
    store.register(
        ph2d_editor_core::grid_snap::ids::GS_SHOW_OVERLAY,
        InteractiveState::Toggle {
            state: ToggleState::Normal,
            on: true,
        },
    );
    // Opacity slider — value matches GridSnapState::default().opacity.
    store.register(
        ph2d_editor_core::grid_snap::ids::GS_OPACITY_SLIDER,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.75, // LITERAL-PX-OK: default opacity (normalized 0..1, not a UI metric)
            orientation: SliderOrientation::Horizontal,
        },
    );

    // NumberInputs — register one entry per reserved id, default
    // values pulled from GridSnapState::default() so paint stays
    // in sync on first frame. Specs list extracted into a separate
    // fn (Wave 11 §2.2) so `populate()` stays under the 200-LOC cap.
    let defaults = GridSnapState::default();
    for (id, value) in default_number_specs(&defaults) {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format_value(value),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
    }
    // A faixa que o COMMIT já enforça, entregue à lei do scrub — ver `crate::limits` para os
    // números e para o que eles custavam enquanto o arrasto não os conhecia (uma componente de
    // cor inteira em 5,1 px; as iterações de Lloyd em 0,16).
    for (id, (min, max, step)) in crate::limits::DECLARED {
        store.set_number_range(id, min, max, step);
    }
}

/// The default `(NodeId, value)` pairs for every NumberInput slot in
/// the Grid Snap panel. Pulled from `GridSnapState::default()` so
/// paint reads the live store on first frame without divergence.
fn default_number_specs(defaults: &GridSnapState) -> Vec<(NodeId, f64)> {
    vec![
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_CELL_SIZE,
            defaults.square_cfg.cell_size as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_ISO_TILE_W,
            defaults.iso_cfg.tile_w as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_ISO_TILE_H,
            defaults.iso_cfg.tile_h as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_QT_MAX_PER_LEAF,
            defaults.quadtree_cfg.max_points_per_leaf as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_QT_MAX_DEPTH,
            defaults.quadtree_cfg.max_depth as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_VORONOI_SEED_COUNT,
            defaults.voronoi_cfg.seed_count as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_VORONOI_RNG_SEED,
            defaults.voronoi_cfg.rng_seed as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_VORONOI_LLOYD_ITERS,
            defaults.voronoi_cfg.lloyd_iterations as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_CHUNKS_SIZE,
            defaults.chunks_cfg.chunk_size_cells as f64,
        ),
        // Universal extras — values mirror the active kind's *Cfg
        // on each frame (apply_event keeps store in sync).
        (ph2d_editor_core::grid_snap::ids::GS_CFG_ORIGIN_X, 0.0),
        (ph2d_editor_core::grid_snap::ids::GS_CFG_ORIGIN_Y, 0.0),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_SPACING_MAJOR,
            defaults.square_cfg.spacing_major as f64,
        ),
        // Grid color RGB — alpha stays implicit (controlled by the
        // opacity slider).
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_COLOR_R,
            defaults.color_rgba[0] as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_COLOR_G,
            defaults.color_rgba[1] as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_COLOR_B,
            defaults.color_rgba[2] as f64,
        ),
        // Snap subdivisions (sub-grid factor; rendering unaffected).
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_SNAP_SUBDIVISIONS,
            defaults.snap_subdivisions as f64,
        ),
        // Snap magnetism radius (world meters; 0 disables the gate).
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_SNAP_MAGNETISM_RADIUS,
            meters_to_display(defaults.snap_magnetism_radius),
        ),
        // Inspect probes A / B (X, Y) — user-editable from the panel.
        (
            ph2d_editor_core::grid_snap::ids::GS_PROBE_A_X,
            defaults.probe_a[0] as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_PROBE_A_Y,
            defaults.probe_a[1] as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_PROBE_B_X,
            defaults.probe_b[0] as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_PROBE_B_Y,
            defaults.probe_b[1] as f64,
        ),
        // Quadtree bounds + demo seeds.
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_QT_BOUNDS_MIN_X,
            defaults.quadtree_cfg.bounds.min[0] as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_QT_BOUNDS_MIN_Y,
            defaults.quadtree_cfg.bounds.min[1] as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_QT_BOUNDS_MAX_X,
            defaults.quadtree_cfg.bounds.max[0] as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_QT_BOUNDS_MAX_Y,
            defaults.quadtree_cfg.bounds.max[1] as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_QT_DEMO_POINTS,
            defaults.quadtree_cfg.demo_point_count as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_QT_DEMO_SEED,
            defaults.quadtree_cfg.demo_rng_seed as f64,
        ),
        // Voronoi bounds.
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_VORONOI_BOUNDS_MIN_X,
            defaults.voronoi_cfg.bounds.min[0] as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_VORONOI_BOUNDS_MIN_Y,
            defaults.voronoi_cfg.bounds.min[1] as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_VORONOI_BOUNDS_MAX_X,
            defaults.voronoi_cfg.bounds.max[0] as f64,
        ),
        (
            ph2d_editor_core::grid_snap::ids::GS_CFG_VORONOI_BOUNDS_MAX_Y,
            defaults.voronoi_cfg.bounds.max[1] as f64,
        ),
    ]
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
