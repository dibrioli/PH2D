//! Inspector panel `populate` — pre-allocates Inspector-only widget
//! state slots in the `WidgetStore`. Called once at host boot via
//! `Panel::populate`.
//!
//! Other widgets historically registered in the same module (gallery
//! showcase samples, blender color picker, hierarchy chrome handles,
//! global context-menu items, scrollbars) remain in
//! `ph2d_editor_core::screens::hero::pre_populate` because they are
//! shared across panels / chrome layers and are not Inspector-specific.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{
    ButtonState, CheckboxState, CheckboxValue, DropdownState, SliderOrientation, SliderState,
    TextInputState,
};

pub fn populate(store: &mut WidgetStore) {
    populate_transform_editor(store);
    populate_visibility_editor(store);
    populate_render_strategy(store);
    populate_region(store);
    populate_sprite_flip(store);
    populate_color_tint(store);
    populate_sprite_sheet(store);
    populate_name_editor(store);
    populate_ordering(store);
    populate_sampling(store);
    populate_visibility_section(store);
    populate_blend(store);
    populate_physics(store);
    populate_joint(store);
}

/// W3 §8 Visibility section: register the segmented + bitmask + toggle ids
/// as `Button`s (is_focusable) and the cutoff/rect NumberInputs. Live
/// values come from the snapshot; defaults match the optional-component
/// "absent" state (cutoff 0.5, rect zero).
fn populate_visibility_section(store: &mut WidgetStore) {
    register_button_ids(store, &ids::INSP_VIS_CLIP);
    register_button_ids(store, &ids::INSP_VIS_MASK);
    register_button_ids(store, &ids::INSP_VIS_LAYER_BIT);
    register_button_ids(store, &[ids::INSP_VIS_MASK_SOURCE, ids::INSP_VIS_ON_SCREEN]);
    for (id, value) in [
        (ids::INSP_VIS_ALPHA_CUTOFF, 0.5_f64),
        (ids::INSP_VIS_RECT_X, 0.0_f64),
        (ids::INSP_VIS_RECT_Y, 0.0_f64),
        (ids::INSP_VIS_RECT_W, 0.0_f64),
        (ids::INSP_VIS_RECT_H, 0.0_f64),
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format_number(value),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
    }
    // Alpha Cutoff is a hard `0..1` mask threshold — drag-scrub spans the whole range (coherent with
    // its limits, like the texture number boxes; Enio 2026-06-26). The Rect fields are pixel extents
    // with no natural ceiling, so they keep the unbounded step-rate (no artificial clamp).
    store.set_number_range(ids::INSP_VIS_ALPHA_CUTOFF, 0.0, 1.0, 0.01); // LITERAL-PX-OK: alpha-cutoff chip 0..1 track step (non-design behaviour value)
}

