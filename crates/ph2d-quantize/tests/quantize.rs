//! **OS GATES DA QUANTIZAÇÃO** (ADR-0161, F4).
//!
//! ⭐ **A régua principal é FORÇA BRUTA.** Para layouts pequenos, enumerar todas
//! as quantizações e ficar com a mais barata dá o ótimo inteiro **sem** partilhar
//! uma linha de código com o solver. É a única forma de uma afirmação de
//! otimalidade não ser o solver a avaliar-se a si próprio.

use ph2d_quantize::{
    ArcSpec, Budget, CornerError, Layout, PatchSpec, quantize, quantize_within, solve_corners,
    verify,
};

/// Um gerador determinístico — nada de `rand` num gate.
struct Lcg(u64);

impl Lcg {
    fn next_f64(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        ((self.0 >> 11) as f64) / ((1u64 << 53) as f64)
    }
}

/// Um layout fechado a partir de faces triangulares: cada aresta vira um arco.
fn from_faces(faces: &[[u32; 3]], targets: &[f64]) -> Layout {
    let mut ids: Vec<(u32, u32)> = Vec::new();
    let mut patches = Vec::new();
    for f in faces {
        let mut sides = Vec::new();
        for k in 0..3 {
            let (a, b) = (f[k], f[(k + 1) % 3]);
            let key = (a.min(b), a.max(b));
            let id = match ids.iter().position(|x| *x == key) {
                Some(i) => i,
                None => {
                    ids.push(key);
                    ids.len() - 1
                }
            };
            sides.push(vec![u32::try_from(id).unwrap()]);
        }
        patches.push(PatchSpec { sides });
    }
    let arcs = (0..ids.len())
        .map(|i| ArcSpec::new(targets[i % targets.len()]))
        .collect();
    Layout::new(arcs, patches).expect("o layout fechado e' valido")
}

/// O tetraedro: 4 patches triangulares, 6 arcos. Pequeno o bastante para força
/// bruta e grande o bastante para ter estrutura bi-dirigida a sério.
fn tetrahedron(targets: &[f64]) -> Layout {
    from_faces(&[[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]], targets)
}

/// O octaedro: **é o layout que o oráculo produz para uma esfera** — 8 patches
/// triangulares e 12 arcos (medido em 2026-08-20 sobre `sphere_uv_96x144`).
fn octahedron(targets: &[f64]) -> Layout {
    from_faces(
        &[
            [0, 2, 4],
            [2, 1, 4],
            [1, 3, 4],
            [3, 0, 4],
            [2, 0, 5],
            [1, 2, 5],
            [3, 1, 5],
            [0, 3, 5],
        ],
        targets,
    )
}

/// O cubo: 6 patches de **quatro** lados — o caso em que a lei geral tem de
/// reproduzir, sozinha, o clássico *"lados opostos iguais"*.
fn cube_layout(targets: &[f64]) -> Layout {
    let faces: [[u32; 4]; 6] = [
        [0, 3, 2, 1],
        [4, 5, 6, 7],
        [0, 1, 5, 4],
        [1, 2, 6, 5],
        [2, 3, 7, 6],
        [3, 0, 4, 7],
    ];
    let mut ids: Vec<(u32, u32)> = Vec::new();
    let mut patches = Vec::new();
    for f in faces {
        let mut sides = Vec::new();
        for k in 0..4 {
            let (a, b) = (f[k], f[(k + 1) % 4]);
            let key = (a.min(b), a.max(b));
            let id = match ids.iter().position(|x| *x == key) {
                Some(i) => i,
                None => {
                    ids.push(key);
                    ids.len() - 1
                }
            };
            sides.push(vec![u32::try_from(id).unwrap()]);
        }
        patches.push(PatchSpec { sides });
    }
    let arcs = (0..ids.len())
        .map(|i| ArcSpec::new(targets[i % targets.len()]))
        .collect();
    Layout::new(arcs, patches).expect("o cubo e' valido")
}

