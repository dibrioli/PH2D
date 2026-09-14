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

/// ⭐⭐⭐ **O MODO MISTO: cada junta fica do lado em que o artista a deixou, E a ponta chega** —
/// o gate da ordem do dono (2026-09-14: *«além de CCW e CW precisamos de um modo misto onde temos
/// ossos com ângulos para os dois lados, e cada osso mantém sua direção inicial»*).
///
/// O corpus é uma corrente em **ZIG-ZAG** — cada junta dobrada para o lado oposto da anterior —,
/// que é a pose que nenhum dos outros três modos sabe exprimir: o `Ccw` e o `Cw` põem TODAS as
/// juntas do mesmo lado, e o `Keep` não defende nenhuma.
///
/// ⭐⭐ **A metade ANTI-VÁCUO é a segunda:** a mesma corrente resolvida com [`BendSide::Keep`]
/// **perde** pelo menos um sinal em `7` dos `12` alvos — sempre o da junta junto à ponta, que é a
/// que mais se move para chegar ao alvo. Sem essa metade, um `Mixed` que não fizesse nada passaria
/// nos alvos em que o `Keep` já acerta por acaso.
///
/// ⛔ **A tolerância da ponta é a do solver** (`TOLERANCE · alcance`), não um número escolhido: a
/// promessa deste modo é chegar ao alvo *e* guardar os lados, e o gate exige as duas ao mesmo tempo.
#[test]
fn a_mixed_chain_keeps_every_joint_on_its_own_side() {
    let mut keep_perdeu = 0usize;
    let mut casos = 0usize;
    for n in [3usize, 4, 5, 6] {
        let lengths: Vec<f64> = (0..n).map(|_| 1.0).collect();
        let total: f64 = lengths.iter().sum();
        // Cada osso aponta para a frente e alterna em `y`: as juntas alternam de lado.
        let mut j: Vec<[f64; 2]> = vec![[0.0, 0.0]];
        for i in 0..n {
            let s = if i % 2 == 0 { 1.0 } else { -1.0 };
            let ultimo = *j.last().unwrap();
            j.push([ultimo[0] + 0.85, ultimo[1] + 0.5 * s]);
        }
        let antes = sinais_crus(&j);
        assert!(
            antes.iter().any(|&s| s > 0.0) && antes.iter().any(|&s| s < 0.0),
            "a fixtura de {n} ossos nao e' um zig-zag: {antes:?}"
        );
        for alvo in [[total * 0.5, 0.2], [total * 0.7, -0.4], [total * 0.35, 0.6]] {
            casos += 1;
            let mut k = j.clone();
            reach(
                &mut k,
                &lengths,
                alvo,
                Reach {
                    bend: BendSide::Mixed,
                    ..Reach::default()
                },
            );
            assert_eq!(
                sinais_crus(&k),
                antes,
                "com {n} ossos e o alvo {alvo:?}, o MISTO trocou o lado de uma junta"
            );
            let erro = (k[n][0] - alvo[0]).hypot(k[n][1] - alvo[1]);
            assert!(
                erro < 1e-3 * total,
                "com {n} ossos e o alvo {alvo:?}, o MISTO guardou os lados e a ponta ficou a \
                 {erro} do alvo (alcance {total})"
            );

            // ANTI-VÁCUO: a corrente LIVRE resolve os mesmos alvos e perde sinais.
            let mut livre = j.clone();
            reach(&mut livre, &lengths, alvo, Reach::default());
            if sinais_crus(&livre) != antes {
                keep_perdeu += 1;
            }
        }
    }
    assert_eq!(casos, 12, "o corpus mudou de tamanho");
    assert!(
        keep_perdeu >= 5,
        "o corpus deixou de discriminar: a corrente LIVRE guardou os sinais em {} dos {casos} \
         alvos, logo este gate passaria sem o modo MISTO fazer nada",
        casos - keep_perdeu
    );
}

/// ⭐ **Resolver duas vezes devolve a MESMA pose** — o modo misto lê o lado da pose que chega, logo
/// a pose que ele devolve tem de ser um ponto fixo dele. ⚠️ Sem isto, um rig parado derivaria de
/// quadro para quadro, que é o defeito que a W16 curou no lado autorado.
#[test]
fn a_mixed_chain_is_a_fixed_point() {
    let lengths = [1.0, 1.0, 1.0, 1.0, 1.0];
    let mut j: Vec<[f64; 2]> = vec![[0.0, 0.0]];
    for i in 0..5 {
        let s = if i % 2 == 0 { 1.0 } else { -1.0 };
        let u = *j.last().unwrap();
        j.push([u[0] + 0.85, u[1] + 0.5 * s]);
    }
    let alvo = [2.6, 0.3];
    let opts = Reach {
        bend: BendSide::Mixed,
        ..Reach::default()
    };
    reach(&mut j, &lengths, alvo, opts);
    let uma = j.clone();
    reach(&mut j, &lengths, alvo, opts);
    for (i, (a, b)) in uma.iter().zip(&j).enumerate() {
        let d = (a[0] - b[0]).hypot(a[1] - b[1]);
        assert!(
            d < 1e-9,
            "a junta {i} andou {d} numa segunda resolucao igual"
        );
    }
}

