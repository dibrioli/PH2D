//! Os gates do **LIMITE DE ÂNGULO** — a quarta fatia do [`super`], pelo corte por RESPONSABILIDADE
//! que este ficheiro já fez três vezes.
//!
//! A LEI (aparar um ângulo num arco) está gateada em `ph2d-skeleton`. Aqui mede-se o que só existe
//! com um mundo ECS: que o limite apara **as duas** mãos que giram um osso (o dedo e o solver), que
//! ele é no-op quando não existe, e — o risco real desta wave — que o par *solver + limite* **assenta
//! em vez de vibrar**.

use super::*;
use ph2d_skeleton_ecs::BoneLimit;

/// Põe um limite em `bone` e devolve a faixa em radianos.
fn limita(sim: &mut SimWorld, bone: Entity, min: f64, max: f64) {
    sim.world_mut().entity_mut(bone).insert(BoneLimit { min, max });
}

fn rot(sim: &SimWorld, e: Entity) -> f64 {
    f64::from(sim.world().get::<Transform>(e).expect("tem pose").rotation)
}

/// ⭐⭐⭐ **O LIMITE APARA O QUE O DEDO PEDE** — não só o que o solver pede.
///
/// ⚠️ É a metade que o Godot não tem: lá o limite vive na restrição de IK, então o gesto de girar o
/// osso à mão atravessa-o. Um limite que obedece ou não conforme QUEM moveu a junta é pior que
/// limite nenhum — o artista não consegue formar um modelo do que a ferramenta faz.
#[test]
fn the_limit_clamps_what_the_finger_asks_for_not_only_the_solver() {
    let (mut sim, [ombro, _]) = braco();
    limita(&mut sim, ombro, -0.2, 0.2);
    // Aponta o ombro para muito acima do que o limite permite.
    assert!(crate::bone_gesture::pose(
        &mut sim,
        ombro,
        [0.0, 10.0],
        ph2d_skeleton_render::BonePart::Body,
    ));
    let r = rot(&sim, ombro);
    assert!(
        (r - 0.2).abs() < 1e-6,
        "o dedo pediu ~90 graus e a junta devia parar em 0,2 rad — parou em {r}"
    );
}

/// ⭐ **SEM LIMITE, O GESTO É O DE SEMPRE AO BIT** — a lei da casa, e a prova de que a junta sem
/// limite (a esmagadora maioria) não paga nada por esta wave existir.
#[test]
fn a_bone_without_a_limit_moves_exactly_as_before() {
    let alvo = [3.0, 7.0];
    let (mut sim_a, [a, _]) = braco();
    let (mut sim_b, [b, _]) = braco();
    limita(&mut sim_b, b, -ph2d_skeleton::FULL_TURN / 2.0, ph2d_skeleton::FULL_TURN / 2.0);
    assert!(crate::bone_gesture::pose(
        &mut sim_a,
        a,
        alvo,
        ph2d_skeleton_render::BonePart::Body,
    ));
    assert!(crate::bone_gesture::pose(
        &mut sim_b,
        b,
        alvo,
        ph2d_skeleton_render::BonePart::Body,
    ));
    assert_eq!(
        rot(&sim_a, a),
        rot(&sim_b, b),
        "um limite da volta inteira mudou a pose — ele devia ser o no-op exacto"
    );
}

/// ⭐⭐⭐ **O LIMITE APARA O SOLVER, e a ponta deixa de alcançar o alvo — que é o CERTO.**
///
/// ⚠️ Uma restrição de IK que atravessasse o limite para chegar ao alvo é precisamente o defeito:
/// o membro alcança por um caminho que um corpo não faz. O Blender responde igual — com *IK
/// limits*, a ponta fica onde a anatomia deixa.
#[test]
fn the_limit_stops_the_solver_and_the_tip_falls_short_on_purpose() {
    let (mut sim, [ombro, cotovelo]) = braco();
    add(&mut sim, cotovelo).expect("a âncora nasce");
    // Sem limite a corrente alcança um alvo lá em cima.
    let alto = [4.0, 16.0];
    quadro(&mut sim, cotovelo, alto);
    let livre = (ponta(&sim, cotovelo)[0] - alto[0]).hypot(ponta(&sim, cotovelo)[1] - alto[1]);
    // Com o ombro preso perto de zero, ela não chega lá.
    limita(&mut sim, ombro, -0.05, 0.05);
    for _ in 0..8 {
        quadro(&mut sim, cotovelo, alto);
    }
    let preso = (ponta(&sim, cotovelo)[0] - alto[0]).hypot(ponta(&sim, cotovelo)[1] - alto[1]);
    assert!(
        preso > livre + 1.0,
        "com o ombro limitado a ponta devia ficar LONGE do alvo (livre {livre}, preso {preso})"
    );
    assert!(
        rot(&sim, ombro).abs() <= 0.05 + 1e-6,
        "o solver atravessou o limite do ombro: {}",
        rot(&sim, ombro)
    );
}

