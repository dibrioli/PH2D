//! §12 Physics Joint — the shell half of the Inspector section (W3): the
//! snapshot the panel reads, the ECS write an edit turns into, and the gesture
//! that creates a joint in the first place.
//!
//! Its own module rather than more of `inspector_physics.rs`, for the same
//! reason that one is not more of `inspector_ordering.rs`: a joint is not a
//! body, and this is the whole answer to what the §12 controls do.

use bevy_ecs::world::World;
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{Entity, Name, SimWorld};
use ph2d_editor::{InspectorJointInfo, JointFieldEdit};
use ph2d_physics_ecs::{JointKind, MotorMode, PhysicsJoint};

use super::inspector_ordering::queue_set;

/// O gesto que CRIA um joint mora no irmão `inspector_joint_create` (extraído
/// no cap de 600 LOC do shell) e é re-exportado aqui, para que todo chamador
/// mantenha o caminho `inspector_joint::create_joint*` que já usava. O corte é o
/// que a UI faz: criar é da §11, editar é da §12.
pub(crate) use super::inspector_joint_create::{create_joint, create_joint_at, ensure_named};

const JOINT: &str = "ph2d::physics::PhysicsJoint";

/// Tag ↔ kind, in one place. The panel speaks tags (it never sees
/// `ph2d-physics-ecs`), so this is the only conversion and both directions
/// live next to each other where a mismatch is visible. `pub(crate)` so the
/// Join gesture can create a joint of the artist's chosen kind (the join-kind
/// selector stores a tag).
pub(crate) fn kind_of(tag: u8) -> JointKind {
    match tag {
        1 => JointKind::Spring,
        2 => JointKind::Rope,
        3 => JointKind::Weld,
        4 => JointKind::Slider,
        5 => JointKind::Rod,
        6 => JointKind::Wheel,
        7 => JointKind::Pulley,
        8 => JointKind::Custom,
        _ => JointKind::Pin,
    }
}

/// A limit range **out** of the component, in the unit the Inspector shows.
///
/// One door, and its twin [`limit_in`] is the other direction — the pair is why a
/// value typed into the Min row and the value that row shows next frame are the
/// same number. Degrees for an angular range, metres verbatim for a stroke.
pub(crate) fn limit_out(kind: JointKind, v: f32) -> f32 {
    if kind.limits_in_metres() {
        v
    } else {
        v.to_degrees()
    }
}

/// A limit range **into** the component, from the unit the Inspector typed.
fn limit_in(kind: JointKind, v: f32) -> f32 {
    if kind.limits_in_metres() {
        v
    } else {
        v.to_radians()
    }
}

/// A motor quantity **out** of the component, in the unit the Inspector shows.
///
/// The twin door of [`limit_out`], and deliberately a SEPARATE one: the question
/// is `JointKind::motor_in_metres`, not `limits_in_metres`. A Rope has no limit
/// range at all and still has a linear motor (the winch), so one door for both
/// would show a winch's target in degrees.
///
/// ⚠️ **Toma o JOINT e não o `kind`, desde que o `Custom` chegou:** ali o eixo
/// do motor é AUTORADO, então metro-ou-radiano é escolhido junto — a porta é
/// `PhysicsJoint::motor_in_metres` (a da INSTÂNCIA), e usar a do tipo rotularia
/// em graus um número que o solver lê em metros. O artista digitaria 90 e a peça
/// andaria 1,57 m.
fn motor_out(j: &PhysicsJoint, v: f32) -> f32 {
    if j.motor_in_metres() {
        v
    } else {
        v.to_degrees()
    }
}

/// A motor quantity **into** the component, from the unit the Inspector typed.
fn motor_in(j: &PhysicsJoint, v: f32) -> f32 {
    if j.motor_in_metres() {
        v
    } else {
        v.to_radians()
    }
}

fn motor_mode_tag(mode: MotorMode) -> u8 {
    match mode {
        MotorMode::Velocity => 0,
        MotorMode::Position => 1,
    }
}

fn motor_mode_of(tag: u8) -> MotorMode {
    if tag == 1 {
        MotorMode::Position
    } else {
        MotorMode::Velocity
    }
}