/// ⭐⭐⭐ **UMA JUNTA EMPURRADA CONTRA A RECTA PÁRA ANTES DELA, NUNCA NELA.**
///
/// ⛔⛔ **Parar NA recta perderia a feature em silêncio no quadro SEGUINTE**: o lado de cada junta é
/// re-lido da pose a cada resolução, e uma junta exactamente recta lê-se «sem lado» — ela deixaria
/// de ser defendida para sempre, e o artista veria uma junta trocar de lado sozinha um quadro
/// depois de nada ter feito.
///
/// ⚠️ **A fixtura foi ACHADA, não escolhida:** uma varredura de `1 176` combinações (contagem de
/// ossos × dobra autorada × padrão de lados × alvo, só alvos ao alcance e só juntas que nasceram
/// com lado) procurou o menor seno final de todas — e ele é **exactamente** `0,008`, a margem, nesta
/// célula. É o único sítio do corpus onde a parede da recta é o que decide.
#[test]
fn a_joint_pushed_against_the_straight_stops_short_of_it() {
    // (ossos, dobra autorada, alvo). A 1.ª é onde a parede DECIDE no produto; a 2.ª é onde uma
    // parede SEM margem deixa a junta exactamente recta — as duas saíram da mesma varredura, uma
    // corrida sobre o produto e outra sobre o mutante.
    for (ossos, dy, alvo) in [(6usize, 0.1, [1.2, 0.9]), (4, 0.01, [2.8, -0.9])] {
        let lengths = vec![1.0; ossos];
        let mut j: Vec<[f64; 2]> = vec![[0.0, 0.0]];
        for i in 0..ossos {
            let s = if i % 2 == 0 { 1.0 } else { -1.0 };
            let u = *j.last().unwrap();
            j.push([u[0] + 0.9, u[1] + dy * s]);
        }
        let antes = sinais_crus(&j);
        let mut k = j.clone();
        reach(
            &mut k,
            &lengths,
            alvo,
            Reach {
                bend: BendSide::Mixed,
                ..Reach::default()
            },
        );
        assert_eq!(
            sinais_crus(&k),
            antes,
            "o alvo {alvo:?} empurrou uma junta para cima da recta (ou para lá dela)"
        );
        // E a folga é MENSURÁVEL, não um resto de arredondamento. ⚠️ A barra é a margem que o produto
        // promete, LIDA dele — repeti-la aqui deixaria o gate a concordar com uma constante que já não
        // existe.
        let margem = crate::reach_mixed::MARGEM.sin();
        let mut encostou = false;
        for i in 1..k.len() - 1 {
            let v1 = [k[i][0] - k[i - 1][0], k[i][1] - k[i - 1][1]];
            let v2 = [k[i + 1][0] - k[i][0], k[i + 1][1] - k[i][1]];
            let seno =
                (v1[0] * v2[1] - v1[1] * v2[0]).abs() / (v1[0].hypot(v1[1]) * v2[0].hypot(v2[1]));
            assert!(
                seno >= margem * 0.999,
                "a junta {i} ficou a {seno} de seno da recta — abaixo da margem {margem}, logo o \
                 proximo quadro le-a como SEM lado"
            );
            if seno < margem * 1.001 {
                encostou = true;
            }
        }
        // ANTI-VÁCUO: sem nenhuma junta a encostar na parede, este gate não mede parede nenhuma.
        // ⚠️ Só a 1.ª célula o promete — na 2.ª a parede decide mais cedo e a junta acaba folgada.
        assert!(
            encostou || ossos == 4,
            "nenhuma junta encostou na parede da recta com {ossos} ossos: a fixtura deixou de \
             exercitar a lei que este gate afirma"
        );
    }
}