/// **O layout DIFÍCIL**: 4 patches, valências 3 e 4, e **duas junções em T** (um
/// lado partido em dois arcos porque o vizinho tem um canto no meio dele).
///
/// ⚠️ **É este que separa uma busca completa de uma busca que parece completa.**
/// O octaedro é todo de valência 3 com um arco por lado — folgado demais para
/// distinguir as duas. Aqui as restrições apertam.
fn t_junction_layout(targets: &[f64]) -> Layout {
    let arcs = targets.iter().map(|t| ArcSpec::new(*t)).collect();
    Layout::new(
        arcs,
        vec![
            // A: quatro lados, o primeiro partido em dois arcos.
            PatchSpec {
                sides: vec![vec![0, 1], vec![2], vec![3], vec![4]],
            },
            // B: vê os arcos 0 e 1 como lados separados — a junção em T.
            PatchSpec {
                sides: vec![vec![0], vec![5], vec![6]],
            },
            PatchSpec {
                sides: vec![vec![1], vec![2], vec![7]],
            },
            PatchSpec {
                sides: vec![vec![3], vec![4], vec![5], vec![6], vec![7]],
            },
        ],
    )
    .expect("o layout com juncao em T e' fechado")
}

/// **O PRISMA de `n` lados**: duas tampas de valência `n` e `n` patches de 4
/// lados a ligá-las. `n+2` patches, `3n` arcos.
///
/// ⚠️ **É o layout apertado.** Os `n` quads forçam `t_i = b_i` **e** todos os
/// verticais iguais entre si; as duas tampas de valência `n` amarram o resto. É
/// grande demais para força bruta e restrito o bastante para separar uma busca
/// completa de uma que só parece.
fn prism(n: usize, targets: &[f64]) -> Layout {
    let t = |i: usize| u32::try_from(i % n).unwrap();
    let b = |i: usize| u32::try_from(n + i % n).unwrap();
    let v = |i: usize| u32::try_from(2 * n + i % n).unwrap();
    let mut patches = vec![
        PatchSpec {
            sides: (0..n).map(|i| vec![t(i)]).collect(),
        },
        PatchSpec {
            sides: (0..n).map(|i| vec![b(i)]).collect(),
        },
    ];
    for i in 0..n {
        patches.push(PatchSpec {
            sides: vec![vec![t(i)], vec![v(i)], vec![b(i)], vec![v(i + n - 1)]],
        });
    }
    let arcs = (0..3 * n)
        .map(|i| ArcSpec::new(targets[i % targets.len()]))
        .collect();
    Layout::new(arcs, patches).expect("o prisma e' fechado")
}

/// Enumera TODAS as quantizações até `max` e devolve o custo mínimo — o oráculo.
fn brute_force(layout: &Layout, max: u32) -> Option<f64> {
    let n = layout.arcs().len();
    let mut x = vec![1u32; n];
    let mut best: Option<f64> = None;
    loop {
        if verify(layout, &x).is_ok() {
            let c = layout.cost(&x);
            if best.is_none_or(|b| c < b) {
                best = Some(c);
            }
        }
        let mut i = 0;
        while i < n {
            x[i] += 1;
            if x[i] <= max {
                break;
            }
            x[i] = 1;
            i += 1;
        }
        if i == n {
            return best;
        }
    }
}

// ─────────────────────────── a lei de um patch ───────────────────────────

#[test]
fn a_triangular_patch_needs_an_even_perimeter() {
    // (2,2,2) fecha com o leque todo em 1.
    assert_eq!(solve_corners(&[2, 2, 2]), Ok(vec![1, 1, 1]));
    // ⚠️ Perímetro ÍMPAR: nenhum inteiro resolve. Este é o caso que a paridade
    // de Takayama nomeia, e ele cai do sistema sozinho.
    assert_eq!(
        solve_corners(&[2, 2, 3]),
        Err(CornerError::Parity { patch: None })
    );
    // Um lado tão longo quanto os outros dois juntos degenera o leque.
    assert_eq!(
        solve_corners(&[2, 2, 4]),
        Err(CornerError::TooShort {
            patch: None,
            corner: 2
        })
    );
    // E a soma fecha: L_i = e_{i-1} + e_{i+1}.
    let lens = [4u32, 4, 6];
    let e = solve_corners(&lens).expect("fecha");
    for (i, l) in lens.iter().enumerate() {
        assert_eq!(*l, e[(i + 2) % 3] + e[(i + 1) % 3], "lado {i}");
    }
}

