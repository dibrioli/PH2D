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

/// ⭐⭐⭐ **UM LADO AUTORADO É UMA FUNÇÃO DOS DADOS, e não da pose de onde a corrente partiu**
/// (report do dono, 2026-09-14: *«IK Bend não está consistente para maior que 2. Muda o ângulo de
/// lado.»*).
///
/// ⛔⛔ **Medido antes da cura:** a mesma restrição (mesmo alvo, mesmo lado) resolvida a partir de
/// QUATRO poses de partida diferentes dava quatro poses finais diferentes — `0,52` de desvio numa
/// corrente de alcance `3` (**17 %**), `1,15` em `4` (**29 %**), `1,58` em `5` (**32 %**). A dois
/// ossos o desvio é **`0,0000`**, porque ali a lei é fechada e não olha para a pose.
///
/// ⇒ o `Ccw`/`Cw` acertava no LADO e o artista via o ÂNGULO mudar, porque cada quadro parte do
/// resultado do anterior e o FABRIK é sensível à pose inicial.
///
/// ⚠️ **A lei é só para o lado AUTORADO.** Com [`BendSide::Keep`] — o gesto de arrastar a ponta — a
/// dependência da pose é o DESENHO: *um gesto preserva o que se vê, uma restrição defende o que se
/// autorou*, e o `Keep` continua byte a byte o que era (há gate irmão a afirmá-lo).
#[test]
fn an_authored_side_is_a_function_of_the_data_not_of_where_the_chain_came_from() {
    for n in 2..=6usize {
        let lengths: Vec<f64> = (0..n).map(|_| 1.0).collect();
        let total: f64 = lengths.iter().sum();
        for alvo_frac in [0.4_f64, 0.6, 0.9] {
            let goal = [total * alvo_frac, 0.3];
            for lado in [BendSide::Ccw, BendSide::Cw] {
                let partidas: Vec<Vec<[f64; 2]>> = vec![
                    (0..=n).map(|i| [i as f64, 0.0]).collect(),
                    (0..=n).map(|i| [i as f64 * 0.8, i as f64 * 0.6]).collect(),
                    (0..=n)
                        .map(|i| [i as f64 * 0.8, -(i as f64) * 0.6])
                        .collect(),
                    (0..=n)
                        .map(|i| {
                            let a = (i as f64) * 0.5;
                            [a.sin() * i as f64, a.cos() * i as f64 - i as f64]
                        })
                        .collect(),
                ];
                let mut finais = Vec::new();
                for mut j in partidas {
                    reach(
                        &mut j,
                        &lengths,
                        goal,
                        Reach {
                            bend: lado,
                            ..Reach::default()
                        },
                    );
                    finais.push(j);
                }
                let base = finais[0].clone();
                for f in &finais {
                    let desvio = f
                        .iter()
                        .zip(base.iter())
                        .map(|(p, q)| (p[0] - q[0]).hypot(p[1] - q[1]))
                        .fold(0.0f64, f64::max);
                    assert!(
                        desvio < 1e-3 * total,
                        "com {n} ossos, alvo a {alvo_frac} e lado {lado:?}, duas poses de partida \
                         diferentes deram poses finais a {desvio} uma da outra (alcance {total}): o \
                         artista carrega no mesmo botao e ve^ um angulo diferente"
                    );
                }
            }
        }
    }
}

/// ⭐⭐⭐ **`Ccw` E `Cw` SÃO ESPELHOS EXACTOS UM DO OUTRO** sobre a recta `raiz → alvo`.
///
/// ⛔ **Esta lei nasceu de uma mutação SOBREVIVENTE:** o arco de partida multiplicava o bojo pelo
/// lado pedido, e apagar esse factor não reprovava nada — o [`mirror_to_side`] a seguir já corrige.
/// ⇒ *o lado tem UM dono*, e daí sai uma propriedade mais forte do que a que se estava a medir: com
/// o bojo sem sinal, as duas respostas são a mesma pose reflectida.
///
/// ⚠️ **Só com lado AUTORADO.** Com `Keep` a pose de partida manda, e duas partidas diferentes não
/// têm porque ser espelhos.
#[test]
fn the_two_authored_sides_are_exact_mirrors_of_each_other() {
    for n in 2..=6usize {
        let lengths: Vec<f64> = (0..n).map(|_| 1.0).collect();
        let total: f64 = lengths.iter().sum();
        for alvo_frac in [0.4_f64, 0.6, 0.9] {
            let goal = [total * alvo_frac, 0.3];
            let mut poses = Vec::new();
            for lado in [BendSide::Ccw, BendSide::Cw] {
                let mut j: Vec<[f64; 2]> = (0..=n).map(|i| [i as f64, 0.0]).collect();
                reach(
                    &mut j,
                    &lengths,
                    goal,
                    Reach {
                        bend: lado,
                        ..Reach::default()
                    },
                );
                poses.push(j);
            }
            let raiz = poses[0][0];
            let du = {
                let v = [goal[0] - raiz[0], goal[1] - raiz[1]];
                let m = v[0].hypot(v[1]);
                [v[0] / m, v[1] / m]
            };
            for (a, b) in poses[0].iter().zip(poses[1].iter()) {
                let v = [a[0] - raiz[0], a[1] - raiz[1]];
                let ao_longo = v[0] * du[0] + v[1] * du[1];
                let refl = [
                    raiz[0] + 2.0 * ao_longo * du[0] - v[0],
                    raiz[1] + 2.0 * ao_longo * du[1] - v[1],
                ];
                let erro = (refl[0] - b[0]).hypot(refl[1] - b[1]);
                assert!(
                    erro < 1e-3 * total,
                    "com {n} ossos e o alvo a {alvo_frac}, `Cw` nao e' o espelho de `Ccw`: {erro} de \
                     desvio (alcance {total})"
                );
            }
        }
    }
}
