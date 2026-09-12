//! Os gates do **LADO DA DOBRA** — a terceira fatia do [`super`], pelo teto de 600 LOC do HR-18 e
//! pelo mesmo corte por RESPONSABILIDADE das outras duas (a agenda, o que o dedo aponta).
//!
//! Aqui mede-se uma coisa só: *para que lado o joelho aponta, e quem decide isso*. A LEI está
//! gateada em `ph2d-skeleton` (incluindo a prova de que a fixtura PRODUZ a inversão); o que só
//! existe com um mundo ECS é a **captura no nascimento** e o defeito de ponta a ponta.

use super::*;
use crate::goal::braco;
// ⚠️ Declarado aqui: o pai deixou de nomear o `Bone` quando a lei foi para a folha.
use ph2d_skeleton_ecs::Bone;

/// O mesmo braço, mas **dobrado** por `theta` radianos no cotovelo.
///
/// ⚠️ O [`braco`] nasce **recto**, e uma corrente recta **não tem lado** — ela não produz o
/// fenómeno que os dois gates de baixo medem. É a mesma armadilha que a fila deste módulo já
/// registou quatro vezes, e é por isso que esta fixtura existe em vez de um `theta` emprestado.
fn braco_dobrado(theta: f32) -> (SimWorld, [Entity; 2]) {
    let (mut sim, ossos) = braco();
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(ossos[1]) {
        t.rotation = theta;
    }
    (sim, ossos)
}

/// ⭐⭐⭐ **A ÂNCORA NASCE A DEFENDER A DOBRA QUE O ARTISTA JÁ POSOU.**
///
/// ⚠️ **O lado é CAPTURADO, não escolhido** — e é isso que faz criar a âncora continuar a ser um
/// no-op visual **e** curar o defeito ao mesmo tempo. Se ela nascesse em `Keep`, a primeira vez que
/// o dono esticasse o membro o joelho inverteria sozinho.
///
/// ⛔ E o lado esperado sai da **própria fixtura** (dobras opostas ⇒ capturas opostas), nunca de um
/// nome que eu escolhi: foi assim que a 1.ª redacção do `the_elbow_keeps_the_side_it_is_already_
/// bent_to` ficou verde a pinar a inversão que dizia proibir.
#[test]
fn the_bend_side_is_captured_when_the_anchor_is_born() {
    let mut capturados = Vec::new();
    for theta in [0.5_f32, -0.5] {
        let (mut sim, [_, cotovelo]) = braco_dobrado(theta);
        add(&mut sim, cotovelo).expect("a âncora nasce");
        let g = *sim
            .world()
            .get::<IkGoal>(cotovelo)
            .expect("a âncora foi escrita no osso da ponta");
        assert_ne!(
            g.bend,
            ph2d_skeleton::BendSide::Keep,
            "uma corrente DOBRADA (theta={theta}) tem lado, e ele tinha de ser capturado"
        );
        capturados.push(g.bend);
    }
    assert_eq!(
        capturados[0],
        capturados[1].flipped(),
        "dobras opostas capturaram o MESMO lado ({capturados:?}) — a captura não está a ler a pose"
    );
}

/// ⭐ **Uma corrente RECTA não tem lado, e a âncora não inventa um.**
///
/// ⛔ Escolher um lado ali seria fabricar uma decisão do artista a partir de ruído de `f32`: perto
/// da extensão máxima um erro de posição de `1e-6` vira `1e-3` de ângulo, e o sinal dele é sorteio.
#[test]
fn a_straight_chain_gives_the_anchor_no_side_to_defend() {
    let (mut sim, [_, cotovelo]) = braco();
    add(&mut sim, cotovelo).expect("a âncora nasce");
    assert_eq!(
        sim.world().get::<IkGoal>(cotovelo).expect("existe").bend,
        ph2d_skeleton::BendSide::Keep
    );
}

