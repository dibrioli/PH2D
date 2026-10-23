//! **Os CAMPOS da família da física** — o irmão de [`super::physics`] pelo tecto de 700 LOC do
//! HR-18: lá os descritores (que componente é, como se anexa), aqui os blocos de campos deles.

use crate::{FieldDesc, FieldKind as K, Propagation, RefKind, View, WHOLE};

/// Um campo de valor puro, com o nome de código (doc CodeEditor 08 §3.11).
const fn f(field_id: u16, label_key: &'static str, kind: K) -> FieldDesc {
    FieldDesc {
        field_id,
        label_key,
        kind,
        policy: Propagation::Propagate,
        is_ref: None,
        name: "",
        key: "",
        view: crate::View::Same,
    }
}

/// **Uma referência a outro objeto**, por `StableId` — ver o cabeçalho do módulo.
///
/// ⚠️ `Propagate`: o campo segue o mestre **depois de remapeado**. Não é `RuntimeOwned` — a
/// referência é AUTORIA (*"prende neste corpo"*), e quem o solver possui é a pose, não o elo.
const fn r(field_id: u16, label_key: &'static str) -> FieldDesc {
    FieldDesc {
        field_id,
        label_key,
        kind: K::Ref,
        policy: Propagation::Propagate,
        is_ref: Some(RefKind::Object),
        name: "",
        key: "",
        view: crate::View::Same,
    }
}

/// **`PhysicsJoint`** — os dois corpos que ele prende. ⚠️ Os `field_id` seguem a ordem de
/// declaração da struct (`joint.rs`), para que uma wave que descreva o resto do tipo não tenha
/// de saltar por cima destes dois. Os nomes são os rótulos do cartão da §12; os limites e o motor
/// guardam a unidade do TIPO (radianos numa dobradiça, metros num trilho) — o cartão pinta graus.
/// Sem descritor (ocultos em `hiding`): `local_a/b` (14, 15), `anchored` (16) e `custom` (25).
pub(super) const JOINT_KINDS: View = crate::options![
    "pin" => Pin,
    "spring" => Spring,
    "rope" => Rope,
    "weld" => Weld,
    "slider" => Slider,
    "rod" => Rod,
    "wheel" => Wheel,
    "pulley" => Pulley,
    "custom" => Custom,
];
pub(super) const JOINT: &[FieldDesc] = &[
    r(1, "component.field.joint.1")
        .named("body_a")
        .viewed(View::Named(crate::Target::NameHash)),
    r(2, "component.field.joint.2")
        .named("body_b")
        .viewed(View::Named(crate::Target::NameHash)),
    f(3, "component.field.joint.3", K::Enum)
        .named("kind")
        .viewed(JOINT_KINDS),
    f(4, "component.field.joint.4", K::Toggle).named_as("limits", "limits_enabled"),
    f(5, "component.field.joint.5", K::Scalar).named_as("min", "limit_min"),
    f(6, "component.field.joint.6", K::Scalar).named_as("max", "limit_max"),
    f(7, "component.field.joint.7", K::Toggle).named_as("motor", "motor_enabled"),
    f(8, "component.field.joint.8", K::Scalar).named_as("speed", "motor_speed"),
    f(9, "component.field.joint.9", K::Scalar).named_as("max_force", "motor_max_force"),
    f(10, "component.field.joint.10", K::Scalar).named("rest_length"),
    f(11, "component.field.joint.11", K::Scalar).named("stiffness"),
    f(12, "component.field.joint.12", K::Scalar).named("damping"),
    f(13, "component.field.joint.13", K::Scalar).named("max_length"),
    f(17, "component.field.joint.17", K::Enum)
        .named_as("mode", "motor_mode")
        .viewed(crate::options!["velocity" => Velocity, "position" => Position]),
    f(18, "component.field.joint.18", K::Scalar).named_as("target", "motor_target"),
    f(19, "component.field.joint.19", K::Toggle).named_as("breakable", "break_enabled"),
    f(20, "component.field.joint.20", K::Scalar).named("break_force"),
    f(21, "component.field.joint.21", K::Scalar).named("break_torque"),
    f(22, "component.field.joint.22", K::Toggle).named("active"),
    f(23, "component.field.joint.23", K::Toggle).named_as("collide", "collide_connected"),
    // «Weld: Rigid | Soft» — o rótulo é a linha da solda, o valor é `soft`.
    f(24, "component.field.joint.24", K::Toggle).named_as("weld", "soft"),
];

