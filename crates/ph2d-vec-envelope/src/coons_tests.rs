//! Gates do patch de Coons (ADR-0129 Fatia D). Módulo irmão do [`super`] — teto de LOC.

use super::*;
use crate::QuadWarp;

/// Cantos de um retângulo `[BL, BR, TR, TL]`.
fn bbox_corners(origin: [f64; 2], size: [f64; 2]) -> [[f64; 2]; 4] {
    let [ox, oy] = origin;
    let [w, h] = size;
    [[ox, oy], [ox + w, oy], [ox + w, oy + h], [ox, oy + h]]
}

/// A cúbica de bordo `i` como o motor a consome, avaliada em `t` — o oráculo independente de
/// [`Side`] para o gate de bordo (mesma fórmula, escrita à mão a partir dos 4 pontos).
fn cubic_at(p: [[f64; 2]; 4], t: f64) -> [f64; 2] {
    let s = 1.0 - t;
    let w = [s * s * s, 3.0 * s * s * t, 3.0 * s * t * t, t * t * t];
    std::array::from_fn(|k| (0..4).map(|i| w[i] * p[i][k]).sum())
}

/// **EM REPOUSO, O MAPA É A IDENTIDADE** — gaiola no retângulo-fonte, lados retos canônicos.
///
/// Metade **presença** do par ausência/presença: afirma pontos concretos IGUAIS à entrada, então
/// não fica verde num motor que não faz nada. É este gate que autoriza o envelope a nascer em Mesh
/// sem mexer numa vírgula da arte.
#[test]
fn at_rest_the_coons_warp_is_the_identity() {
    let origin = [3.0, -2.0];
    let size = [10.0, 6.0];
    let corners = bbox_corners(origin, size);
    let w = CoonsWarp::new(origin, size, corners, &rest_edges(&corners)).unwrap();
    for p in [[3.0, -2.0], [8.0, 1.0], [13.0, 4.0], [5.5, 0.0]] {
        let q = w.map(p);
        assert!(
            (q[0] - p[0]).abs() < 1e-12 && (q[1] - p[1]).abs() < 1e-12,
            "repouso não é identidade em {p:?}: {q:?}"
        );
    }
}

/// **O BORDO DESENHADO É O BORDO DO MAPA.** Para os 4 lados: a imagem da aresta do retângulo-fonte
/// coincide com a cúbica que o artista vê, ponto a ponto.
///
/// É o que o termo bilinear negativo compra, e é a razão de o gesto ser utilizável: uma alça que
/// pousasse *perto* do bordo tornaria a gaiola uma sugestão, não um contrato. Falha se alguém
/// "simplificar" o patch somando só as duas réguas.
#[test]
fn each_drawn_side_is_exactly_the_image_of_that_edge() {
    let origin = [0.0, 0.0];
    let size = [10.0, 6.0];
    let corners = [[0.0, 0.0], [10.0, 0.0], [10.0, 6.0], [0.0, 6.0]];
    // Cada lado puxado para fora de um jeito diferente — nenhum reto, nenhum simétrico.
    let edges: CageEdges = [
        [[3.0, -2.0], [7.0, 1.5]],
        [[11.5, 2.0], [12.5, 4.5]],
        [[7.0, 8.0], [3.0, 5.0]],
        [[-1.5, 4.0], [-2.5, 1.0]],
    ];
    let w = CoonsWarp::new(origin, size, corners, &edges).unwrap();
    /// Um lado do gate: onde o ponto-fonte anda com `t`, e os 4 controles da cúbica **no sentido em
    /// que ela é DESENHADA** (a mesma ordem que o artista vê no componente).
    type SideCase = (fn(f64) -> [f64; 2], [[f64; 2]; 4]);
    let sides: [SideCase; 4] = [
        (
            |t| [10.0 * t, 0.0],
            [corners[0], edges[0][0], edges[0][1], corners[1]],
        ),
        (
            |t| [10.0, 6.0 * t],
            [corners[1], edges[1][0], edges[1][1], corners[2]],
        ),
        (
            |t| [10.0 * (1.0 - t), 6.0],
            [corners[2], edges[2][0], edges[2][1], corners[3]],
        ),
        (
            |t| [0.0, 6.0 * (1.0 - t)],
            [corners[3], edges[3][0], edges[3][1], corners[0]],
        ),
    ];
    for (side, (src, ctrl)) in sides.into_iter().enumerate() {
        for i in 0..=10 {
            let t = f64::from(i) / 10.0;
            let got = w.map(src(t));
            let want = cubic_at(ctrl, t);
            assert!(
                (got[0] - want[0]).abs() < 1e-9 && (got[1] - want[1]).abs() < 1e-9,
                "lado {side} em t={t}: mapa deu {got:?}, a curva desenhada está em {want:?}"
            );
        }
    }
}

/// **Um lado reto canônico é AFIM em `t`.** É a razão de [`rest_edges`] emitir (⅓, ⅔) e não a
/// degenerada `(P0,P0,P3,P3)` — a `ph2d-vec-blend` já pagou por essa confusão (as intermediárias
/// ondulavam), e aqui o preço seria o repouso deixar de ser identidade EXATA.
#[test]
fn a_canonical_straight_side_is_affine_in_t() {
    let corners = [[0.0, 0.0], [9.0, 3.0], [9.0, 9.0], [0.0, 6.0]];
    let edges = rest_edges(&corners);
    for i in 0..4 {
        let (a, b) = (corners[i], corners[(i + 1) % 4]);
        let ctrl = [a, edges[i][0], edges[i][1], b];
        for k in 0..=8 {
            let t = f64::from(k) / 8.0;
            let got = cubic_at(ctrl, t);
            let want = [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])];
            assert!(
                (got[0] - want[0]).abs() < 1e-12 && (got[1] - want[1]).abs() < 1e-12,
                "lado reto {i} não é afim em t={t}: {got:?} != {want:?}"
            );
        }
    }
}

