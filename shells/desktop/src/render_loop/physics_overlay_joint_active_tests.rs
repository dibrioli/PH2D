//! **O desenho de um joint DESLIGADO** (W-J8) — irmão dos testes de ruptura, e
//! cortado pela mesma linha: ali um joint que **deixou** de segurar, aqui um que
//! o artista **desarmou**.
//!
//! Três afirmações, e cada uma falha por um motivo próprio: ele apaga (a mesma
//! figura, um terço da tinta), ele **não** fica vermelho nem ganha estouro (isso
//! é ruptura, e dizer *partiu* onde o artista disse *desliga* é uma cena mentindo
//! sobre o que aconteceu), e o **envelope acompanha** — o arco de limite de um
//! joint apagado não pode ficar brilhando sozinho.

use super::*;
use crate::render_loop::physics_overlay_joints::{
    JOINT_BROKEN_RGBA, JOINT_DIM_RGBA, JOINT_OFF_DIM_RGBA, JOINT_OFF_RGBA, JOINT_RGBA,
};

/// Um Pin com limite e motor — a view mais CHEIA que o overlay desenha, para que
/// "o envelope acompanha" tenha um envelope de verdade para acompanhar.
fn view(active: bool) -> JointView {
    JointView {
        entity: ph2d_ecs::Entity::from_bits(1),
        kind: JointKind::Pin,
        anchor_a: [0.0, 0.0],
        anchor_b: [0.0, 0.0],
        centre_a: [-1.0, 0.0],
        centre_b: [1.0, 0.0],
        body_b: ph2d_ecs::Entity::from_bits(3),
        angle_a: 0.0,
        angle_b: 0.0,
        limits: Some([-0.7, 0.7]),
        motor_speed: Some(2.0),
        length: None,
        wheel_start: 0,
        wheel_count: 0,
        axis: None,
        broken: false,
        active,
        load: ph2d_physics_ecs::JointLoad::ZERO,
        peak: ph2d_physics_ecs::JointLoad::ZERO,
        break_force: f32::INFINITY,
        break_torque: f32::INFINITY,
    }
}

fn segments(paths: &[(BezPath, [f32; 4])], rgba: [f32; 4]) -> usize {
    paths
        .iter()
        .filter(|(_, c)| *c == rgba)
        .map(|(p, _)| p.elements().len())
        .sum()
}

fn marks(active: bool) -> Vec<(BezPath, [f32; 4])> {
    joint_marks(
        true,
        &[view(active)],
        &wheels(),
        [0.0, -9.81],
        &camera(),
        window(),
    )
}

/// **Um joint desligado desenha a MESMA figura, apagada — e nada dele fica
/// aceso.**
///
/// A segunda metade é a que tem dentes: pintar só o glifo de apagado e deixar o
/// arco de limite no âmbar cheio seria o defeito real (o desenho dizendo *este
/// alcance está em vigor* sobre um joint que não impõe nada), e é exatamente o
/// que o código fazia antes desta wave, porque o envelope usava as constantes
/// acesas em vez do par escolhido para a view.
///
/// Mutação: o envelope de volta a `JOINT_DIM_RGBA` fixo → o joint desligado
/// deixa 12 elementos acesos, vermelho.
#[test]
fn an_inactive_joint_is_drawn_dim_and_nothing_of_it_stays_lit() {
    let off = marks(false);
    assert!(
        segments(&off, JOINT_OFF_RGBA) > 0,
        "o glifo e o vao desenham na tinta apagada"
    );
    assert!(
        segments(&off, JOINT_OFF_DIM_RGBA) > 0,
        "e as linhas de posse tambem"
    );
    assert_eq!(
        segments(&off, JOINT_RGBA) + segments(&off, JOINT_DIM_RGBA),
        0,
        "e NADA dele fica na tinta acesa — o arco de limite inclusive"
    );
}

/// O controle: o mesmo joint ATIVO desenha aceso, e nada dele apagado. Sem ele
/// "está apagado" seria satisfeito por um overlay que apagou tudo.
#[test]
fn the_same_joint_active_is_drawn_lit() {
    let on = marks(true);
    assert!(segments(&on, JOINT_RGBA) > 0, "o glifo acende");
    assert_eq!(
        segments(&on, JOINT_OFF_RGBA) + segments(&on, JOINT_OFF_DIM_RGBA),
        0,
        "e nada dele usa a tinta de desligado"
    );
}

/// **Desligar NÃO é romper: sem vermelho, sem estouro.**
///
/// A distinção que a wave inteira depende de manter — as duas escrevem a mesma
/// flag do rapier, e só o `desc` as separa. Um joint desarmado pintado de
/// vermelho diria ao artista que o rig cedeu sob carga.
///
/// Mutação: `broken` derivado só do solver (sem o `&& j.rest.enabled` na ponte) →
/// a view chega com `broken: true` e este gate fica vermelho no lugar certo.
#[test]
fn switching_a_joint_off_is_not_the_same_as_breaking_it() {
    let off = marks(false);
    assert_eq!(
        segments(&off, JOINT_BROKEN_RGBA),
        0,
        "nada de vermelho: ele nao partiu, foi desarmado"
    );
    // O estouro é a assinatura da ruptura; a contagem total de elementos de um
    // joint desligado tem de ser a de um aceso, sem os 6 do estouro.
    let on = marks(true);
    let total =
        |v: &[(BezPath, [f32; 4])]| -> usize { v.iter().map(|(p, _)| p.elements().len()).sum() };
    assert_eq!(
        total(&off),
        total(&on),
        "a GEOMETRIA e a mesma — desligar muda a tinta, nao a figura"
    );
}