/// **`PlatformPlayer`** — as linhas da §14, na ordem da struct (`components/player.rs`).
pub(super) const PLATFORM_PLAYER: &[FieldDesc] = &[
    f(0, "component.field.platform_player.0", K::Scalar).named("float_height"),
    f(1, "component.field.platform_player.1", K::Scalar).named("cling_distance"),
    f(2, "component.field.platform_player.2", K::Scalar)
        .named_as("leg_stiffness", "spring_strength"),
    f(3, "component.field.platform_player.3", K::Scalar).named_as("leg_damping", "spring_damping"),
    f(4, "component.field.platform_player.4", K::Scalar).named("speed"),
    f(5, "component.field.platform_player.5", K::Scalar).named("acceleration"),
    f(6, "component.field.platform_player.6", K::Scalar).named("air_acceleration"),
    f(7, "component.field.platform_player.7", K::Angle)
        .named_as("max_slope", "max_slope_deg")
        .viewed(View::Degrees),
    f(8, "component.field.platform_player.8", K::Scalar).named("jump_height"),
    f(9, "component.field.platform_player.9", K::Int).named("air_jumps"),
    f(10, "component.field.platform_player.10", K::Scalar).named("air_jump_height"),
    f(11, "component.field.platform_player.11", K::Scalar).named("takeoff_gravity"),
    f(12, "component.field.platform_player.12", K::Scalar)
        .named_as("takeoff_above", "takeoff_speed"),
    f(13, "component.field.platform_player.13", K::Scalar).named("peak_gravity"),
    f(14, "component.field.platform_player.14", K::Scalar).named_as("peak_window", "peak_speed"),
    f(15, "component.field.platform_player.15", K::Scalar).named("fall_gravity"),
    f(16, "component.field.platform_player.16", K::Scalar).named("cut_gravity"),
    f(17, "component.field.platform_player.17", K::Scalar).named("coyote_time"),
    f(18, "component.field.platform_player.18", K::Scalar).named("jump_buffer"),
    f(19, "component.field.platform_player.19", K::Int).named_as("foot_rays", "foot_samples"),
    f(20, "component.field.platform_player.20", K::Scalar)
        .named_as("foot_ray_spread", "foot_spread"),
    f(21, "component.field.platform_player.21", K::Int).named_as("corner_rays", "corner_samples"),
    f(22, "component.field.platform_player.22", K::Scalar)
        .named_as("corner_look_ahead", "corner_lookahead"),
    f(23, "component.field.platform_player.23", K::Int).named_as("wall_rays", "wall_samples"),
    f(24, "component.field.platform_player.24", K::Scalar)
        .named_as("wall_ray_spread", "wall_spread"),
    f(25, "component.field.platform_player.25", K::Scalar).named("corner_reach"),
    f(26, "component.field.platform_player.26", K::Scalar).named("lift_momentum"),
    f(27, "component.field.platform_player.27", K::Scalar)
        .named_as("weight_on_ground", "reaction_support"),
    f(28, "component.field.platform_player.28", K::Scalar)
        .named_as("push_on_ground", "reaction_movement"),
    f(29, "component.field.platform_player.29", K::Scalar)
        .named_as("push_on_bodies", "reaction_push"),
    f(30, "component.field.platform_player.30", K::Scalar)
        .named_as("wall_slide", "wall_slide_speed"),
    f(31, "component.field.platform_player.31", K::Scalar)
        .named_as("wall_jump", "wall_jump_height"),
    f(32, "component.field.platform_player.32", K::Scalar).named_as("wall_push", "wall_jump_push"),
    f(33, "component.field.platform_player.33", K::Scalar).named("wall_reach"),
    f(34, "component.field.platform_player.34", K::Scalar)
        .named_as("wall_lockout", "wall_jump_lockout"),
    f(35, "component.field.platform_player.35", K::Scalar)
        .named_as("wall_grab", "wall_grab_stamina"),
    f(36, "component.field.platform_player.36", K::Scalar).named("dash_speed"),
    f(37, "component.field.platform_player.37", K::Scalar).named("dash_time"),
    f(38, "component.field.platform_player.38", K::Scalar).named("dash_cooldown"),
    f(39, "component.field.platform_player.39", K::Scalar).named("crouch_height"),
    f(40, "component.field.platform_player.40", K::Scalar).named("crouch_speed"),
    f(41, "component.field.platform_player.41", K::Scalar).named("swim_speed"),
    f(42, "component.field.platform_player.42", K::Scalar)
        .named_as("swim_accel", "swim_acceleration"),
    f(43, "component.field.platform_player.43", K::Scalar).named_as("swim_line", "swim_enter"),
    f(44, "component.field.platform_player.44", K::Scalar).named("ledge_grab"),
    f(45, "component.field.platform_player.45", K::Scalar).named_as("grab_window", "ledge_reach_y"),
    f(46, "component.field.platform_player.46", K::Scalar).named_as("grab_span", "ledge_span"),
    f(47, "component.field.platform_player.47", K::Scalar)
        .named_as("grab_offset_y", "ledge_offset_y"),
    f(48, "component.field.platform_player.48", K::Scalar).named("ledge_speed"),
    f(49, "component.field.platform_player.49", K::Scalar)
        .named_as("glide_fall", "glide_fall_speed"),
    f(50, "component.field.platform_player.50", K::Scalar).named_as("brake", "brake_scale"),
    f(51, "component.field.platform_player.51", K::Scalar).named_as("max_fall", "max_fall_speed"),
    f(52, "component.field.platform_player.52", K::Enum)
        .named("platform_lift")
        .viewed(crate::options![
            "full" => Full,
            "up_only" => UpOnly,
            "none" => Nothing,
        ]),
    f(53, "component.field.platform_player.53", K::Toggle).named("walk_off_ledges"),
    f(54, "component.field.platform_player.54", K::Toggle)
        .named_as("when_crouching", "crouch_walk_off_ledges"),
];

