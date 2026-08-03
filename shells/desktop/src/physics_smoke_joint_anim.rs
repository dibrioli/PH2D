//! **A cena da MÁQUINA ANIMADA** (`PH2D_PHYSICS_SMOKE=78`, W-JointAnim).
//!
//! Até esta wave um parâmetro de joint era um número que o artista digitava e a
//! cena segurava para sempre. Um servo apontava para um lugar; um guincho tinha
//! um comprimento. Animar a máquina exigia animar os CORPOS — que é assar o
//! resultado em vez de dirigir a causa.
//!
//! Agora os quatro números que fazem uma máquina se mover são **canais de
//! timeline**: o alvo do servo, a taxa do motor, o comprimento que a mola quer e
//! o que a corda governa. O artista põe keys neles e a física responde.
//!
//! ⚠️ **O que esta cena tem de provar não é que a track existe — é que ela chega
//! ao solver TICK A TICK.** Um parâmetro keyframado é uma entrada por tick, do
//! mesmo jeito que a pose de um corpo cinemático (a lição do W4b), e o modo de
//! falha é silencioso: o número chega uma vez por QUADRO no play e **nunca** num
//! replay. É por isso que o roteiro manda arrastar a régua para trás.
//!
//! Os números abaixo saíram da sonda `probe_smoke_78`, rodada ANTES desta
//! mensagem ser escrita.

use crate::physics_smoke::spawn_floor;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform, World, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, MotorMode, PhysicsJoint, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_timeline::{PropKind, TimelineDoc};

const GREY: [f32; 4] = [0.75, 0.75, 0.8, 1.0];
const HOT: [f32; 4] = [0.95, 0.6, 0.2, 1.0];
const COOL: [f32; 4] = [0.4, 0.8, 0.95, 1.0];
const DEAD: [f32; 4] = [0.45, 0.46, 0.5, 1.0];

/// A altura de cada bancada.
pub(crate) const LANE_Y: [f32; 4] = [4.2, 1.2, -1.6, -4.2];

fn body(
    world: &mut World,
    name: &str,
    kind: BodyKind,
    shape: ColliderShape,
    size: [f32; 2],
    rgba: [f32; 4],
    at: [f32; 2],
) {
    world.spawn((
        Transform::from_translation(Vec2::new(at[0], at[1])),
        Sprite::atlas(WHITE_TILE_KEY, size, rgba),
        Name::new(name.to_string()),
        RigidBody { kind },
        Collider {
            shape,
            ..Collider::default()
        },
    ));
}

fn joint(world: &mut World, name: &str, a: &str, b: &str, at: [f32; 2], j: PhysicsJoint) -> Entity {
    world
        .spawn((
            Name::new(name.to_string()),
            PhysicsJoint {
                body_a: stable_name_id(a),
                body_b: stable_name_id(b),
                ..j
            },
            Transform::from_translation(Vec2::new(at[0], at[1])),
        ))
        .id()
}

