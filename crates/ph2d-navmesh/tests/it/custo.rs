//! ⭐⭐⭐ **A PROCURA COM CUSTOS** (W7, plano 30 §2.5) — o Polyanya ponderado contra o ORÁCULO
//! PONDERADO (cantos + Steiner nas fronteiras, convergência medida na sonda `medir_custo`), em cenas
//! desenhadas à mão que isolam cada fenómeno, com o CONTROLO ao lado de cada uma.

use ph2d_nav::cost::path_cost;
use ph2d_nav::oracle::WeightedOracle;
use ph2d_nav::{NavMesh, Polyanya, V2};
use ph2d_navmesh::{Area, Params, Shape, build_with_areas};

use crate::cena::retangulo;

fn quadrado(c: V2, h: f64) -> Shape {
    Shape::Convex(vec![
        [c[0] - h, c[1] - h],
        [c[0] + h, c[1] - h],
        [c[0] + h, c[1] + h],
        [c[0] - h, c[1] + h],
    ])
}

/// Um chão `20 × 12` com UMA lama quadrada (lado `6`) no meio, sem obstáculos.
fn lama_no_meio() -> NavMesh {
    build_with_areas(
        &retangulo(20.0, 12.0),
        &[],
        &[Area {
            shape: quadrado([10.0, 6.0], 3.0),
            id: 1,
        }],
        &Params::default(),
    )
    .expect("constrói")
    .mesh
}

/// O custo da procura, o do oráculo, e a validade do caminho (o custo dito = o andado).
fn mede(m: &NavMesh, costs: &[f64], a: V2, z: V2) -> (f64, f64, Vec<V2>) {
    let mut s = Polyanya::new();
    let p = s.find_path_costs(m, costs, a, z).expect("há caminho");
    let andado = path_cost(m, costs, &p.points).expect("o caminho fica na malha");
    assert!(
        (andado - p.cost).abs() <= 1e-9 * andado.max(1.0),
        "diz {} e custa {andado}",
        p.cost
    );
    let (_, o) = WeightedOracle::new(m, costs, 0.05)
        .shortest(a, z)
        .expect("o oráculo acha");
    (p.cost, o, p.points)
}

#[test]
fn contorna_a_lama_cara_e_atravessa_a_barata() {
    let m = lama_no_meio();
    let (a, z) = ([2.0, 6.0], [18.0, 6.0]);
    // Cara: contornar pelos cantos (`2·√(5² + 3²) + 6` = `17,66`) é mais barato que atravessar.
    let (c, o, pts) = mede(&m, &[1.0, 10.0], a, z);
    let volta = 2.0 * (25.0f64 + 9.0).sqrt() + 6.0;
    assert!(
        (c - volta).abs() < 1e-6,
        "custo {c}, a volta pelos cantos é {volta}"
    );
    assert!(c <= o * (1.0 + 1e-9), "procura {c} acima do oráculo {o}");
    assert!(pts.len() >= 4, "não dobrou nos cantos: {pts:?}");
    // CONTROLO: barata, a recta é a resposta (`10 + 6 · 0,1` a mais).
    let (c, _, pts) = mede(&m, &[1.0, 1.1], a, z);
    assert!((c - 16.6).abs() < 1e-6, "custo {c}");
    assert_eq!(pts.len(), 2, "devia ir a direito: {pts:?}");
}

#[test]
fn dentro_da_lama_sai_e_corre_encostado_a_fronteira() {
    let m = lama_no_meio();
    // Os dois a `d = 0,2` da fronteira de cima, a `5` m um do outro. O óptimo sai em diagonal no
    // ÂNGULO CRÍTICO (`sin θ = 1/w`), corre EM CIMA da fronteira (custo 1) e volta a entrar:
    // `5 + 2d·√(w² − 1)` — enquanto as duas saídas (`d/√(w² − 1)` cada) couberem nos 5 m.
    let (a, z) = ([7.5, 8.8], [12.5, 8.8]);
    for w in [10.0f64, 1.5, 1.02] {
        let (c, o, pts) = mede(&m, &[1.0, w], a, z);
        let esperado = 5.0 + 0.4 * (w * w - 1.0).sqrt();
        assert!(
            (c - esperado).abs() <= 1e-9 * esperado,
            "w = {w}: custo {c}, o ângulo crítico dá {esperado} (oráculo {o}) {pts:?}"
        );
        assert!(
            c <= o * (1.0 + 1e-9),
            "w = {w}: procura {c} acima do oráculo {o}"
        );
    }
    // CONTROLO: a 1,001 cada saída pedia `3,2` m e não cabe — a direito, sem pontos no meio. ⚠️ Os
    // dois controlos escritos aqui antes (1,5 e 1,02, «sair não compensa») estavam errados: as
    // saídas a olho eram perpendiculares, e a lei é a do ângulo crítico.
    let (c, o, pts) = mede(&m, &[1.0, 1.001], a, z);
    assert!((c - 5.005).abs() < 1e-9, "custo {c} (oráculo {o}) {pts:?}");
    assert_eq!(pts.len(), 2, "devia ir a direito: {pts:?}");
}

#[test]
fn a_refraccao_obedece_a_snell() {
    // A fronteira é a recta `x = 10`: chão (1) à esquerda, lama (w) à direita. De `a` a `z` o óptimo
    // cruza em `y*` com `sin θ₁ = w · sin θ₂` — calculado aqui por bissecção na derivada, sem a malha.
    let m = build_with_areas(
        &retangulo(20.0, 12.0),
        &[],
        &[Area {
            shape: Shape::Convex(vec![[10.0, -1.0], [21.0, -1.0], [21.0, 13.0], [10.0, 13.0]]),
            id: 1,
        }],
        &Params::default(),
    )
    .expect("constrói")
    .mesh;
    let casos: [(f64, V2, V2); 3] = [
        (2.0, [3.0, 2.0], [16.0, 10.0]),
        (3.5, [1.0, 9.0], [19.0, 1.5]),
        (1.25, [8.0, 1.0], [11.0, 11.0]),
    ];
    for (w, a, z) in casos {
        let f = |y: f64| {
            let d1 = ((10.0 - a[0]).powi(2) + (y - a[1]).powi(2)).sqrt();
            let d2 = ((z[0] - 10.0).powi(2) + (z[1] - y).powi(2)).sqrt();
            d1 + w * d2
        };
        let (mut lo, mut hi) = (a[1].min(z[1]), a[1].max(z[1]));
        for _ in 0..200 {
            let (m1, m2) = (lo + (hi - lo) / 3.0, hi - (hi - lo) / 3.0);
            if f(m1) < f(m2) {
                hi = m2;
            } else {
                lo = m1;
            }
        }
        let esperado = f(0.5 * (lo + hi));
        let (c, _, pts) = mede(&m, &[1.0, w], a, z);
        assert!(
            (c - esperado).abs() <= 1e-9 * esperado,
            "w = {w}: procura {c}, Snell {esperado} ({pts:?})"
        );
    }
}