#[test]
fn a_four_sided_patch_forces_opposite_sides_equal() {
    // ⭐ A lei geral NÃO conhece "lados opostos"; ela devolve dois ciclos de
    // comprimento 2, e a condição de fecho deles É `L_0 = L_2`, `L_1 = L_3`.
    assert!(solve_corners(&[5, 3, 5, 3]).is_ok());
    assert!(matches!(
        solve_corners(&[5, 3, 6, 3]),
        Err(CornerError::Inconsistent { .. })
    ));
    // ⚠️ E um patch de 4 lados com perímetro PAR (16) ainda é inválido — a
    // paridade não é a única condição, e um gate só de paridade a deixaria
    // passar.
    assert!(solve_corners(&[5, 3, 7, 1]).is_err());
}

#[test]
fn a_six_sided_patch_is_determined_not_free() {
    // Seis lados dão dois ciclos de comprimento **3** — ímpares — logo solução
    // única e uma paridade **por ciclo**, ao contrário do caso de 4 lados.
    let lens = [4u32, 4, 4, 4, 4, 4];
    let e = solve_corners(&lens).expect("fecha");
    assert_eq!(e, vec![2, 2, 2, 2, 2, 2]);
    for (i, l) in lens.iter().enumerate() {
        assert_eq!(*l, e[(i + 5) % 6] + e[(i + 1) % 6], "lado {i}");
    }
    // ⚠️ E a paridade é de CADA ciclo, não do perímetro: aqui o perímetro é 23,
    // mas o que reprova é o ciclo ímpar dos lados {2,4,0}.
    assert!(matches!(
        solve_corners(&[3, 4, 4, 4, 4, 4]),
        Err(CornerError::Parity { .. })
    ));
}

// ─────────────────────── o solver contra a força bruta ───────────────────────

#[test]
fn the_solver_reaches_the_brute_force_optimum_on_a_tetrahedron() {
    let mut rng = Lcg(0x5EED_1234);
    for case in 0..12 {
        let targets: Vec<f64> = (0..6).map(|_| 1.0 + rng.next_f64() * 4.0).collect();
        let layout = tetrahedron(&targets);
        let (q, report) = quantize(&layout).expect("o tetraedro fecha");
        // ⚠️ **A força bruta é limitada e o solver não.** Se o solver sair da
        // faixa varrida, a comparação deixa de ser uma comparação — e o gate tem
        // de dizer ISSO, não "o solver errou".
        assert!(
            q.arc.iter().all(|&v| v <= 7),
            "caso {case}: o solver saiu da faixa da forca bruta ({:?})",
            q.arc
        );
        let truth = brute_force(&layout, 7).expect("a forca bruta acha alguma");
        assert!(
            (report.cost - truth).abs() < 1e-9,
            "caso {case}: o solver deu {:.6} e o otimo e' {truth:.6} (alvos {targets:?})",
            report.cost
        );
        // ⭐ O certificado tem de ser um LIMITE: nunca acima do que se atingiu.
        assert!(
            report.lower_bound <= report.cost + 1e-9,
            "caso {case}: limite {:.6} acima do custo {:.6}",
            report.lower_bound,
            report.cost
        );
        assert_eq!(report.cap_binding, 0, "caso {case}: o teto mordeu");
        verify(&layout, &q.arc).expect("todo patch fecha");
    }
}

#[test]
fn the_solver_reaches_the_brute_force_optimum_on_a_glued_pair() {
    // Dois triângulos colados pelos três lados — a esfera mínima. Três arcos.
    let mut rng = Lcg(0xC0FF_EE01);
    for case in 0..16 {
        let targets: Vec<f64> = (0..3).map(|_| 1.0 + rng.next_f64() * 6.0).collect();
        let arcs = targets.iter().map(|t| ArcSpec::new(*t)).collect();
        let layout = Layout::new(
            arcs,
            vec![
                PatchSpec {
                    sides: vec![vec![0], vec![1], vec![2]],
                },
                PatchSpec {
                    sides: vec![vec![0], vec![2], vec![1]],
                },
            ],
        )
        .expect("valido");
        let (_, report) = quantize(&layout).expect("fecha");
        let truth = brute_force(&layout, 16).expect("existe");
        assert!(
            (report.cost - truth).abs() < 1e-9,
            "caso {case}: solver {:.6} vs otimo {truth:.6} (alvos {targets:?})",
            report.cost
        );
    }
}

