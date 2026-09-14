//! ⭐⭐⭐ **O LADO DA DOBRA NUMA CORRENTE LONGA** — irmão do [`super::reach`] pelo tecto de 700 LOC
//! (`architecture_workspace_file_loc_cap`), cortado por RESPONSABILIDADE: ali mora a LEI do
//! alcance, aqui a pergunta que um report do dono fez sobre ela.

use crate::reach::*;

/// ⭐⭐⭐ **O LADO DA DOBRA VALE PARA QUALQUER COMPRIMENTO DE CORRENTE, e não só para DOIS.**
///
/// ⛔ **Report do dono** (2026-09-14): *«IK Bend só funciona se o IK Chain for 2. Não é possível
/// funcionar para mais de 2?»* — **medido, e a premissa é falsa**: o lado é honrado de `2` a `6`
/// ossos, em três distâncias de alvo, com a ponta a chegar. O caminho fechado (dois ossos) e o
/// FABRIK (três ou mais) aplicam-no por mecanismos diferentes — aquele escolhe o sinal da lei
/// dos cossenos, este arqueia e **espelha** a corrente antes de iterar —, e os dois respondem.
///
/// ⇒ o que o dono viu não estava aqui. O painel mostrava `Chain = 0` (nunca era semeado do
/// documento), logo o número que ele lia não era o que a restrição usava. *Um gate que defende
/// uma premissa refutada vale mais do que a refutação sozinha: ele impede que ela volte.*
///
/// ⛔ O CONTROLO é exigir que os dois lados dêem poses **diferentes**: um solver que ignorasse
/// o pedido passaria numa asserção que só olhasse o lado obtido de UM deles.
#[test]
fn the_bend_side_is_honoured_for_chains_longer_than_two() {
    for n in 2..=6usize {
        let lengths: Vec<f64> = (0..n).map(|_| 1.0).collect();
        let total: f64 = lengths.iter().sum();
        for alvo_frac in [0.4_f64, 0.7, 0.95] {
            let goal = [total * alvo_frac, 0.35];
            let mut poses = Vec::new();
            for lado in [BendSide::Ccw, BendSide::Cw] {
                let mut joints: Vec<[f64; 2]> = (0..=n).map(|i| [i as f64, 0.0]).collect();
                reach(
                    &mut joints,
                    &lengths,
                    goal,
                    Reach {
                        bend: lado,
                        ..Reach::default()
                    },
                );
                assert_eq!(
                    bend_side_of(&joints, goal, total),
                    lado,
                    "com {n} ossos e o alvo a {alvo_frac} do alcance, pedi {lado:?} e a corrente \
                     dobrou para o outro lado"
                );
                let erro = (joints[n][0] - goal[0]).hypot(joints[n][1] - goal[1]);
                assert!(
                    erro < 1e-2,
                    "defender o lado nao pode custar o alvo: erro {erro} com {n} ossos"
                );
                poses.push(joints);
            }
            assert!(
                poses[0]
                    .iter()
                    .zip(poses[1].iter())
                    .any(|(p, q)| (p[0] - q[0]).abs() > 1e-6 || (p[1] - q[1]).abs() > 1e-6),
                "controlo: os dois lados TE^M de dar poses diferentes ({n} ossos)"
            );
        }
    }
}