/// Monta as quatro bancadas. Devolve as entidades-joint que a cena vai animar,
/// na ordem `[servo, guincho, músculo, giro]`.
pub(crate) fn build_joint_anim_scene(world: &mut World) -> [Entity; 4] {
    spawn_floor(world);
    let peg = ColliderShape::Ball { radius: 0.08 };
    let arm = ColliderShape::Cuboid {
        half_x: 0.7,
        half_y: 0.09,
    };
    let load = ColliderShape::Cuboid {
        half_x: 0.22,
        half_y: 0.22,
    };

    // ── 1. O SERVO: o alvo é keyframado, e o braço vai atrás.
    let y = LANE_Y[0];
    body(
        world,
        "ServoPost",
        BodyKind::Static,
        peg,
        [0.16, 0.16],
        GREY,
        [-3.0, y],
    );
    body(
        world,
        "ServoArm",
        BodyKind::Dynamic,
        arm,
        [1.4, 0.18],
        HOT,
        [-2.3, y],
    );
    let servo = joint(
        world,
        "Servo",
        "ServoPost",
        "ServoArm",
        [-3.0, y],
        PhysicsJoint {
            kind: JointKind::Pin,
            motor_enabled: true,
            motor_mode: MotorMode::Position,
            motor_target: 0.0,
            motor_max_force: 400.0,
            ..PhysicsJoint::default()
        },
    );
    // O CONTROLE, na mesma bancada: o MESMO braço, sem track nenhuma. Sem ele a
    // cena não distingue *o alvo foi animado* de *tudo se mexe sozinho*.
    body(
        world,
        "CtrlPost",
        BodyKind::Static,
        peg,
        [0.16, 0.16],
        GREY,
        [1.4, y],
    );
    body(
        world,
        "CtrlArm",
        BodyKind::Dynamic,
        arm,
        [1.4, 0.18],
        DEAD,
        [2.1, y],
    );
    joint(
        world,
        "CtrlServo",
        "CtrlPost",
        "CtrlArm",
        [1.4, y],
        PhysicsJoint {
            kind: JointKind::Pin,
            motor_enabled: true,
            motor_mode: MotorMode::Position,
            motor_target: 0.0,
            motor_max_force: 400.0,
            ..PhysicsJoint::default()
        },
    );

    // ── 2. O GUINCHO: o TETO da corda é keyframado, e a carga sobe.
    let y = LANE_Y[1];
    body(
        world,
        "WinchPost",
        BodyKind::Static,
        peg,
        [0.16, 0.16],
        GREY,
        [-3.0, y],
    );
    body(
        world,
        "WinchLoad",
        BodyKind::Dynamic,
        load,
        [0.44, 0.44],
        COOL,
        [-3.0, y - 2.4],
    );
    let winch = joint(
        world,
        "Winch",
        "WinchPost",
        "WinchLoad",
        [-3.0, y],
        PhysicsJoint {
            kind: JointKind::Rope,
            max_length: 2.4,
            ..PhysicsJoint::default()
        },
    );

    // ── 3. O MÚSCULO: o comprimento que a mola QUER é keyframado.
    let y = LANE_Y[2];
    body(
        world,
        "MusclePost",
        BodyKind::Static,
        peg,
        [0.16, 0.16],
        GREY,
        [0.0, y],
    );
    body(
        world,
        "MuscleWeight",
        BodyKind::Dynamic,
        load,
        [0.44, 0.44],
        HOT,
        [0.0, y - 2.0],
    );
    let muscle = joint(
        world,
        "Muscle",
        "MusclePost",
        "MuscleWeight",
        [0.0, y],
        PhysicsJoint {
            kind: JointKind::Spring,
            rest_length: 2.0,
            stiffness: 260.0,
            damping: 14.0,
            ..PhysicsJoint::default()
        },
    );

    // ── 4. O GIRO: a TAXA do motor é keyframada, e a roda acelera.
    let y = LANE_Y[3];
    body(
        world,
        "SpinPost",
        BodyKind::Static,
        peg,
        [0.16, 0.16],
        GREY,
        [2.6, y],
    );
    body(
        world,
        "SpinBlade",
        BodyKind::Dynamic,
        arm,
        [1.4, 0.18],
        COOL,
        [2.6, y],
    );
    let spin = joint(
        world,
        "Spinner",
        "SpinPost",
        "SpinBlade",
        [2.6, y],
        PhysicsJoint {
            kind: JointKind::Pin,
            motor_enabled: true,
            motor_mode: MotorMode::Velocity,
            motor_speed: 0.0,
            motor_max_force: 60.0,
            ..PhysicsJoint::default()
        },
    );

    [servo, winch, muscle, spin]
}

/// Uma key linear em `(entidade, prop)` no tempo `t`.
fn key(doc: &mut TimelineDoc, e: Entity, prop: PropKind, t: f64, v: f32) {
    doc.insert_key(
        e.to_bits(),
        prop,
        RationalTime::from_seconds(t),
        AnimValue::Float(v),
        Interp::Linear,
    );
}

