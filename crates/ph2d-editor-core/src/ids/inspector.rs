use super::*;

// ── Inspector Transform editor (M14.A) ──────────────────────────────────────
// Live binding for `ph2d_ecs::Transform` on the selected entity. The
// section paints when `HeroScreen::inspector_transform` is `Some` and
// commits via `pending_transform_edit` → `EditorCommandQueue` at the
// shell boundary (first real consumer of the editor command pipeline).
// Z is intentionally hidden — `Transform` is 2D by design (SKILL §3,
// ADR-0025); X/Y NumberInputs only, with R/G axis-color labels.
/// Collapsible section header for the Transform editor.
pub const INSP_TRANSFORM_SECTION: NodeId = hash_node_id("insp_transform_section");
/// Position X NumberInput (meters, R-tinted label).
pub const INSP_TRANSFORM_POS_X: NodeId = hash_node_id("insp_transform_pos_x");
/// Position Y NumberInput (meters, G-tinted label).
pub const INSP_TRANSFORM_POS_Y: NodeId = hash_node_id("insp_transform_pos_y");
/// Rotation NumberInput (displayed in degrees; stored in radians).
pub const INSP_TRANSFORM_ROT: NodeId = hash_node_id("insp_transform_rot");
/// Scale X NumberInput (unitless, R-tinted label).
pub const INSP_TRANSFORM_SCALE_X: NodeId = hash_node_id("insp_transform_scale_x");
/// Scale Y NumberInput (unitless, G-tinted label).
pub const INSP_TRANSFORM_SCALE_Y: NodeId = hash_node_id("insp_transform_scale_y");
/// Skew X NumberInput (degrees in UI, R-tinted label; ADR-0025-amendment-1).
pub const INSP_TRANSFORM_SKEW_X: NodeId = hash_node_id("insp_transform_skew_x");
/// Skew Y NumberInput (degrees in UI, G-tinted label).
pub const INSP_TRANSFORM_SKEW_Y: NodeId = hash_node_id("insp_transform_skew_y");
/// Reset-to-Identity button in the Transform section header.
pub const INSP_TRANSFORM_RESET: NodeId = hash_node_id("insp_transform_reset");

// ── Inspector Visibility checkbox (M14.D) ───────────────────────────────────
// Mirrors the Hierarchy eye toggle (M14.6A). Painted as a single row
// above the Transform section. Click commits via
// `pending_visibility_edit` → `EditorCommand::SetComponent` for the
// `ph2d_ecs::Visibility` component (same pipeline as Transform).
/// Visibility checkbox in the Inspector header strip.
pub const INSP_VISIBILITY_CHECK: NodeId = hash_node_id("insp_visibility_check");

// ── Inspector Render Source — Strategy switcher (M14.C) ─────────────────────
// Three segmented buttons in the Render Source section that let the
// user switch the sprite's source-storage strategy. Pressed = current
// strategy (driven from the host snapshot every frame). Click on a
// non-pressed button raises `pending_sprite_source_change`; the shell
// does the renderer-side swap. Atlas ↔ Individual is wired in v1;
// HandPacked transitions surface a toast (asset-picker arrives in
// M14.C+).
pub const INSP_RENDER_STRATEGY_ATLAS: NodeId = hash_node_id("insp_render_strategy_atlas");
pub const INSP_RENDER_STRATEGY_INDIVIDUAL: NodeId = hash_node_id("insp_render_strategy_individual");
pub const INSP_RENDER_STRATEGY_HANDPACKED: NodeId = hash_node_id("insp_render_strategy_handpacked");

/// M14.E: editable entity-name TextInput at the top of the Inspector
/// body. Replaces the read-only name display that previously lived in
/// the Inspector header subtitle and again as a "Name" row inside the
/// Render Source section. Edits commit live via `TextChanged` →
/// `EditorCommand::SetComponent` for `ph2d_ecs::Name`.
pub const INSP_ENTITY_NAME: NodeId = hash_node_id("insp_entity_name");

// ── Inspector live section headers (Wave 4.1 restore) ─────────────────
// Right-click on these header areas opens the SectionOutline context
// menu — same affordance the Widget Gallery (showcase) has for its 10
// `INSP_SECTION_*` headers. The live Inspector originally had Transform
// + Render Source + Visibility + Name "sections" without right-click
// hit areas; restoring the outline feature here means each editable
// block now registers a header rect under one of these ids and reads
// `store.section_outline_color(...)` to paint the colored frame.
pub const INSP_LIVE_NAME_SECTION: NodeId = hash_node_id("insp_live_name_section");
pub const INSP_LIVE_VISIBILITY_SECTION: NodeId = hash_node_id("insp_live_visibility_section");
pub const INSP_LIVE_TRANSFORM_SECTION: NodeId = hash_node_id("insp_live_transform_section");
pub const INSP_LIVE_RENDER_SECTION: NodeId = hash_node_id("insp_live_render_section");
/// W2 Sprite Inspector v2 — Color & Tint live section header.
pub const INSP_LIVE_COLOR_SECTION: NodeId = hash_node_id("insp_live_color_section");
/// W2 Sprite Inspector v2 — Sprite Sheet live section header.
pub const INSP_LIVE_SHEET_SECTION: NodeId = hash_node_id("insp_live_sheet_section");
/// W3 Sprite Inspector v2 §7 — Ordering / Sorting live section header.
pub const INSP_LIVE_ORDERING_SECTION: NodeId = hash_node_id("insp_live_ordering_section");
/// W3 Sprite Inspector v2 §9 — Sampling live section header.
pub const INSP_LIVE_SAMPLING_SECTION: NodeId = hash_node_id("insp_live_sampling_section");
/// Sprite Inspector v2 §10 — Material & Blend live section header.
pub const INSP_LIVE_BLEND_SECTION: NodeId = hash_node_id("insp_live_blend_section");
/// Color-circle hit NodeIds — one per Inspector live section, parallel
/// to [`LIVE_SECTION_IDS`]. Clicking the circle opens the canonical
/// BlenderPicker pointing at this id; the picker writes the chosen
/// rgba back via `set_widget_color(<color_id>, rgba)`, and the next
/// `paint_section_header` call paints the dot in that color.
pub const INSP_LIVE_NAME_COLOR: NodeId = hash_node_id("insp_live_name_color");
pub const INSP_LIVE_VISIBILITY_COLOR: NodeId = hash_node_id("insp_live_visibility_color");
pub const INSP_LIVE_TRANSFORM_COLOR: NodeId = hash_node_id("insp_live_transform_color");
pub const INSP_LIVE_RENDER_COLOR: NodeId = hash_node_id("insp_live_render_color");
/// Color-circle hit id for the Color & Tint section header.
pub const INSP_LIVE_COLOR_COLOR: NodeId = hash_node_id("insp_live_color_color");
/// Color-circle hit id for the Sprite Sheet section header.
pub const INSP_LIVE_SHEET_COLOR: NodeId = hash_node_id("insp_live_sheet_color");
/// Color-circle hit id for the Ordering / Sorting section header.
pub const INSP_LIVE_ORDERING_COLOR: NodeId = hash_node_id("insp_live_ordering_color");

