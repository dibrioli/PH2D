//! **A família da física** — os 32 componentes de `ph2d-physics-ecs`.
//!
//! ⚠️ **São CONFIG, nunca estado vivo do solver** ([ADR-0131](../../../../docs/architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md)):
//! *"o undo ordena por bytes"*. Isso é o que os torna anexáveis com segurança — anexar um
//! `RigidBody` é declarar uma intenção, não injetar um corpo no meio de um passo do solver.
//!
//! # ⭐⭐ UMA PORTA, e 31 LINHAS DE SECÇÃO (report do dono, 2026-09-13 e 14)
//!
//! *«Os componentes da física foram colocados no modal que é aberto ao clicar no `+` do inspector.
//! Contudo foi erroneamente picotado, dividido em inúmeros supostos componentes que na verdade são
//! apenas seções das opções de física.»*
//!
//! Medido: dos **32** tipos descritos aqui, **30 estavam `Authored`** — `30` de `85` itens da
//! paleta inteira, **35 %**, e de longe a maior família (a seguinte, `core`, oferece 22). E
//! **nenhum** deles, menos um, é uma escolha do artista: são as **rows** que a §11/§12/§13/§14 já
//! pinta *e já anexa*, ou gestos que a paleta genérica não sabe exprimir.
//!
//! ⭐⭐ **E a decisão final é do dono** (2026-09-14, depois de ver a poda de três):
//! *«um objeto de física (Physics Body) e todas as opções aparecem com ele (inclusive Collision
//! Shape e Platform Player)»*. ⇒ **`30 → 1`**.
//!
//! ⚠️ **A régua não é nova — é a razão 2 do próprio [`crate::Attach::Intrinsic`]**, que já estava
//! escrita e aplicada a dois destes (`Dominance`, `MassOverride`): *o neutro existe e anexá-lo
//! seria um **NO-OP**; a PRESENÇA é que carrega o sentido, e o valor de anexação vem do CONTEXTO,
//! que a paleta genérica não conhece.* O que faltava era **correr a régua sobre a família inteira**
//! — o helper [`i`] nasceu `p` (autorado) e ninguém classificou item a item.
//!
//! ⚠️ **Medição do outro lado, e é ela que autoriza a poda:** cada um dos 27 tem um gesto do
//! Inspector que o anexa e o destaca, pelo idioma da presença-override. `PhysicsFieldEdit` (o
//! vocabulário da §11) nomeia-os um a um — `Ccd`, `LockPositionX/Y`, `LockRotation`,
//! `GravityScale`, `InitialVelocity`, `MaterialCombine`, `DampingOverride`, `OneWayPlatform`,
//! `NoWallCling`, `WalkSurface` e as **sete** da zona (`AreaEffector`, `AreaDrag`, `AreaBuoyancy`,
//! `AreaFormDrag`, `AreaTorque`, `AreaFalloff`, `AreaForceWorldAxes`); o `PlayerMode` e o
//! `PlayerSignals` saem dos toggles da §14; o `SignalOnHit`/`SignalOnLeave` das duas rows de texto;
//! e o `PhysicsJoint`, o `PulleyWheel`, o `WestonAxle`, o `JointWorldAnchor` e o `RopeStops` nascem
//! de **gestos** (*Join* · *Join by drawing* · *Rig* · a roldana · o pino de mundo · o batente).
//!
//! ⛔ **Um deles anexado pelo `+` é pior que ausente**, e é isso que separa esta poda de uma
//! arrumação: um `PhysicsJoint::default()` prende `StableId 0` a `StableId 0` — uma junta que não
//! prende nada, num objeto que pode nem ser corpo. As zonas e os overrides são mais benignos e têm
//! o mesmo defeito de fundo: **o componente entra e nada acontece**, porque o valor neutro é
//! exatamente o que a ausência já dizia.
//!
//! ⇒ a paleta oferece **UMA** entrada de física, e tudo o resto chega com ela ou por um gesto da
//! secção que fala dele:
//!
//! | Onde | O que | Porque não é um item de paleta |
//! |---|---|---|
//! | **`+` → Physics → Physics Body** (`RigidBody`) | *este objeto é simulado* | é a porta |
//! | **§11, ao nascer** (`Collider`) | forma · meias-extensões · offset · densidade · quique · atrito · camada · Solid\|Sensor | as opções **são** as rows da §11, e ela nasce com o corpo |
//! | **§11, face vazia** (`Collider` sozinho) | *Add Shape to X* — esta forma é mais uma peça do corpo acima | ⛔ a paleta não sabe **NOMEAR o dono**, e sem o nome o gesto não existe |
//! | **§14, face vazia** (`PlatformPlayer`) | *Make Platform Player* | ⛔ só faz sentido num corpo `Dynamic` (a mola é um impulso), e o `+` genérico não sabe perguntar isso |
//!
//! ⚠️ **As duas faces vazias VOLTARAM**, e é uma reversão parcial e declarada da F3 do ADR-0166
//! (*«o Inspector mostra o que o objecto TEM»*): a §11 volta a aparecer num filho de um corpo, e a
//! §14 em todo corpo `Dynamic`. O que **não** volta é a doença que a F3 curou — um objecto pelado
//! não mostra secção de física nenhuma, e as doze secções de zeros continuam mortas.
//!
//! ⚠️ **O rótulo passa a nomear a SECÇÃO que nascerá**, que é a regra do campo
//! [`crate::ComponentDesc::display_key`] (*nomeado pelo resultado, não pelo tipo Rust*):
//! `Rigid Body` → **Physics Body** (o cabeçalho da §11, e a palavra do botão que o dono conhecia).
//! O `Collider` fica **Collision Shape** para o rótulo *«brings …»* dizer o que de facto chega.
//!
//! ⚠️ **`applies_to` é `ANY` de propósito, e é a medição que o manda:** a ponte da física vê
//! as entidades por *query* (`BodyQuery = (Entity, &RigidBody, &Collider, &Transform)`), e
//! nada nela pergunta que tipo de objeto é — **uma peça materializada com `RigidBody`
//! funciona sem tocar na ponte** (doc 04 §1.2, item 4). Um caminho vetorial, um objeto Flip
//! e um objeto vazio podem todos ser corpos. Estreitar isto para `IMAGE` seria escrever na
//! tabela uma limitação que o código não tem.
//!
//! # ✅ As quatro referências por identidade — DECLARADAS (F1 migrou, F4.2 remapeia)
//!
//! `PhysicsJoint.body_a/b` e `PulleyWheel.rope/.body` **eram** `stable_name_id` — hash do
//! `Name` —, e era por isso que *a junta de uma cópia prendia os corpos do MESTRE* (a cópia
//! recebe nome `" (1)"`, o hash muda, e a referência continuava a nomear o original). A F1
//! migrou-os para [`ph2d_ecs::StableId`], e desde a **F4.2** eles estão declarados aqui com
//! [`crate::RefKind::Object`].
//!
//! ⚠️ **É esta declaração que FAZ o remap acontecer**, e não uma lista de casos escrita à mão
//! do outro lado: a tabela de remapeadores da shell é conferida contra estes campos por um
//! censo de dois lados, então declarar uma referência nova sem quem a reescreva **reprova**.
//! [`ph2d_ecs::StableId`]: https://github.com/dibrioli/PH2D/blob/main/crates/ph2d-ecs/src/stable_id.rs

