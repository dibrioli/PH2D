//! Os gates de POSAR um osso — o que cada alça FAZ (girar · deslocar · força · IK).
//!
//! ⚠️ **Irmão do [`super::bone_gesture_tests`] pelo teto de 600 LOC, e o corte é por
//! RESPONSABILIDADE:** ali mede-se o que o dedo APONTA (acerto, realce, a decisão do press); aqui,
//! o que o gesto ESCREVE no documento. As duas perguntas falham de maneiras diferentes — uma acende
//! a alça errada, a outra move a coisa errada.

use super::*;
// ⚠️ As duas ajudas de fixtura vivem no irmão que faz o hit-test — é ele que sabe CRIAR um osso e
// dizer onde ele está. O corte de 2026-09-07 separou *o que o dedo aponta* de *o que a mão faz*, e
// a fixtura ficou do lado de quem a produz.
use crate::bone_gesture::{create, test_chain, test_segment};
use ph2d_skeleton_render::BonePart;

/// ⭐⭐⭐ **AGARRAR O CORPO GIRA; AGARRAR A JUNTA DESLOCA.** As duas metades, porque uma sozinha
/// deixa metade do rig inalcançável — sem a rotação não se posa, sem o deslocamento o esqueleto
/// nunca sai de onde nasceu.
#[test]
fn grabbing_the_body_turns_the_bone_and_grabbing_the_joint_moves_it() {
    let mut sim = SimWorld::default();
    let osso = Entity::from_bits(create(&mut sim, None, [0.0, 0.0], [10.0, 0.0]).expect("osso"));
    // Apontar para cima: a ponta sobe, a ORIGEM fica.
    assert!(pose(&mut sim, osso, [0.0, 7.0], BonePart::Body));
    let (a, b) = test_segment(&sim, osso.to_bits());
    assert!(
        a[0].abs() < 1e-5 && a[1].abs() < 1e-5,
        "a origem andou: {a:?}"
    );
    assert!(
        b[0].abs() < 1e-4 && (b[1] - 10.0).abs() < 1e-4,
        "a ponta devia ir para (0,10) e foi para {b:?}"
    );
    // Pela junta: a origem vai para o ponteiro e o osso leva a direcção consigo.
    assert!(pose(&mut sim, osso, [4.0, 4.0], BonePart::Joint));
    let (a2, b2) = test_segment(&sim, osso.to_bits());
    assert!(
        (a2[0] - 4.0).abs() < 1e-5 && (a2[1] - 4.0).abs() < 1e-5,
        "a junta nao foi para o ponteiro: {a2:?}"
    );
    assert!(
        (b2[1] - 14.0).abs() < 1e-4,
        "deslocar mudou a DIRECCAO do osso: {b2:?}"
    );
}

/// ⛔ **Apontar para a PRÓPRIA origem não move nada.** Ali não há direcção, e um `atan2(0,0)` daria
/// um ângulo arbitrário — o osso saltaria no instante em que o ponteiro cruzasse a junta.
#[test]
fn aiming_at_the_bones_own_origin_does_nothing() {
    let mut sim = SimWorld::default();
    let osso = Entity::from_bits(create(&mut sim, None, [3.0, 1.0], [9.0, 1.0]).expect("osso"));
    let antes = test_segment(&sim, osso.to_bits());
    assert!(!pose(&mut sim, osso, [3.0, 1.0], BonePart::Body));
    assert_eq!(antes, test_segment(&sim, osso.to_bits()));
}

/// ⭐⭐⭐ **A MANCHA MOSTRA EXACTAMENTE A REGIÃO QUE O PESO USA** — a costura que, partida, faz o
/// desenho MENTIR ao artista.
///
/// A lei da pele (`ph2d_skeleton::SkinBone::new`) faz `raio = |eixo| × força`, e o overlay faz o
/// mesmo com o eixo de MUNDO. ⚠️ São **duas contas** em duas crates, e a única coisa que as mantém
/// juntas é este gate: divergindo, a mancha passa a cobrir uma área e a deformação a obedecer
/// outra — o defeito mais caro possível numa ferramenta cuja razão de existir é *ver* o alcance.
#[test]
fn the_influence_blob_covers_exactly_the_region_the_weight_law_uses() {
    let mut sim = SimWorld::default();
    let osso = create(&mut sim, None, [0.0, 0.0], [40.0, 0.0]).expect("osso");
    for forca in [0.25, 1.0, 2.5] {
        {
            let mut b = sim
                .world_mut()
                .get_mut::<ph2d_skeleton_ecs::Bone>(Entity::from_bits(osso))
                .expect("Bone");
            b.strength = forca;
        }
        let desenhado = crate::skeleton_live::influence_radius(&sim, osso).expect("raio");
        // A MESMA pergunta, feita à lei da pele: um osso em repouso sobre uma forma na identidade.
        let lei = ph2d_skeleton::SkinBone::new(
            ph2d_skeleton::Xform::IDENTITY,
            40.0,
            forca,
            ph2d_skeleton::Xform::IDENTITY,
            ph2d_skeleton::Xform::IDENTITY,
        )
        .expect("o osso da pele")
        .radius;
        assert!(
            (desenhado - lei).abs() < 1e-9,
            "a forca {forca} desenha {desenhado} e pesa {lei} - a mancha esta' a mentir"
        );
    }
}