/// Register the W3 segmented-tab + dropdown-option ids as `Button`s so
/// the pointer dispatcher routes their clicks (an unregistered hit id is
/// rejected by `is_focusable` and never emits `Click`). The selected
/// visual is snapshot-driven via `Tabs::selected`; these states exist
/// purely so the click reaches the event handler. §7 Sort Point tabs,
/// §7 Sorting Layer dropdown options, and the §9 Sampling tabs.
fn register_button_ids(store: &mut WidgetStore, ids: &[ph2d_a11y::NodeId]) {
    for &id in ids {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// §11 Physics Body (ADR-0131 D8): the two segmented groups and the two
/// buttons register as `Button`s (is_focusable → clicks route), the five
/// dimensions as `NumberInput`s.
///
/// **Every one gets a range**, and that is not decoration: `set_number_range`
/// is what makes drag-scrub proportional to the field's own span, and its
/// absence is the known gotcha (a `0..1` field dragged at the unbounded rate
/// crosses its whole domain in a few pixels). Bounce is physically `0..=1`;
/// friction above 1 is legal (it means "grips harder than it weighs"), so it
/// gets headroom rather than a false ceiling; density and the extents have no
/// natural maximum, so their caps are generous rather than meaningful.
/// §12 Physics Joint (W3). Registering these is what makes the widgets
/// FOCUSABLE — a control that is painted and hit-registered but never
/// registered here is dead under the mouse, and looks perfectly fine.
fn populate_joint(store: &mut WidgetStore) {
    register_button_ids(store, &ids::INSP_JOINT_KIND);
    register_button_ids(store, &ids::INSP_JOINT_LIMITS);
    register_button_ids(store, &ids::INSP_JOINT_MOTOR);
    register_button_ids(store, &[ids::INSP_JOINT_REMOVE, ids::INSP_PHYS_JOIN]);
    // Physical quantities again — degrees, meters, N·m, spring constants —
    // so none of these literals has a design token to come from.
    for (id, value, min, max, step) in [
        // A hinge limit is an angle; a full turn each way is the widest thing
        // that still means something.
        (ids::INSP_JOINT_LIMIT_MIN, -45.0, -360.0, 360.0, 1.0), // LITERAL-PX-OK: degrees
        (ids::INSP_JOINT_LIMIT_MAX, 45.0, -360.0, 360.0, 1.0),  // LITERAL-PX-OK: degrees
        // Motor speed in degrees/second, either direction.
        (ids::INSP_JOINT_MOTOR_SPEED, 114.0, -3600.0, 3600.0, 1.0), // LITERAL-PX-OK: deg/s
        (ids::INSP_JOINT_MOTOR_FORCE, 10.0, 0.0, 10000.0, 0.1),     // LITERAL-PX-OK: N·m ceiling
        (ids::INSP_JOINT_REST_LENGTH, 1.0, 0.0, 1000.0, 0.01),      // LITERAL-PX-OK: meters
        (ids::INSP_JOINT_STIFFNESS, 30.0, 0.0, 100000.0, 1.0), // LITERAL-PX-OK: spring constant
        (ids::INSP_JOINT_DAMPING, 0.5, 0.0, 1000.0, 0.1),      // LITERAL-PX-OK: damping constant
        (ids::INSP_JOINT_MAX_LENGTH, 1.0, 0.001, 1000.0, 0.01), // LITERAL-PX-OK: meters
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format_number(value),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
        store.set_number_range(id, min, max, step);
    }
}

fn populate_physics(store: &mut WidgetStore) {
    register_button_ids(store, &ids::INSP_PHYS_KIND);
    register_button_ids(store, &ids::INSP_PHYS_SHAPE);
    register_button_ids(store, &ids::INSP_PHYS_LAYER);
    register_button_ids(store, &ids::INSP_PHYS_SENSOR);
    register_button_ids(store, &ids::INSP_PHYS_CCD);
    register_button_ids(store, &ids::INSP_PHYS_LOCKROT);
    register_button_ids(store, &ids::INSP_PHYS_LOCKX);
    register_button_ids(store, &ids::INSP_PHYS_LOCKY);
    register_button_ids(store, &ids::INSP_PHYS_MASSMODE);
    register_button_ids(store, &ids::INSP_PHYS_REST_COMBINE);
    register_button_ids(store, &ids::INSP_PHYS_FRIC_COMBINE);
    register_button_ids(store, &ids::INSP_PHYS_DAMPMODE);
    register_button_ids(store, &ids::INSP_PHYS_ONEWAY);
    register_button_ids(store, &ids::INSP_PHYS_BAKE_CH);
    register_button_ids(
        store,
        &[
            ids::INSP_PHYS_ADD,
            ids::INSP_PHYS_BAKE,
            ids::INSP_PHYS_REMOVE,
        ],
    );
    // Every literal below is a PHYSICAL quantity — meters, kg/m², or a
    // dimensionless coefficient — not a design measurement, so none of them
    // has a token to come from.
    for (id, value, min, max, step) in [
        (ids::INSP_PHYS_RADIUS, 0.5, 0.001, 1000.0, 0.01), // LITERAL-PX-OK: meters
        (ids::INSP_PHYS_HALF_X, 0.5, 0.001, 1000.0, 0.01), // LITERAL-PX-OK: meters
        (ids::INSP_PHYS_HALF_Y, 0.5, 0.001, 1000.0, 0.01), // LITERAL-PX-OK: meters
        // The capsule's STRAIGHT segment: min 0.0, not 0.001 — a zero-segment
        // capsule is exactly a ball, which is the honest bottom of the range
        // (and what Ball -> Capsule converts to).
        (ids::INSP_PHYS_CAP_HALF_H, 0.25, 0.0, 1000.0, 0.01), // LITERAL-PX-OK: meters
        // Collider offset (signed — the offset is a POSITION, can go either way).
        // Bounds the drag only; the component/BodyDesc take any f32.
        (ids::INSP_PHYS_OFFSET_X, 0.0, -1000.0, 1000.0, 0.01), // LITERAL-PX-OK: meters
        (ids::INSP_PHYS_OFFSET_Y, 0.0, -1000.0, 1000.0, 0.01), // LITERAL-PX-OK: meters
        // Initial velocity (W9): signed. The range bounds the DRAG only —
        // the component/BodyDesc/rapier take any f32 — so it spans a sane
        // authoring range around zero, like gravity scale does.
        (ids::INSP_PHYS_LINVEL_X, 0.0, -100.0, 100.0, 0.1), // LITERAL-PX-OK: m/s
        (ids::INSP_PHYS_LINVEL_Y, 0.0, -100.0, 100.0, 0.1), // LITERAL-PX-OK: m/s
        (ids::INSP_PHYS_ANGVEL, 0.0, -3600.0, 3600.0, 1.0), // LITERAL-PX-OK: deg/s
        (ids::INSP_PHYS_DENSITY, 1.0, 0.0, 1000.0, 0.01),   // LITERAL-PX-OK: kg/m^2
        // Explicit mass override (W-Mass), Manual mode. min 0.001 (mass must be
        // positive — a zero-mass dynamic body is degenerate); the range bounds the
        // DRAG only, the component/BodyDesc take any positive f32.
        (ids::INSP_PHYS_MASS, 1.0, 0.001, 100000.0, 0.1), // LITERAL-PX-OK: kg
        (ids::INSP_PHYS_RESTITUTION, 0.0, 0.0, 1.0, 0.01), // LITERAL-PX-OK: bounciness is 0..=1 by physics
        (ids::INSP_PHYS_FRICTION, 0.5, 0.0, 10.0, 0.01), // LITERAL-PX-OK: Coulomb coefficient, >1 is legal
        // Per-body gravity multiplier (W8). The range bounds the DRAG only — the
        // component/`BodyDesc`/rapier take any f32 (a loaded project may carry
        // more); -10..10 covers the authoring span (balloon → 10× heavy), the
        // same soft-UI bound `RADIUS` uses.
        (ids::INSP_PHYS_GRAVITY_SCALE, 1.0, -10.0, 10.0, 0.1), // LITERAL-PX-OK: dimensionless gravity multiplier
        // Dominance (W-Dominance): a signed integer collision priority, step 1. The
        // range bounds the DRAG only — the component/BodyDesc take any i8, and the
        // event arm rounds+clamps; -10..10 covers the authoring span (rapier's i8
        // allows ±127, but a legible priority is a small number).
        (ids::INSP_PHYS_DOMINANCE, 0.0, -10.0, 10.0, 1.0), // LITERAL-PX-OK: integer collision priority
        // Per-body damping (drag), Dynamic-only (W-Damping). Default 0.0 (the world
        // default drag). The drag range bounds only the scrub; the component/BodyDesc
        // take any f32 >= 0. 0..10 mirrors the world drag's own MAX (`MAX_DAMPING`) —
        // a coefficient past 10 is essentially "instant stop".
        (ids::INSP_PHYS_LINEAR_DAMPING, 0.0, 0.0, 10.0, 0.05), // LITERAL-PX-OK: linear drag coefficient
        (ids::INSP_PHYS_ANGULAR_DAMPING, 0.0, 0.0, 10.0, 0.05), // LITERAL-PX-OK: angular drag coefficient
        // Force zone (W-Area): newtons, signed (a wind blows either way). The range
        // is generous because it is weighed against a body's MASS — a 20 kg crate
        // needs ~200 N just to hold it against gravity.
        (ids::INSP_PHYS_FORCE_X, 0.0, -1000.0, 1000.0, 0.5), // LITERAL-PX-OK: newtons
        (ids::INSP_PHYS_FORCE_Y, 0.0, -1000.0, 1000.0, 0.5), // LITERAL-PX-OK: newtons
        // Torque zone (W-AreaTorque): N·m, signed (the sign is the spin direction). The
        // linear/rotational pair with Force, so it wears the same generous signed range —
        // weighed against the body's MOMENT OF INERTIA, which is smaller than its mass, so
        // a modest number already spins a compact body briskly.
        (ids::INSP_PHYS_AREA_TORQUE, 0.0, -1000.0, 1000.0, 0.5), // LITERAL-PX-OK: N*m
        // The medium's resistance. Same range as the other drag knobs in this app —
        // it is the same law, so it must be the same numbers.
        (ids::INSP_PHYS_AREA_DRAG, 0.0, 0.0, 10.0, 0.05), // LITERAL-PX-OK: drag coefficient
        // Densidade do fluido: MESMA faixa do `Density` do collider, porque a
        // comparação entre os dois é justamente a leitura (menor boia, maior afunda).
        (ids::INSP_PHYS_AREA_DENSITY, 0.0, 0.0, 1000.0, 0.05), // LITERAL-PX-OK: kg/m^2
        // Resistência de forma: mesma faixa dos outros arrastos, porque a leitura do
        // artista é comparar os dois knobs de resistência lado a lado.
        (ids::INSP_PHYS_AREA_FORM_DRAG, 0.0, 0.0, 10.0, 0.05), // LITERAL-PX-OK: coef. de forma
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format_number(value),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
        store.set_number_range(id, min, max, step);
    }
}

/// §10 Material & Blend: register the 6 blend-mode segmented ids as
/// `Button`s (is_focusable → clicks route). Selection is snapshot-driven.
fn populate_blend(store: &mut WidgetStore) {
    register_button_ids(store, &ids::INSP_SAMPLE_BLEND);
}

fn populate_sampling(store: &mut WidgetStore) {
    register_button_ids(store, &ids::INSP_SAMPLE_FILTER);
    register_button_ids(store, &ids::INSP_SAMPLE_REPEAT);
    register_button_ids(
        store,
        &[
            ids::INSP_ORDER_SP_CENTER,
            ids::INSP_ORDER_SP_PIVOT,
            ids::INSP_ORDER_SP_CUSTOM,
        ],
    );
    register_button_ids(store, &ids::INSP_ORDER_LAYER_OPT);
    // UV tiling/scroll NumberInputs (scale default 1.0, offset 0.0).
    for (id, value) in [
        (ids::INSP_SAMPLE_UV_SCALE_X, 1.0_f64),
        (ids::INSP_SAMPLE_UV_SCALE_Y, 1.0_f64),
        (ids::INSP_SAMPLE_UV_OFFSET_X, 0.0_f64),
        (ids::INSP_SAMPLE_UV_OFFSET_Y, 0.0_f64),
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format_number(value),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
    }
}

/// W3 Sprite Inspector v2 §7 Ordering / Sorting: 7 toggles + 2 integer
/// NumberInputs. Defaults match the optional-component "absent" state
/// (everything off, Z as Relative on per Godot). Live values sync from
/// the snapshot.
fn populate_ordering(store: &mut WidgetStore) {
    for (id, on) in [
        (ids::INSP_ORDER_Z_RELATIVE, true),
        (ids::INSP_ORDER_SHOW_BEHIND, false),
        (ids::INSP_ORDER_YSORT_ENABLED, false),
        (ids::INSP_ORDER_SORTING_GROUP, false),
        (ids::INSP_ORDER_SORT_AT_ROOT, false),
        (ids::INSP_ORDER_TOP_LEVEL, false),
    ] {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: if on {
                    CheckboxValue::Checked
                } else {
                    CheckboxValue::Unchecked
                },
            },
        );
    }
    for id in [
        ids::INSP_ORDER_Z_INDEX,
        ids::INSP_ORDER_ORDER_IN_LAYER,
        ids::INSP_ORDER_AXIS_X,
        ids::INSP_ORDER_AXIS_Y,
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: format!("{:.0}", 0.0),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
    }
    // Sorting Layer dropdown (default = "Default" layer index 2).
    store.register(
        ids::INSP_ORDER_SORTING_LAYER,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(2),
        },
    );
}

