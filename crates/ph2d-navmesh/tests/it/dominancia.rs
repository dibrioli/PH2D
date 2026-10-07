//! ⭐⭐ **A DOMINÂNCIA ENTRE FRENTES** (W9, plano 30 §17) — a procura ponderada corta as frentes
//! paralelas que a grelha de uma fronteira abre, sem pagar um centésimo de custo por isso. A régua é a
//! MESMA procura com a dominância desligada (o CONTROLO), sobre as cenas da sonda `medir_custo`.

use ph2d_nav::cost::path_cost;
use ph2d_nav::oracle::WeightedOracle;
use ph2d_nav::{NavMesh, Polyanya, V2};
use ph2d_navmesh::{Area, Params, Shape, build_with_areas};

/// O gerador da sonda `examples/medir_custo.rs`, ao número (a mesma semente dá a mesma cena).
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn caixa(r: &mut Lcg, c: V2, hmin: f64, hmax: f64) -> Shape {
    let (hx, hy) = (
        hmin + r.next() * (hmax - hmin),
        hmin + r.next() * (hmax - hmin),
    );
    let (a, b) = (r.next() * 2.0 - 1.0, r.next() * 2.0 - 1.0);
    let l = (a * a + b * b).sqrt().max(1e-6);
    let (co, si) = (a / l, b / l);
    Shape::Convex(
        [[-hx, -hy], [hx, -hy], [hx, hy], [-hx, hy]]
            .iter()
            .map(|p| [c[0] + p[0] * co - p[1] * si, c[1] + p[0] * si + p[1] * co])
            .collect(),
    )
}

/// A cena `30 × 20` da sonda: 10 obstáculos e 4 lamas (ids `1..=4`).
pub(super) fn cena(seed: u64) -> NavMesh {
    let (w, h) = (30.0, 20.0);
    let mut r = Lcg(seed);
    let obs: Vec<Shape> = (0..10)
        .map(|i| {
            let c = [r.next() * w, r.next() * h];
            if i % 3 == 0 {
                Shape::Circle {
                    center: c,
                    radius: 0.2 + r.next() * 1.0,
                }
            } else {
                caixa(&mut r, c, 0.2, 1.5)
            }
        })
        .collect();
    let areas: Vec<Area> = (0..4)
        .map(|i| {
            let c = [r.next() * w, r.next() * h];
            let shape = if i % 2 == 0 {
                caixa(&mut r, c, 1.0, 4.0)
            } else {
                Shape::Circle {
                    center: c,
                    radius: 1.0 + r.next() * 3.0,
                }
            };
            Area {
                shape,
                id: (i + 1) as u16,
                dentro: false,
            }
        })
        .collect();
    let params = Params {
        agent_radius: 0.4,
        ..Params::default()
    };
    let reg = vec![[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]];
    build_with_areas(&reg, &obs, &areas, &params)
        .expect("constrói")
        .mesh
}

/// O custo andado de uma procura e os nós que ela expandiu.
fn procura(s: &mut Polyanya, m: &NavMesh, costs: &[f64], a: V2, z: V2) -> (f64, u64) {
    let e0 = s.stats.expanded;
    let p = s.find_path_costs(m, costs, a, z).expect("mesma ilha");
    let andado = path_cost(m, costs, &p.points).expect("o caminho fica na malha");
    assert!(
        (andado - p.cost).abs() <= 1e-9 * andado.max(1.0),
        "diz {} e custa {andado}",
        p.cost
    );
    (andado, s.stats.expanded - e0)
}