pub(crate) fn tag_of(kind: JointKind) -> u8 {
    match kind {
        JointKind::Pin => 0,
        JointKind::Spring => 1,
        JointKind::Rope => 2,
        JointKind::Weld => 3,
        JointKind::Slider => 4,
        JointKind::Rod => 5,
        JointKind::Wheel => 6,
        JointKind::Pulley => 7,
        JointKind::Custom => 8,
    }
}

/// The name of whichever entity hashes to `id`, or empty when none does.
///
/// A linear scan, on purpose: it runs for the ONE selected joint, twice, and
/// only while §12 is on screen. A cached index would be a second copy of a
/// fact the `Name`s already hold — and one that goes stale on every rename,
/// which is precisely the event this has to report correctly.
/// O NOME do objeto de identidade `id` — o documento guarda id, o painel mostra nome
/// (ADR-0164 F1). Era o contrário: guardava o hash do nome e comparava-o, e por isso renomear
/// um corpo esvaziava a linha do joint que o citava.
fn name_for(
    world: &World,
    id: u64,
    q: &mut bevy_ecs::query::QueryState<(&Name, &ph2d_ecs::StableId)>,
) -> String {
    if id == 0 {
        return String::new();
    }
    q.iter(world)
        .find(|(_, s)| s.0 == id)
        .map(|(n, _)| n.as_str().to_string())
        .unwrap_or_default()
}

/// Build the §12 snapshot. `None` for anything that is not a joint — unlike
/// §11, this section has no empty face: there is nothing useful to offer on an
/// object that is not a joint, and the button that CREATES one lives in §11.
///
/// `pick_armed` (`0` none / `1` A / `2` B) is the shell's armed-pick state for
/// THIS joint, mirrored into the snapshot so the painter draws the waiting
/// eyedropper pressed.
pub(crate) fn build_joint_info(
    sim: &mut SimWorld,
    entity_bits: u64,
    pick_armed: u8,
    paste_targets: usize,
) -> Option<InspectorJointInfo> {
    let entity = Entity::from_bits(entity_bits);
    let joint = *sim.world().get::<PhysicsJoint>(entity)?;
    // Quantas roldanas esta corda tem — contadas no estado AUTORADO, não na arena
    // do solver: é o número que o artista gerencia (ele acabou de criar uma), e
    // uma corda cujo joint ainda não foi construído mostraria zero se a pergunta
    // fosse ao solver.
    let wheel_count = super::inspector_joint_wheel::rope_wheel_count(sim, entity);
    let mut q = sim.world_mut().query::<(&Name, &ph2d_ecs::StableId)>();
    let world = sim.world();
    let a = name_for(world, joint.body_a, &mut q);
    let b = name_for(world, joint.body_b, &mut q);
    let world_anchored = world
        .get::<ph2d_physics_ecs::JointWorldAnchor>(entity)
        .is_some();
    // ⚠️ **Os batentes por eixo saem na unidade da ROW** — metros nos lineares,
    // GRAUS na rotação —, pela mesma porta que o `limit_min_ui` ao lado usa.
    // Guardar radianos e rotular graus é como o artista digita 90 e a peça anda
    // 1,57.
    let axis = |i: usize| joint.custom.axes[i];
    let axis_ui = |i: usize, v: f32| if i == 2 { v.to_degrees() } else { v };
    Some(InspectorJointInfo {
        entity_bits,
        axis_mode_tag: [axis(0).mode.tag(), axis(1).mode.tag(), axis(2).mode.tag()],
        axis_min_ui: [
            axis_ui(0, axis(0).min),
            axis_ui(1, axis(1).min),
            axis_ui(2, axis(2).min),
        ],
        axis_max_ui: [
            axis_ui(0, axis(0).max),
            axis_ui(1, axis(1).max),
            axis_ui(2, axis(2).max),
        ],
        motor_axis_tag: joint.custom.motor_axis.tag(),
        kind_tag: tag_of(joint.kind),
        soft: joint.soft,
        wheel_count,
        paste_targets,
        // `bound` is about the NAMES resolving, which is the thing the artist
        // can act on. Whether the solver also built it depends on those bodies
        // having colliders, and saying "not connected" for a body that is
        // merely not physical yet would point at the wrong problem.
        // ⚠️ **Um pino de MUNDO é `bound` com UM corpo só** — o cenário não é um
        // objeto que possa estar ausente, então exigir os dois nomes chamaria de
        // quebrado um joint que está segurando. A pergunta vai à porta única
        // (`is_anchored`), que é a mesma que o reconcile faz.
        bound: joint.is_anchored(world_anchored)
            && !a.is_empty()
            && (world_anchored || !b.is_empty()),
        world_anchored,
        body_a_name: a,
        body_b_name: b,
        limits_enabled: joint.limits_enabled,
        limit_min_ui: limit_out(joint.kind, joint.limit_min),
        limit_max_ui: limit_out(joint.kind, joint.limit_max),
        motor_enabled: joint.motor_enabled,
        motor_mode_tag: motor_mode_tag(joint.motor_mode),
        motor_speed_ui: motor_out(&joint, joint.motor_speed),
        motor_target_ui: motor_out(&joint, joint.motor_target),
        motor_max_force: joint.motor_max_force,
        rest_length: joint.rest_length,
        stiffness: joint.stiffness,
        damping: joint.damping,
        max_length: joint.max_length,
        pick_armed,
        break_enabled: joint.break_enabled,
        break_force: joint.break_force,
        break_torque: joint.break_torque,
        // The panel does not know `ph2d-physics-ecs` (loose-coupled, like every
        // sibling section), so the ENGINE's answer travels in the snapshot rather
        // than being re-derived from `kind_tag` on the far side — a second copy of
        // "which kinds can report a torque" would be a second thing to keep true.
        // ⚠️ À INSTÂNCIA, não ao tipo: uma solda MOLE motoriza o eixo angular
        // que a rígida trava, e rapier só publica reação de eixo motorizado ou
        // limitado (medido: 0,9619 N·m contra 0,0000). A ponte pergunta à MESMA
        // porta antes de entregar o limiar ao solver.
        breaks_on_torque: joint.breaks_on_torque(),
        active: joint.active,
        collide_connected: joint.collide_connected,
    })
}