/// W2 Sprite Inspector v2 Sprite Sheet grid: Centered toggle (default
/// on) + Offset X/Y (default 0) + HFrames / VFrames (default 1) + Frame
/// (default 0). Live values sync from the snapshot.
fn populate_sprite_sheet(store: &mut WidgetStore) {
    store.register(
        ids::INSP_SPRITE_CENTERED,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Checked,
        },
    );
    for id in [ids::INSP_SPRITE_OFFSET_X, ids::INSP_SPRITE_OFFSET_Y] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: format!("{:.0}", 0.0),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
    }
    for (id, value) in [
        (ids::INSP_SPRITE_HFRAMES, 1.0_f64),
        (ids::INSP_SPRITE_VFRAMES, 1.0_f64),
        (ids::INSP_SPRITE_FRAME, 0.0_f64),
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format!("{value:.0}"),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
    }
}

/// W2 Sprite Inspector v2 Color & Tint controls: Opacity Slider (0..1
/// storage, default 1.0) with a linked percent chip (0..100), + Tint Fill
/// checkbox (default off). Live values sync from the snapshot.
fn populate_color_tint(store: &mut WidgetStore) {
    // Opacity Slider 0..1 + linked chip showing 0..100 % (spec §3.6).
    store.register(
        ids::INSP_SPRITE_OPACITY,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 1.0,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        ids::INSP_SPRITE_OPACITY_CHIP,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 100.0, // LITERAL-PX-OK: opacity percent scale (1.0 → 100 %), not a design token
            buffer: format_number(100.0), // LITERAL-PX-OK: opacity percent scale
            caret: 0,
            last_committed: 100.0, // LITERAL-PX-OK: opacity percent scale
            selection_anchor: None,
        },
    );
    // chip_display = slider_storage * 100 (+0); integer-snapped so the
    // chip is whole percents while the slider track stays continuous.
    store.link_slider_number_mapped_integer(
        ids::INSP_SPRITE_OPACITY,
        ids::INSP_SPRITE_OPACITY_CHIP,
        100.0, // LITERAL-PX-OK: opacity percent scale (slider 0..1 → chip 0..100)
        0.0,
    );
    // Opacity is a hard `0..100 %` — drag-scrub on the chip spans the whole range proportionally
    // (coherent with its limits, like the texture number boxes; Enio 2026-06-26).
    store.set_number_range(ids::INSP_SPRITE_OPACITY_CHIP, 0.0, 100.0, 1.0); // LITERAL-PX-OK: opacity percent scale
    store.register(
        ids::INSP_SPRITE_TINT_FILL,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Unchecked,
        },
    );
    // Tint / Self Tint + 4 per-corner color swatches. Registered as
    // `Plain` (like the section color-dots in `pre_populate` and
    // grid-snap's swatch) so `is_focusable` is true and the pointer
    // dispatch arms `active` on Down → emits `Click` on Up. Without this
    // leg the click is silently dropped and the picker never opens (the
    // swatch carries no value of its own — its color lives in the
    // `widget_colors` side-table).
    for id in [
        ids::INSP_SPRITE_TINT_SWATCH,
        ids::INSP_SPRITE_SELF_TINT_SWATCH,
        ids::INSP_SPRITE_CORNER_TL,
        ids::INSP_SPRITE_CORNER_TR,
        ids::INSP_SPRITE_CORNER_BL,
        ids::INSP_SPRITE_CORNER_BR,
    ] {
        store.register(id, InteractiveState::Plain);
    }
    // "Equalize corners" button (copies TL → the other three).
    store.register(
        ids::INSP_SPRITE_CORNER_EQUALIZE,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    // (The Color & Tint sub-tabs were retired 2026-05-31 — the section
    // now stacks every control visible at once, so no tab Button group.)
}