/// ⭐⭐⭐ **O PAR *SOLVER + LIMITE* ASSENTA, E NÃO VIBRA** — o risco real desta wave.
///
/// ⚠️ O solver resolve, o limite apara, e o quadro seguinte parte da pose **aparada**. Se aparar
/// mudasse a entrada da resolução seguinte o bastante para ela pedir outra coisa, a junta oscilaria
/// a 60 Hz entre duas poses — e nada num teste que resolva UMA vez o veria.
///
/// ⚠️ A régua é o movimento entre quadros **decrescer**, não ser zero: o FABRIK sai cedo quando o
/// erro cai abaixo da tolerância, então uma corrente que ainda refina é sã. *Convergir e oscilar são
/// coisas diferentes* — foi a mesma correcção que a régua do lado da dobra precisou.
#[test]
fn a_limited_chain_settles_instead_of_oscillating() {
    let (mut sim, [ombro, cotovelo]) = braco();
    add(&mut sim, cotovelo).expect("a âncora nasce");
    limita(&mut sim, ombro, -0.3, 0.3);
    limita(&mut sim, cotovelo, -0.8, 0.8);
    let alvo = [6.0, 14.0];
    let mut movs = Vec::new();
    let mut ant = (rot(&sim, ombro), rot(&sim, cotovelo));
    for _ in 0..12 {
        quadro(&mut sim, cotovelo, alvo);
        let agora = (rot(&sim, ombro), rot(&sim, cotovelo));
        movs.push((agora.0 - ant.0).abs().max((agora.1 - ant.1).abs()));
        ant = agora;
    }
    let cedo = movs[1];
    let tarde = movs[11];
    assert!(
        tarde <= cedo + 1e-9,
        "a corrente limitada está a OSCILAR: o movimento por quadro foi {cedo} e ficou {tarde} \
         (série {movs:?})"
    );
    assert!(
        rot(&sim, ombro).abs() <= 0.3 + 1e-6 && rot(&sim, cotovelo).abs() <= 0.8 + 1e-6,
        "uma das juntas acabou fora do próprio limite"
    );
}

/// ⭐⭐⭐ **A CENA DE SMOKE DÁ AO DONO UMA PAREDE PARA SENTIR — e um vizinho livre ao lado.**
///
/// ⚠️ É o irmão do `the_smoke_scene_gives_the_anchor_a_side_to_defend`, e existe pela mesma razão
/// (`CLAUDE.md` §5.0): uma cena em que **tudo** tem limite não distingue *«o limite funciona»* de
/// *«o osso não roda»*, e uma em que **nada** tem deixa o dono a clicar num botão sem ver efeito.
/// O que ensina é o CONTRASTE, e é ele que este gate fixa.
#[test]
fn the_smoke_scene_has_one_limited_bone_and_a_free_neighbour() {
    use crate::vec_bone_smoke::TENTACLE_LIMIT_HALF;
    // A cena põe o limite no `TENTACLE_LIMITED_BONE`-ésimo osso e deixa os outros livres. Uma
    // corrente de dois ossos reproduz a estrutura: o limitado e o vizinho.
    let (mut sim, [livre, preso]) = braco();
    limita(
        &mut sim,
        preso,
        -TENTACLE_LIMIT_HALF,
        TENTACLE_LIMIT_HALF,
    );
    let longe = [0.0, 20.0];
    assert!(crate::bone_gesture::pose(
        &mut sim,
        preso,
        longe,
        ph2d_skeleton_render::BonePart::Body,
    ));
    assert!(crate::bone_gesture::pose(
        &mut sim,
        livre,
        longe,
        ph2d_skeleton_render::BonePart::Body,
    ));
    assert!(
        rot(&sim, preso).abs() <= TENTACLE_LIMIT_HALF + 1e-6,
        "o osso limitado passou a parede: {}",
        rot(&sim, preso)
    );
    assert!(
        rot(&sim, livre).abs() > TENTACLE_LIMIT_HALF + 1e-6,
        "o vizinho devia girar LIVRE, e parou em {} — sem contraste o smoke não ensina nada",
        rot(&sim, livre)
    );
}