/// ⭐⭐ **ARRASTAR A ALÇA MUDA A FORÇA, e a grandeza é a distância ao SEGMENTO** — a mesma que a lei
/// do peso mede (`dist2_to_segment`). ⇒ o artista arrasta literalmente a borda que a mistura usa.
#[test]
fn dragging_the_influence_handle_sets_the_strength_to_what_the_pointer_reaches() {
    let mut sim = SimWorld::default();
    let osso = Entity::from_bits(create(&mut sim, None, [0.0, 0.0], [20.0, 0.0]).expect("osso"));
    // A 30 de distância perpendicular, num osso de 20 ⇒ força 1,5.
    assert!(pose(&mut sim, osso, [10.0, 30.0], BonePart::Influence));
    let f = sim
        .world()
        .get::<ph2d_skeleton_ecs::Bone>(osso)
        .expect("Bone")
        .strength;
    assert!((f - 1.5).abs() < 1e-9, "a forca saiu {f} e devia ser 1,5");
    // ⛔ E ela nunca fica negativa — mas o ZERO é legal: significa *"só alcança pelo desempate do
    // órfão"*, e um piso acima de zero tiraria esse estado do artista.
    assert!(pose(&mut sim, osso, [10.0, 0.0], BonePart::Influence));
    assert_eq!(
        sim.world()
            .get::<ph2d_skeleton_ecs::Bone>(osso)
            .expect("Bone")
            .strength,
        0.0
    );
    // ⚠️ E arrastar a alça NÃO move o osso: a força não é uma pose.
    assert_eq!(
        test_segment(&sim, osso.to_bits()),
        ([0.0, 0.0], [20.0, 0.0])
    );
}

/// ⭐⭐⭐ **ARRASTAR A PONTA DOBRA A CORRENTE** — a cinemática inversa, pelo gesto.
///
/// É o degrau que separa *"um editor de esqueletos"* de *"um editor de animação"*: o artista sabe
/// onde a MÃO tem de estar, e os ângulos são exactamente o que ele não quer digitar.
#[test]
fn dragging_the_tip_bends_the_whole_chain_until_it_reaches() {
    for n in [2usize, 3, 6] {
        let mut sim = SimWorld::default();
        let ossos = test_chain(&mut sim, n);
        let ponta = Entity::from_bits(*ossos.last().expect("ponta"));
        let alcance = f64::from(u16::try_from(n).unwrap_or(1)) * 10.0;
        let alvo = [alcance * 0.4, alcance * 0.35];
        assert!(pose(&mut sim, ponta, alvo, BonePart::Tip));
        let (_, chegou) = test_segment(&sim, ponta.to_bits());
        let erro = (chegou[0] - alvo[0]).hypot(chegou[1] - alvo[1]);
        assert!(
            erro < 1e-2 * alcance,
            "com {n} ossos a ponta parou a {erro} do alvo {alvo:?}"
        );
    }
}

/// ⛔⛔ **A IK NUNCA ESTICA UM OSSO, E A RAIZ NÃO SAI DO SÍTIO** — os dois invariantes que fazem o
/// resultado ler-se como um membro e não como um elástico.
///
/// ⚠️ Eles saem de graça da escolha de escrever a pose pela porta da ROTAÇÃO: uma rotação não muda
/// comprimento nenhum, e o osso roda em torno da própria origem. Este gate existe para que essa
/// escolha não seja desfeita por um atalho que escreva posições.
#[test]
fn inverse_kinematics_never_stretches_a_bone_nor_unpins_the_root() {
    let mut sim = SimWorld::default();
    let ossos = test_chain(&mut sim, 4);
    let ponta = Entity::from_bits(*ossos.last().expect("ponta"));
    let raiz_antes = test_segment(&sim, ossos[0]).0;
    for alvo in [[10.0, 10.0], [-20.0, 5.0], [1e4, 1e4], [0.0, 0.0]] {
        pose(&mut sim, ponta, alvo, BonePart::Tip);
        for (i, &b) in ossos.iter().enumerate() {
            let (a, t) = test_segment(&sim, b);
            let comp = (t[0] - a[0]).hypot(t[1] - a[1]);
            assert!(
                (comp - 10.0).abs() < 1e-4,
                "com alvo {alvo:?} o osso {i} ficou com {comp} em vez de 10"
            );
        }
        let raiz = test_segment(&sim, ossos[0]).0;
        assert!(
            (raiz[0] - raiz_antes[0]).abs() < 1e-6 && (raiz[1] - raiz_antes[1]).abs() < 1e-6,
            "com alvo {alvo:?} a RAIZ da corrente andou para {raiz:?}"
        );
    }
}
