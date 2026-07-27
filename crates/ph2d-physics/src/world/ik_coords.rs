//! **A ÁLGEBRA DA COORDENADA DE UMA JUNTA** (W-IK) — irmão do [`super::ik`].
//!
//! Módulo próprio porque as duas metades crescem por motivos diferentes: aquele
//! é *o que uma árvore de pose É e como se resolve*; este é *que número uma
//! junta tem, e como escrevê-lo*.
//!
//! E é UMA pergunta, com **três consumidores** que sem ela seriam três leituras
//! do rapier a divergir:
//!
//! - **semear** — pôr a árvore recém-construída na pose em que a cadeia está;
//! - **limitar** — trazer a junta de volta à faixa autorada depois do solve;
//! - **construir** — os frames locais que o joint reduzido carrega.
//!
//! Os dois primeiros são a MESMA correção com sinais diferentes: leia a
//! coordenada que existe, leia a que se quer, aplique a diferença. Uma passada é
//! exata porque a coordenada é linear no deslocamento e `apply_displacements`
//! soma.

use rapier2d::dynamics::{
    FixedJointBuilder, Multibody, PrismaticJointBuilder, RevoluteJointBuilder, RigidBodySet,
};
use rapier2d::na::{DVector, Isometry2, Point2, Vector2};

use super::joints::{JointDesc, JointKind};

use super::ik::{IkLink, LimitedDof};

/// **A coordenada que esta junta de fato tem, dadas as poses de MUNDO.**
///
/// `local_to_parent` de um elo é `F1 · J · F2⁻¹`, onde `F1`/`F2` são os frames
/// locais que [`multibody_joint`] montou e `J` é o movimento da própria junta.
/// Isolando: `J = F1⁻¹ · (P⁻¹·C) · F2` — e aí a coordenada é o **ângulo** de `J`
/// numa dobradiça e a componente **x** da translação num trilho.
///
/// Uma fórmula, dois tipos, sem caso especial: é a mesma expressão que o rapier
/// compõe, resolvida para o lado que falta.
pub(super) fn joint_coordinate(
    kind: JointKind,
    desc: &JointDesc,
    rel: &Isometry2<f32>,
) -> Option<f32> {
    let (f1, f2) = joint_frames(kind, desc)?;
    let j = f1.inverse() * rel * f2;
    match kind {
        JointKind::Pin => Some(j.rotation.angle()),
        JointKind::Slider => Some(j.translation.x),
        // Um Weld não tem grau de liberdade: não há coordenada a semear, e a
        // pose relativa dele é o que o `JointDesc` já diz.
        _ => None,
    }
}

/// Os dois frames locais do joint reduzido — a MESMA construção de
/// [`multibody_joint`], expressa como isometrias.
///
/// ⚠️ Derivada aqui e não lida do `MultibodyJoint`: são os números que NÓS
/// passamos ao builder, então derivá-los é reproduzir a nossa própria receita,
/// enquanto lê-los seria depender de como o rapier a guardou.
pub(super) fn joint_frames(
    kind: JointKind,
    desc: &JointDesc,
) -> Option<(Isometry2<f32>, Isometry2<f32>)> {
    let a = Vector2::new(desc.anchor_a[0], desc.anchor_a[1]);
    let b = Vector2::new(desc.anchor_b[0], desc.anchor_b[1]);
    match kind {
        // O `RevoluteJointBuilder` põe só as âncoras: os frames não giram, e é
        // por isso que o ângulo de `local_to_parent` É a coordenada.
        JointKind::Pin => Some((Isometry2::new(a, 0.0), Isometry2::new(b, 0.0))),
        // O prismático gira `+X` até o eixo do trilho — a rotação que faz a
        // pose relativa NÃO entregar a distância percorrida de graça.
        JointKind::Slider => {
            let ang = |v: [f32; 2]| {
                let n = (v[0] * v[0] + v[1] * v[1]).sqrt();
                if n > 0.0 {
                    (v[1] / n).atan2(v[0] / n)
                } else {
                    0.0
                }
            };
            Some((
                Isometry2::new(a, ang(desc.axis_a)),
                Isometry2::new(b, ang(desc.axis_b)),
            ))
        }
        _ => None,
    }
}

/// Põe as coordenadas da árvore de acordo com as poses REAIS dos corpos.
///
/// Uma passada: para cada elo, a diferença entre a coordenada que o mundo tem e
/// a que a árvore acha que tem, aplicada como deslocamento. Ver a nota em
/// `ik_chain` para por que isto não é opcional.
pub(super) fn seed_coordinates(mb: &mut Multibody, bodies: &RigidBodySet, links: &[IkLink]) {
    let ndofs = mb.ndofs();
    let mut disp = DVector::zeros(ndofs);
    let mut dof = 0usize;
    let mut any = false;
    // Uma passada só de leitura para colher os deslocamentos, porque o
    // `apply_displacements` toma `&mut mb` e a leitura precisa de `&mb`.
    let mut wanted: Vec<(usize, f32)> = Vec::new();
    for l in mb.links() {
        let n = l.joint().ndofs();
        if n == 1
            && let Some(src) = links.iter().find(|k| k.child == l.rigid_body_handle())
            && let (Some(p), Some(c)) = (bodies.get(src.parent), bodies.get(src.child))
            && let Some(world) = joint_coordinate(
                src.joint.kind,
                &src.joint,
                &(p.position().inverse() * c.position()),
            )
            && let Some(tree) = joint_coordinate(src.joint.kind, &src.joint, l.local_to_parent())
        {
            let mut d = world - tree;
            // ⚠️ Só para o eixo ANGULAR: uma diferença de ângulos é definida a
            // menos de uma volta, e semear `+2π` em vez de `0` gira o elo
            // inteiro sem mudar a pose — invisível aqui e visível na próxima
            // vez que alguém ler a coordenada.
            if matches!(src.joint.kind, JointKind::Pin) {
                let tau = std::f32::consts::TAU;
                while d > std::f32::consts::PI {
                    d -= tau;
                }
                while d <= -std::f32::consts::PI {
                    d += tau;
                }
            }
            if d != 0.0 {
                wanted.push((dof, d));
                any = true;
            }
        }
        dof += n;
    }
    if !any {
        return;
    }
    for (i, d) in wanted {
        if i < ndofs {
            disp[i] = d;
        }
    }
    mb.apply_displacements(disp.as_slice());
    mb.forward_kinematics(bodies, false);
}