// W3 Sprite Inspector v2 §7 — Ordering / Sorting control ids.
// (INSP_ORDER_Z_OVERRIDE retired 2026-05-31 — the Z Index field IS the
// override: a non-zero value attaches `ZIndexOverride`, 0 detaches it.)
/// Z Index value (NumberInput); 0 = no override (default / DFS order).
pub const INSP_ORDER_Z_INDEX: NodeId = hash_node_id("insp_order_z_index");
/// "Z as Relative" toggle (shown when override on).
pub const INSP_ORDER_Z_RELATIVE: NodeId = hash_node_id("insp_order_z_relative");
/// "Show Behind Parent" marker toggle.
pub const INSP_ORDER_SHOW_BEHIND: NodeId = hash_node_id("insp_order_show_behind");
/// "Order in Layer" value (NumberInput).
pub const INSP_ORDER_ORDER_IN_LAYER: NodeId = hash_node_id("insp_order_order_in_layer");
/// "Y-Sort" enabled toggle.
pub const INSP_ORDER_YSORT_ENABLED: NodeId = hash_node_id("insp_order_ysort_enabled");
/// "Sorting Group" toggle.
pub const INSP_ORDER_SORTING_GROUP: NodeId = hash_node_id("insp_order_sorting_group");
/// "Sort At Root" toggle (shown when Sorting Group on).
pub const INSP_ORDER_SORT_AT_ROOT: NodeId = hash_node_id("insp_order_sort_at_root");
/// "Top Level" marker toggle.
pub const INSP_ORDER_TOP_LEVEL: NodeId = hash_node_id("insp_order_top_level");
/// Sorting Layer dropdown chip.
pub const INSP_ORDER_SORTING_LAYER: NodeId = hash_node_id("insp_order_sorting_layer");
/// Sorting Layer dropdown option ids (one per canonical project layer,
/// index = LayerId). Spec §5.2 default set: Background / Midground /
/// Default / Foreground / UI.
pub const INSP_ORDER_LAYER_OPT: [NodeId; 5] = [
    hash_node_id("insp_order_layer_opt_0"),
    hash_node_id("insp_order_layer_opt_1"),
    hash_node_id("insp_order_layer_opt_2"),
    hash_node_id("insp_order_layer_opt_3"),
    hash_node_id("insp_order_layer_opt_4"),
];
/// Y-Sort Sort Point segmented control items (Center / Pivot / Custom).
pub const INSP_ORDER_SP_CENTER: NodeId = hash_node_id("insp_order_sp_center");
pub const INSP_ORDER_SP_PIVOT: NodeId = hash_node_id("insp_order_sp_pivot");
pub const INSP_ORDER_SP_CUSTOM: NodeId = hash_node_id("insp_order_sp_custom");
/// Y-Sort Custom Axis NumberInputs (shown only when Sort Point = Custom).
pub const INSP_ORDER_AXIS_X: NodeId = hash_node_id("insp_order_axis_x");
pub const INSP_ORDER_AXIS_Y: NodeId = hash_node_id("insp_order_axis_y");

/// W3 §9 Sampling — section accent color dot.
pub const INSP_LIVE_SAMPLING_COLOR: NodeId = hash_node_id("insp_live_sampling_color");
/// Texture Filter segmented items (Inherit / Nearest / Linear → tags 0/1/2).
pub const INSP_SAMPLE_FILTER: [NodeId; 3] = [
    hash_node_id("insp_sample_filter_inherit"),
    hash_node_id("insp_sample_filter_nearest"),
    hash_node_id("insp_sample_filter_linear"),
];
/// Texture Repeat segmented items (Inherit / Disabled / Enabled / Mirror
/// → tags 0/1/2/3).
pub const INSP_SAMPLE_REPEAT: [NodeId; 4] = [
    hash_node_id("insp_sample_repeat_inherit"),
    hash_node_id("insp_sample_repeat_disabled"),
    hash_node_id("insp_sample_repeat_enabled"),
    hash_node_id("insp_sample_repeat_mirror"),
];
/// UV tiling/scroll NumberInputs (W3 UvTransform): scale X/Y, offset X/Y.
pub const INSP_SAMPLE_UV_SCALE_X: NodeId = hash_node_id("insp_sample_uv_scale_x");
pub const INSP_SAMPLE_UV_SCALE_Y: NodeId = hash_node_id("insp_sample_uv_scale_y");
pub const INSP_SAMPLE_UV_OFFSET_X: NodeId = hash_node_id("insp_sample_uv_offset_x");
pub const INSP_SAMPLE_UV_OFFSET_Y: NodeId = hash_node_id("insp_sample_uv_offset_y");

/// §10 Material & Blend — section accent color dot.
pub const INSP_LIVE_BLEND_COLOR: NodeId = hash_node_id("insp_live_blend_color");
/// §10 Blend Mode segmented items, indexed by `BlendMode::tag()` (0..5):
/// Mix / Add / Subtract / Multiply / Screen / Premult. Tag 0 (Mix) =
/// detach the optional `BlendMode` component (default).
pub const INSP_SAMPLE_BLEND: [NodeId; 6] = [
    hash_node_id("insp_sample_blend_mix"),
    hash_node_id("insp_sample_blend_add"),
    hash_node_id("insp_sample_blend_subtract"),
    hash_node_id("insp_sample_blend_multiply"),
    hash_node_id("insp_sample_blend_screen"),
    hash_node_id("insp_sample_blend_premult"),
];