/// ⭐⭐⭐ **O DEFEITO DO DONO, DE PONTA A PONTA NO MUNDO** — e é o gate que fecha a wave.
///
/// O artista dobra o cotovelo, **estica o membro** arrastando a âncora para longe (a corrente fica
/// recta, e uma recta não tem lado) e traz a mão de volta ao mesmo sítio. Com o lado capturado ela
/// volta **ao mesmo lado**; sem ele, ao contrário.
///
/// ⚠️ A régua é a da LEI ([`ph2d_skeleton::dominant_side`]), e não um produto vectorial escrito
/// aqui: os dois medem o mesmo facto com **sinais opostos**, e escrever o segundo foi o que fez a
/// 1.ª redacção do gate irmão acusar a implementação certa.
#[test]
fn the_anchor_defends_its_side_across_the_straight_pose() {
    for theta in [0.5_f32, -0.5] {
        let (mut sim, [ombro, cotovelo]) = braco_dobrado(theta);
        add(&mut sim, cotovelo).expect("a âncora nasce");
        let poiso = ponta(&sim, cotovelo);

        let lado_agora = |sim: &SimWorld| {
            let juntas = [
                ph2d_skeleton_live::skin_live::bone_segments(sim)
                    .iter()
                    .find(|(x, _, _)| *x == ombro.to_bits())
                    .map(|(_, a, _)| *a)
                    .expect("o ombro tem segmento"),
                ponta(sim, ombro),
                ponta(sim, cotovelo),
            ];
            ph2d_skeleton::dominant_side(&juntas, juntas[2])
        };

        let antes = lado_agora(&sim);
        assert!(
            antes.abs() > 0.0,
            "a fixtura tem de ter lado (theta={theta}), senão este gate é vácuo"
        );
        // Estica: o alvo vai muito para além do alcance ⇒ a corrente deita-se na recta.
        quadro(&mut sim, cotovelo, [400.0, 0.0]);
        // E volta exactamente para onde estava.
        quadro(&mut sim, cotovelo, poiso);
        let depois = lado_agora(&sim);
        assert!(
            antes.signum() == depois.signum(),
            "o joelho INVERTEU ao passar pela recta (theta={theta}): {antes} -> {depois}"
        );
    }
}

/// ⭐⭐⭐ **A CENA DE SMOKE DÁ À ÂNCORA UM LADO PARA DEFENDER** — e sem este gate ela não daria.
///
/// ⛔⛔ **O `CLAUDE.md` §5.0 nomeia este modo de falha e ele quase aconteceu aqui:** o braço da cena
/// nascia **RECTO**, e uma corrente recta **não tem lado**. O `add` capturaria `Auto` — que é
/// precisamente o modo em que o joelho inverte ao passar pela posição esticada — e o dono faria o
/// smoke da cura vendo o defeito, com tudo verde deste lado.
///
/// ⚠️ Ele mede a MESMA geometria que a cena monta (a tabela `ARM_*`), nunca uma cópia dos números:
/// uma sonda que alimenta outros números mede outro programa.
#[test]
fn the_smoke_scene_gives_the_anchor_a_side_to_defend() {
    use ph2d_app_vec::smoke_bone::{ARM_A, ARM_B, ARM_BONES, ARM_ELBOW_BEND};
    let mut sim = SimWorld::default();
    let raiz = ph2d_app_vec::smoke_bone::cadeia(&mut sim, ARM_A, ARM_B, ARM_BONES)
        .expect("a cadeia do braço monta-se");
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    // A cena dobra o ÚLTIMO osso — é ele que está dentro da corrente de `DEFAULT_CHAIN`.
    let mut ponta = raiz;
    while let Some(f) = sim.world().get::<ph2d_ecs::Children>(ponta).and_then(|c| {
        c.iter()
            .find(|c| sim.world().get::<Bone>(**c).is_some())
            .copied()
    }) {
        ponta = f;
    }
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(ponta) {
        t.rotation = ARM_ELBOW_BEND;
    }
    add(&mut sim, ponta).expect("a âncora nasce na ponta do braço");
    let g = *sim.world().get::<IkGoal>(ponta).expect("a âncora existe");
    assert_ne!(
        g.bend,
        ph2d_skeleton::BendSide::Keep,
        "a cena de smoke monta o braço sem dobra: o dono arrastaria o losango e veria o joelho \
         INVERTER, que é o defeito que esta wave cura. Ponha `ARM_ELBOW_BEND` fora do zero."
    );
}