/// **O limite deste tipo de joint É a coordenada da junta?**
///
/// Para o Pin sim: `local_frame1`/`local_frame2` que [`multibody_joint`] monta
/// carregam **só translação**, então o ângulo de `local_to_parent` É a
/// coordenada, e o limite (radianos) mede exactamente esse número.
///
/// ⚠️ Para o **Slider NÃO**, e isso é honesto em vez de silencioso: o
/// `local_frame1` dele carrega a ROTAÇÃO que leva `+X` ao eixo do trilho, então
/// a pose relativa não entrega a distância percorrida sem desfazer aquele
/// frame. Um Slider limitado **posa sem limite** e o Play o traz de volta ao
/// curso — nomeado no ADR e gateado, para ninguém "descobrir" isso num smoke.
pub(super) fn limit_is_a_coordinate(kind: JointKind) -> bool {
    matches!(kind, JointKind::Pin)
}

/// **A projeção dos limites, depois do solve.**
///
/// ⚠️ **O `inverse_kinematics` do rapier IGNORA limites** — `apply_displacement`
/// é `integrate(1.0, disp)`, aritmética pura sobre a coordenada, sem clamp
/// (conferido no source, não suposto). MEDIDO: uma dobradiça limitada a
/// `[0, 0.3]` rad, puxada para baixo, dobrava até **−90°**. Uma pose que o
/// solver do Play desfaz no primeiro tick não é uma pose: é uma promessa que o
/// produto quebra assim que o artista aperta Play.
///
/// A cura é uma projeção: leia o ângulo que a junta de fato ficou, clampe, e
/// aplique a DIFERENÇA como mais um deslocamento. Para uma dobradiça (um grau
/// de liberdade, e o ângulo é linear na coordenada) **uma passada é exata** —
/// não é iteração, é uma correção fechada.
///
/// O preço, que é o certo: a ponta deixa de alcançar o alvo quando um limite
/// está no caminho. Um cotovelo que não dobra para trás é um cotovelo que não
/// dobra para trás.
pub(super) fn project_limits(
    mb: &mut Multibody,
    limits: &[LimitedDof],
    scratch: &mut DVector<f32>,
) {
    scratch.fill(0.0);
    let mut any = false;
    for l in limits {
        let Some(link) = mb.link(l.link) else {
            continue;
        };
        let theta = link.local_to_parent().rotation.angle();
        let clamped = theta.clamp(l.min, l.max);
        let delta = clamped - theta;
        if delta != 0.0 && l.dof < scratch.len() {
            scratch[l.dof] = delta;
            any = true;
        }
    }
    if any {
        mb.apply_displacements(scratch.as_slice());
    }
}

/// O joint de coordenadas reduzidas equivalente a um joint autorado.
///
/// ⚠️ **Só os rígidos chegam aqui** — o construtor da árvore não gera aresta
/// para Spring nem Rope (ver a nota do módulo). Os braços existem para o `match`
/// ser exaustivo sem um `_ =>` que engoliria um tipo novo em silêncio: um Weld é
/// a resposta conservadora (trava tudo), que é visivelmente errado num smoke em
/// vez de sutilmente errado.
pub(super) fn multibody_joint(desc: &JointDesc) -> rapier2d::dynamics::GenericJoint {
    let a = Point2::new(desc.anchor_a[0], desc.anchor_a[1]);
    let b = Point2::new(desc.anchor_b[0], desc.anchor_b[1]);
    match desc.kind {
        JointKind::Pin => {
            let mut builder = RevoluteJointBuilder::new()
                .local_anchor1(a)
                .local_anchor2(b);
            // ⚠️ Os limites do joint VALEM na pose: um cotovelo que não dobra
            // para trás na simulação não pode dobrar para trás ao ser posado, ou
            // a pose que o artista autora é uma que o Play desfaz no 1º tick.
            if let Some([min, max]) = desc.limits {
                builder = builder.limits([min, max]);
            }
            builder.into()
        }
        JointKind::Slider => {
            let mut builder = PrismaticJointBuilder::new(super::joints::unit_or_x(desc.axis_a))
                .local_axis1(super::joints::unit_or_x(desc.axis_a))
                .local_axis2(super::joints::unit_or_x(desc.axis_b))
                .local_anchor1(a)
                .local_anchor2(b);
            if let Some([min, max]) = desc.limits {
                builder = builder.limits([min, max]);
            }
            builder.into()
        }
        JointKind::Weld | JointKind::Spring | JointKind::Rope => FixedJointBuilder::new()
            .local_anchor1(a)
            .local_anchor2(b)
            .into(),
    }
}

/// **Este tipo de joint vira elo de uma árvore de pose?**
///
/// A porta única: o construtor da árvore (na ponte do ECS) pergunta a ELA quais
/// arestas existem, e enumerar os tipos lá seria a lista que nasce incompleta no
/// dia em que um tipo novo chegar.
#[must_use]
pub fn is_rigid_link(kind: JointKind) -> bool {
    matches!(kind, JointKind::Pin | JointKind::Weld | JointKind::Slider)
}