/// Re-bind slot A (or B, if `slot_b`) of the joint at `joint_bits` to `target`
/// — the resolve half of the §12 eyedropper pick, called from `input_dispatch`
/// when the armed pick's next canvas click lands on a body. Returns whether the
/// bind took (so the caller clears the armed pick only on success).
///
/// The target is NAMED if it lacks one (a joint refers to bodies by name hash,
/// the requirement `create_joint` also has), then the hash is written **in
/// place** — not through the editor queue like the field edits, because the pick
/// resolves mid-frame in the pointer handler, and the global diff-based undo
/// captures a direct write the same as a queued one. The value still goes
/// through `clamped()`, the one door before the solver.
///
/// ⚠️ **Refuses a self-joint.** Picking the body already in the OTHER slot would
/// name both ends the same body — a joint that can never bind — so it returns
/// `false` and the pick stays armed for another click, rather than writing a
/// silently-dormant joint.
#[must_use]
pub(crate) fn set_joint_body(
    sim: &mut SimWorld,
    joint_bits: u64,
    slot_b: bool,
    target: Entity,
) -> bool {
    let joint_e = Entity::from_bits(joint_bits);
    let Some(&current) = sim.world().get::<PhysicsJoint>(joint_e) else {
        return false;
    };
    let Some(name) = ensure_named(sim, target, "Body") else {
        return false;
    };
    // ⚠️ A IDENTIDADE do alvo, pela entidade que este sítio tem em mãos. O `name` continua
    // garantido (um corpo sem nome é ilegível na Hierarquia), mas não é ele que se guarda.
    let _ = &name;
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let hash = ph2d_ecs::stable_id_of(sim.world(), target).map_or(0, |s| s.0);
    let mut next = current;
    if slot_b {
        next.body_b = hash;
    } else {
        next.body_a = hash;
    }
    if next.body_a == next.body_b {
        return false; // a self-joint is dormant — keep the pick armed instead
    }
    // Re-picking a body changes which frame the stored local anchor belongs to,
    // so it is meaningless for the new body: mark the joint un-anchored. The next
    // reconcile re-derives both body-local anchors from the joint's current
    // display pivot against the NEW bodies, re-gluing the pin where it visibly is.
    next.anchored = false;
    let next = next.clamped();
    if next != current
        && let Some(mut j) = sim.world_mut().get_mut::<PhysicsJoint>(joint_e)
    {
        *j = next;
    }
    true
}