/// **As quatro tracks** — a autoria que esta cena existe para demonstrar.
pub(crate) fn author_joint_anim_tracks(doc: &mut TimelineDoc, j: [Entity; 4]) {
    // O servo varre: centro → baixo → cima.
    key(doc, j[0], PropKind::JointMotorTarget, 0.0, 0.0);
    key(doc, j[0], PropKind::JointMotorTarget, 1.2, -1.15);
    key(doc, j[0], PropKind::JointMotorTarget, 2.6, 0.85);
    key(doc, j[0], PropKind::JointMotorTarget, 4.0, 0.0);
    // O guincho recolhe.
    key(doc, j[1], PropKind::JointMaxLength, 0.0, 2.4);
    key(doc, j[1], PropKind::JointMaxLength, 3.0, 0.5);
    // O músculo contrai e solta.
    key(doc, j[2], PropKind::JointRestLength, 0.0, 2.0);
    key(doc, j[2], PropKind::JointRestLength, 1.5, 0.7);
    key(doc, j[2], PropKind::JointRestLength, 3.5, 2.0);
    // A roda acelera.
    key(doc, j[3], PropKind::JointMotorSpeed, 0.0, 0.0);
    key(doc, j[3], PropKind::JointMotorSpeed, 3.5, 11.0);
}

#[cfg(test)]
#[path = "physics_smoke_joint_anim_tests.rs"]
mod tests;

impl crate::App {
    /// **Cena 78 (W-JointAnim).** Quatro máquinas dirigidas por keyframes.
    pub(crate) fn physics_smoke_joint_anim(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let joints = build_joint_anim_scene(gfx.sim.world_mut());
        gfx.camera.center = [-0.4, 0.0];
        gfx.camera.height_world = 15.0;
        author_joint_anim_tracks(&mut self.timeline.doc, joints);

        eprintln!(
            "[physics-smoke 78] A MAQUINA ANIMADA -- os parametros de um joint sao\n  \
               canais de timeline.\n\n  \
               Ate' aqui um param de joint era um numero que a cena segurava para\n  \
               sempre: o servo apontava para um lugar, o guincho tinha um comprimento.\n  \
               Animar a maquina exigia animar os CORPOS -- assar o resultado em vez de\n  \
               dirigir a causa. Agora os quatro numeros que fazem uma maquina se mexer\n  \
               tem track.\n\n  \
               Quatro bancadas, uma por canal. **Aperte Espaco** e assista:\n     \
                  - EM CIMA, o SERVO (laranja): 'Motor Target' keyframado. O braco\n       \
                    varre centro -> baixo -> cima -> centro; medido, ele passa por\n       \
                    -0,90 rad em 1 s e +0,64 rad em 3 s. Ao lado dele, em CINZA, o\n       \
                    MESMO braco sem track: medido, ele nao sai de -0,005 rad. Esse\n       \
                    e' o controle, e sem ele a cena nao distingue *o alvo foi\n       \
                    animado* de *tudo se mexe sozinho*.\n     \
                  - O GUINCHO (azul): 'Max Length' keyframado de 2,4 m para 0,5 m. A\n       \
                    carga SOBE 1,90 m, recolhida pela corda.\n     \
                  - O MUSCULO (laranja): 'Rest Length' keyframado. A mola contrai\n       \
                    1,00 m e SOLTA -- e a rigidez dela nunca foi tocada.\n     \
                  - EMBAIXO, o GIRO (azul): 'Motor Speed' keyframado de 0 a 11 rad/s.\n       \
                    A pa' acelera.\n\n  \
               (!) A PERGUNTA DA WAVE E' A REGUA. Deixe tocar ate' o fim, depois\n      \
                   **arraste a regua para tras** e solte no meio. A cena tem de mostrar\n      \
                   a pose daquele instante, nao a do fim -- e tem de faze-lo de novo,\n      \
                   igual, quantas vezes voce arrastar. Um param que so' chega ao solver\n      \
                   uma vez por quadro sobrevive ao play e MORRE aqui.\n\n  \
               (!) E AUTORE UMA: selecione o objeto 'Servo' na Hierarquia, abra o\n      \
                   '+ Track' da timeline e escolha **Motor Target**. A lista tem os\n      \
                   quatro canais novos no fim. Com a track criada, mova o playhead,\n      \
                   mude o alvo na secao Physics Joint e aperte **K**.\n"
        );
    }
}