#[test]
fn the_solver_reaches_the_brute_force_optimum_where_the_constraints_bite() {
    // Força bruta contra o solver num layout com **junções em T e valências
    // mistas** — mais apertado que o octaedro, que só tem lados de um arco.
    //
    // ⚠️ **Ele NÃO apanha a ramificação em pontos** (verificado por mutação em
    // 2026-08-20; quem a apanha é
    // `the_two_branches_partition_the_range_they_came_from`). O que ele prova é
    // outra coisa e vale por si: que o ótimo declarado é o ótimo **verdadeiro**
    // quando as restrições apertam.
    // ⚠️ A faixa é apertada de propósito: a força bruta é `faixa^8`, e um gate
    // lento é um gate que alguém salta.
    let mut rng = Lcg(0x7A11_5EED);
    for case in 0..4 {
        let targets: Vec<f64> = (0..8).map(|_| 1.0 + rng.next_f64() * 2.5).collect();
        let layout = t_junction_layout(&targets);
        let (q, report) = quantize(&layout).expect("fecha");
        assert!(report.proved, "caso {case}: a busca devia esgotar");
        assert!(
            q.arc.iter().all(|&v| v <= 5),
            "caso {case}: o solver saiu da faixa da forca bruta ({:?})",
            q.arc
        );
        let truth = brute_force(&layout, 5).expect("a forca bruta acha alguma");
        assert!(
            (report.cost - truth).abs() < 1e-9,
            "caso {case}: solver {:.6} vs otimo {truth:.6} (alvos {targets:?})",
            report.cost
        );
    }
}

// ────────────────────────── os layouts de verdade ──────────────────────────

#[test]
fn the_sphere_layout_of_the_oracle_quantizes_and_every_patch_closes() {
    let mut rng = Lcg(0x00A1_1CE5);
    for case in 0..8 {
        let targets: Vec<f64> = (0..12).map(|_| 2.0 + rng.next_f64() * 12.0).collect();
        let layout = octahedron(&targets);
        let (q, report) = quantize(&layout).expect("o octaedro fecha");
        // ⚠️ Aqui a força bruta é impossível (12 arcos); o que se afirma é a
        // VALIDADE e o certificado, não a otimalidade.
        let corners = verify(&layout, &q.arc).expect("todo patch fecha");
        assert_eq!(corners.len(), 8);
        // ⚠️ **A forma da rede é uma invariante, não um detalhe**: um nó por LADO
        // de patch, e uma aresta por arco MAIS uma por lado (a do leque). Se
        // alguém trocar o template por-patch, é aqui que se vê.
        assert_eq!(report.nodes, 8 * 3, "um no por lado de patch");
        assert_eq!(report.edges, 12 + 8 * 3, "um arco + um leque por lado");
        assert!(q.arc.iter().all(|&v| v >= 1), "nenhum arco colapsa");
        assert!(report.gap >= -1e-9, "caso {case}: gap negativo");
        assert_eq!(report.cap_binding, 0);
    }
}

#[test]
fn the_cube_layout_makes_opposite_sides_agree_without_being_told() {
    let mut rng = Lcg(0xBEEF_0042);
    let targets: Vec<f64> = (0..12).map(|_| 3.0 + rng.next_f64() * 9.0).collect();
    let layout = cube_layout(&targets);
    let (q, _) = quantize(&layout).expect("o cubo fecha");
    for p in 0..layout.patches().len() {
        let l: Vec<u32> = (0..4).map(|i| layout.side_len(p, i, &q.arc)).collect();
        assert_eq!(l[0], l[2], "patch {p}: lados opostos discordam");
        assert_eq!(l[1], l[3], "patch {p}: lados opostos discordam");
    }
}