// ─── §11 Physics Body (ADR-0131 D8) ───────────────────────────────────
/// §11 Physics Body — collapsible section header.
pub const INSP_LIVE_PHYSICS_SECTION: NodeId = hash_node_id("insp_live_physics_section");
/// §11 Physics Body — section accent color dot.
pub const INSP_LIVE_PHYSICS_COLOR: NodeId = hash_node_id("insp_live_physics_color");
/// §11 Attach `RigidBody` + a sprite-shaped `Collider`. Shown ONLY when the
/// entity has no body — it is the door into physics for a plain sprite.
pub const INSP_PHYS_ADD: NodeId = hash_node_id("insp_phys_add");
/// §11 Detach both components; the entity goes back to being plain art.
pub const INSP_PHYS_REMOVE: NodeId = hash_node_id("insp_phys_remove");
/// §11 Bake this body's simulated motion into timeline curves (W4). Lives in
/// the body section because it is a thing you do TO a body, next to the kind it
/// is about to change.
pub const INSP_PHYS_BAKE: NodeId = hash_node_id("insp_phys_bake");
/// §11 Bake channel selector — group id + the three options (All / Position /
/// Rotation). A GLOBAL bake option: which pose channels the Bake button writes.
pub const INSP_PHYS_BAKE_CH_GROUP: NodeId = hash_node_id("insp_phys_bake_ch_group");
pub const INSP_PHYS_BAKE_CH: [NodeId; 3] = [
    hash_node_id("insp_phys_bake_ch_all"),
    hash_node_id("insp_phys_bake_ch_position"),
    hash_node_id("insp_phys_bake_ch_rotation"),
];
/// §11 Body kind segmented, indexed by `BodyKind` tag: Dynamic / Static /
/// Kinematic. The third chip lands with W4 — it is what a **baked** body is,
/// and offering it only to the bake would leave the artist looking at a state
/// they can see, cannot author, and cannot leave.
pub const INSP_PHYS_KIND: [NodeId; 3] = [
    hash_node_id("insp_phys_kind_dynamic"),
    hash_node_id("insp_phys_kind_static"),
    hash_node_id("insp_phys_kind_kinematic"),
];
/// §11 Collider shape segmented, indexed by `ColliderShape` tag: Ball / Box.
pub const INSP_PHYS_SHAPE: [NodeId; 3] = [
    hash_node_id("insp_phys_shape_ball"),
    hash_node_id("insp_phys_shape_box"),
    hash_node_id("insp_phys_shape_capsule"),
];
/// §11 Ball radius, meters (shown only for the Ball shape).
pub const INSP_PHYS_RADIUS: NodeId = hash_node_id("insp_phys_radius");
/// §11 Box HALF-extents, meters (shown only for the Box shape).
pub const INSP_PHYS_HALF_X: NodeId = hash_node_id("insp_phys_half_x");
pub const INSP_PHYS_HALF_Y: NodeId = hash_node_id("insp_phys_half_y");
/// §11 Capsule straight-segment HALF-length, meters (shown only for the Capsule
/// shape). The capsule's total half-extent along Y is this plus the radius.
pub const INSP_PHYS_CAP_HALF_H: NodeId = hash_node_id("insp_phys_cap_half_h");
/// §11 Collider offset from the sprite centre, meters (local axes). Not
/// Dynamic-only — any collider can be offset (a character's feet, an off-centre
/// hitbox). The overlay draws the outline there so the offset is visible.
pub const INSP_PHYS_OFFSET_X: NodeId = hash_node_id("insp_phys_offset_x");
pub const INSP_PHYS_OFFSET_Y: NodeId = hash_node_id("insp_phys_offset_y");
/// §11 Mass density (kg/m² in 2D).
pub const INSP_PHYS_DENSITY: NodeId = hash_node_id("insp_phys_density");
/// §11 Bounciness, `0..=1`.
pub const INSP_PHYS_RESTITUTION: NodeId = hash_node_id("insp_phys_restitution");
/// §11 Coulomb friction.
pub const INSP_PHYS_FRICTION: NodeId = hash_node_id("insp_phys_friction");
/// §11 Authored initial linear velocity, world axes, m/s (W9). Dynamic-only.
pub const INSP_PHYS_LINVEL_X: NodeId = hash_node_id("insp_phys_linvel_x");
pub const INSP_PHYS_LINVEL_Y: NodeId = hash_node_id("insp_phys_linvel_y");
/// §11 Authored initial angular velocity. Shown as deg/s; the panel converts
/// to the component's radians at its boundary.
pub const INSP_PHYS_ANGVEL: NodeId = hash_node_id("insp_phys_angvel");
/// §11 Per-body gravity multiplier (W8). Shown only for a Dynamic body — the
/// only kind rapier applies gravity to.
pub const INSP_PHYS_GRAVITY_SCALE: NodeId = hash_node_id("insp_phys_gravity_scale");
/// The eight collision-layer chips (W2c). A fixed array, not runtime-hashed
/// ids: the count is a const, so every one can be checked against the others
/// and against all chrome by `node_id_collisions` — which does NOT see
/// registrations made inside a loop, and so cannot police dynamic ids.
/// Group id for the layer segmented control (the label/aria owner; the eight
/// chips below are its options).
pub const INSP_LIVE_PHYSICS_LAYER: NodeId = hash_node_id("insp_live_physics_layer");
pub const INSP_PHYS_LAYER: [NodeId; 8] = [
    hash_node_id("insp_phys_layer_0"),
    hash_node_id("insp_phys_layer_1"),
    hash_node_id("insp_phys_layer_2"),
    hash_node_id("insp_phys_layer_3"),
    hash_node_id("insp_phys_layer_4"),
    hash_node_id("insp_phys_layer_5"),
    hash_node_id("insp_phys_layer_6"),
    hash_node_id("insp_phys_layer_7"),
];
/// §11 Sensor (trigger) toggle — group id + the two options. A sensor passes
/// through but reports its overlaps (W7); the overlay lights it up. Modelled as
/// a two-segment control so it reuses the same paint/populate/event path as the
/// Kind and Layer segments.
pub const INSP_LIVE_PHYSICS_SENSOR: NodeId = hash_node_id("insp_live_physics_sensor");
pub const INSP_PHYS_SENSOR: [NodeId; 2] = [
    hash_node_id("insp_phys_sensor_solid"),
    hash_node_id("insp_phys_sensor_trigger"),
];
/// §11 Continuous collision detection toggle — group id + the two options
/// (Discrete / Continuous). A Dynamic-only two-segment control (like the Sensor
/// toggle) so it reuses the same paint/populate/event path (W-CCD). `Continuous`
/// makes a fast body sweep its motion instead of tunnelling through thin geometry.
pub const INSP_LIVE_PHYSICS_CCD: NodeId = hash_node_id("insp_live_physics_ccd");
pub const INSP_PHYS_CCD: [NodeId; 2] = [
    hash_node_id("insp_phys_ccd_discrete"),
    hash_node_id("insp_phys_ccd_continuous"),
];
/// §11 Lock-rotation toggle — group id + the two options (Free / Locked). A
/// Dynamic-only two-segment control (like the Sensor and CCD toggles) so it reuses
/// the same paint/populate/event path (Freeze Rotation). `Locked` pins the body's
/// orientation so it translates but never rotates.
pub const INSP_LIVE_PHYSICS_LOCKROT: NodeId = hash_node_id("insp_live_physics_lockrot");
pub const INSP_PHYS_LOCKROT: [NodeId; 2] = [
    hash_node_id("insp_phys_lockrot_free"),
    hash_node_id("insp_phys_lockrot_locked"),
];
/// §11 Freeze-Position-X toggle — group id + the two options (Free / Locked). A
/// Dynamic-only two-segment control (like the Lock-rotation toggle) so it reuses the
/// same paint/populate/event path (Freeze Position, W-LockPos). `Locked` pins the
/// body's X so the solver never moves it sideways.
pub const INSP_LIVE_PHYSICS_LOCKX: NodeId = hash_node_id("insp_live_physics_lockx");
pub const INSP_PHYS_LOCKX: [NodeId; 2] = [
    hash_node_id("insp_phys_lockx_free"),
    hash_node_id("insp_phys_lockx_locked"),
];
/// §11 Freeze-Position-Y toggle — group id + the two options (Free / Locked). The
/// vertical sibling of the X toggle; `Locked` pins Y so gravity cannot pull it down.
pub const INSP_LIVE_PHYSICS_LOCKY: NodeId = hash_node_id("insp_live_physics_locky");
pub const INSP_PHYS_LOCKY: [NodeId; 2] = [
    hash_node_id("insp_phys_locky_free"),
    hash_node_id("insp_phys_locky_locked"),
];
/// §11 Mass-source toggle — group id + the two options (Auto / Manual) (W-Mass).
/// Auto = mass is density×area (the Density row); Manual = an explicit mass in kg
/// (the Mass row). Dynamic-only, reusing the same paint/populate/event path as the
/// other two-segment controls. Density and mass are the same quantity by two roads,
/// so only one row is ever shown.
pub const INSP_LIVE_PHYSICS_MASSMODE: NodeId = hash_node_id("insp_live_physics_massmode");
pub const INSP_PHYS_MASSMODE: [NodeId; 2] = [
    hash_node_id("insp_phys_massmode_auto"),
    hash_node_id("insp_phys_massmode_manual"),
];
/// §11 Mass (kg) NumberInput — the live control in Manual mode (W-Mass).
pub const INSP_PHYS_MASS: NodeId = hash_node_id("insp_phys_mass");
/// §11 Dominance (collision priority) NumberInput — Dynamic-only (W-Dominance). A
/// higher value bulldozes lower-dominance bodies; `0` is neutral.
pub const INSP_PHYS_DOMINANCE: NodeId = hash_node_id("insp_phys_dominance");
/// §11 Restitution / Friction combine rule (W-Material): two 4-segment controls —
/// group id + the four options (Average / Min / Multiply / Max), indexed by the
/// `CombineRule` tag. NOT Dynamic-only — a collider material property, so it reuses
/// the same paint/populate/event path as the Sensor/CCD/Layer segments. `Max` makes
/// a superball bounce off any floor; `Average` (tag 0) detaches the component.
pub const INSP_LIVE_PHYSICS_REST_COMBINE: NodeId = hash_node_id("insp_live_physics_rest_combine");
pub const INSP_PHYS_REST_COMBINE: [NodeId; 4] = [
    hash_node_id("insp_phys_rest_combine_average"),
    hash_node_id("insp_phys_rest_combine_min"),
    hash_node_id("insp_phys_rest_combine_multiply"),
    hash_node_id("insp_phys_rest_combine_max"),
];
pub const INSP_LIVE_PHYSICS_FRIC_COMBINE: NodeId = hash_node_id("insp_live_physics_fric_combine");
pub const INSP_PHYS_FRIC_COMBINE: [NodeId; 4] = [
    hash_node_id("insp_phys_fric_combine_average"),
    hash_node_id("insp_phys_fric_combine_min"),
    hash_node_id("insp_phys_fric_combine_multiply"),
    hash_node_id("insp_phys_fric_combine_max"),
];
/// §11 Per-body damping (drag), Dynamic-only (W-Damping): two NumberInputs (linear /
/// angular) + a Combine|Replace mode toggle (group id + 2 options). Combine adds to
/// the world default drag, Replace ignores it. Detaches at neutral (0 drag + Combine).
pub const INSP_PHYS_LINEAR_DAMPING: NodeId = hash_node_id("insp_phys_linear_damping");
pub const INSP_PHYS_ANGULAR_DAMPING: NodeId = hash_node_id("insp_phys_angular_damping");
pub const INSP_LIVE_PHYSICS_DAMPMODE: NodeId = hash_node_id("insp_live_physics_dampmode");
pub const INSP_PHYS_DAMPMODE: [NodeId; 2] = [
    hash_node_id("insp_phys_dampmode_combine"),
    hash_node_id("insp_phys_dampmode_replace"),
];
/// §11 One-way (jump-through) platform toggle (W-OneWay) — group id + the two options
/// (Off / On). A COLLIDER property, so it is offered for ANY body kind: a platform is
/// usually Static, which is exactly the case a Dynamic-only gate would delete.
pub const INSP_LIVE_PHYSICS_ONEWAY: NodeId = hash_node_id("insp_live_physics_oneway");
pub const INSP_PHYS_ONEWAY: [NodeId; 2] = [
    hash_node_id("insp_phys_oneway_off"),
    hash_node_id("insp_phys_oneway_on"),
];
/// §11 Force zone (W-Area) — the push, in newtons, this SENSOR applies to whatever
/// overlaps it. Two number rows, one per world axis. Painted only for a sensor
/// collider: an area you cannot enter is not an area, and the narrow phase reports no
/// overlap for a solid one. Detaches its `AreaEffector` at zero on both axes.
pub const INSP_PHYS_FORCE_X: NodeId = hash_node_id("insp_phys_force_x");
pub const INSP_PHYS_FORCE_Y: NodeId = hash_node_id("insp_phys_force_y");
/// §11 O FRAME da força da zona (W-AreaFrame) — grupo + as duas opções (Zone / World).
/// Um controle de dois segmentos, sensor-only como as rows de força que ele qualifica.
/// `Zone` (o default) autora a força no referencial da zona, então **girar o sensor gira o
/// vento**; `World` prende a direção aos eixos de mundo (o `useGlobalAngle` da Unity).
/// Governa a força e SÓ ela — o torque 2D é um escalar sobre Z e não tem o que girar, o
/// arrasto é isotrópico e o empuxo mede pela gravidade. Marcador `AreaForceWorldAxes`:
/// presente = World, ausente = Zone, então o default não custa componente nenhum.
pub const INSP_LIVE_PHYSICS_FORCE_AXES: NodeId = hash_node_id("insp_live_physics_force_axes");
pub const INSP_PHYS_FORCE_AXES: [NodeId; 2] = [
    hash_node_id("insp_phys_force_axes_zone"),
    hash_node_id("insp_phys_force_axes_world"),
];
/// §11 Torque de área (W-AreaTorque) — o giro (N·m) que este SENSOR imprime a cada corpo
/// dentro dele, um redemoinho ou mesa giratória. O SINAL é o sentido; destaca seu
/// `AreaTorque` em zero exato (não clampa negativo, ao contrário dos irmãos de arrasto).
pub const INSP_PHYS_AREA_TORQUE: NodeId = hash_node_id("insp_phys_area_torque");
/// §11 Falloff da zona (W-AreaFalloff) — o quanto a força e o torque ENFRAQUECEM do centro
/// até a borda deste SENSOR. `0` (o default) é um campo uniforme; `1` desvanece até zero
/// exatamente na borda, em toda direção. A régua é a silhueta da própria zona, então não há
/// um raio à parte para discordar do tamanho dela. Pesa os dois EMPURRÕES e nada mais — o
/// arrasto e o empuxo descrevem um meio, e um meio não fica ralo perto da própria margem.
/// Destaca seu `AreaFalloff` em zero.
pub const INSP_PHYS_AREA_FALLOFF: NodeId = hash_node_id("insp_phys_area_falloff");
/// §11 Area drag (W-AreaDrag) — the resistance the medium inside this SENSOR offers.
/// The other half of a force zone: force is the push, this is the water. Same law as
/// the world default drag; detaches its `AreaDrag` at zero.
pub const INSP_PHYS_AREA_DRAG: NodeId = hash_node_id("insp_phys_area_drag");
/// §11 Densidade do FLUIDO (W-Buoyancy) — o empuxo de Arquimedes dentro deste sensor.
/// Comparável ao `Density` do collider: um corpo menos denso que isto boia, mais denso
/// afunda. Destaca seu `AreaBuoyancy` em zero.
pub const INSP_PHYS_AREA_DENSITY: NodeId = hash_node_id("insp_phys_area_density");
/// §11 Arrasto de FORMA (W-FormDrag) — a resistência que sabe para onde o corpo aponta.
/// Irmã de `Drag` (viscosidade, uniforme) e não substituta: são mecanismos diferentes.
pub const INSP_PHYS_AREA_FORM_DRAG: NodeId = hash_node_id("insp_phys_area_form_drag");