/// W2 Sprite Inspector v2: Flip H / Flip V checkboxes. Default
/// Unchecked (the Sprite default `flip_x = flip_y = false`); the live
/// value is synced from the snapshot each frame in `sync.rs`.
fn populate_sprite_flip(store: &mut WidgetStore) {
    for id in [ids::INSP_SPRITE_FLIP_X, ids::INSP_SPRITE_FLIP_Y] {
        store.register(
            id,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Unchecked,
            },
        );
    }
}

/// W2 Sprite Inspector v2 — Region sampling (Render Source section,
/// spec §3.3): enable toggle (default off) + 4 px NumberInputs (x/y/w/h,
/// default 0) + filter-clip toggle (default ON, the Atlas anti-bleed
/// default). Live values sync from the snapshot.
fn populate_region(store: &mut WidgetStore) {
    store.register(
        ids::INSP_REGION_ENABLED,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Unchecked,
        },
    );
    for id in [
        ids::INSP_REGION_X,
        ids::INSP_REGION_Y,
        ids::INSP_REGION_W,
        ids::INSP_REGION_H,
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: format!("{:.0}", 0.0),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
    }
    store.register(
        ids::INSP_REGION_FILTER_CLIP,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Checked,
        },
    );
}

fn populate_name_editor(store: &mut WidgetStore) {
    store.register(
        ids::INSP_ENTITY_NAME,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
}

fn populate_render_strategy(store: &mut WidgetStore) {
    for id in [
        ids::INSP_RENDER_STRATEGY_ATLAS,
        ids::INSP_RENDER_STRATEGY_INDIVIDUAL,
        ids::INSP_RENDER_STRATEGY_HANDPACKED,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    store.register(
        ids::INSP_RENDER_FORMAT_RGBA8,
        InteractiveState::Button {
            state: ButtonState::Pressed,
        },
    );
    store.register(
        ids::INSP_RENDER_FORMAT_RGBA16,
        InteractiveState::Button {
            state: ButtonState::Disabled,
        },
    );
    store.register(
        ids::INSP_RENDER_SOURCE_REIMPORT,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

fn populate_visibility_editor(store: &mut WidgetStore) {
    store.register(
        ids::INSP_VISIBILITY_CHECK,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Checked,
        },
    );
}

fn populate_transform_editor(store: &mut WidgetStore) {
    let identity_pairs = [
        (ids::INSP_TRANSFORM_POS_X, 0.0_f64),
        (ids::INSP_TRANSFORM_POS_Y, 0.0_f64),
        (ids::INSP_TRANSFORM_ROT, 0.0_f64),
        (ids::INSP_TRANSFORM_SCALE_X, 1.0_f64),
        (ids::INSP_TRANSFORM_SCALE_Y, 1.0_f64),
        (ids::INSP_TRANSFORM_SKEW_X, 0.0_f64),
        (ids::INSP_TRANSFORM_SKEW_Y, 0.0_f64),
    ];
    for (id, value) in identity_pairs {
        let buffer = format!("{value}");
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer,
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
    }
    store.register(
        ids::INSP_TRANSFORM_RESET,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}