#[test]
fn a_side_made_of_two_arcs_is_a_t_junction_and_still_closes() {
    // ⚠️ **Junção em T**: um lado do patch A é partido em dois arcos porque o
    // patch B tem um canto no meio dele. É a razão de a variável ser do ARCO.
    let arcs = vec![
        ArcSpec::new(3.0),
        ArcSpec::new(2.0),
        ArcSpec::new(4.0),
        ArcSpec::new(5.0),
        ArcSpec::new(6.0),
    ];
    let layout = Layout::new(
        arcs,
        vec![
            // A: um lado com DOIS arcos.
            PatchSpec {
                sides: vec![vec![0, 1], vec![2], vec![3]],
            },
            // B: os mesmos arcos, mas o canto cai entre 0 e 1.
            PatchSpec {
                sides: vec![vec![0], vec![1], vec![4]],
            },
            PatchSpec {
                sides: vec![vec![2], vec![3], vec![4]],
            },
        ],
    )
    .expect("valido");
    let (q, report) = quantize(&layout).expect("fecha");
    verify(&layout, &q.arc).expect("todo patch fecha");
    assert!(report.gap >= -1e-9);
}

// ───────────────────────────── as cercas ─────────────────────────────

#[test]
fn a_layout_with_a_boundary_arc_is_refused_not_guessed() {
    // ⚠️ Um arco usado UMA vez é bordo, e esta fase pressupõe superfície fechada.
    // Aceitar em silêncio devolveria um número que parece uma resposta.
    let arcs = vec![ArcSpec::new(2.0), ArcSpec::new(2.0), ArcSpec::new(2.0)];
    let err = Layout::new(
        arcs,
        vec![PatchSpec {
            sides: vec![vec![0], vec![1], vec![2]],
        }],
    );
    assert!(err.is_err(), "um layout aberto tem de ser recusado");
}

#[test]
fn a_spent_budget_returns_a_valid_answer_never_an_error() {
    // ⚠️ **O orçamento não é uma porta de saída.** Com zero expansões a busca não
    // corre de todo — e mesmo assim tem de sair uma quantização VÁLIDA, porque o
    // mergulho corre antes dela. O que se perde é só a prova.
    let mut rng = Lcg(0x0D1E_7A5C);
    for case in 0..6 {
        let targets: Vec<f64> = (0..12).map(|_| 2.0 + rng.next_f64() * 12.0).collect();
        let layout = octahedron(&targets);
        let (q, report) = quantize_within(&layout, Budget::new(0, Budget::default().solves))
            .expect("o mergulho responde sozinho");
        verify(&layout, &q.arc).expect("todo patch fecha mesmo sem busca");
        assert_eq!(
            report.expansions, 0,
            "caso {case}: a busca nao devia correr"
        );
        // ⭐ E o certificado continua válido: o custo nunca fica ABAIXO do limite.
        assert!(
            report.cost >= report.lower_bound - 1e-9,
            "caso {case}: custo {:.6} abaixo do limite {:.6}",
            report.cost,
            report.lower_bound
        );
    }
}

#[test]
fn the_search_never_returns_worse_than_the_dive_it_started_from() {
    // ⭐ O incumbente do mergulho é o teto inicial da busca; ela só substitui por
    // algo estritamente melhor. Um caso em que a busca PIORA seria uma poda
    // errada, e ela passaria despercebida — a resposta continuaria válida.
    let mut rng = Lcg(0x5EED_1234);
    for case in 0..12 {
        let targets: Vec<f64> = (0..6).map(|_| 1.0 + rng.next_f64() * 4.0).collect();
        let layout = tetrahedron(&targets);
        let (_, dived) =
            quantize_within(&layout, Budget::new(0, Budget::default().solves)).expect("mergulho");
        let (_, searched) = quantize(&layout).expect("busca");
        assert!(
            searched.cost <= dived.cost + 1e-9,
            "caso {case}: a busca ({:.6}) piorou o mergulho ({:.6})",
            searched.cost,
            dived.cost
        );
        assert!(searched.proved, "caso {case}: a busca devia ter esgotado");
    }
}