// ─── W3 §8 Visibility-section controls (ClipChildren / Mask / Layer) ───
/// Clip Children segmented: Disabled / ClipOnly / ClipAndDraw (tags 0/1/2).
pub const INSP_VIS_CLIP: [NodeId; 3] = [
    hash_node_id("insp_vis_clip_disabled"),
    hash_node_id("insp_vis_clip_clip_only"),
    hash_node_id("insp_vis_clip_clip_and_draw"),
];
/// Mask Interaction segmented: None / VisibleInside / VisibleOutside (0/1/2).
pub const INSP_VIS_MASK: [NodeId; 3] = [
    hash_node_id("insp_vis_mask_none"),
    hash_node_id("insp_vis_mask_inside"),
    hash_node_id("insp_vis_mask_outside"),
];
/// Mask alpha-cutoff NumberInput (shown when Mask != None).
pub const INSP_VIS_ALPHA_CUTOFF: NodeId = hash_node_id("insp_vis_alpha_cutoff");
/// "Mask Source (Mask2D)" toggle — makes this sprite a mask source.
pub const INSP_VIS_MASK_SOURCE: NodeId = hash_node_id("insp_vis_mask_source");
/// Collapsible sub-header for the Visibility Layer 4×8 bitmask grid — a
/// `mark_collapsible_section` id so clicking the row folds the grid.
pub const INSP_VIS_LAYER_HEADER: NodeId = hash_node_id("insp_vis_layer_header");
/// On-Screen Enabler toggle + its Rect2 editor (x/y/w/h, shown when on).
pub const INSP_VIS_ON_SCREEN: NodeId = hash_node_id("insp_vis_on_screen");
pub const INSP_VIS_RECT_X: NodeId = hash_node_id("insp_vis_rect_x");
pub const INSP_VIS_RECT_Y: NodeId = hash_node_id("insp_vis_rect_y");
pub const INSP_VIS_RECT_W: NodeId = hash_node_id("insp_vis_rect_w");
pub const INSP_VIS_RECT_H: NodeId = hash_node_id("insp_vis_rect_h");
/// VisibilityLayer bitmask — 32 checkboxes (4 cols × 8 rows), bit `n`.
pub const INSP_VIS_LAYER_BIT: [NodeId; 32] = [
    hash_node_id("insp_vis_layer_bit_0"),
    hash_node_id("insp_vis_layer_bit_1"),
    hash_node_id("insp_vis_layer_bit_2"),
    hash_node_id("insp_vis_layer_bit_3"),
    hash_node_id("insp_vis_layer_bit_4"),
    hash_node_id("insp_vis_layer_bit_5"),
    hash_node_id("insp_vis_layer_bit_6"),
    hash_node_id("insp_vis_layer_bit_7"),
    hash_node_id("insp_vis_layer_bit_8"),
    hash_node_id("insp_vis_layer_bit_9"),
    hash_node_id("insp_vis_layer_bit_10"),
    hash_node_id("insp_vis_layer_bit_11"),
    hash_node_id("insp_vis_layer_bit_12"),
    hash_node_id("insp_vis_layer_bit_13"),
    hash_node_id("insp_vis_layer_bit_14"),
    hash_node_id("insp_vis_layer_bit_15"),
    hash_node_id("insp_vis_layer_bit_16"),
    hash_node_id("insp_vis_layer_bit_17"),
    hash_node_id("insp_vis_layer_bit_18"),
    hash_node_id("insp_vis_layer_bit_19"),
    hash_node_id("insp_vis_layer_bit_20"),
    hash_node_id("insp_vis_layer_bit_21"),
    hash_node_id("insp_vis_layer_bit_22"),
    hash_node_id("insp_vis_layer_bit_23"),
    hash_node_id("insp_vis_layer_bit_24"),
    hash_node_id("insp_vis_layer_bit_25"),
    hash_node_id("insp_vis_layer_bit_26"),
    hash_node_id("insp_vis_layer_bit_27"),
    hash_node_id("insp_vis_layer_bit_28"),
    hash_node_id("insp_vis_layer_bit_29"),
    hash_node_id("insp_vis_layer_bit_30"),
    hash_node_id("insp_vis_layer_bit_31"),
];

