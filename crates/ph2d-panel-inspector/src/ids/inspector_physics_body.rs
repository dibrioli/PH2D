//! **A secção Physics Body** do Inspector (§11, ADR-0131 D8) — cortada do `inspector.rs` por
//! assunto, pelo tecto de 600 linhas por ficheiro de painel.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/inspector.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 43 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

// ─── §11 Physics Body (ADR-0131 D8) ───────────────────────────────────
/// ⭐ **O `+` do cabeçalho do Inspector** — abre a paleta de componentes (ADR-0166 / F3).
///
/// ⚠️ **UMA porta, e ela SUBSUME as cinco de hoje** (`INSP_PLAYER_ADD` · `INSP_ANCHOR_ADD` ·
/// `INSP_ANIM_ADD` · `INSP_PHYS_ADD` + o botão de anexar da §5 9-Slice). Duas respostas a *"como
/// se adiciona um componente?"* é a divergência que esta fase existe para apagar.
///
/// ⚠️ Um botão por-seção pode sobreviver como **atalho da seção já visível** — o que não
/// sobrevive é ser a ÚNICA rota. E as que SEMEIAM um valor do contexto vivo (a massa que o corpo
/// tem agora) fazem algo que o `+` genérico não pode fazer: ver a nota do `Attach::Intrinsic`.
pub const INSP_ADD_COMPONENT: NodeId = hash_node_id("insp_add_component");

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

/// **De que é feito este chão para quem ANDA sobre ele** (`W-Surface`) — o
/// multiplicador de tração. Neutro `1.0`; gelo é baixo, borracha é alto.
///
/// ⚠️ **Oferecido em TODO collider, e é o caso de uso inteiro:** a superfície
/// que importa é quase sempre um chão ESTÁTICO, então gateá-lo em Dynamic
/// deletaria o controle exatamente onde ele serve.
pub const INSP_PHYS_WALK_GRIP: NodeId = hash_node_id("insp_phys_walk_grip");

/// **A ESTEIRA** (`W-Surface`) — a velocidade que esta superfície apresenta a
/// quem está sobre ela, em m/s COM SINAL, ao longo da tangente dela.
pub const INSP_PHYS_WALK_BELT: NodeId = hash_node_id("insp_phys_walk_belt");

/// **Esta superfície é PAREDE?** (`W-WallMaterial`) — o `platform_wall_layers` do
/// Godot, por CORPO/PEÇA em vez de por camada. Grupo + as duas opções
/// (`On` = agarra-se, o default; `Off` = a mão não a segura).
///
/// ⚠️ **Ao lado do Grip e da Belt, e não junto do One-Way**, porque é a mesma
/// pergunta que aquelas duas fazem — *de que esta superfície é feita para quem
/// encosta nela* —, enquanto o One-Way é sobre a GEOMETRIA do collider (*de que
/// lado ele é sólido*). E é oferecida em TODO collider pela razão que a família
/// da superfície já tem escrita: a parede que importa é quase sempre ESTÁTICA.
pub const INSP_LIVE_PHYSICS_WALLMAT: NodeId = hash_node_id("insp_live_physics_wallmat");

pub const INSP_PHYS_WALLMAT: [NodeId; 2] = [
    hash_node_id("insp_phys_wallmat_on"),
    hash_node_id("insp_phys_wallmat_off"),
];

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
