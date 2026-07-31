//! **Criar um joint cujo lado B é o MUNDO** (W-JointWorld).
//!
//! Irmão de [`super::inspector_joint::create_joint_at`], e separado dele porque
//! aquele exige DOIS corpos por construção — ele toma bits de entidade, e o
//! mundo não tem entidade nenhuma (`Entity::from_bits(0)` não existe).
//!
//! ⚠️ **A política de âncora é a MESMA**, e é de propósito: o par nomeado pelo
//! gesto é *(press no corpo, release no mundo)*, e um tipo que COMPARTILHA um
//! ponto (Pin/Weld) tem de pousar os dois no mesmo lugar — senão a criação
//! começa com um TRANCO, arrancando o corpo do ponto onde ele estava para o
//! ponto onde o artista soltou. Duas cópias desta regra divergiriam na primeira
//! vez que qualquer uma mudasse.

use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{JointKind, JointWorldAnchor, PhysicsJoint};

use super::inspector_joint::ensure_named;

/// **Qual ponto do gesto é o quê**, quando um dos lados é o cenário.
///
/// `from_body` diz se o gesto SAIU de um corpo. Devolve `(ponto no corpo, âncora
/// no mundo)` — e é só isso que a direção muda: um pino desenhado do corpo para
/// a parede e um desenhado da parede para o corpo são o MESMO objeto.
///
/// ⚠️ **Porta própria porque a alternativa é a troca escrita nos dois braços do
/// `match`**, e uma delas nasceria invertida no dia em que um terceiro braço
/// aparecer. Pura, então o gate pode perguntar direto — o `joint_draw_release`
/// exige janela e nenhum teste de unidade o alcança.
#[must_use]
pub(crate) fn gesture_points(
    from_body: bool,
    press: [f32; 2],
    release: [f32; 2],
) -> ([f32; 2], [f32; 2]) {
    if from_body {
        (press, release)
    } else {
        (release, press)
    }
}

/// **O pino de parede, nascido de um gesto.**
///
/// `press` é onde o arrasto começou (sobre o corpo) e `anchor` é onde ele
/// terminou (no cenário). Devolve a entidade-joint, ou `None` se o corpo não
/// puder ser nomeado — a mesma condição que o irmão de dois corpos tem.
///
/// ⚠️ **A POLIA é recusada aqui**, e não por herança: a corda puxa as DUAS
/// pontas, e uma delas presa ao cenário é outra máquina (o `motor_rate` já é o
/// guincho). O reconcile também a recusa; recusar no gesto é o que impede o
/// artista de criar um objeto que nasce dormente.
pub(crate) fn create_world_pin_at(
    sim: &mut SimWorld,
    a_bits: u64,
    kind: JointKind,
    press: [f32; 2],
    anchor: [f32; 2],
) -> Option<Entity> {
    if kind == JointKind::Pulley {
        return None;
    }
    let a = Entity::from_bits(a_bits);
    let pose = {
        let t = sim.world().get::<Transform>(a)?;
        [t.translation.x, t.translation.y, t.rotation]
    };
    let name_a = ensure_named(sim, a, "Body")?;
    // **Um joint é nomeado pelo que ele junta** (W-J8) — e aqui a outra ponta é
    // o cenário, então ela se chama pelo que é.
    let label = crate::name_unique::unique_name(sim, &format!("{name_a} : World"));
    // A âncora em A: o ponto do PRESS num tipo de duas pontas (a mola prende
    // onde você agarrou), e o ponto do RELEASE num que compartilha um ponto —
    // porque ali os dois lados são o MESMO lugar, e esse lugar é o pivô que o
    // artista acabou de apontar.
    let wa = if kind.shares_a_point() { anchor } else { press };
    let local_a = ph2d_physics_ecs::PhysicsWorld::local_anchor_at_pose(pose, wa);
    let base = PhysicsJoint::of_kind(kind);
    let joint = sim
        .world_mut()
        .spawn((
            Name::new(label),
            PhysicsJoint {
                body_a: stable_name_id(&name_a),
                // O mundo não tem nome a apontar; quem diz que este zero é o
                // cenário — e não uma ponta que falta — é o marcador abaixo.
                body_b: 0,
                local_a,
                // A âncora É o ponto, então o local dela nele é a origem.
                local_b: [0.0, 0.0],
                anchored: true,
                ..base
            },
            JointWorldAnchor,
            Transform::from_translation(ph2d_core::Vec2::new(anchor[0], anchor[1])),
        ))
        .id();
    // O MESMO fecho do irmão: toda raiz ganha z explícito, senão a árvore
    // desempata por bits de entidade — que o respawn do undo TROCA.
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    Some(joint)
}