/// W2 Sprite Inspector v2 — Color & Tint section controls.
/// Final opacity Slider (`0..1` storage; renders today via
/// `RenderInstance.opacity`). Paired with [`INSP_SPRITE_OPACITY_CHIP`].
pub const INSP_SPRITE_OPACITY: NodeId = hash_node_id("insp_sprite_opacity");
/// Numeric chip linked to the Opacity slider, displaying `0..100`
/// (percent) via an integer-mapped projection.
pub const INSP_SPRITE_OPACITY_CHIP: NodeId = hash_node_id("insp_sprite_opacity_chip");
/// Tint Fill (silhouette) checkbox; renders today via `flip_uv` bit 2.
pub const INSP_SPRITE_TINT_FILL: NodeId = hash_node_id("insp_sprite_tint_fill");
/// Inherited modulate color swatch (`Sprite::tint`, cascades to
/// children). Clicking opens the shared `INSP_BLENDER_PICKER`
/// targeting this id; the chosen color round-trips through
/// `widget_color(id)` and is dispatched as `SpriteFieldEdit::Tint`.
/// Renders today via `RenderInstance.tint`.
pub const INSP_SPRITE_TINT_SWATCH: NodeId = hash_node_id("insp_sprite_tint_swatch");
/// Local modulate color swatch (`Sprite::self_tint`, does NOT cascade).
/// Same picker mechanism as [`INSP_SPRITE_TINT_SWATCH`]; dispatched as
/// `SpriteFieldEdit::SelfTint`.
pub const INSP_SPRITE_SELF_TINT_SWATCH: NodeId = hash_node_id("insp_sprite_self_tint_swatch");
/// Per-corner tint swatches `[TL, TR, BL, BR]` — a 4-stop bilinear
/// gradient (Phaser-style). Each opens the shared picker; the chosen
/// color replaces one corner of the `[[f32;4];4]` array and dispatches
/// `SpriteFieldEdit::PerCornerTint`. Renders via the shader's
/// `@location(9..12)` per-corner attributes.
pub const INSP_SPRITE_CORNER_TL: NodeId = hash_node_id("insp_sprite_corner_tl");
/// Per-corner tint swatch — top-right. See [`INSP_SPRITE_CORNER_TL`].
pub const INSP_SPRITE_CORNER_TR: NodeId = hash_node_id("insp_sprite_corner_tr");
/// Per-corner tint swatch — bottom-left. See [`INSP_SPRITE_CORNER_TL`].
pub const INSP_SPRITE_CORNER_BL: NodeId = hash_node_id("insp_sprite_corner_bl");
/// Per-corner tint swatch — bottom-right. See [`INSP_SPRITE_CORNER_TL`].
pub const INSP_SPRITE_CORNER_BR: NodeId = hash_node_id("insp_sprite_corner_br");
/// "Equalize corners" button — copies the top-left corner color to the
/// other three (spec §3.6 hotkey); dispatches `SpriteFieldEdit::PerCornerTint`.
pub const INSP_SPRITE_CORNER_EQUALIZE: NodeId = hash_node_id("insp_sprite_corner_equalize");
// (The Color & Tint sub-tab ids `INSP_COLOR_TAB_*` were retired
// 2026-05-31 — the section now stacks every control visible at once.)

