//! ⭐⭐⭐ **POLYANYA = O EXACTO** (plano 30 §2.5, o gate S1): em cenas aleatórias com caixas rodadas,
//! círculos e cápsulas, recuadas por vários raios, com e sem fusão em convexos, o comprimento da
//! procura bate o do grafo de visibilidade + Dijkstra a `1e-9` relativo — e o caminho devolvido é
//! VÁLIDO (cada troço vê o seguinte dentro da malha, pela régua do oráculo, que não usa a procura).
//!
//! ⚠️ O piso de população é load-bearing: uma varredura cujas cenas saíssem todas vazias (ou todos os
//! pontos fora da malha) ficaria verde a medir nada.

use ph2d_nav::{NoPath, Polyanya, oracle};
use ph2d_navmesh::{Corner, Params, build};

use crate::cena::{Lcg, obstaculos, retangulo};

#[test]
fn o_caminho_da_procura_e_o_mais_curto_exacto() {
    let mut s = Polyanya::new();
    let mut comparados = 0usize;
    let mut inalcancaveis = 0usize;
    let mut com_volta = 0usize;
    for seed in 1..=24u64 {
        let mut rng = Lcg(seed);
        let n = 3 + (seed as usize % 6);
        let obs = obstaculos(&mut rng, n, 16.0, 12.0);
        let raio = [0.0, 0.3, 0.7][(seed % 3) as usize];
        let params = Params {
            agent_radius: raio,
            corner: if seed % 5 == 0 {
                Corner::Miter
            } else {
                Corner::Round
            },
            disk_sides: 8,
            merge: seed % 2 == 0,
        };
        let built = build(&retangulo(16.0, 12.0), &obs, &params).expect("a cena constrói");
        let m = &built.mesh;
        let mut pontos = Vec::new();
        while pontos.len() < 14 {
            let p = [rng.range(0.0, 16.0), rng.range(0.0, 12.0)];
            if m.locate(p).is_some() {
                pontos.push(p);
            }
        }
        for i in 0..pontos.len() {
            for j in (i + 1)..pontos.len() {
                let (a, b) = (pontos[i], pontos[j]);
                let ex = oracle::shortest(m, a, b);
                match (s.find_path(m, a, b), ex) {
                    (Ok(p), Some((_, el))) => {
                        let tol = 1e-9 * el.max(1.0);
                        assert!(
                            (p.length - el).abs() <= tol,
                            "semente {seed}: {a:?} → {b:?}: procura {} contra exacto {el}",
                            p.length
                        );
                        for w in p.points.windows(2) {
                            assert!(
                                oracle::visible(m, w[0], w[1]),
                                "semente {seed}: troço {w:?} atravessa uma parede"
                            );
                        }
                        if p.points.len() > 2 {
                            com_volta += 1;
                        }
                        comparados += 1;
                    }
                    (Err(NoPath::Unreachable), None) => inalcancaveis += 1,
                    (r, e) => {
                        panic!("semente {seed}: {a:?} → {b:?}: procura {r:?} contra exacto {e:?}")
                    }
                }
            }
        }
    }
    assert!(
        comparados >= 1_500,
        "a varredura comparou só {comparados} pares"
    );
    assert!(
        com_volta >= 300,
        "só {com_volta} caminhos viraram — as cenas não têm obstáculos a sério"
    );
    assert!(
        inalcancaveis > 0,
        "nenhuma cena partiu a região — o ramo das ilhas ficou por medir"
    );
}