/// O sinal CRU de cada junta interior — sem o limiar de recta do produto, de propósito: um gate que
/// partilhasse a régua do produto não veria uma junta empurrada para cima da fronteira.
fn sinais_crus(p: &[[f64; 2]]) -> Vec<f64> {
    (1..p.len() - 1)
        .map(|i| {
            let v1 = [p[i][0] - p[i - 1][0], p[i][1] - p[i - 1][1]];
            let v2 = [p[i + 1][0] - p[i][0], p[i + 1][1] - p[i][1]];
            let cruz = v1[0] * v2[1] - v1[1] * v2[0];
            // ⚠️ `f64::signum` devolve `±1` para o ZERO — uma junta exactamente recta lia-se como
            // tendo lado, e um gate construído sobre isso não vê a junta que ficou sem nenhum.
            if cruz == 0.0 { 0.0 } else { cruz.signum() }
        })
        .collect()
}

/// ⭐⭐ **O INSTRUMENTO que achou as duas fixturas do gate da parede** — varre `1 176` células
/// (contagem de ossos × dobra autorada × padrão de lados × alvo) e imprime o **menor seno final**
/// de todas as juntas que nasceram com lado, com o alvo em alcance.
///
/// ⚠️ Corra-o **também sobre o mutante** que quer matar: uma fixtura calibrada no caminho do
/// produto não discrimina, porque o mutante anda por outro caminho — foi exactamente assim que a
/// 1.ª redacção do gate da parede passou com a margem da recta apagada. No produto ele lê
/// `0,008000` (a margem, encostada); sem a margem lê `0,000000` noutra célula, e é essa a que o
/// gate carrega.
#[test]
#[ignore]
fn varre_fixturas() {
    let mut pior = f64::INFINITY;
    let mut onde = String::new();
    for n in [3usize, 4, 5, 6] {
        let lengths: Vec<f64> = (0..n).map(|_| 1.0).collect();
        for dy in [0.01f64, 0.03, 0.1, 0.3, 0.5, 0.7, 0.9] {
            for padrao in 0..4 {
                let mut j: Vec<[f64; 2]> = vec![[0.0, 0.0]];
                for i in 0..n {
                    let s = match padrao {
                        0 => {
                            if i % 2 == 0 {
                                1.0
                            } else {
                                -1.0
                            }
                        }
                        1 => {
                            if i < n / 2 {
                                1.0
                            } else {
                                -1.0
                            }
                        }
                        2 => {
                            if i % 3 == 0 {
                                1.0
                            } else {
                                -1.0
                            }
                        }
                        _ => 1.0,
                    };
                    let u = *j.last().unwrap();
                    j.push([u[0] + 0.9, u[1] + dy * s]);
                }
                for gx in [0.2f64, 0.3, 0.5, 0.7, 0.9, 0.99] {
                    for gy in [-0.9f64, -0.5, -0.2, 0.0, 0.2, 0.5, 0.9] {
                        let mut k = j.clone();
                        let alvo = [n as f64 * gx, gy];
                        if alvo[0].hypot(alvo[1]) > n as f64 * 0.98 {
                            continue; // fora de alcance: a recta e' a resposta, sem lado nenhum
                        }
                        reach(
                            &mut k,
                            &lengths,
                            alvo,
                            Reach {
                                bend: BendSide::Mixed,
                                ..Reach::default()
                            },
                        );
                        let erro = (k[n][0] - alvo[0]).hypot(k[n][1] - alvo[1]);
                        let sinais_ok = sinais_crus(&k) == sinais_crus(&j);
                        println!(
                            "F n={n} dy={dy} p={padrao} g={gx},{gy} erro={erro:.5} ok={sinais_ok}"
                        );
                        for i in 1..k.len() - 1 {
                            let a1 = [j[i][0] - j[i - 1][0], j[i][1] - j[i - 1][1]];
                            let a2 = [j[i + 1][0] - j[i][0], j[i + 1][1] - j[i][1]];
                            let autorado = (a1[0] * a2[1] - a1[1] * a2[0]).abs()
                                / (a1[0].hypot(a1[1]) * a2[0].hypot(a2[1]));
                            if autorado <= 1e-3 {
                                continue; // nasceu recta: nao tem lado para defender
                            }
                            let v1 = [k[i][0] - k[i - 1][0], k[i][1] - k[i - 1][1]];
                            let v2 = [k[i + 1][0] - k[i][0], k[i + 1][1] - k[i][1]];
                            let sen = (v1[0] * v2[1] - v1[1] * v2[0]).abs()
                                / (v1[0].hypot(v1[1]) * v2[0].hypot(v2[1]));
                            if sen < pior {
                                pior = sen;
                                onde = format!("n={n} dy={dy} p={padrao} alvo={alvo:?} junta={i}");
                            }
                        }
                    }
                }
            }
        }
    }
    println!(
        "MENOR seno final = {pior:.6}  (margem {:.6})  em {onde}",
        crate::reach_mixed::MARGEM.sin()
    );
}