/// W2 Sprite Inspector v2 — Region sampling controls (Render Source
/// section, spec §3.3). Toggle + 4 px NumberInputs (x/y/w/h) + filter
/// clip. Renders via the extract `region_subrect` sub-UV (W2.T2.4).
pub const INSP_REGION_ENABLED: NodeId = hash_node_id("insp_region_enabled");
/// Region rect X NumberInput (source pixels). See [`INSP_REGION_ENABLED`].
pub const INSP_REGION_X: NodeId = hash_node_id("insp_region_x");
/// Region rect Y NumberInput (source pixels).
pub const INSP_REGION_Y: NodeId = hash_node_id("insp_region_y");
/// Region rect W NumberInput (source pixels, `>= 0`).
pub const INSP_REGION_W: NodeId = hash_node_id("insp_region_w");
/// Region rect H NumberInput (source pixels, `>= 0`).
pub const INSP_REGION_H: NodeId = hash_node_id("insp_region_h");
/// Region filter-clip toggle (anti atlas-bleed). See [`INSP_REGION_ENABLED`].
pub const INSP_REGION_FILTER_CLIP: NodeId = hash_node_id("insp_region_filter_clip");

/// W2 Sprite Inspector v2 — origin controls (Sprite Sheet section, spec
/// §3.4). Centered toggle + Offset X/Y px NumberInputs. Renders via
/// `Sprite::resolve_anchor` (W2.T2.6).
pub const INSP_SPRITE_CENTERED: NodeId = hash_node_id("insp_sprite_centered");
/// Intrinsic offset X NumberInput (px). See [`INSP_SPRITE_CENTERED`].
pub const INSP_SPRITE_OFFSET_X: NodeId = hash_node_id("insp_sprite_offset_x");
/// Intrinsic offset Y NumberInput (px). See [`INSP_SPRITE_CENTERED`].
pub const INSP_SPRITE_OFFSET_Y: NodeId = hash_node_id("insp_sprite_offset_y");