/// Renumera os arcos por uma permutação e reordena os patches — a MESMA
/// superfície, escrita de outra maneira.
fn relabel(layout: &Layout, shift: usize) -> Layout {
    let n = layout.arcs().len();
    let map = |a: u32| u32::try_from((a as usize + shift) % n).unwrap();
    let mut arcs = vec![ArcSpec::new(1.0); n];
    for (a, spec) in layout.arcs().iter().enumerate() {
        arcs[map(u32::try_from(a).unwrap()) as usize] = *spec;
    }
    let mut patches: Vec<PatchSpec> = layout
        .patches()
        .iter()
        .map(|p| PatchSpec {
            sides: p
                .sides
                .iter()
                .map(|s| s.iter().map(|&a| map(a)).collect())
                .collect(),
        })
        .collect();
    patches.reverse();
    Layout::new(arcs, patches).expect("renomear nao muda a validade")
}

#[test]
fn the_two_branches_partition_the_range_they_came_from() {
    // ⭐ **O gate que MATA a ramificação em pontos**, e o único que o faz.
    // Verificado por mutação em 2026-08-20: trocar as meias-retas por
    // `{cut}` e `{cut+1}` passa incólume pelo octaedro, pelo prisma e pelo layout
    // com junção em T — os três continuam verdes e continuam a dizer `proved`.
    // *Um sobrevivente de mutação é um gate em falta, e o gate em falta era este:
    // a propriedade não é do resultado, é da partição.*
    for lo in -3i64..6 {
        for hi in lo..lo + 9 {
            for cut in lo - 1..=hi + 1 {
                let [(a1, b1), (a2, b2)] = ph2d_quantize::branch(lo, hi, cut);
                // Todo inteiro da faixa cai em exatamente UM dos dois lados.
                for x in lo..=hi {
                    let left = x >= a1 && x <= b1;
                    let right = x >= a2 && x <= b2;
                    assert!(
                        left ^ right,
                        "lo={lo} hi={hi} cut={cut}: {x} cai em {} ramos",
                        usize::from(left) + usize::from(right)
                    );
                }
                // E nenhum ramo inventa valores fora da faixa.
                assert!(
                    a1 >= lo && b2 <= hi,
                    "lo={lo} hi={hi} cut={cut}: saiu da faixa"
                );
            }
        }
    }
}

#[test]
fn a_proven_optimum_does_not_depend_on_how_the_arcs_are_numbered() {
    // A ordem dos índices decide qual aresta a busca ramifica primeiro. Se a
    // busca é completa, o ótimo demonstrado é o mesmo escreva-se o layout como se
    // escrever — e o limite inferior também, porque a relaxação não conhece
    // nomes.
    //
    // ⚠️ **Este gate NÃO apanha a ramificação em pontos** (verificado por mutação
    // em 2026-08-20 — ele fica verde sobre ela). Ele apanha uma classe vizinha e
    // igualmente silenciosa: qualquer dependência do resultado na **ordem de
    // construção** do layout, que é o que quebra o determinismo entre máquinas.
    // ⚠️ **O prisma, não o octaedro.** Verificado por mutação em 2026-08-20: o
    // octaedro é folgado demais e deixa a ramificação em pontos passar.
    // ⚠️ E o prisma é PEQUENO de propósito: num prisma de 5 lados com alvos até
    // 10 o ramifica-e-limita gasta o orçamento inteiro (4 096 expansões) em cada
    // caso, e o gate passa a minutos. *Um gate lento é um gate que alguém salta.*
    let mut rng = Lcg(0x7A11_5EED);
    for case in 0..4 {
        let targets: Vec<f64> = (0..9).map(|_| 1.0 + rng.next_f64() * 4.0).collect();
        let layout = prism(3, &targets);
        let (_, a) = quantize(&layout).expect("fecha");
        for shift in [1usize, 4] {
            let (_, b) = quantize(&relabel(&layout, shift)).expect("fecha renomeado");
            assert!(
                a.proved && b.proved,
                "caso {case}: sem prova nao ha' o que comparar"
            );
            assert!(
                (a.cost - b.cost).abs() < 1e-9,
                "caso {case}, shift {shift}: {:.6} != {:.6} — a busca descarta solucoes",
                a.cost,
                b.cost
            );
            assert!(
                (a.lower_bound - b.lower_bound).abs() < 1e-9,
                "caso {case}, shift {shift}: o limite tambem devia ser o mesmo"
            );
        }
    }
}

