//! **Como um joint é DESCRITO ao solver** — a tradução `PhysicsJoint` (ECS,
//! autorado, `serde`, formato de arquivo) → `JointDesc` (plain data do wrapper).
//!
//! Módulo irmão do `bridge/joints.rs` pelo cap de 700 LOC, cortado por
//! responsabilidade: *o que este joint É para o solver* × *manter os joints do
//! solver em dia com a cena*.
//!
//! ⚠️ **Todo parâmetro que o TIPO ignora é resolvido AQUI**, perguntando à porta
//! do tipo (`has_limits`, `has_motor`, `can_be_soft`, `breaks_on_torque`) — as
//! MESMAS que o painel pergunta para decidir que row pintar. É isso que impede um
//! valor deixado para trás por uma troca de tipo de seguir em vigor em silêncio.

use ph2d_physics::{JointDesc, MotorDesc};

use crate::joint::{JointKind, PhysicsJoint};

/// Translate the component + the already-LOCAL anchors into the plain
/// [`JointDesc`] the wrapper takes.
///
/// `pub` (re-exported from the crate root) because it is the documented place
/// where "which parameters does this kind actually use?" is answered, and that is
/// a question a gate needs to ask directly: the alternative is inferring the
/// answer from a body's motion, which cannot distinguish *the threshold was not
/// passed* from *the threshold was passed and the reading is structurally zero* —
/// exactly the pair the torque row exists to keep apart.
///
/// ⚠️ **`None` para uma [`JointKind::Pulley`]**, e não é um caso de erro: o
/// rapier não tem polia, então aquele tipo não tem `JointDesc` nenhum a
/// produzir — ele é roteado para o passe de impulso de
/// `ph2d_physics::world::pulley`. Devolver `Option` em vez de mapear a polia
/// para o tipo "mais parecido" é o que impede uma corda de virar uma corda-de-
/// rapier em silêncio.
pub fn joint_desc(
    j: &PhysicsJoint,
    local_a: [f32; 2],
    local_b: [f32; 2],
    axis: ([f32; 2], [f32; 2]),
) -> Option<JointDesc> {
    Some(JointDesc {
        kind: match j.kind {
            JointKind::Pulley => return None,
            JointKind::Pin => ph2d_physics::JointKind::Pin,
            JointKind::Spring => ph2d_physics::JointKind::Spring,
            JointKind::Rope => ph2d_physics::JointKind::Rope,
            JointKind::Weld => ph2d_physics::JointKind::Weld,
            JointKind::Slider => ph2d_physics::JointKind::Slider,
            JointKind::Rod => ph2d_physics::JointKind::Rod,
            JointKind::Wheel => ph2d_physics::JointKind::Wheel,
        },
        anchor_a: local_a,
        anchor_b: local_b,
        // A parameter the kind ignores is not *passed* to the solver either:
        // `is_hinge` is asked here exactly as the Inspector asks it to decide
        // which rows to paint, so a limit left over from a previous kind
        // cannot quietly still be in force.
        // `has_limits`, not `is_hinge`: a Slider is limited too, and its range is
        // a stroke in metres (`limits_in_metres`) rather than an angle. The
        // Inspector asks the same door to decide which label to paint.
        limits: (j.kind.has_limits() && j.limits_enabled).then_some([j.limit_min, j.limit_max]),
        // `has_motor`, not `is_hinge`: a Slider's rail and a Rope's distance are
        // driven too (W-J6). The wrapper asks the SAME question through
        // `ph2d_physics::motor_axis` — which is where a Spring's exclusion is
        // load-bearing rather than cosmetic (rapier models a spring AS a motor,
        // so a second one there would eat its stiffness).
        motor: (j.kind.has_motor() && j.motor_enabled).then_some(MotorDesc {
            mode: match j.motor_mode {
                crate::joint::MotorMode::Velocity => ph2d_physics::MotorMode::Velocity,
                crate::joint::MotorMode::Position => ph2d_physics::MotorMode::Position,
            },
            speed: j.motor_speed,
            target: j.motor_target,
            max_force: j.motor_max_force,
        }),
        rest_length: j.rest_length,
        stiffness: j.stiffness,
        damping: j.damping,
        max_length: j.max_length,
        axis_a: axis.0,
        axis_b: axis.1,
        // ⚠️ `∞` is what "off" IS (W-J7, P7), so the checkbox is resolved HERE
        // rather than carried into the solver: the wrapper never learns that a
        // joint *could* be breakable, only what it can take. A joint with the box
        // clear packs to the same `user_data` a joint from before this wave has,
        // which is why every scene that predates it is byte-identical.
        break_force: if j.break_enabled {
            j.break_force
        } else {
            f32::INFINITY
        },
        // O limiar de torque só é entregue a quem pode produzir uma reação
        // angular: rapier não publica nada de um eixo TRAVADO. ⚠️ A pergunta é à
        // INSTÂNCIA (`PhysicsJoint::breaks_on_torque`) e não ao tipo, porque uma
        // solda MOLE motoriza o eixo que a rígida trava — medido, 0,9619 N·m
        // contra 0,0000. Perguntado aqui e no painel, para que um valor deixado
        // por uma troca de tipo não siga em vigor — a razão do `has_limits`.
        break_torque: if j.break_enabled && j.breaks_on_torque() {
            j.break_torque
        } else {
            f32::INFINITY
        },
        // W-J8. Both are carried verbatim — they are the same fact on both sides
        // of this boundary, unlike the break thresholds two lines up (a checkbox
        // here, an infinity there). An inactive joint is still BUILT and still
        // reconciled; what stops is the constraint.
        enabled: j.active,
        contacts_enabled: j.collide_connected,
        // ⚠️ **O `can_be_soft` aqui é HIGIENE, e a mutação o provou** — eu havia
        // escrito que ele impedia um `soft` deixado por uma troca de tipo de
        // seguir em vigor, e isso é falso: `desc.soft` tem UM leitor no wrapper
        // inteiro (o braço `JointKind::Weld` do `spawn_joint`), então um Pin com
        // o flag ligado já toma o próprio braço e o ignora. Neutralizar esta
        // pergunta deixa a workspace VERDE, e a nota fica para ninguém a
        // re-derivar como se fosse correção.
        //
        // Ela fica porque a resposta do wrapper é dele, não nossa: o dia em que
        // um segundo tipo ler `desc.soft` é o dia em que isto passa a segurar. E
        // a MESMA porta é load-bearing um andar acima — `PhysicsJoint::
        // breaks_on_torque` a usa para decidir se o limiar de torque é
        // alcançável, e lá a mutação sangra.
        soft: j.kind.can_be_soft() && j.soft,
    })
}