/// W2 Sprite Inspector v2 — Sprite Sheet grid controls (render today via
/// the extract atlas-rect sub-division). HFrames / VFrames / Frame.
pub const INSP_SPRITE_HFRAMES: NodeId = hash_node_id("insp_sprite_hframes");
/// Sprite-sheet rows NumberInput.
pub const INSP_SPRITE_VFRAMES: NodeId = hash_node_id("insp_sprite_vframes");
/// Active sheet frame index NumberInput.
pub const INSP_SPRITE_FRAME: NodeId = hash_node_id("insp_sprite_frame");

/// Title-bar color dot for the Grid Snap panel. Kept (Grid Snap is a
/// settings panel, not an image tool). The original broadcast added
/// PAD/BGR/CEQ/UPS/EQS dots too, but those were removed 2026-05-24
/// per user feedback: image-tool panels are transient operation
/// surfaces, not annotation surfaces.
pub const GS_TITLE_COLOR: NodeId = hash_node_id("gs_title_color");
/// Widget Gallery floating panel — root id. The gallery is a Procreate-
/// style floating reference panel that hosts the canonical widget
/// showcase. Toggle visibility via [`TOPBAR_WIDGET_GALLERY`].
pub const GAL_PANEL: NodeId = hash_node_id("gal_panel");
/// Audio Mixer floating panel — root id. Must match
/// `ph2d_panel_audio_mixer::AMIX_PANEL` (same `hash_node_id` string) so the
/// z-order paint walk in `screens::hero::paint` resolves + paints it. Toggle
/// visibility via [`crate::ids::TOPBAR_AUDIO_MIXER`].
pub const AUDIO_MIXER_PANEL: NodeId = hash_node_id("audio_mixer_panel");
/// Audio Editor docked panel — root id. Must match
/// `ph2d_panel_audio_editor::AEDIT_PANEL` (same `hash_node_id` string) so the
/// z-order paint walk in `screens::hero::paint` resolves + paints it. Toggle
/// visibility via [`crate::ids::TOPBAR_AUDIO_EDITOR`]. The docked panel holds the
/// transport + load/export controls; the big waveform + timeline live in the
/// separate floating [`AUDIO_OVERLAY_PANEL`] on the canvas.
pub const AUDIO_EDITOR_PANEL: NodeId = hash_node_id("audio_editor_panel");
/// Audio Editor **floating overlay** — root id for the resizable waveform +
/// timeline window that floats over the canvas in the gap between the Hierarchy
/// and Inspector docks. Drag/resize reuse the panel-agnostic
/// `blender_picker_offset` + `panel_resize_delta` store, keyed by this id
/// (mirror of the Inspector dock).
pub const AUDIO_OVERLAY_PANEL: NodeId = hash_node_id("audio_overlay_panel");
/// Audio Editor overlay — title-bar drag handle. Registered as
/// `BlenderHit { parent: AUDIO_OVERLAY_PANEL, kind: DragHandle }` by the editor
/// panel's populate; the panel-agnostic dispatch moves the overlay via
/// `blender_picker_offset`.
pub const AUDIO_OVERLAY_DRAG_HANDLE: NodeId = hash_node_id("audio_overlay_drag_handle");
/// Audio Editor overlay — bottom-right resize gripper (`ResizeHandle`).
pub const AUDIO_OVERLAY_RESIZE_HANDLE: NodeId = hash_node_id("audio_overlay_resize_handle");
/// Audio Editor overlay — bottom-left resize gripper (`ResizeHandleBl`).
pub const AUDIO_OVERLAY_RESIZE_HANDLE_BL: NodeId = hash_node_id("audio_overlay_resize_handle_bl");
/// Drag handle pill at the top of the Widget Gallery panel.
pub const GAL_DRAG_HANDLE: NodeId = hash_node_id("gal_drag_handle");
/// Resize gripper at the Widget Gallery's bottom-right corner.
pub const GAL_RESIZE_HANDLE: NodeId = hash_node_id("gal_resize_handle");
/// Resize gripper at the Widget Gallery's bottom-LEFT corner. Mirror
/// of [`GAL_RESIZE_HANDLE`].
pub const GAL_RESIZE_HANDLE_BL: NodeId = hash_node_id("gal_resize_handle_bl");
/// Close (X) button at the top-right of the Widget Gallery — alternate
/// way to dismiss the panel beyond clicking the TopBar palette pill.
pub const GAL_CLOSE: NodeId = hash_node_id("gal_close");
/// Drag handle at the top of the Hierarchy.
pub const HIER_DRAG_HANDLE: NodeId = hash_node_id("hier_drag_handle");
/// Resize gripper at the Hierarchy's bottom-right corner.
pub const HIER_RESIZE_HANDLE: NodeId = hash_node_id("hier_resize_handle");
/// Resize gripper at the Hierarchy's bottom-LEFT corner. Mirror of
/// [`HIER_RESIZE_HANDLE`].
pub const HIER_RESIZE_HANDLE_BL: NodeId = hash_node_id("hier_resize_handle_bl");

// ---------------------------------------------------------------------------
// §12 Physics Joint (W3). A joint is an ENTITY, so this section describes the
// selected joint object — kind, the two bodies it names, and the parameters
// the chosen kind actually uses.
// ---------------------------------------------------------------------------

/// The §12 section header (collapse state owner) and its colour circle.
pub const INSP_LIVE_JOINT_SECTION: NodeId = hash_node_id("insp_live_joint_section");
pub const INSP_LIVE_JOINT_COLOR: NodeId = hash_node_id("insp_live_joint_color");

/// Group ids for the three segmented controls in the section.
///
/// Separate constants rather than reusing the section/colour ids. ⚠️ The
/// *reason* is latent, not live: `SegmentedAdaptive`'s `id`/`label` are read
/// only by `build_a11y`, which **has no callers yet** — so today these are
/// inert and nothing observes a collision. They are distinct anyway because
/// §11 next door does reuse `INSP_LIVE_PHYSICS_SECTION` and
/// `INSP_LIVE_PHYSICS_COLOR` as group ids, and the day accessibility is wired
/// that is two rects answering to one id. Cheap to get right now; a rename
/// hunt later.
pub const INSP_JOINT_KIND_GROUP: NodeId = hash_node_id("insp_joint_kind_group");
pub const INSP_JOINT_LIMITS_GROUP: NodeId = hash_node_id("insp_joint_limits_group");
pub const INSP_JOINT_MOTOR_GROUP: NodeId = hash_node_id("insp_joint_motor_group");