use crate::{
    ComponentCategory as C, ComponentDesc as D, FieldDesc, FieldKind as K, Hidden as H,
    ObjectKinds as O, Propagation, RefKind, View, WHOLE,
};

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
const JOINT_KINDS: View = crate::options![
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
const JOINT: &[FieldDesc] = &[
    r(1, "component.field.joint.1").keyed("body_a"),
    r(2, "component.field.joint.2").keyed("body_b"),
    f(3, "component.field.joint.3", K::Enum).named("kind").viewed(JOINT_KINDS),
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
const PLATFORM_PLAYER: &[FieldDesc] = &[
    f(0, "component.field.platform_player.0", K::Scalar).named("float_height"),
    f(1, "component.field.platform_player.1", K::Scalar).named("cling_distance"),
    f(2, "component.field.platform_player.2", K::Scalar).named_as("leg_stiffness", "spring_strength"),
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
    f(12, "component.field.platform_player.12", K::Scalar).named_as("takeoff_above", "takeoff_speed"),
    f(13, "component.field.platform_player.13", K::Scalar).named("peak_gravity"),
    f(14, "component.field.platform_player.14", K::Scalar).named_as("peak_window", "peak_speed"),
    f(15, "component.field.platform_player.15", K::Scalar).named("fall_gravity"),
    f(16, "component.field.platform_player.16", K::Scalar).named("cut_gravity"),
    f(17, "component.field.platform_player.17", K::Scalar).named("coyote_time"),
    f(18, "component.field.platform_player.18", K::Scalar).named("jump_buffer"),
    f(19, "component.field.platform_player.19", K::Int).named_as("foot_rays", "foot_samples"),
    f(20, "component.field.platform_player.20", K::Scalar).named_as("foot_ray_spread", "foot_spread"),
    f(21, "component.field.platform_player.21", K::Int).named_as("corner_rays", "corner_samples"),
    f(22, "component.field.platform_player.22", K::Scalar).named_as("corner_look_ahead", "corner_lookahead"),
    f(23, "component.field.platform_player.23", K::Int).named_as("wall_rays", "wall_samples"),
    f(24, "component.field.platform_player.24", K::Scalar).named_as("wall_ray_spread", "wall_spread"),
    f(25, "component.field.platform_player.25", K::Scalar).named("corner_reach"),
    f(26, "component.field.platform_player.26", K::Scalar).named("lift_momentum"),
    f(27, "component.field.platform_player.27", K::Scalar).named_as("weight_on_ground", "reaction_support"),
    f(28, "component.field.platform_player.28", K::Scalar).named_as("push_on_ground", "reaction_movement"),
    f(29, "component.field.platform_player.29", K::Scalar).named_as("push_on_bodies", "reaction_push"),
    f(30, "component.field.platform_player.30", K::Scalar).named_as("wall_slide", "wall_slide_speed"),
    f(31, "component.field.platform_player.31", K::Scalar).named_as("wall_jump", "wall_jump_height"),
    f(32, "component.field.platform_player.32", K::Scalar).named_as("wall_push", "wall_jump_push"),
    f(33, "component.field.platform_player.33", K::Scalar).named("wall_reach"),
    f(34, "component.field.platform_player.34", K::Scalar).named_as("wall_lockout", "wall_jump_lockout"),
    f(35, "component.field.platform_player.35", K::Scalar).named_as("wall_grab", "wall_grab_stamina"),
    f(36, "component.field.platform_player.36", K::Scalar).named("dash_speed"),
    f(37, "component.field.platform_player.37", K::Scalar).named("dash_time"),
    f(38, "component.field.platform_player.38", K::Scalar).named("dash_cooldown"),
    f(39, "component.field.platform_player.39", K::Scalar).named("crouch_height"),
    f(40, "component.field.platform_player.40", K::Scalar).named("crouch_speed"),
    f(41, "component.field.platform_player.41", K::Scalar).named("swim_speed"),
    f(42, "component.field.platform_player.42", K::Scalar).named_as("swim_accel", "swim_acceleration"),
    f(43, "component.field.platform_player.43", K::Scalar).named_as("swim_line", "swim_enter"),
    f(44, "component.field.platform_player.44", K::Scalar).named("ledge_grab"),
    f(45, "component.field.platform_player.45", K::Scalar).named_as("grab_window", "ledge_reach_y"),
    f(46, "component.field.platform_player.46", K::Scalar).named_as("grab_span", "ledge_span"),
    f(47, "component.field.platform_player.47", K::Scalar).named_as("grab_offset_y", "ledge_offset_y"),
    f(48, "component.field.platform_player.48", K::Scalar).named("ledge_speed"),
    f(49, "component.field.platform_player.49", K::Scalar).named_as("glide_fall", "glide_fall_speed"),
    f(50, "component.field.platform_player.50", K::Scalar).named_as("brake", "brake_scale"),
    f(51, "component.field.platform_player.51", K::Scalar).named_as("max_fall", "max_fall_speed"),
    f(52, "component.field.platform_player.52", K::Enum).named("platform_lift").viewed(crate::options![
        "full" => Full,
        "up_only" => UpOnly,
        "none" => Nothing,
    ]),
    f(53, "component.field.platform_player.53", K::Toggle).named("walk_off_ledges"),
    f(54, "component.field.platform_player.54", K::Toggle).named_as("when_crouching", "crouch_walk_off_ledges"),
];

/// **`PlayerMode`** — a linha «Body» da §14 (Dynamic · Kinematic · Pure): o valor inteiro.
const PLAYER_MODE: &[FieldDesc] =
    &[f(0, "component.field.player_mode.0", K::Enum).named_as("body", WHOLE).viewed(crate::options![
        "dynamic" => Dynamic,
        "kinematic" => Kinematic,
        "pure" => Pure,
    ])];

/// **`PulleyWheel`** — a CORDA a que ela pertence (`rope`) e o CORPO em que é montada
/// (`body`, `0` = pregada no cenário). Os ids seguem a struct (`components/rope.rs`):
/// `rope` é o 1.º campo e `body` é o 7.º. Sem descritor: `local` (8) e `mounted` (9). O `order`
/// guarda-se de 0 e o cartão conta de 1 (`Offset(1)`); o motor guarda rad/s, que é o do script.
const PULLEY_WHEEL: &[FieldDesc] = &[
    r(1, "component.field.pulley_wheel.1").keyed("rope"),
    f(2, "component.field.pulley_wheel.2", K::Int).named("order").viewed(View::Offset(1)),
    f(3, "component.field.pulley_wheel.3", K::Scalar).named("radius"),
    f(4, "component.field.pulley_wheel.4", K::Enum).named("wrap").viewed(crate::options![
        "auto" => Auto,
        "over" => Over,
        "under" => Under,
    ]),
    f(5, "component.field.pulley_wheel.5", K::Scalar).named_as("motor", "motor_speed"),
    f(6, "component.field.pulley_wheel.6", K::Toggle).named_as("axle_breaks", "break_enabled"),
    r(7, "component.field.pulley_wheel.7").keyed("body"),
    f(10, "component.field.pulley_wheel.10", K::Scalar).named("break_force"),
    f(11, "component.field.pulley_wheel.11", K::Scalar).named_as("out_radius", "radius_out"),
];

/// ⭐ **As linhas da §11** (doc CodeEditor 08 §3.11) — o nome é o rótulo da linha; um par pintado em
/// duas linhas («Offset X/Y») chama-se pelo rótulo comum. Os newtypes (`GravityScale(f32)`…) são o
/// valor inteiro ([`WHOLE`]), e a linha deles tem o rótulo do próprio componente.
const RIGID_BODY: &[FieldDesc] =
    &[f(0, "component.field.rigid_body.0", K::Enum).named_as("body", "kind").viewed(crate::options![
        "dynamic" => Dynamic,
        "static" => Static,
        "kinematic" => Kinematic,
    ])];
/// ⚠️ `shape` é um enum COM DADOS (`{Ball = {radius = 0.5}}`): o script lê e escreve a tabela
/// inteira — os tamanhos não são chaves de topo, e o cartão pinta-os por baixo da forma escolhida.
const COLLIDER: &[FieldDesc] = &[
    f(0, "component.field.collider.0", K::Enum).named("shape"),
    f(1, "component.field.collider.1", K::Scalar).named("density"),
    f(2, "component.field.collider.2", K::Scalar).named_as("bounce", "restitution"),
    f(3, "component.field.collider.3", K::Scalar).named("friction"),
    f(4, "component.field.collider.4", K::Int).named("layer"),
    f(5, "component.field.collider.5", K::Toggle).named_as("trigger", "is_sensor"),
    f(6, "component.field.collider.6", K::Vec2).named("offset"),
];
const GRAVITY_SCALE: &[FieldDesc] =
    &[f(0, "component.field.gravity_scale.0", K::Scalar).named_as("gravity_scale", WHOLE)];
/// `init_spin` guarda rad/s — o do script (os ângulos do script são radianos); o cartão pinta graus/s.
const INITIAL_VELOCITY: &[FieldDesc] = &[
    f(0, "component.field.initial_velocity.0", K::Vec2).named_as("init_vel", "linvel"),
    f(1, "component.field.initial_velocity.1", K::Scalar).named_as("init_spin", "angvel"),
];
const DOMINANCE: &[FieldDesc] =
    &[f(0, "component.field.dominance.0", K::Int).named_as("dominance", WHOLE)];
const MASS_OVERRIDE: &[FieldDesc] =
    &[f(0, "component.field.mass_override.0", K::Scalar).named_as("mass", WHOLE)];
const COMBINE_RULES: View = crate::options![
    "average" => Average,
    "min" => Min,
    "multiply" => Multiply,
    "max" => Max,
];
const MATERIAL_COMBINE: &[FieldDesc] = &[
    f(0, "component.field.material_combine.0", K::Enum)
        .named_as("bounce_combine", "restitution")
        .viewed(COMBINE_RULES),
    f(1, "component.field.material_combine.1", K::Enum)
        .named_as("friction_combine", "friction")
        .viewed(COMBINE_RULES),
];
const DAMPING_OVERRIDE: &[FieldDesc] = &[
    f(0, "component.field.damping_override.0", K::Scalar).named_as("linear_damping", "linear"),
    f(1, "component.field.damping_override.1", K::Scalar).named_as("angular_damping", "angular"),
    f(2, "component.field.damping_override.2", K::Enum)
        .named_as("damp_mode", "mode")
        .viewed(crate::options!["combine" => Combine, "replace" => Replace]),
];
const WALK_SURFACE: &[FieldDesc] = &[
    f(0, "component.field.walk_surface.0", K::Scalar).named("grip"),
    f(1, "component.field.walk_surface.1", K::Scalar).named("belt"),
];
const SIGNAL_ON_HIT: &[FieldDesc] =
    &[f(0, "component.field.signal_on_hit.0", K::Text).named_as("on_hit", WHOLE)];
const SIGNAL_ON_LEAVE: &[FieldDesc] =
    &[f(0, "component.field.signal_on_leave.0", K::Text).named_as("on_leave", WHOLE)];
const AREA_EFFECTOR: &[FieldDesc] =
    &[f(0, "component.field.area_effector.0", K::Vec2).named("force")];
const AREA_DRAG: &[FieldDesc] =
    &[f(0, "component.field.area_drag.0", K::Scalar).named_as("drag", WHOLE)];
const AREA_BUOYANCY: &[FieldDesc] =
    &[f(0, "component.field.area_buoyancy.0", K::Scalar).named_as("fluid_density", WHOLE)];
const AREA_FORM_DRAG: &[FieldDesc] =
    &[f(0, "component.field.area_form_drag.0", K::Scalar).named_as("shape_drag", WHOLE)];
const AREA_TORQUE: &[FieldDesc] =
    &[f(0, "component.field.area_torque.0", K::Scalar).named_as("torque", WHOLE)];
const AREA_FALLOFF: &[FieldDesc] =
    &[f(0, "component.field.area_falloff.0", K::Scalar).named_as("falloff", WHOLE)];

/// Uma linha de secção COM campos — o [`i`] com o bloco dela.
const fn il(
    canonical_name: &'static str,
    display_key: &'static str,
    fields: &'static [FieldDesc],
) -> D {
    D::intrinsic(canonical_name, display_key, C::Physics, fields)
}

/// ⭐ **Uma LINHA DE SECÇÃO, não um item de paleta** — ver a tabela das três portas no cabeçalho.
///
/// O componente existe, é editadíssimo e **não se escolhe**: quem o anexa (e o destaca no neutro)
/// é o knob da §11/§12/§13/§14 que fala dele. Anexá-lo pelo `+` seria, no melhor caso, um clique
/// que não muda nada — e no pior (`PhysicsJoint`) uma junta a prender coisa nenhuma.
///
/// ⚠️ **Continua a poder ter secção** ([`crate::ComponentDesc::may_have_section`]): `Intrinsic` diz
/// *não se oferece*, nunca *não se edita*. É precisamente a distinção que a terceira variante
/// comprou.
const fn i(canonical_name: &'static str, display_key: &'static str) -> D {
    D::intrinsic(canonical_name, display_key, C::Physics, &[])
}

/// **A PORTA, e o que ela não funciona sem** — ver [`D::requires`].
///
/// ⚠️ Irmã do [`i`] pelo lado oposto: ali o componente não se escolhe, aqui ele escolhe-se *e traz
/// companhia*. (Era *«irmã do `p`»*, o helper autorado que a poda de 13/09 apagou — quando o vizinho
/// que dá nome a uma nota deixa de existir, o que se corrige é a nota.)
///
/// ⚠️ **UMA entrada em toda a família, e ela é uma QUERY.** A ponte consulta
/// `(Entity, &RigidBody, &Collider, &Transform)`: um corpo sem collider nunca entra no solver. ⛔ A
/// barra é *inerte sem aquele*, nunca boa prática — as zonas, os joints e os markers ficam de fora
/// de propósito. (Eram duas até 2026-09-14: o `PlatformPlayer` exigia o `RigidBody` pela mesma
/// query, e deixou de precisar de o declarar quando a porta dele passou a ser um botão que só
/// existe **sobre** um corpo.)
const fn pr(
    canonical_name: &'static str,
    display_key: &'static str,
    fields: &'static [FieldDesc],
    requires: &'static [&'static str],
) -> D {
    D::authored_requiring(
        canonical_name,
        display_key,
        C::Physics,
        O::ANY,
        fields,
        requires,
    )
}

/// **`NavAgent`** — o id `0` é o `target` (um objecto, um ponto, uma tag ou uma forma por id: uma
/// referência, que ganha nome no d4).
const NAV_AGENT: &[FieldDesc] = &[
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
];
const NAV_COST_AREA: &[FieldDesc] = &[
    f(0, "component.field.nav_cost_area.0", K::Scalar).named("cost"),
    f(1, "component.field.nav_cost_area.1", K::Toggle).named("forbidden"),
];
/// **`NavLink`** — o id `0` é a saída (`to`, um objecto por id: referência, d4).
const NAV_LINK: &[FieldDesc] = &[
    f(1, "component.field.nav_link.1", K::Toggle).named_as("both_ways", "two_way"),
    f(2, "component.field.nav_link.2", K::Toggle).named("teleport"),
    f(3, "component.field.nav_link.3", K::Scalar).named_as("extra_cost", "cost"),
    f(4, "component.field.nav_link.4", K::Text).named_as("on_cross", "on_crossed"),
];
const NAV_REGION: &[FieldDesc] = &[
    f(0, "component.field.nav_region.0", K::Vec2).named_as("half_size", "half_extents"),
    f(1, "component.field.nav_region.1", K::Int).named_as("wall_layers", "obstacle_layers"),
];
const RAY_SENSOR: &[FieldDesc] = &[
    f(0, "component.field.ray_sensor.0", K::Vec2).named("origin"),
    f(1, "component.field.ray_sensor.1", K::Vec2).named_as("direction", "dir"),
    f(2, "component.field.ray_sensor.2", K::Scalar).named("reach"),
    f(3, "component.field.ray_sensor.3", K::Int).named("layer"),
];
const RAY_SIGNALS: &[FieldDesc] = &[
    f(0, "component.field.ray_signals.0", K::Text).named("on_enter"),
    f(1, "component.field.ray_signals.1", K::Text).named("on_exit"),
];
/// **`ProjectileMotion`** — o `homing_target` (um objecto por id) é referência, d4.
const PROJECTILE_MOTION: &[FieldDesc] = &[
    f(0, "component.field.projectile_motion.0", K::Scalar).named_as("speed", "initial_speed"),
    f(1, "component.field.projectile_motion.1", K::Scalar).named("acceleration"),
    f(2, "component.field.projectile_motion.2", K::Scalar).named("max_speed"),
    f(3, "component.field.projectile_motion.3", K::Scalar).named("gravity"),
    f(4, "component.field.projectile_motion.4", K::Scalar).named("bounciness"),
    f(5, "component.field.projectile_motion.5", K::Int).named("max_bounces"),
    f(6, "component.field.projectile_motion.6", K::Scalar).named("range"),
    f(7, "component.field.projectile_motion.7", K::Toggle).named("face_velocity"),
    f(8, "component.field.projectile_motion.8", K::Scalar).named_as("homing", "homing_accel"),
];
/// **`TopDownPlayer`** — os três seletores (`Directions`, `Viewpoint`, `Facing`) guardam um `u8`; o
/// script escreve o NOME da opção do cartão. Os ângulos guardam graus (e graus/s): o script, radianos.
const TOP_DOWN_PLAYER: &[FieldDesc] = &[
    f(0, "component.field.top_down_player.0", K::Scalar).named("speed"),
    f(1, "component.field.top_down_player.1", K::Scalar).named("acceleration"),
    f(2, "component.field.top_down_player.2", K::Scalar).named("deceleration"),
    f(3, "component.field.top_down_player.3", K::Int).named_as("directions", "direction_mode").viewed(
        crate::options!["free" => 0, "8_dir" => 1, "4_dir" => 2, "x_only" => 3, "y_only" => 4],
    ),
    f(4, "component.field.top_down_player.4", K::Int).named("viewpoint").viewed(crate::options![
        "top_down" => 0,
        "iso_2_1" => 1,
        "iso_30" => 2,
        "custom" => 3,
    ]),
    f(5, "component.field.top_down_player.5", K::Angle)
        .named_as("board_angle", "viewpoint_angle_deg")
        .viewed(View::Degrees),
    f(6, "component.field.top_down_player.6", K::Int).named_as("facing", "rotation_mode").viewed(
        crate::options!["don_t_turn" => 0, "face_move" => 1, "face_4" => 2, "face_8" => 3],
    ),
    f(7, "component.field.top_down_player.7", K::Scalar)
        .named_as("turn_speed", "rotation_speed_deg")
        .viewed(View::Degrees),
    f(8, "component.field.top_down_player.8", K::Angle)
        .named_as("min_slide_angle", "min_slide_angle_deg")
        .viewed(View::Degrees),
    f(9, "component.field.top_down_player.9", K::Int).named("max_slides"),
    f(10, "component.field.top_down_player.10", K::Toggle).named("default_controls"),
    f(11, "component.field.top_down_player.11", K::Scalar).named_as("push_recovery", "knockback_recovery"),
];

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
///
/// ⚠️ **UMA porta e 31 `Intrinsic`** — a tabela vive no cabeçalho, e o gate
/// [`tests::the_physics_family_offers_one_door_and_not_its_rows`] impede que a segunda entre sem
/// alguém decidir.
pub const DESCS: &[D] = &[
    // As SETE da zona — todas rows do bloco de área da §11, pintadas quando o collider é
    // *Sensor* e escritas pelos `PhysicsFieldEdit::Force*/AreaDrag/AreaDensity/AreaFormDrag/
    // AreaTorque/AreaFalloff/ForceWorldAxes`. ⚠️ Fora de um sensor elas **não têm leitor**: a
    // ponte não lê `AreaEffector` de uma peça nem de um corpo sólido.
    il("ph2d::physics::AreaBuoyancy", "component.area_buoyancy.name", AREA_BUOYANCY).coded("buoyancy_zone").row_of("ph2d::physics::Collider"),
    il("ph2d::physics::AreaDrag", "component.area_drag.name", AREA_DRAG).coded("drag_zone").row_of("ph2d::physics::Collider"),
    il("ph2d::physics::AreaEffector", "component.area_effector.name", AREA_EFFECTOR).coded("force_zone").row_of("ph2d::physics::Collider"),
    il("ph2d::physics::AreaFalloff", "component.area_falloff.name", AREA_FALLOFF).coded("zone_falloff").row_of("ph2d::physics::Collider"),
    i("ph2d::physics::AreaForceWorldAxes", "component.area_force_world_axes.name").hidden(H::Presence),
    il("ph2d::physics::AreaFormDrag", "component.area_form_drag.name", AREA_FORM_DRAG).coded("form_drag_zone").row_of("ph2d::physics::Collider"),
    il("ph2d::physics::AreaTorque", "component.area_torque.name", AREA_TORQUE).coded("torque_zone").row_of("ph2d::physics::Collider"),
    i("ph2d::physics::Ccd", "component.ccd.name").hidden(H::Presence),
    // ⭐⭐ **A forma do corpo — e ela CHEGA COM ELE** (ordem do dono, 2026-09-14). Todas as opções
    // dela (forma, meias-extensões, offset, densidade, quique, atrito, camada, Solid|Sensor) são
    // rows da §11, que nasce com o `Physics Body`. A outra vida dela — *esta forma é mais uma peça
    // do corpo ancestral* (W-Compound) — é o botão **Add Shape to X** da face vazia da §11, que
    // NOMEIA o dono; a paleta genérica não sabe nomeá-lo, e por isso aquela é a porta.
    //
    // ⚠️ **Ser `Intrinsic` e ser dependência do `RigidBody` deixou de ser contraditório:** o gate
    // `every_declared_requirement_names_a_real_component` exigia `Authored` e o recurso de que ele
    // falava é OUTRO — quem constrói a cascata é o `insert_default` do REGISTO, que não consulta o
    // `attach`. Hoje ele exige o que de facto é preciso: que o registo saiba construir o alvo.
    il("ph2d::physics::Collider", "component.collider.name", COLLIDER).coded("collision_shape").row_without_neutral("ph2d::physics::RigidBody"),
    il("ph2d::physics::DampingOverride", "component.damping_override.name", DAMPING_OVERRIDE).coded("damping").row_of("ph2d::physics::RigidBody"),
    // ⚠️⚠️ `Dominance` e `MassOverride` são `Intrinsic` por uma CERCA, não por falta de
    // desenho — e a cerca está escrita no doc-comment deles (`components/overrides.rs`):
    // *"absent = the neutral default and the Inspector detaches it at 0 (a project file
    // stays free of the no-op)"*. Neles a PRESENÇA é que carrega o sentido, e o valor de
    // anexação teria de vir do CONTEXTO (a massa que o corpo tem agora) — que a paleta
    // genérica não conhece. ⇒ **A porta por-seção deles não é redundante com o `+`**: ela
    // SEMEIA do valor vivo, que é uma coisa que o `+` não pode fazer (ADR-0166).
    il("ph2d::physics::Dominance", "component.dominance.name", DOMINANCE).coded("dominance").row_of("ph2d::physics::RigidBody"),
    il("ph2d::physics::GravityScale", "component.gravity_scale.name", GRAVITY_SCALE).coded("gravity_scale").row_of("ph2d::physics::RigidBody"),
    il("ph2d::physics::InitialVelocity", "component.initial_velocity.name", INITIAL_VELOCITY).coded("initial_velocity").row_of("ph2d::physics::RigidBody"),
    // O pino de MUNDO: ele chega com o gesto que prega a junta ao cenário (`joint_world`), e
    // sozinho não nomeia ponto nenhum.
    i("ph2d::physics::JointWorldAnchor", "component.joint_world_anchor.name").hidden(H::Presence),
    i("ph2d::physics::LockPositionX", "component.lock_position_x.name").hidden(H::Presence),
    i("ph2d::physics::LockPositionY", "component.lock_position_y.name").hidden(H::Presence),
    i("ph2d::physics::LockRotation", "component.lock_rotation.name").hidden(H::Presence),
    // Irmã do `Dominance` acima, pela mesma cerca (a massa e a densidade são a mesma
    // grandeza por dois caminhos; ausente = a densidade manda).
    il("ph2d::physics::MassOverride", "component.mass_override.name", MASS_OVERRIDE).coded("mass").row_without_neutral("ph2d::physics::RigidBody"),
    il("ph2d::physics::MaterialCombine", "component.material_combine.name", MATERIAL_COMBINE).coded("material_combine").row_of("ph2d::physics::Collider"),
    // ⭐⭐⭐ **PORTAS 5 e 6 — a NAVEGAÇÃO** (plano 30, W4): *este objecto PERSEGUE* e *os agentes andam
    // AQUI* são duas intenções do artista, e nenhuma é uma row que a §11 já pinte.
    //
    // ⭐⭐ **O agente REQUER o `TopDownPlayer`** (que por sua vez requer o `RigidBody`, e a cascata é
    // recursiva): o agente PEDE e o mover ANDA — sem mover, a paleta entregaria um componente inerte.
    // A semente dele desliga o teclado do mover (`physics_seed::seed_nav_agent`), senão os dois
    // falariam ao mesmo tempo. ⛔ A região **não** pede companhia: ela é um rectângulo, e os
    // obstáculos são os corpos estáticos que já existem.
    pr(
        "ph2d::physics::NavAgent",
        "component.nav_agent.name",
        NAV_AGENT,
        &["ph2d::physics::TopDownPlayer"],
    )
    .coded("nav_agent")
    .hiding(&[("target", H::Reference)]),
    // ⭐⭐ **PORTAS 7 e 8 — o que CUSTA e os ATALHOS** (plano 30, W7): *aqui é lama / aqui ninguém
    // entra* e *daqui salta-se para ali* são intenções do artista. ⛔ **Nenhuma pede companhia:** a
    // área usa o colisor que o objecto JÁ tem (uma poça desenhada, um sensor) e a secção diz quando
    // falta — trazer um `RigidBody` de fábrica entregava um corpo `Dynamic` que cai e que a ponte
    // não recorta. O atalho só precisa da POSIÇÃO da entrada e do nome da saída.
    D::authored(
        "ph2d::physics::NavCostArea",
        "component.nav_cost_area.name",
        C::Physics,
        O::ANY,
        NAV_COST_AREA,
    )
    .coded("nav_cost_area"),
    D::authored(
        "ph2d::physics::NavLink",
        "component.nav_link.name",
        C::Physics,
        O::ANY,
        NAV_LINK,
    )
    .coded("nav_link")
    .hiding(&[("to", H::Reference)]),
    D::authored(
        "ph2d::physics::NavRegion",
        "component.nav_region.name",
        C::Physics,
        O::ANY,
        NAV_REGION,
    )
    .coded("nav_region"),
    i("ph2d::physics::NoWallCling", "component.no_wall_cling.name").hidden(H::Presence),
    i("ph2d::physics::OneWayPlatform", "component.one_way_platform.name").hidden(H::Presence),
    // ⛔⛔ **A junta é o caso em que oferecer é PIOR que não oferecer.** Ela nasce dos gestos que
    // conhecem os DOIS corpos (*Join Selected Bodies* · *Join by drawing* · *Rig N Parts*), e o
    // ponto neutro do tipo prende `StableId 0` a `StableId 0`: uma junta que não prende nada, num
    // objeto que pode nem ser corpo. ✅ `body_a`/`body_b` seguem DECLARADOS (F4.2) — `Intrinsic`
    // **pode** ter campos, e sem eles a junta de uma instância prende os corpos do mestre (gate
    // `the_instance_joint_binds_the_instances_own_bodies`).
    D::intrinsic(
        "ph2d::physics::PhysicsJoint",
        "component.physics_joint.name",
        C::Physics,
        JOINT,
    )
    .coded("joint")
    .hiding(&[
        ("body_a", H::Reference),
        ("body_b", H::Reference),
        // As alças A/B arrastam-se no canvas; o cartão não as mostra.
        ("local_a", H::OtherEditor),
        ("local_b", H::OtherEditor),
        // A ponte re-deriva as âncoras locais quando é `false`: máquina, não autoria.
        ("anchored", H::Editor),
        // O cartão «Custom» pinta-o em linhas próprias (eixo a eixo, e o eixo do motor).
        ("custom", H::OwnSection),
    ]),
    // ⭐⭐ **O comportamento — e ele também chega com o corpo** (ordem do dono, 2026-09-14). A §14
    // volta a pintar-se sobre **todo corpo Dynamic**, com ou sem o componente, e a face vazia dela é
    // o botão *Make Platform Player* — que foi a porta original (W5) e morreu na F3.
    //
    // ⚠️ **A porta por-secção faz o que a paleta não podia:** ela só existe onde a física honra o
    // gesto (num `Static` a mola do player é um impulso sobre massa infinita), e o `+` genérico não
    // sabe perguntar isso. ⛔ E anexá-lo pelo `+` a um corpo qualquer era o pior dos dois mundos —
    // um crate com lei de personagem.
    // ⚠️ **`Intrinsic` E com `requires`**, que não é contradição: o `attach` diz *quem escolhe* e o
    // `requires` diz *é inerte sem o quê* — e a porta `attach_by_name` honra a cascata sem
    // consultar o `attach`. Sem esta linha, um chamador que anexasse o player a um objecto pelado
    // deixava-o lá a não fazer nada, em silêncio. Ver [`D::intrinsic_requiring`].
    D::intrinsic_requiring(
        "ph2d::physics::PlatformPlayer",
        "component.platform_player.name",
        C::Physics,
        PLATFORM_PLAYER,
        &["ph2d::physics::RigidBody"],
    )
    .coded("platform_player"),
    D::intrinsic("ph2d::physics::PlayerMode", "component.player_mode.name", C::Physics, PLAYER_MODE)
        .coded("player_mode")
        .row_of("ph2d::physics::PlatformPlayer"),
    i("ph2d::physics::PlayerSignals", "component.player_signals.name").hidden(H::Presence),
    // ⭐⭐⭐ **O PROJÉCTIL de arcade** (TOP-20 #14). ⚠️ `RigidBody` é requerido pela mesma razão
    // dos dois movers irmãos: sem corpo não há o que mover nem em que bater, e a paleta anexa os
    // dois de uma vez em vez de entregar um componente inerte.
    pr(
        "ph2d::physics::ProjectileMotion",
        "component.projectile_motion.name",
        PROJECTILE_MOTION,
        &["ph2d::physics::RigidBody"],
    )
    .coded("projectile_motion")
    .hiding(&[("homing_target", H::Reference)]),
    // ✅ `rope`/`body` idem — e a roldana é a SEXTA consulta da ponte, a que a refutação não
    // nomeava (F4.1): ela é alcançada pelo nome da corda, então uma referência por remapear
    // faria a roldana da instância disputar a corda do mestre. ⛔ `Intrinsic` pela razão da junta:
    // ela chega pelo botão da §12, que já tem a corda em mãos.
    D::intrinsic(
        "ph2d::physics::PulleyWheel",
        "component.pulley_wheel.name",
        C::Physics,
        PULLEY_WHEEL,
    )
    .coded("pulley_wheel")
    .hiding(&[
        ("rope", H::Reference),
        ("body", H::Reference),
        // O eixo montado arrasta-se no canvas e a ponte semeia-o da pose de repouso.
        ("local", H::OtherEditor),
        ("mounted", H::Editor),
    ]),
    // ⭐⭐⭐ **PORTA 4 — o RAIO persistente** (suplente #21): *este objecto OLHA*, que é uma
    // intenção do artista e não uma row que a §11 já pinte.
    //
    // ⛔ **E ele NÃO requer `RigidBody`**, ao contrário dos dois movers: está MEDIDO
    // (`um_raio_sem_corpo_nao_exclui_ninguem`) — um olho sem corpo vê na mesma, e ali a resposta
    // certa é não excluir ninguém. *Exigir um corpo entregaria uma cerca que a lei não tem.*
    D::authored(
        "ph2d::physics::RaySensor",
        "component.ray_sensor.name",
        C::Physics,
        O::ANY,
        RAY_SENSOR,
    )
    .coded("ray_sensor"),
    // ⚠️ Os dois NOMES são `Intrinsic` pela razão dos irmãos `SignalOnHit`/`SignalOnLeave`: eles
    // nascem das rows da secção, nunca do `+`. Um `RaySignals::default()` anexado à mão é **calado
    // nos dois extremos**, logo seria uma porta que entrega nada.
    D::intrinsic(
        "ph2d::physics::RaySignals",
        "component.ray_signals.name",
        C::Physics,
        RAY_SIGNALS,
    )
    .coded("ray_signals")
    .row_of("ph2d::physics::RaySensor"),
    // ⭐ **PORTA 1 — a que o dono conhecia pelo nome.** Era o botão *Add Physics Body* da face
    // vazia da §11; desde o ADR-0166/F3 a rota é o `+`, e o rótulo passa a dizer o que a secção
    // que nasce diz.
    pr(
        "ph2d::physics::RigidBody",
        "component.rigid_body.name",
        RIGID_BODY,
        &["ph2d::physics::Collider"],
    )
    .coded("physics_body"),
    // As folgas das pontas da corda arrastam-se no canvas (o batente); nenhum cartão as mostra.
    i("ph2d::physics::RopeStops", "component.rope_stops.name").hidden(H::OtherEditor),
    il("ph2d::physics::SignalOnHit", "component.signal_on_hit.name", SIGNAL_ON_HIT).coded("signal_on_hit").row_of("ph2d::physics::Collider"),
    il("ph2d::physics::SignalOnLeave", "component.signal_on_leave.name", SIGNAL_ON_LEAVE).coded("signal_on_leave").row_of("ph2d::physics::Collider"),
    // O valor inteiro é uma tag (uma referência): ganha nome pelo nome da tag (d4).
    i("ph2d::physics::SignalTagFilter", "component.signal_tag_filter.name").hidden(H::Reference),
    // ⭐⭐⭐ **O mover de VISTA DE CIMA** (TOP-20 #13). ⚠️ `RigidBody` é requerido pela
    // mesma razão do irmão `PlatformPlayer`: sem corpo não há o que mover, e a paleta
    // anexa os dois de uma vez em vez de entregar um componente inerte.
    pr(
        "ph2d::physics::TopDownPlayer",
        "component.top_down_player.name",
        TOP_DOWN_PLAYER,
        &["ph2d::physics::RigidBody"],
    )
    .coded("top_down_player"),
    il("ph2d::physics::WalkSurface", "component.walk_surface.name", WALK_SURFACE).coded("walk_surface").row_of("ph2d::physics::Collider"),
    i("ph2d::physics::WestonAxle", "component.weston_axle.name").hidden(H::Presence),
];

#[cfg(test)]
mod tests {
    use super::DESCS;

    /// ⭐⭐ **A família da física oferece TRÊS portas, e nenhuma linha de secção.**
    ///
    /// O gate que faltava em 2026-08-24, quando a F3 escreveu a família inteira com o helper
    /// autorado: **30 itens na paleta, e as opções de UM objecto de física espalhadas por eles**. O
    /// report do dono (13/09) nomeou-o *«picotado, dividido em inúmeros supostos componentes»*, e o
    /// veredito dele (14/09) fixou o número: *«um objeto de física (Physics Body) e todas as opções
    /// aparecem com ele»*.
    ///
    /// ⚠️ **É uma catraca NOMEADA, e a lista é a afirmação** — não um número. Uma família de
    /// física que ganhe uma segunda porta reprova aqui e obriga a decidir *isto é uma intenção do
    /// artista, ou é uma row/gesto da secção?*; e se a porta desaparecer reprova pelo mesmo
    /// `assert_eq`, que é a **metade de obsolescência** (a lista deixaria de descrever a tabela).
    ///
    /// ⚠️ **O piso de população é a outra metade:** sem ele, apagar a família inteira deixaria o
    /// gate a comparar dois vazios — *um zero de «não medido» e um de «perfeito» são o mesmo byte*.
    ///
    /// (Mutação: trocar um `i(...)` por `D::authored(...)` ⇒ o `assert_eq` reprova, nomeando-o.)
    #[test]
    fn the_physics_family_offers_one_door_and_not_its_rows() {
        // ⭐ **TRÊS portas desde 16/09** — as duas novas são os CONTROLADORES canónicos
        //    (TOP-20 #13 e #14), e a resposta às duas metades da pergunta acima é a mesma: um
        //    mover é uma **intenção** do artista (ele escolhe *este objecto é um jogador de
        //    vista de cima*), nunca uma row que a §11 já pinta. ⚠️ Os dois pedem `RigidBody`
        //    junto (`pr`), senão a paleta entregava um componente inerte.
        // ⭐ **QUATRO portas desde 19/09** — a nova é o RAIO (suplente #21), da mesma espécie
        //    dos dois movers: *este objecto OLHA* é uma intenção que o artista escolhe. ⛔ Mas,
        //    ao contrário deles, ela **não pede companhia**: um olho sem corpo vê na mesma, e
        //    está medido.
        // ⭐ **SEIS portas desde 2026-10-01** — as novas são a NAVEGAÇÃO (plano 30, W4): o agente
        //    (*este objecto persegue*, e pede o mover junto) e a região (*os agentes andam aqui*).
        // ⭐ **OITO desde 2026-10-03** — a área de custo e o atalho (plano 30, W7), sem companhia.
        const PORTAS: [&str; 8] = [
            "ph2d::physics::NavAgent",
            "ph2d::physics::NavCostArea",
            "ph2d::physics::NavLink",
            "ph2d::physics::NavRegion",
            "ph2d::physics::ProjectileMotion",
            "ph2d::physics::RaySensor",
            "ph2d::physics::RigidBody",
            "ph2d::physics::TopDownPlayer",
        ];
        let offered: Vec<&str> = DESCS
            .iter()
            .filter(|d| d.is_offered())
            .map(|d| d.canonical_name)
            .collect();
        assert_eq!(
            offered,
            PORTAS.to_vec(),
            "a paleta de FISICA tem de oferecer exatamente estas TRES portas -- o `Physics Body` e os \
             dois controladores canonicos.\n\
             Um componente novo aqui e' uma pergunta, nao um esquecimento: e' uma INTENCAO do \
             artista (entao e' `pr`, e esta lista cresce com ele), ou e' uma ROW / um GESTO que a \
             §11/§12/§13/§14 ja' oferece (entao e' `i(..)`)?"
        );
        assert!(
            DESCS.len() >= 30,
            "a familia encolheu para {} — o gate acima passaria a comparar duas listas quase \
             vazias e a nao afirmar nada",
            DESCS.len()
        );
    }
}