/// **`PlayerMode`** — a linha «Body» da §14 (Dynamic · Kinematic · Pure): o valor inteiro.
pub(super) const PLAYER_MODE: &[FieldDesc] = &[f(0, "component.field.player_mode.0", K::Enum)
    .named_as("body", WHOLE)
    .viewed(crate::options![
        "dynamic" => Dynamic,
        "kinematic" => Kinematic,
        "pure" => Pure,
    ])];

/// **`PulleyWheel`** — a CORDA a que ela pertence (`rope`) e o CORPO em que é montada
/// (`body`, `0` = pregada no cenário). Os ids seguem a struct (`components/rope.rs`):
/// `rope` é o 1.º campo e `body` é o 7.º. Sem descritor: `local` (8) e `mounted` (9). O `order`
/// guarda-se de 0 e o cartão conta de 1 (`Offset(1)`); o motor guarda rad/s, que é o do script.
pub(super) const PULLEY_WHEEL: &[FieldDesc] = &[
    r(1, "component.field.pulley_wheel.1")
        .named("rope")
        .viewed(View::Named(crate::Target::NameHash)),
    f(2, "component.field.pulley_wheel.2", K::Int)
        .named("order")
        .viewed(View::Offset(1)),
    f(3, "component.field.pulley_wheel.3", K::Scalar).named("radius"),
    f(4, "component.field.pulley_wheel.4", K::Enum)
        .named("wrap")
        .viewed(crate::options![
            "auto" => Auto,
            "over" => Over,
            "under" => Under,
        ]),
    f(5, "component.field.pulley_wheel.5", K::Scalar).named_as("motor", "motor_speed"),
    f(6, "component.field.pulley_wheel.6", K::Toggle).named_as("axle_breaks", "break_enabled"),
    r(7, "component.field.pulley_wheel.7")
        .named_as("mounted_on", "body")
        .viewed(View::Named(crate::Target::NameHash)),
    f(10, "component.field.pulley_wheel.10", K::Scalar).named("break_force"),
    f(11, "component.field.pulley_wheel.11", K::Scalar).named_as("out_radius", "radius_out"),
];