#[test]
fn a_dominancia_corta_nos_e_nunca_encarece_um_caminho() {
    let mut com = Polyanya::new();
    let mut sem = Polyanya::new();
    sem.set_front_dominance(false);
    let (mut nos_com, mut nos_sem, mut pares) = (0u64, 0u64, 0);
    for seed in 1..=4u64 {
        let m = cena(seed);
        let mut r = Lcg(seed * 31 + 7);
        for peso in [2.0, 4.0, 10.0] {
            let costs = [1.0, peso, peso, peso, peso];
            let mut n = 0;
            while n < 8 {
                let (a, z) = (
                    [r.next() * 30.0, r.next() * 20.0],
                    [r.next() * 30.0, r.next() * 20.0],
                );
                if m.locate(a).is_none() || com.find_path(&m, a, z).is_err() {
                    continue;
                }
                n += 1;
                let (cc, ec) = procura(&mut com, &m, &costs, a, z);
                let (cs, es) = procura(&mut sem, &m, &costs, a, z);
                assert!(
                    cc <= cs * (1.0 + 1e-9),
                    "semente {seed}, peso {peso}, {a:?} → {z:?}: com a dominância {cc}, sem {cs}"
                );
                nos_com += ec;
                nos_sem += es;
                pares += 1;
            }
        }
    }
    // O CONTROLO expande mais (medido: `2,6×` na cena grande; aqui `26 493` nós contra `43 543`, `1,64×`).
    assert!(
        (nos_com as f64) * 1.5 <= nos_sem as f64,
        "{pares} pares: {nos_com} nós com a dominância, {nos_sem} sem"
    );
}

/// O par da sonda onde a dominância escolheu, entre dois caminhos discretos de custo IGUAL, o que
/// guarda um ponto a mais na quina arredondada da lama — e o polimento ficava preso nele (`1,0002`
/// do oráculo) enquanto a gama de deslize de uma raiz de refracção era «o que a raiz vê». Com a
/// ARESTA inteira, a travessia desliza para onde o vizinho a vê, e o ponto da quina fica colinear.
#[test]
fn a_quina_da_lama_nao_prende_o_polimento() {
    let m = cena(5);
    let costs = [1.0, 4.0, 4.0, 4.0, 4.0];
    let (a, z) = (
        [21.213_907_483_677_346, 16.625_414_616_786_91],
        [26.739_704_185_901_4, 4.362_066_306_318_788],
    );
    let (_, o) = WeightedOracle::new(&m, &costs, 0.1)
        .shortest(a, z)
        .expect("o oráculo acha");
    let (c, _) = procura(&mut Polyanya::new(), &m, &costs, a, z);
    assert!(c / o <= 1.0001, "custo / oráculo = {}", c / o);
}

/// (W14) **O trabalho de uma procura** ([`ph2d_nav::Stats::work`], a unidade do orçamento da fila):
/// sem lama é o número de nós AO BIT (as cenas sem lama ficam com a mesma fila); na lama pesa as
/// raízes de fronteira e as frentes comparadas — medido (plano 30 §22.1): o nó da ponderada custa
/// `1,7–3,2×` o da uniforme conforme a densidade da lama. Aqui: `16 837` de trabalho sobre `9 290` nós,
/// `1,81×` — a faixa `1,5–2,5` apanha um peso dobrado ou a metade.
#[test]
fn o_trabalho_sem_lama_sao_os_nos_e_na_lama_pesa_o_que_custa() {
    let mut s = Polyanya::new();
    let (mut nos, mut trabalho, mut pend, mut frentes) = (0u64, 0u64, 0u64, 0u64);
    for seed in 1..=4u64 {
        let m = cena(seed);
        let mut r = Lcg(seed * 31 + 7);
        let mut n = 0;
        while n < 8 {
            let (a, z) = (
                [r.next() * 30.0, r.next() * 20.0],
                [r.next() * 30.0, r.next() * 20.0],
            );
            if m.locate(a).is_none() || s.find_path(&m, a, z).is_err() {
                continue;
            }
            n += 1;
            let antes = s.stats;
            s.find_path_costs(&m, &[1.0], a, z).expect("mesma ilha");
            assert_eq!(
                s.stats.work() - antes.work(),
                s.stats.expanded - antes.expanded,
                "sem lama, semente {seed}, {a:?} → {z:?}"
            );
            let antes = s.stats;
            s.find_path_costs(&m, &[1.0, 4.0, 4.0, 4.0, 4.0], a, z)
                .expect("mesma ilha");
            nos += s.stats.expanded - antes.expanded;
            trabalho += s.stats.work() - antes.work();
            pend += s.stats.pending - antes.pending;
            frentes += s.stats.compared - antes.compared;
        }
    }
    // A população: a lama refracta e a dominância compara (os dois termos mordem).
    assert!(pend > 0 && frentes > 0, "{pend} raízes, {frentes} frentes");
    let k = trabalho as f64 / nos as f64;
    assert!(
        (1.5..=2.5).contains(&k),
        "trabalho / nós = {k:.2} ({trabalho} / {nos})"
    );
}