/// Pin · Spring · Rope · Weld · Slider. Indexed by the `JointKind` tag the
/// snapshot carries.
///
/// ⚠️ **Este array e o `KIND_LABELS` do painel têm de ter o MESMO tamanho.** O
/// `seg_row` faz `option_ids.zip(labels)`, e um `zip` **trunca**: com um rótulo a
/// mais que ids, o último chip simplesmente não é pintado — sem erro, sem
/// warning. Foi o que aconteceu quando o Slider chegou (W-J5), e o gate de seam
/// não pegou porque ele iterava justamente a lista CURTA. Há um teste no painel
/// comparando os dois comprimentos.
pub const INSP_JOINT_KIND: [NodeId; 5] = [
    hash_node_id("insp_joint_kind_pin"),
    hash_node_id("insp_joint_kind_spring"),
    hash_node_id("insp_joint_kind_rope"),
    hash_node_id("insp_joint_kind_weld"),
    hash_node_id("insp_joint_kind_slider"),
];

/// Off · On, for the two Pin-only switches. Segmented rather than a checkbox
/// because that is the widget this section already speaks, and a two-option
/// segmented is exactly a switch.
pub const INSP_JOINT_LIMITS: [NodeId; 2] = [
    hash_node_id("insp_joint_limits_off"),
    hash_node_id("insp_joint_limits_on"),
];
pub const INSP_JOINT_MOTOR: [NodeId; 2] = [
    hash_node_id("insp_joint_motor_off"),
    hash_node_id("insp_joint_motor_on"),
];

/// Velocity · Position — what a driven joint is AIMING at (W-J6). Not another
/// on/off switch: the two carry different instructions (*keep going at this
/// rate* vs *go to this place and hold*) and each shows its own row underneath.
pub const INSP_JOINT_MOTOR_MODE_GROUP: NodeId = hash_node_id("insp_joint_motor_mode_group");
pub const INSP_JOINT_MOTOR_MODE: [NodeId; 2] = [
    hash_node_id("insp_joint_motor_mode_velocity"),
    hash_node_id("insp_joint_motor_mode_position"),
];

/// Pin: the angular range, in DEGREES at this boundary (the component stores
/// radians, like `Transform::rotation_rad`).
pub const INSP_JOINT_LIMIT_MIN: NodeId = hash_node_id("insp_joint_limit_min");
pub const INSP_JOINT_LIMIT_MAX: NodeId = hash_node_id("insp_joint_limit_max");
/// The motor — a Pin's hinge, a Slider's rail, a Rope's winch. `SPEED` is the
/// Velocity mode's rate and `TARGET` is the Position mode's place, each in the
/// free degree of freedom's own unit (degrees on a hinge, metres on the other
/// two — `JointKind::motor_in_metres`).
pub const INSP_JOINT_MOTOR_SPEED: NodeId = hash_node_id("insp_joint_motor_speed");
pub const INSP_JOINT_MOTOR_TARGET: NodeId = hash_node_id("insp_joint_motor_target");
pub const INSP_JOINT_MOTOR_FORCE: NodeId = hash_node_id("insp_joint_motor_force");
/// Spring.
pub const INSP_JOINT_REST_LENGTH: NodeId = hash_node_id("insp_joint_rest_length");
pub const INSP_JOINT_STIFFNESS: NodeId = hash_node_id("insp_joint_stiffness");
pub const INSP_JOINT_DAMPING: NodeId = hash_node_id("insp_joint_damping");
/// Rope.
pub const INSP_JOINT_MAX_LENGTH: NodeId = hash_node_id("insp_joint_max_length");
/// Delete the joint object.
pub const INSP_JOINT_REMOVE: NodeId = hash_node_id("insp_joint_remove");

/// The eyedropper next to each of the joint's two body rows (§12). Clicking it
/// ARMS a canvas pick for that slot; the next click on a body re-binds that end,
/// with no other object pre-selected (the app's pick idiom — arm, then click the
/// target, like the colour eyedropper). Fixes a mis-joined pair without deleting
/// and re-creating the joint.
pub const INSP_JOINT_PICK_A: NodeId = hash_node_id("insp_joint_pick_a");
pub const INSP_JOINT_PICK_B: NodeId = hash_node_id("insp_joint_pick_b");

/// The creation gesture, and it lives in §11 (Physics Body) rather than here:
/// a joint does not exist yet when you want to make one, so the button has to
/// be somewhere you already are — looking at two bodies you have selected.
pub const INSP_PHYS_JOIN: NodeId = hash_node_id("insp_phys_join");
/// **Arm the canvas drawing gesture** (W-J4) — press a body, drag, release on
/// another, and the joint is born with its anchors AT the two points.
///
/// The sibling route, and the one that puts the anchors where the artist
/// pointed: `INSP_PHYS_JOIN` has no points to offer, so its anchors come from
/// the seed policy (body B's CENTRE for a spring/rope). Both stay — the button
/// is how a CHAIN is made, the gesture is how a placement is made.
pub const INSP_PHYS_JOIN_DRAW: NodeId = hash_node_id("insp_phys_join_draw");
/// §11 join-kind selector, indexed by `JointKind` tag (Pin / Spring / Rope /
/// Weld / Slider). Painted beside *Join Selected Bodies* so the artist creates
/// the joint TYPE they want in one gesture, instead of making a Pin and
/// converting it — and it qualifies the canvas DRAW gesture too.
///
/// ⚠️ **A lista de tipos existe DUAS vezes** — aqui (o tipo que o próximo gesto
/// CRIA) e em [`INSP_JOINT_KIND`] (o tipo que a joint selecionada É) — e as duas
/// têm de conhecer todo `JointKind`. O Slider chegou só na segunda (W-J5), e o
/// resultado foi um tipo que a simulação tinha e o artista **não conseguia
/// escolher** (Enio: *"Slider não aparece no painel de joints"*): o `seg_row` faz
/// `option_ids.zip(labels)`, então o rótulo a mais foi silenciosamente descartado.
/// Há um gate que compara os comprimentos dos DOIS pares.
pub const INSP_PHYS_JOIN_KIND: [NodeId; 5] = [
    hash_node_id("insp_phys_join_kind_pin"),
    hash_node_id("insp_phys_join_kind_spring"),
    hash_node_id("insp_phys_join_kind_rope"),
    hash_node_id("insp_phys_join_kind_weld"),
    hash_node_id("insp_phys_join_kind_slider"),
];
