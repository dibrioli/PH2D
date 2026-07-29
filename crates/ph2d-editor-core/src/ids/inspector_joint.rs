//! **Os ids da §12 — Physics Joint.**
//!
//! Irmão de `inspector.rs`, separado dele quando os dois juntos passaram do cap
//! de 700 LOC, e o corte é por SEÇÃO: tudo aqui descreve a seção que fala do
//! objeto-joint selecionado (tipo, as duas pontas que ele nomeia, os parâmetros
//! do tipo escolhido, e o teto sob o qual ele parte).
//!
//! ⚠️ **O gesto de CRIAR não está aqui** — ele mora nos ids da §11
//! (`INSP_PHYS_JOIN*`), porque um joint não existe ainda quando você quer fazer
//! um: o botão tem de estar onde você já está, olhando dois corpos selecionados.

use super::hash_node_id;
use ph2d_a11y::NodeId;

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

/// Pin · Spring · Rope · Weld · Slider · Rod · Wheel · Pulley. Indexed by the
/// `JointKind` tag the snapshot carries.
///
/// ⚠️ **Este array e o `KIND_LABELS` do painel têm de ter o MESMO tamanho.** O
/// `seg_row` faz `option_ids.zip(labels)`, e um `zip` **trunca**: com um rótulo a
/// mais que ids, o último chip simplesmente não é pintado — sem erro, sem
/// warning. Foi o que aconteceu quando o Slider chegou (W-J5), e o gate de seam
/// não pegou porque ele iterava justamente a lista CURTA. Há um teste no painel
/// comparando os dois comprimentos.
pub const INSP_JOINT_KIND: [NodeId; 8] = [
    hash_node_id("insp_joint_kind_pin"),
    hash_node_id("insp_joint_kind_spring"),
    hash_node_id("insp_joint_kind_rope"),
    hash_node_id("insp_joint_kind_weld"),
    hash_node_id("insp_joint_kind_slider"),
    hash_node_id("insp_joint_kind_rod"),
    hash_node_id("insp_joint_kind_wheel"),
    hash_node_id("insp_joint_kind_pulley"),
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
/// **Break force** (W-J7): the switch, and the two thresholds it gates.
///
/// A switch and not an `f32::INFINITY` typed into a number row, for the reason
/// the Limits and Motor switches above exist: `∞ = off` (P7) has to be a state
/// the artist can SEE, and "inf" in a numeric field is a value they would have
/// to overwrite to find the control. The bridge resolves the switch into the
/// infinity the solver wants.
pub const INSP_JOINT_BREAK_GROUP: NodeId = hash_node_id("insp_joint_break_group");
pub const INSP_JOINT_BREAK: [NodeId; 2] = [
    hash_node_id("insp_joint_break_off"),
    hash_node_id("insp_joint_break_on"),
];
/// Newtons. No conversion at this boundary (unlike the angle rows above).
pub const INSP_JOINT_BREAK_FORCE: NodeId = hash_node_id("insp_joint_break_force");
/// Newton-metres. ⚠️ Painted for a **Pin only** — rapier reports no reaction for
/// a LOCKED angular axis, so a Weld's threshold could never fire
/// (`JointKind::breaks_on_torque`).
pub const INSP_JOINT_BREAK_TORQUE: NodeId = hash_node_id("insp_joint_break_torque");

/// **Active** (W-J8) — is the constraint in force at all? Off stops the joint
/// holding without the object going anywhere, which is the thing Delete cannot
/// give you (*try the rig without this one*). Newton ships the same checkbox.
///
/// First in the section because it qualifies everything below it: the parameters
/// stay authored and stay editable, they are simply not being imposed.
pub const INSP_JOINT_ACTIVE_GROUP: NodeId = hash_node_id("insp_joint_active_group");
pub const INSP_JOINT_ACTIVE: [NodeId; 2] = [
    hash_node_id("insp_joint_active_off"),
    hash_node_id("insp_joint_active_on"),
];

/// **Collide Connected** (W-J8) — do the two bodies this joins still bump into
/// each other? Default off, and the default is the MEASURED one (a chain link
/// overlaps its neighbour at the pin by construction).
///
/// Lives with the two body rows and the swap, because it is a fact about the
/// PAIR rather than about the constraint.
pub const INSP_JOINT_COLLIDE_GROUP: NodeId = hash_node_id("insp_joint_collide_group");
pub const INSP_JOINT_COLLIDE: [NodeId; 2] = [
    hash_node_id("insp_joint_collide_off"),
    hash_node_id("insp_joint_collide_on"),
];

/// **Swap A↔B** (W-J8) — exchange the two ends. Behaviour-preserving by
/// construction (`PhysicsJoint::swapped` negates every signed quantity measured
/// between the bodies), so what it changes is which end is called A: the rows,
/// the anchor dot's owner, the overlay's solid/dashed lines, and which end each
/// eyedropper re-picks.
pub const INSP_JOINT_SWAP: NodeId = hash_node_id("insp_joint_swap");

/// Delete the joint object.
pub const INSP_JOINT_REMOVE: NodeId = hash_node_id("insp_joint_remove");

/// The eyedropper next to each of the joint's two body rows (§12). Clicking it
/// ARMS a canvas pick for that slot; the next click on a body re-binds that end,
/// with no other object pre-selected (the app's pick idiom — arm, then click the
/// target, like the colour eyedropper). Fixes a mis-joined pair without deleting
/// and re-creating the joint.
pub const INSP_JOINT_PICK_A: NodeId = hash_node_id("insp_joint_pick_a");
pub const INSP_JOINT_PICK_B: NodeId = hash_node_id("insp_joint_pick_b");

/// **Acrescentar uma roldana à corda** (W-Pulley W1) — o pedido (4) do artista,
/// *"escolher o número de roldanas, em tempo real, acrescentando após a criação"*.
///
/// Mora na seção da CORDA e não na da roda, porque é a corda que possui a lista —
/// e porque é ali que o artista está quando pensa *"esta corda precisa de mais
/// uma"*. A roda nova entra ao FIM da rota (o `order` seguinte) e o gesto de
/// posicioná-la é o mesmo de qualquer roldana: arrastar o dot dela.
pub const INSP_JOINT_ADD_WHEEL: NodeId = hash_node_id("insp_joint_add_wheel");

/// **O cabeçalho da §13 — Pulley Wheel** (dono do estado de colapso) e o
/// círculo de cor dele.
///
/// ⚠️ Seção PRÓPRIA, e não mais rows na §12: uma roldana é uma ENTIDADE (a
/// espinha do W1), então ela é o objeto SELECIONADO quando estas rows importam,
/// e a §12 só existe com a CORDA selecionada. Uma seção que trocasse de assunto
/// conforme o que está selecionado teria um estado de colapso descrevendo dois
/// objetos diferentes.
pub const INSP_LIVE_WHEEL_SECTION: NodeId = hash_node_id("insp_live_wheel_section");
pub const INSP_LIVE_WHEEL_COLOR: NodeId = hash_node_id("insp_live_wheel_color");

/// **O RAIO da roldana selecionada**, metros (W-Pulley W1).
///
/// O número que substituiu o `Ratio`. A alça do aro no canvas edita o MESMO
/// campo — dois gestos, um valor.
pub const INSP_WHEEL_RADIUS: NodeId = hash_node_id("insp_wheel_radius");
/// **O SEGUNDO diâmetro** do tambor diferencial (W4) — `0` é uma roldana comum.
///
/// A vantagem mecânica da corda é `Radius / Out Radius`, e ela não é digitada em
/// lugar nenhum: cai das duas circunferências que o artista dimensiona.
pub const INSP_WHEEL_RADIUS_OUT: NodeId = hash_node_id("insp_wheel_radius_out");

/// **A POSIÇÃO da roldana ao longo da corda** — 1º nó, 2º nó, … (W-Pulley W1).
///
/// Editável porque a rota é uma SEQUÊNCIA: duas roldanas trocadas de ordem
/// descrevem outra corda, e sem esta row a única forma de reordenar seria apagar
/// e recriar.
pub const INSP_WHEEL_ORDER: NodeId = hash_node_id("insp_wheel_order");

/// **De que lado a corda passa nesta roldana** — `Auto | Over | Under`, o escape
/// manual ao lado do algoritmo (W-Pulley W1, pedido 7).
/// **O MOTOR desta roldana** — graus por segundo, positivo RECOLHE (W2).
///
/// A grandeza é angular, e é isso que faz o diâmetro ser o câmbio: a corda anda
/// `ω·r`, então o mesmo motor num tambor maior recolhe mais depressa. Graus na
/// fronteira e radianos no componente, a convenção do motor do Pin.
pub const INSP_WHEEL_MOTOR: NodeId = hash_node_id("insp_wheel_motor");

/// **Este eixo pode ceder?** — o par switch/limiar da §13 (W2).
pub const INSP_WHEEL_BREAK_GROUP: NodeId = hash_node_id("insp_wheel_break_group");
/// Os dois chips do switch acima.
pub const INSP_WHEEL_BREAK: [NodeId; 2] = [
    hash_node_id("insp_wheel_break_off"),
    hash_node_id("insp_wheel_break_on"),
];
/// **O que este eixo aguenta**, newtons.
pub const INSP_WHEEL_BREAK_FORCE: NodeId = hash_node_id("insp_wheel_break_force");

/// **Em que CORPO esta roldana se monta** (W3) — o eyedropper que ARMA o pick de
/// canvas, irmão exato dos `INSP_JOINT_PICK_A/B`.
pub const INSP_WHEEL_MOUNT_PICK: NodeId = hash_node_id("insp_wheel_mount_pick");
/// **Desmontar**: a roldana volta a ser um ponto do cenário. Pintado só quando há
/// o que desmontar — um botão que não faz nada é pior que um botão que falta.
pub const INSP_WHEEL_UNMOUNT: NodeId = hash_node_id("insp_wheel_unmount");

pub const INSP_WHEEL_WRAP_GROUP: NodeId = hash_node_id("insp_wheel_wrap_group");
pub const INSP_WHEEL_WRAP: [NodeId; 3] = [
    hash_node_id("insp_wheel_wrap_auto"),
    hash_node_id("insp_wheel_wrap_over"),
    hash_node_id("insp_wheel_wrap_under"),
];