#[test]
fn running_out_of_budget_says_so_instead_of_calling_the_layout_impossible() {
    // ⭐ **`Exhausted` e `Infeasible` são afirmações sobre coisas DIFERENTES.**
    // Uma é sobre o solver, a outra sobre o layout. Fundi-las faria um layout
    // perfeitamente quantizável parecer impossível — e o chamador a jusante
    // (F5) tomaria a decisão errada: desistir em vez de insistir.
    // ⚠️ O octaedro, não o prisma: é ele que sai meio-inteiro de forma fiável
    // (valências ímpares). O prisma, todo de valência 4 nas laterais, costuma dar
    // raiz já inteira — e o gate passaria a testar outra coisa.
    let mut rng = Lcg(0x0B0D_6E70);
    let targets: Vec<f64> = (0..12).map(|_| 2.0 + rng.next_f64() * 6.0).collect();
    let layout = octahedron(&targets);
    // ⚠️ Primeiro a premissa: este layout tem MESMO de precisar do mergulho.
    // Sem meia-integralidade a raiz já é inteira e uma resolução basta — o gate
    // passaria a testar outra coisa.
    let (q, full) = quantize(&layout).expect("o layout fecha com orcamento cheio");
    verify(&layout, &q.arc).expect("todo patch fecha");
    assert!(
        full.half_integral > 0 && full.solves > 1,
        "premissa: o layout precisa do mergulho ({full:?})"
    );
    // Agora o orçamento mínimo: nem a raiz cabe.
    let err = quantize_within(&layout, Budget::new(0, 1)).unwrap_err();
    assert!(
        matches!(err, ph2d_quantize::SolveError::Exhausted { .. }),
        "com orcamento minimo tem de sair Exhausted, saiu {err:?}"
    );
}

#[test]
fn the_network_ceiling_scales_instead_of_deciding_the_answer() {
    // ⭐ **O teto da rede não é um limite físico e não pode DECIDIR nada.**
    // Medido em 2026-08-20 na `sphere_noisy`: com o teto apertado o solver
    // devolvia *"não existe quantização"* — uma afirmação sobre o LAYOUT —
    // quando o que não cabia era o teto. Duas coisas o impedem, e as duas são
    // gateadas aqui: os degraus sobem, e a resposta nunca encosta no teto.
    let targets: Vec<f64> = (0..12).map(|i| 1.0 + f64::from(i % 7) * 3.0).collect();
    let layout = octahedron(&targets);
    let tight = ph2d_quantize::BiNetwork::build_scaled(&layout, 1);
    for &step in &ph2d_quantize::network::CAP_STEPS[1..] {
        let wide = ph2d_quantize::BiNetwork::build_scaled(&layout, step);
        for (a, b) in tight.edges().iter().zip(wide.edges()) {
            assert!(
                b.hi >= a.hi,
                "degrau {step}: o teto encolheu ({} -> {})",
                a.hi,
                b.hi
            );
            assert_eq!(b.lo, a.lo, "degrau {step}: o PISO nao pode mexer");
        }
        assert!(
            wide.edges()
                .iter()
                .zip(tight.edges())
                .any(|(w, t)| w.hi > t.hi),
            "degrau {step}: nao alargou nada"
        );
    }
    // E a resposta real não encosta no teto — se encostasse, teria sido ele a
    // escolhê-la.
    let (_, report) = quantize(&layout).expect("fecha");
    assert_eq!(report.cap_binding, 0, "a resposta encostou no teto");
    assert_eq!(report.cap_step, 1, "este layout cabe no degrau rapido");
}

#[test]
fn the_verifier_rebuilds_the_fan_instead_of_trusting_the_answer() {
    // Uma quantização inventada à mão, que não fecha, tem de ser rejeitada —
    // mesmo tendo perímetro par em todo patch.
    let targets = [4.0; 6];
    let layout = tetrahedron(&targets);
    let bad = vec![1, 1, 1, 1, 1, 8];
    assert!(verify(&layout, &bad).is_err());
}