/// Apply one [`JointFieldEdit`].
///
/// Every arm reads the live joint and writes it back changed — a partial write
/// would drop the fields not being edited, and this component has eleven of
/// them. `Remove` is not here: deleting a joint is deleting an OBJECT, and the
/// shell already knows how to do that.
pub(crate) fn apply_joint_edit(
    sim: &SimWorld,
    entity_bits: u64,
    edit: JointFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let entity = Entity::from_bits(entity_bits);
    let Some(&current) = sim.world().get::<PhysicsJoint>(entity) else {
        return;
    };
    let Some(next) = joint_with_edit(current, edit) else {
        return;
    };
    if next != current {
        queue_set(queue, registry, entity_bits, JOINT, &next);
    }
}

/// **Colar as propriedades de `source` neste joint** (W-JointCopy).
///
/// Irmã de [`apply_joint_edit`] com outra entrada: um paste não é a edição de um
/// campo (não há tag nem número a converter), é a substituição da metade
/// *"o que a restrição faz"* inteira, decidida por
/// [`PhysicsJoint::with_properties_of`] — a porta única que sabe o que é uma
/// propriedade.
///
/// Sai pelo MESMO `clamped()` e pela MESMA fila que toda edição da §12, então um
/// paste não pode autorar um estado que uma edição à mão não poderia.
pub(crate) fn paste_joint_properties(
    sim: &SimWorld,
    entity_bits: u64,
    source: &PhysicsJoint,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let entity = Entity::from_bits(entity_bits);
    let Some(&current) = sim.world().get::<PhysicsJoint>(entity) else {
        return; // não é um joint (a seleção pode ter sprites dentro) — o fan-out passa reto
    };
    let next = current.with_properties_of(source).clamped();
    if next != current {
        queue_set(queue, registry, entity_bits, JOINT, &next);
    }
}