/// **A jacobiana fechada bate com a diferença central de `map`.** Guarda a consistência que a espinha
/// exige — e barata, ao contrário do sintoma de quebrá-la (o `fit_to_bezpath` deixar de convergir).
#[test]
fn the_closed_form_jacobian_matches_finite_difference() {
    let origin = [0.0, 0.0];
    let size = [10.0, 6.0];
    let corners = [[0.5, 0.5], [10.0, -0.5], [8.5, 6.5], [1.5, 5.5]];
    let mut edges = rest_edges(&corners);
    edges[0][0] = [3.0, -1.5];
    edges[2][1] = [3.0, 7.5];
    let w = CoonsWarp::new(origin, size, corners, &edges).unwrap();
    let step = 1e-6;
    for p in [[2.0, 1.0], [7.0, 4.0], [5.0, 3.0], [9.0, 5.0]] {
        let j = w.jacobian(p);
        let d = |dx: f64, dy: f64| {
            let a = w.map([p[0] + dx, p[1] + dy]);
            let b = w.map([p[0] - dx, p[1] - dy]);
            [(a[0] - b[0]) / (2.0 * step), (a[1] - b[1]) / (2.0 * step)]
        };
        let (du, dv) = (d(step, 0.0), d(0.0, step));
        assert!(
            (j[0][0] - du[0]).abs() < 1e-5
                && (j[1][0] - du[1]).abs() < 1e-5
                && (j[0][1] - dv[0]).abs() < 1e-5
                && (j[1][1] - dv[1]).abs() < 1e-5,
            "jacobiana != diferença finita em {p:?}: J={j:?} du={du:?} dv={dv:?}"
        );
    }
}

/// **AUSÊNCIA:** repouso e uma dobra moderada NÃO dobram o mapa — o gesto normal nunca é recusado.
#[test]
fn a_reasonable_bend_does_not_fold() {
    let origin = [0.0, 0.0];
    let size = [10.0, 6.0];
    let corners = bbox_corners(origin, size);
    let rest = rest_edges(&corners);
    assert!(
        !CoonsWarp::new(origin, size, corners, &rest)
            .unwrap()
            .folds(),
        "o repouso foi acusado de dobrar"
    );
    let mut bent = rest;
    bent[0][0] = [3.0, -2.0]; // barriga para fora, ~1/3 da altura
    bent[0][1] = [7.0, -2.0];
    assert!(
        !CoonsWarp::new(origin, size, corners, &bent)
            .unwrap()
            .folds(),
        "uma barriga moderada foi acusada de dobrar"
    );
}

/// **PRESENÇA:** o MESMO amostrador vê a dobra quando ela existe — um lado empurrado ATRAVÉS do
/// patch, para além do lado oposto.
///
/// Sem este irmão, o gate de ausência acima ficaria verde num `folds` que responde `false` sempre
/// ([[feedback_absence_gate_needs_a_presence_sibling]]) — e o guard do gesto seria decorativo.
#[test]
fn the_sampler_detects_a_fold_when_there_is_one() {
    let origin = [0.0, 0.0];
    let size = [10.0, 6.0];
    let corners = bbox_corners(origin, size);
    let mut folded = rest_edges(&corners);
    // O lado de baixo empurrado para MUITO acima do lado de cima: o patch vira do avesso.
    folded[0][0] = [3.0, 20.0];
    folded[0][1] = [7.0, 20.0];
    assert!(
        CoonsWarp::new(origin, size, corners, &folded)
            .unwrap()
            .folds(),
        "o amostrador não viu uma dobra grosseira"
    );
}

/// **OS DOIS GESTOS CONCORDAM EM REPOUSO E DIVERGEM FORA DELE** — e é este par que prova que o modo
/// não é um chip morto.
///
/// Em repouso ambos são a identidade, então o artista pode trocar de gesto sem que nada se mova. Com
/// a MESMA gaiola de lados retos fora do repouso (um trapézio), a homografia mantém as retas
/// interiores retas e o bilinear as encurva — a diferença tem de ser VISÍVEL, não epsilon. Se este
/// gate ficasse verde nos dois ramos, um dos dois mapas seria supérfluo.
#[test]
fn perspective_and_mesh_agree_at_rest_and_differ_off_it() {
    let origin = [0.0, 0.0];
    let size = [10.0, 6.0];
    let center = [5.0, 3.0];

    let rest = bbox_corners(origin, size);
    let q = QuadWarp::new(origin, size, rest).unwrap();
    let c = CoonsWarp::new(origin, size, rest, &rest_edges(&rest)).unwrap();
    let (a, b) = (q.map(center), c.map(center));
    assert!(
        (a[0] - b[0]).abs() < 1e-12 && (a[1] - b[1]).abs() < 1e-12,
        "em repouso os dois gestos deviam concordar: {a:?} vs {b:?}"
    );

    // Trapézio (topo estreito) = perspectiva forte, lados ainda RETOS.
    let trap = [[0.0, 0.0], [10.0, 0.0], [7.0, 6.0], [3.0, 6.0]];
    let q = QuadWarp::new(origin, size, trap).unwrap();
    let c = CoonsWarp::new(origin, size, trap, &rest_edges(&trap)).unwrap();
    let (a, b) = (q.map(center), c.map(center));
    let gap = (a[0] - b[0]).hypot(a[1] - b[1]);
    assert!(
        gap > 0.1,
        "projetivo e bilinear deviam divergir no miolo do trapézio, mas o vão é {gap:.3e} \
         ({a:?} vs {b:?})"
    );
}
