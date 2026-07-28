//! **O desenho de um joint ROMPIDO** (W-J7) — irmão do
//! `physics_overlay_joints_tests`, separado dele pelo cap de 600 LOC do shell e
//! cortado por assunto: aqui tudo descreve um joint que **não está segurando**.
//!
//! Duas afirmações, e elas falham por motivos diferentes: a COR (o joint inteiro
//! tinge de vermelho, então ele não se confunde com um vizinho que segura) e o
//! ENVELOPE (o arco de limite, o anel de comprimento e a seta do motor somem,
//! porque nenhum deles está mais em vigor — desenhá-los descreveria uma regra que
//! o solver deixou de aplicar, que é a divergência desenho×solver que o P2 do
//! plano proíbe).

use super::*;
use crate::render_loop::physics_overlay_joints::{JOINT_BROKEN_DIM_RGBA, JOINT_BROKEN_RGBA};

/// Um Pin com limite, motor e as duas âncoras — a view mais CHEIA que o overlay
/// desenha, de propósito: é a que perde mais coisas ao romper.
fn broken_view(broken: bool) -> JointView {
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
        broken,
        active: true,
        // W-J7b: a fixture descreve um joint SEM teto e sem carga — o readout é
        // assunto do irmão `physics_overlay_joint_readout_tests`, e um teto aqui
        // faria estes gates contarem um rótulo que não é sobre eles.
        load: ph2d_physics_ecs::JointLoad::ZERO,
        peak: ph2d_physics_ecs::JointLoad::ZERO,
        break_force: f32::INFINITY,
        break_torque: f32::INFINITY,
    }
}

/// Quantos elementos o estouro tem: três diâmetros = 3 `move_to` + 3 `line_to`.
const BURST_ELEMENTS: usize = 6;

fn segments(paths: &[(BezPath, [f32; 4])], rgba: [f32; 4]) -> usize {
    paths
        .iter()
        .filter(|(_, c)| *c == rgba)
        .map(|(p, _)| p.elements().len())
        .sum()
}

/// **Um joint rompido é VERMELHO, e nenhuma parte dele segue âmbar.**
///
/// A cor é a diferença que o artista lê de longe — numa corrente de dez elos,
/// procurar qual parou de segurar não pode ser um exercício de comparar
/// espessuras.
///
/// Mutação: `joint_marks` ignorando `v.broken` na escolha da paleta — todo traço
/// volta a ser âmbar e as duas metades do assert caem.
#[test]
fn a_broken_joint_is_drawn_in_red_and_nothing_of_it_stays_amber() {
    let out = marks(&broken_view(true));
    assert!(
        segments(&out, JOINT_BROKEN_RGBA) > 0,
        "o joint rompido tem de desenhar em vermelho"
    );
    assert!(
        segments(&out, JOINT_BROKEN_DIM_RGBA) > 0,
        "e as linhas de posse junto — elas seguem dizendo QUAIS dois objetos ele nomeia"
    );
    assert_eq!(
        segments(&out, JOINT_RGBA) + segments(&out, JOINT_DIM_RGBA),
        0,
        "e NADA dele continua âmbar, senão ele lê como um joint que segura"
    );
}

/// **O ENVELOPE some** — o arco de limite e a seta do motor não são desenhados
/// para um joint que não está impondo nada.
///
/// ⚠️ **O oráculo é EXATO e não "desenha menos", e a mutação é por quê.** A 1ª
/// versão comparava o total VERMELHO do rompido com o total ÂMBAR do que segura,
/// e tirar o `continue` do braço rompido **passava por ela**: o arco e o glifo de
/// giro voltam pintados em ÂMBAR (as duas `push` os nomeiam direto), então a
/// contagem vermelha não se move e o gate não podia falhar pelo motivo que
/// alegava. Agora a afirmação é a identidade: um joint rompido desenha
/// exatamente o que um joint SEM limite e SEM motor desenha, mais o estouro.
#[test]
fn a_broken_joint_draws_no_envelope_but_a_holding_one_does() {
    let total = |v: &JointView| -> usize { marks(v).iter().map(|(p, _)| p.elements().len()).sum() };
    let bare = JointView {
        limits: None,
        motor_speed: None,
        ..broken_view(false)
    };
    // O controle, na mesma medida: com limite E motor um joint desenha MAIS.
    assert!(
        total(&broken_view(false)) > total(&bare),
        "o controle: um Pin com limite e motor desenha o envelope"
    );
    assert_eq!(
        total(&broken_view(true)),
        total(&bare) + BURST_ELEMENTS,
        "um joint rompido desenha o que um sem envelope desenha, mais o estouro"
    );
}

/// **O estouro marca ONDE ele partiu**, e existe só no rompido.
///
/// Desenhado do ESTADO e não do evento: um clarão de seis ticks sobre uma cena
/// que segue rompida some antes de o artista olhar, e a pergunta *onde isto
/// arrebentou?* continua sendo feita depois.
#[test]
fn only_a_broken_joint_gets_a_burst() {
    // O estouro são três diâmetros = 6 elementos (3 move + 3 line), no MEIO das
    // duas âncoras. Contado pela diferença: a view que segura não o tem.
    let broken = marks(&broken_view(true));
    let burst = broken
        .iter()
        .filter(|(_, c)| *c == JOINT_BROKEN_RGBA)
        .any(|(p, _)| p.elements().len() == BURST_ELEMENTS);
    assert!(burst, "o joint rompido desenha o estouro de seis pontas");
    let holding = marks(&broken_view(false));
    assert_eq!(
        segments(&holding, JOINT_BROKEN_RGBA),
        0,
        "e um joint que segura não desenha nada em vermelho de rompimento"
    );
}