/// ⭐ **As linhas da §11** (doc CodeEditor 08 §3.11) — o nome é o rótulo da linha; um par pintado em
/// duas linhas («Offset X/Y») chama-se pelo rótulo comum. Os newtypes (`GravityScale(f32)`…) são o
/// valor inteiro ([`WHOLE`]), e a linha deles tem o rótulo do próprio componente.
pub(super) const RIGID_BODY: &[FieldDesc] = &[f(0, "component.field.rigid_body.0", K::Enum)
    .named_as("body", "kind")
    .viewed(crate::options![
        "dynamic" => Dynamic,
        "static" => Static,
        "kinematic" => Kinematic,
    ])];
/// ⚠️ `shape` é um enum COM DADOS (`{Ball = {radius = 0.5}}`): o script lê e escreve a tabela
/// inteira — os tamanhos não são chaves de topo, e o cartão pinta-os por baixo da forma escolhida.
pub(super) const COLLIDER: &[FieldDesc] = &[
    f(0, "component.field.collider.0", K::Enum).named("shape"),
    f(1, "component.field.collider.1", K::Scalar).named("density"),
    f(2, "component.field.collider.2", K::Scalar).named_as("bounce", "restitution"),
    f(3, "component.field.collider.3", K::Scalar).named("friction"),
    f(4, "component.field.collider.4", K::Int).named("layer"),
    f(5, "component.field.collider.5", K::Toggle).named_as("trigger", "is_sensor"),
    f(6, "component.field.collider.6", K::Vec2).named("offset"),
];
pub(super) const GRAVITY_SCALE: &[FieldDesc] =
    &[f(0, "component.field.gravity_scale.0", K::Scalar).named_as("gravity_scale", WHOLE)];
/// `init_spin` guarda rad/s — o do script (os ângulos do script são radianos); o cartão pinta graus/s.
pub(super) const INITIAL_VELOCITY: &[FieldDesc] = &[
    f(0, "component.field.initial_velocity.0", K::Vec2).named_as("init_vel", "linvel"),
    f(1, "component.field.initial_velocity.1", K::Scalar).named_as("init_spin", "angvel"),
];
pub(super) const DOMINANCE: &[FieldDesc] =
    &[f(0, "component.field.dominance.0", K::Int).named_as("dominance", WHOLE)];
pub(super) const MASS_OVERRIDE: &[FieldDesc] =
    &[f(0, "component.field.mass_override.0", K::Scalar).named_as("mass", WHOLE)];
pub(super) const COMBINE_RULES: View = crate::options![
    "average" => Average,
    "min" => Min,
    "multiply" => Multiply,
    "max" => Max,
];
pub(super) const MATERIAL_COMBINE: &[FieldDesc] = &[
    f(0, "component.field.material_combine.0", K::Enum)
        .named_as("bounce_combine", "restitution")
        .viewed(COMBINE_RULES),
    f(1, "component.field.material_combine.1", K::Enum)
        .named_as("friction_combine", "friction")
        .viewed(COMBINE_RULES),
];
pub(super) const DAMPING_OVERRIDE: &[FieldDesc] = &[
    f(0, "component.field.damping_override.0", K::Scalar).named_as("linear_damping", "linear"),
    f(1, "component.field.damping_override.1", K::Scalar).named_as("angular_damping", "angular"),
    f(2, "component.field.damping_override.2", K::Enum)
        .named_as("damp_mode", "mode")
        .viewed(crate::options!["combine" => Combine, "replace" => Replace]),
];
pub(super) const WALK_SURFACE: &[FieldDesc] = &[
    f(0, "component.field.walk_surface.0", K::Scalar).named("grip"),
    f(1, "component.field.walk_surface.1", K::Scalar).named("belt"),
];
pub(super) const SIGNAL_ON_HIT: &[FieldDesc] =
    &[f(0, "component.field.signal_on_hit.0", K::Text).named_as("on_hit", WHOLE)];
pub(super) const SIGNAL_ON_LEAVE: &[FieldDesc] =
    &[f(0, "component.field.signal_on_leave.0", K::Text).named_as("on_leave", WHOLE)];
pub(super) const AREA_EFFECTOR: &[FieldDesc] =
    &[f(0, "component.field.area_effector.0", K::Vec2).named("force")];
pub(super) const AREA_DRAG: &[FieldDesc] =
    &[f(0, "component.field.area_drag.0", K::Scalar).named_as("drag", WHOLE)];
