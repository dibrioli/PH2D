//! ⭐⭐ (W15) **A PROCURA EM FATIAS é a procura inteira, ao bit** (plano 30 §23.2) — parada de `k` em
//! `k` unidades de trabalho, e REFEITA de buffers novos até ao trabalho de cada pausa (o que um scrub
//! faz), dá o mesmo caminho que de uma vez. Sobre as cenas da dominância (lama a pesos 4 e 10, e — W19 —
//! com duas áreas BARATAS, a cota pela distância a elas), com e sem atalhos (o Dijkstra deles pára a meio
//! de um troço).

use ph2d_nav::{Link, NavMesh, Planeado, Plano, Polyanya, Query, V2};

use super::dominancia::{cena, cena_com};

fn comeca(m: &NavMesh, q: &Query<'_>, a: V2, z: V2) -> Result<Plano, Planeado> {
    Plano::begin(m, &mut Polyanya::new(), q, a, z)
}

fn ate_ao_fim(m: &NavMesh, q: &Query<'_>, p: &mut Plano) -> Planeado {
    loop {
        if let Some(r) = p.run(m, q, u64::MAX) {
            return r;
        }
    }
}

/// Em fatias de `k`: a resposta e o trabalho de cada pausa.
fn em_fatias(m: &NavMesh, q: &Query<'_>, a: V2, z: V2, k: u64) -> (Planeado, Vec<u64>) {
    let mut p = match comeca(m, q, a, z) {
        Ok(p) => p,
        Err(r) => return (r, Vec::new()),
    };
    let mut pausas = Vec::new();
    loop {
        let antes = p.trabalho();
        if let Some(r) = p.run(m, q, antes + k) {
            return (r, pausas);
        }
        let w = p.trabalho();
        // (Um `pop` soma o nó e as frentes que a dominância comparou: a pausa passa o tecto por um.)
        assert!(
            w > antes + k,
            "a pausa nunca vem antes de passar o tecto: {antes} + {k} → {w}"
        );
        pausas.push(w);
    }
}

#[test]
fn a_procura_em_fatias_e_a_procura_inteira_ao_bit() {
    let atalhos = [
        Link {
            id: 0,
            from: [2.0, 2.0],
            to: [27.0, 17.0],
            two_way: true,
            teleport: true,
            cost: 1.0,
        },
        Link {
            id: 1,
            from: [4.0, 16.0],
            to: [24.0, 4.0],
            two_way: false,
            teleport: false,
            cost: 0.0,
        },
    ];
    let (mut fatiadas, mut refeitas, mut com_atalho) = (0usize, 0usize, 0usize);
    for seed in [1u64, 2, 3] {
        let (m, m_baratas) = (cena(seed), cena_com(seed, &[2, 4], None));
        for (m, costs) in [4.0, 10.0].into_iter().flat_map(|peso| {
            [
                (&m, [1.0, peso, peso, peso, peso]),
                (&m_baratas, [1.0, peso, 0.3, peso, 0.3]),
            ]
        }) {
            let peso = costs[1];
            for links in [&[][..], &atalhos[..]] {
                let q = Query {
                    costs: &costs,
                    links,
                };
                for (a, z) in [
                    ([1.0, 1.0], [29.0, 19.0]),
                    ([1.0, 19.0], [29.0, 1.0]),
                    ([15.0, 1.0], [15.0, 19.0]),
                    ([1.0, 10.0], [29.0, 10.0]),
                ] {
                    let inteira = match comeca(m, &q, a, z) {
                        Ok(mut p) => ate_ao_fim(m, &q, &mut p),
                        Err(r) => r,
                    };
                    com_atalho += usize::from(inteira.as_ref().is_some_and(|c| !c.1.is_empty()));
                    for k in [1, 7, 61] {
                        let (r, pausas) = em_fatias(m, &q, a, z, k);
                        assert_eq!(r, inteira, "seed {seed} peso {peso} {a:?}→{z:?} k {k}");
                        fatiadas += usize::from(!pausas.is_empty());
                        if k != 61 {
                            continue;
                        }
                        // O scrub: buffers novos, de uma vez até ao trabalho da pausa — e daí ao fim.
                        // A 1.ª pausa, a do meio e a última (cada uma é uma procura inteira).
                        let n = pausas.len();
                        let tres = [0, n / 2, n.saturating_sub(1)];
                        for &w in tres.iter().filter(|_| n > 0).filter_map(|&i| pausas.get(i)) {
                            let mut p = comeca(m, &q, a, z).expect("a corrida parou a meio");
                            assert!(p.run(m, &q, w - 1).is_none(), "pára no mesmo pop");
                            assert_eq!(p.trabalho(), w);
                            assert_eq!(ate_ao_fim(m, &q, &mut p), inteira, "refeita até {w}");
                            refeitas += 1;
                        }
                    }
                }
            }
        }
    }
    // A população (medida: 140 de 144 · 132 · 9): procuras que pararam a meio, pausas refeitas, e
    // caminhos que usam um atalho.
    assert!(fatiadas >= 120, "{fatiadas} procuras pararam a meio");
    assert!(refeitas >= 100, "{refeitas} pausas refeitas");
    assert!(com_atalho >= 4, "{com_atalho} caminhos por um atalho");
}