/// **One edit applied to one joint** — the pure half of [`apply_joint_edit`],
/// and the single funnel every author of these fields goes through.
///
/// Extracted in W-J3 so that DRAGGING a limit wall or a length ring writes the
/// same way TYPING the number does: same degrees-to-radians boundary, same
/// per-field floors, same `clamped()`. Two conversions of "what does this edit
/// mean" is how a posed limit and a typed one would come to disagree — and the
/// disagreement would be invisible, because each looks right on its own.
///
/// `None` for the edits that are not a component write (the eyedroppers arm
/// shell state; `Remove` deletes the entity).
#[must_use]
pub(crate) fn joint_with_edit(current: PhysicsJoint, edit: JointFieldEdit) -> Option<PhysicsJoint> {
    let mut next = current;
    let prev_kind = current.kind;
    match edit {
        // ⚠️ **A unidade da row decide a conversão, e ela é do EIXO** — graus na
        // rotação, metros nos dois lineares. A mesma lei do `limit_in`, aqui
        // perguntada ao eixo em vez de ao tipo.
        JointFieldEdit::AxisMode(ax, tag) => {
            let a = ph2d_physics_ecs::CustomAxis::from_tag(ax);
            next.custom.axis_mut(a).mode = ph2d_physics_ecs::AxisMode::from_tag(tag);
        }
        JointFieldEdit::AxisMin(ax, v) => {
            let a = ph2d_physics_ecs::CustomAxis::from_tag(ax);
            next.custom.axis_mut(a).min = if a.in_metres() { v } else { v.to_radians() };
        }
        JointFieldEdit::AxisMax(ax, v) => {
            let a = ph2d_physics_ecs::CustomAxis::from_tag(ax);
            next.custom.axis_mut(a).max = if a.in_metres() { v } else { v.to_radians() };
        }
        // ⚠️ **Trocar o eixo do motor TROCA A UNIDADE do alvo** (metro ↔
        // radiano), e o alvo guardado passaria a significar outra coisa em
        // silêncio — o número que valia 90° viraria 1,571 m. Ele é RE-SEMEADO
        // em zero, que é a mesma cura que a troca de TIPO aplica ao par de
        // limites logo abaixo, pelo mesmo motivo.
        JointFieldEdit::MotorAxis(tag) => {
            let a = ph2d_physics_ecs::CustomAxis::from_tag(tag);
            if a != next.custom.motor_axis {
                next.custom.motor_axis = a;
                next.motor_target = 0.0;
            }
        }
        JointFieldEdit::Kind(tag) => {
            next.kind = kind_of(tag);
            // The anchor POLICY depends on the kind: a shared-point joint
            // (Pin/Weld) anchors both bodies at the pivot, a two-ended one
            // (Spring/Rope) anchors body B at its own centre. So a kind change is
            // a reposition of the B-end — mark it un-anchored so the next
            // reconcile re-derives the body-local anchors under the NEW policy
            // (the 4th authoring site, beside the dot drag, Position and re-pick).
            // Without this a Pin turned into a Rope keeps the Pin's shared-point
            // anchor and the rope hangs from the wrong spot on body B.
            next.anchored = false;
            // ⚠️ **And the limit RANGE is re-seeded when the unit changes.**
            // `limit_min/max` carry the kind's own unit, so a Pin's ±45°
            // (±0.785 rad) reinterpreted as a stroke is ±0.785 **metres** — a
            // number nobody typed — and a 0.5 m rail read as radians is a 28.6°
            // hinge. Only on a unit CHANGE: Pin→Weld→Pin still returns the angles
            // the artist had, which is the promise the component makes about
            // switching kinds.
            let fresh = PhysicsJoint::of_kind(next.kind);
            if next.kind.limits_in_metres() != prev_kind.limits_in_metres() {
                next.limit_min = fresh.limit_min;
                next.limit_max = fresh.limit_max;
            }
            // ⚠️ **And the MOTOR's unit is a separate question** — `motor_speed`
            // and `motor_target` carry radians on a hinge and metres on a rail or
            // a winch, so a Pin's 2 rad/s reinterpreted as a Slider's is 2 m/s.
            // Gated on `motor_in_metres` and not on the limits' door because a
            // Rope flips this one without flipping that one (it has a linear
            // motor and no limit range at all): sharing the condition would leave
            // a winch running at 114 m/s under a label reading metres.
            if next.motor_in_metres() != current.motor_in_metres() {
                next.motor_speed = fresh.motor_speed;
                // Zero is the joint's own zero in either unit, so the target
                // needs no per-kind default — only the reset.
                next.motor_target = 0.0;
            }
            // ⚠️ **E a MOLA é a terceira, com a diferença de que o que muda nela
            // não é a unidade e sim a ESCALA.** `stiffness`/`damping` são um par
            // de campos com dois donos: pendurar um corpo (uma Spring, 30) e
            // suspender um veículo (a suspensão de um Wheel, 400 — medido). Uma
            // Spring virada Wheel guardando 30 põe o carro sentado no batente no
            // primeiro tick, e nada na tela diz por quê. Mesma regra das outras
            // duas: só quando o PAPEL muda, então Wheel→Pin→Wheel devolve os
            // números que o artista digitou.
            if PhysicsJoint::suspends(next.kind) != PhysicsJoint::suspends(prev_kind) {
                next.stiffness = fresh.stiffness;
                next.damping = fresh.damping;
            }
        }
        JointFieldEdit::LimitsEnabled(on) => next.limits_enabled = on,
        // The UNIT is the kind's (`limits_in_metres`): degrees→radians for a
        // hinge, metres verbatim for a slider's stroke. One door each way, so the
        // label the panel paints and the number the component holds cannot
        // disagree about what was typed.
        JointFieldEdit::LimitMin(v) => next.limit_min = limit_in(next.kind, v),
        JointFieldEdit::LimitMax(v) => next.limit_max = limit_in(next.kind, v),
        JointFieldEdit::MotorEnabled(on) => next.motor_enabled = on,
        JointFieldEdit::MotorMode(tag) => next.motor_mode = motor_mode_of(tag),
        // The UNIT is the kind's (`motor_in_metres`), through the same pair of
        // doors the limits use — and a DIFFERENT pair, because a Rope's answers
        // differ (no limits, linear motor).
        JointFieldEdit::MotorSpeed(v) => next.motor_speed = motor_in(&next, v),
        JointFieldEdit::MotorTarget(v) => next.motor_target = motor_in(&next, v),
        JointFieldEdit::MotorMaxForce(v) => next.motor_max_force = v.max(0.0),
        JointFieldEdit::RestLength(v) => next.rest_length = v.max(0.0),
        JointFieldEdit::Stiffness(v) => next.stiffness = v.max(0.0),
        JointFieldEdit::Damping(v) => next.damping = v.max(0.0),
        // A rope of zero length is a weld nobody asked for, and rapier's own
        // docs require the distance to be strictly positive.
        JointFieldEdit::MaxLength(v) => next.max_length = v.max(1e-3),
        // Sem piso aqui: o `clamped()` na saída desta função é a porta única
        // que conhece o `MIN_RATIO`, e repeti-lo seria o segundo lugar onde a
        // razão mínima é decidida.
        // W-J7. No unit conversion in either direction: a newton and a
        // newton-metre mean the same thing on both sides of this boundary, which
        // is exactly why these two rows need no `limit_in`/`motor_in` twin.
        // W-J8. Two plain switches — no conversion, no unit, no gating on kind:
        // every joint can be turned off, and every joint has a pair.
        // A ponte gateia por `can_be_soft`, então guardar o flag num tipo que não
        // o usa é inerte — e é o mesmo comportamento que todo outro param tem
        // aqui: trocar Weld→Pin→Weld devolve a solda que o artista tinha.
        JointFieldEdit::Soft(on) => next.soft = on,
        JointFieldEdit::Active(on) => next.active = on,
        JointFieldEdit::CollideConnected(on) => next.collide_connected = on,
        // **The pair, exchanged.** The whole operation lives on the component
        // (`PhysicsJoint::swapped`) because it is arithmetic about the joint and
        // nothing else: the anchors travel with their bodies and every signed
        // quantity measured between them is negated, so the joint keeps doing
        // exactly what it did. ⚠️ Deliberately does NOT clear `anchored` — the
        // three sites that do are *repositions*, and this is a re-labelling; a
        // re-seed here would send a Spring's B end back to the body's centre and
        // throw away where the artist put it.
        JointFieldEdit::Swap => next = next.swapped(),
        JointFieldEdit::BreakEnabled(on) => next.break_enabled = on,
        JointFieldEdit::BreakForce(v) => next.break_force = v.max(0.0),
        JointFieldEdit::BreakTorque(v) => next.break_torque = v.max(0.0),
        // The eyedropper ARMS a pick in the action loop (it sets shell state,
        // not a component), and the pick RESOLVES in `input_dispatch` via
        // `set_joint_body`. Neither reaches this per-joint apply; listed so the
        // match stays exhaustive, exactly like `Remove`.
        JointFieldEdit::PickBodyA | JointFieldEdit::PickBodyB => return None,
        // Estruturais: não escrevem campo de `PhysicsJoint` nenhum. O `Remove`
        // apaga a entidade, o `AddWheel` spawna uma, e o `AnchorToWorld`
        // acrescenta/remove um MARCADOR — os três moram no laço de ações, onde
        // a shell tem `&mut self`.
        JointFieldEdit::Remove | JointFieldEdit::AddWheel | JointFieldEdit::AnchorToWorld(_) => {
            return None;
        }
        // W-JointCopy: o Copy ARMA a área de transferência da shell (nenhum
        // componente muda) e o Paste precisa da FONTE, que mora nela — os dois
        // moram no laço onde a shell tem `&mut self`, como os conta-gotas. E o
        // Paste escreve por `paste_joint_properties`, que é o funil deste
        // arquivo com outra entrada, não uma segunda resposta.
        JointFieldEdit::CopyProperties | JointFieldEdit::PasteProperties => return None,
    }
    // Through the SAME clamp the bridge uses on the way to the solver, so the
    // Inspector cannot author a state the loader would have to repair.
    Some(next.clamped())
}