pub(super) const AREA_BUOYANCY: &[FieldDesc] =
    &[f(0, "component.field.area_buoyancy.0", K::Scalar).named_as("fluid_density", WHOLE)];
pub(super) const AREA_FORM_DRAG: &[FieldDesc] =
    &[f(0, "component.field.area_form_drag.0", K::Scalar).named_as("shape_drag", WHOLE)];
pub(super) const AREA_TORQUE: &[FieldDesc] =
    &[f(0, "component.field.area_torque.0", K::Scalar).named_as("torque", WHOLE)];
pub(super) const AREA_FALLOFF: &[FieldDesc] =
    &[f(0, "component.field.area_falloff.0", K::Scalar).named_as("falloff", WHOLE)];

/// **`NavAgent`** — o `target` é um enum com dados: «Target» escolhe a variante e cada linha dela é um
/// campo de variante (`target:{molde}`), lido pelo NOME do alvo.
pub(super) const NAV_AGENT: &[FieldDesc] = &[
    f(0, "component.field.nav_agent.0", K::Enum)
        .named("target")
        .viewed(crate::options![
            "none" => None,
            "object" => {"Named": 0},
            "point" => {"Point": [0.0, 0.0]},
            "tag" => {"NearestTagged": 0},
            "patrol" => {"Patrol": 0},
        ]),
    f(1, "component.field.nav_agent.1", K::Scalar).named("radius"),
    f(2, "component.field.nav_agent.2", K::Scalar).named_as("arrive_at", "arrive_distance"),
    f(3, "component.field.nav_agent.3", K::Scalar).named_as("repath_at", "repath_distance"),
    f(4, "component.field.nav_agent.4", K::Scalar).named_as("stuck_after", "stuck_after_s"),
    f(5, "component.field.nav_agent.5", K::Toggle).named("active"),
    f(6, "component.field.nav_agent.6", K::Text).named_as("on_arrive", "on_arrived"),
    f(7, "component.field.nav_agent.7", K::Text).named("on_no_path"),
    f(8, "component.field.nav_agent.8", K::Text).named("on_stuck"),
    f(9, "component.field.nav_agent.9", K::Toggle).named_as("avoid_others", "avoidance"),
    f(10, "component.field.nav_agent.10", K::Toggle).named("avoid_harm"),
    f(11, "component.field.nav_agent.11", K::Text)
        .named_as("object", r#"target:{"Named":0}"#)
        .viewed(View::Named(crate::Target::NameHash)),
    f(12, "component.field.nav_agent.12", K::Vec2)
        .named_as("point", r#"target:{"Point":[0.0,0.0]}"#),
    f(13, "component.field.nav_agent.13", K::Text)
        .named_as("tag", r#"target:{"NearestTagged":0}"#)
        .viewed(View::Named(crate::Target::Tag)),
    f(14, "component.field.nav_agent.14", K::Text)
        .named_as("shape", r#"target:{"Patrol":0}"#)
        .viewed(View::Named(crate::Target::NameHash)),
];
pub(super) const NAV_COST_AREA: &[FieldDesc] = &[
    f(0, "component.field.nav_cost_area.0", K::Scalar).named("cost"),
    f(1, "component.field.nav_cost_area.1", K::Toggle).named("forbidden"),
];
/// **`NavLink`** — a saída (`to`) é um objecto pelo `stable_name_id`, lido pelo NOME.
pub(super) const NAV_LINK: &[FieldDesc] = &[
    f(0, "component.field.nav_link.0", K::Text)
        .named_as("exit", "to")
        .viewed(View::Named(crate::Target::NameHash)),
    f(1, "component.field.nav_link.1", K::Toggle).named_as("both_ways", "two_way"),
    f(2, "component.field.nav_link.2", K::Toggle).named("teleport"),
    f(3, "component.field.nav_link.3", K::Scalar).named_as("extra_cost", "cost"),
    f(4, "component.field.nav_link.4", K::Text).named_as("on_cross", "on_crossed"),
];
pub(super) const SIGNAL_TAG_FILTER: &[FieldDesc] =
    &[f(0, "component.field.signal_tag_filter.0", K::Text)
        .named_as("only_for_tag", WHOLE)
        .viewed(View::Named(crate::Target::Tag))];
pub(super) const NAV_REGION: &[FieldDesc] = &[
    f(0, "component.field.nav_region.0", K::Vec2).named_as("half_size", "half_extents"),
    f(1, "component.field.nav_region.1", K::Int).named_as("wall_layers", "obstacle_layers"),
];
pub(super) const RAY_SENSOR: &[FieldDesc] = &[
    f(0, "component.field.ray_sensor.0", K::Vec2).named("origin"),
    f(1, "component.field.ray_sensor.1", K::Vec2).named_as("direction", "dir"),
    f(2, "component.field.ray_sensor.2", K::Scalar).named("reach"),
    f(3, "component.field.ray_sensor.3", K::Int).named("layer"),
];
pub(super) const RAY_SIGNALS: &[FieldDesc] = &[
    f(0, "component.field.ray_signals.0", K::Text).named("on_enter"),
    f(1, "component.field.ray_signals.1", K::Text).named("on_exit"),
];
/// **`ProjectileMotion`** — o `homing_target` é um objecto pelo `stable_name_id`, lido pelo NOME.
pub(super) const PROJECTILE_MOTION: &[FieldDesc] = &[
    f(0, "component.field.projectile_motion.0", K::Scalar).named_as("speed", "initial_speed"),
    f(1, "component.field.projectile_motion.1", K::Scalar).named("acceleration"),
    f(2, "component.field.projectile_motion.2", K::Scalar).named("max_speed"),
    f(3, "component.field.projectile_motion.3", K::Scalar).named("gravity"),
    f(4, "component.field.projectile_motion.4", K::Scalar).named("bounciness"),
    f(5, "component.field.projectile_motion.5", K::Int).named("max_bounces"),
    f(6, "component.field.projectile_motion.6", K::Scalar).named("range"),
    f(7, "component.field.projectile_motion.7", K::Toggle).named("face_velocity"),
    f(8, "component.field.projectile_motion.8", K::Scalar).named_as("homing", "homing_accel"),
    f(9, "component.field.projectile_motion.9", K::Text)
        .named_as("target", "homing_target")
        .viewed(View::Named(crate::Target::NameHash)),
];
/// **`TopDownPlayer`** — os três seletores (`Directions`, `Viewpoint`, `Facing`) guardam um `u8`; o
/// script escreve o NOME da opção do cartão. Os ângulos guardam graus (e graus/s): o script, radianos.
pub(super) const TOP_DOWN_PLAYER: &[FieldDesc] = &[
    f(0, "component.field.top_down_player.0", K::Scalar).named("speed"),
    f(1, "component.field.top_down_player.1", K::Scalar).named("acceleration"),
    f(2, "component.field.top_down_player.2", K::Scalar).named("deceleration"),
    f(3, "component.field.top_down_player.3", K::Int)
        .named_as("directions", "direction_mode")
        .viewed(
            crate::options!["free" => 0, "8_dir" => 1, "4_dir" => 2, "x_only" => 3, "y_only" => 4],
        ),
    f(4, "component.field.top_down_player.4", K::Int)
        .named("viewpoint")
        .viewed(crate::options![
            "top_down" => 0,
            "iso_2_1" => 1,
            "iso_30" => 2,
            "custom" => 3,
        ]),
    f(5, "component.field.top_down_player.5", K::Angle)
        .named_as("board_angle", "viewpoint_angle_deg")
        .viewed(View::Degrees),
    f(6, "component.field.top_down_player.6", K::Int)
        .named_as("facing", "rotation_mode")
        .viewed(crate::options!["don_t_turn" => 0, "face_move" => 1, "face_4" => 2, "face_8" => 3]),
    f(7, "component.field.top_down_player.7", K::Scalar)
        .named_as("turn_speed", "rotation_speed_deg")
        .viewed(View::Degrees),
    f(8, "component.field.top_down_player.8", K::Angle)
        .named_as("min_slide_angle", "min_slide_angle_deg")
        .viewed(View::Degrees),
    f(9, "component.field.top_down_player.9", K::Int).named("max_slides"),
    f(10, "component.field.top_down_player.10", K::Toggle).named("default_controls"),
    f(11, "component.field.top_down_player.11", K::Scalar)
        .named_as("push_recovery", "knockback_recovery"),
];
