//! Gates da cena `=81` — **o vocabulário da utilidade** (doc 89, folha 08).
//!
//! ⚠️ **O oráculo desta cena é a FORMA, não a altura**, e é a primeira das cenas de
//! conferência em que isso é verdade: três dos quatro pares respondem *que figura
//! sai daqui*. Um gate que medisse excursão passaria com uma lente no lugar de um
//! círculo e com uma diagonal no lugar de uma espiral.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

/// As posições de uma célula, já sem o deslocamento que a coloca na grelha.
fn points(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId, cell: usize) -> Vec<[f32; 2]> {
    let mut c = Cook::new();
    let out = c.cook(&doc.graph, reg, sink, 0.0).expect("coze");
    let CookValue::Instances(s) = &out[0] else {
        panic!("stream")
    };
    let Some(Column::Vec2(v)) = Stream::get(s, "P") else {
        panic!("P")
    };
    let row = cell / 2;
    let half = cell % 2;
    let cx = if half == 0 { -COL_X } else { COL_X };
    let cy = (ROWS_TABLE.len() as f32 - 1.0) * 0.5 * ROW_GAP - row as f32 * ROW_GAP;
    v.iter().map(|p| [p[0] - cx, p[1] - cy]).collect()
}

/// A distância de cada ponto à origem da célula.
fn radii(p: &[[f32; 2]]) -> Vec<f32> {
    p.iter().map(|q| q[0].hypot(q[1])).collect()
}

fn span(v: &[f32]) -> f32 {
    v.iter().copied().fold(f32::NEG_INFINITY, f32::max)
        - v.iter().copied().fold(f32::INFINITY, f32::min)
}

/// **A cena constrói as oito células.**
#[test]
fn the_util_scene_builds_every_cell() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_util_demo_document(&mut doc, &reg).expect("a cena constroi");
    assert_eq!(sinks.len(), ROWS_TABLE.len() * 2, "duas celulas por linha");
    let (n, count, shift) = authored();
    assert_eq!(n, ROWS_TABLE.len(), "o anuncio conta a mesma tabela");
    for (k, sink) in sinks.iter().enumerate() {
        let p = points(&doc, &reg, *sink, k);
        assert_eq!(p.len(), count as usize, "celula {k}: contagem de pecas");
    }
    assert!(
        shift > 1.0 && shift < count,
        "o shift tem de ser uma rotacao real"
    );
}

/// **A MISTURA: `Avg` dá uma RETA, `Min` dá uma TENDA.**
///
/// ⚠️ O oráculo é a ALTURA DO MEIO contra as PONTAS, e não a excursão: as duas
/// figuras percorrem o mesmo Y no total, e o que as separa é onde elas o percorrem.
#[test]
fn the_average_is_a_line_and_the_min_is_a_tent() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_util_demo_document(&mut doc, &reg).expect("constroi");
    let ys = |cell: usize| -> Vec<f32> {
        points(&doc, &reg, sinks[cell], cell)
            .iter()
            .map(|p| p[1])
            .collect()
    };
    let avg = ys(0);
    let min = ys(1);
    // `Avg` de duas lanes cruzadas é PLANA — o ponto médio é o mesmo em toda peça.
    assert!(
        span(&avg) < 1e-4,
        "a media devia ser uma reta, percorreu {}",
        span(&avg)
    );
    // `Min` é uma TENDA: o meio está ACIMA das duas pontas por uma margem real.
    let mid = min[min.len() / 2];
    let ends = min[0].max(min[min.len() - 1]);
    assert!(
        mid - ends > R * 0.5,
        "a tenda tem de ter um pico: meio {mid} contra pontas {ends}"
    );
}

/// **A FORMA: misturada é uma LENTE, presa a uma lane é o CÍRCULO.**
///
/// ⚠️ **É a leitura que a folha 08 pedia, desenhada.** O oráculo é o RAIO: um
/// círculo tem raio constante, e a média de um círculo com uma reta tem-no a variar
/// — é literalmente *uma terceira forma que nenhuma das duas entradas tinha*.
#[test]
fn the_mixed_geometry_is_a_lens_and_the_pinned_one_is_the_circle() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_util_demo_document(&mut doc, &reg).expect("constroi");
    let mixed = radii(&points(&doc, &reg, sinks[2], 2));
    let pinned = radii(&points(&doc, &reg, sinks[3], 3));
    // Preso: raio constante `R` — um círculo. (A senoide parabólica fica a ~2%.)
    for r in &pinned {
        assert!(
            (r - R).abs() < R * 0.05,
            "o circulo tem raio {r}, esperava {R}"
        );
    }
    // Misturado: o raio VARIA — a figura não é um círculo nenhum.
    assert!(
        span(&mixed) > R * 0.3,
        "a mistura devia deformar o raio, variou {}",
        span(&mixed)
    );
}

/// **A ORDEM: o `shift` RODA a escada, e nenhuma peça se perde.**
///
/// ⚠️ **A segunda metade é o gate.** Uma implementação que DESLOCASSE em vez de
/// rodar desenharia uma escada parecida e teria peças a menos — e a contagem é a
/// coisa que um nó de ordem nunca pode mudar. O oráculo é o CONJUNTO das alturas:
/// ele tem de ser o mesmo nas duas metades, com os pontos noutra ordem.
#[test]
fn the_shift_rotates_the_staircase_without_losing_a_piece() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_util_demo_document(&mut doc, &reg).expect("constroi");
    // ⚠️ **O oráculo é a coluna X, e a 1ª versão deste gate mediu a Y — e reprovou
    // sobre produto correcto.** A altura é o POSTO na lista de saída, que é a rampa
    // `0..1` sempre: a rotação move *quem* está em cada altura, não as alturas. Ela
    // só aparece na relação entre o X de uma peça e a altura em que ela ficou.
    let xs = |cell: usize| -> Vec<i64> {
        points(&doc, &reg, sinks[cell], cell)
            .iter()
            .map(|p| (p[0] * 1e4).round() as i64)
            .collect()
    };
    let mut a = xs(4);
    let mut b = xs(5);
    assert_ne!(a, b, "o shift nao mudou a ordem");
    a.sort_unstable();
    b.sort_unstable();
    assert_eq!(
        a, b,
        "o conjunto de pecas tem de ser o MESMO — roda, nao perde"
    );
    // E a rotação é UMA rotação: a sequência da direita é a da esquerda rodada.
    let (l, r) = (xs(4), xs(5));
    let k = SHIFT as usize % l.len();
    let rotated: Vec<i64> = l[k..].iter().chain(&l[..k]).copied().collect();
    assert_eq!(r, rotated, "a direita tem de ser a esquerda rodada em {k}");
}

/// **O PONTO: os MESMOS números dão uma DIAGONAL e uma ESPIRAL.**
///
/// ⚠️ O oráculo da espiral é o raio CRESCENTE **mais** a volta fechada — não «não é
/// uma reta». Uma implementação que trocasse `cos` por `sin` também não seria uma
/// reta: ela desenharia a mesma espiral rodada um quarto de volta, e passaria em
/// qualquer teste de raio. O que a fixa é a peça inicial estar na origem e a figura
/// visitar os **quatro quadrantes** — uma volta inteira, coisa que nenhuma reta faz.
#[test]
fn the_same_numbers_draw_a_diagonal_and_a_spiral() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_util_demo_document(&mut doc, &reg).expect("constroi");
    let cart = points(&doc, &reg, sinks[6], 6);
    let pol = points(&doc, &reg, sinks[7], 7);
    // Cartesian: uma DIAGONAL — `y` cresce com `x`, monotonamente.
    for w in cart.windows(2) {
        assert!(
            w[1][0] >= w[0][0] - 1e-6 && w[1][1] >= w[0][1] - 1e-6,
            "nao e' diagonal"
        );
    }
    // Polar: o raio cresce de 0 a R, monotonamente…
    let r = radii(&pol);
    assert!(
        r[0].abs() < 1e-4,
        "a espiral comeca na origem, comecou em {}",
        r[0]
    );
    assert!((r[r.len() - 1] - R).abs() < R * 0.05, "e acaba no raio R");
    for w in r.windows(2) {
        assert!(w[1] >= w[0] - 1e-4, "o raio da espiral tem de crescer");
    }
    // …e ela DÁ A VOLTA: a espiral visita os QUATRO quadrantes, coisa que uma
    // diagonal nunca faz. ⚠️ É um oráculo melhor que contar cruzamentos de eixo:
    // uma reta inclinada ao contrário também cruza um eixo.
    let quadrants = |v: &[[f32; 2]]| -> usize {
        let mut seen = [false; 4];
        for q in v.iter().skip(v.len() / 8) {
            seen[usize::from(q[0] < 0.0) + 2 * usize::from(q[1] < 0.0)] = true;
        }
        seen.iter().filter(|b| **b).count()
    };
    assert_eq!(
        quadrants(&pol),
        4,
        "a espiral tem de visitar os quatro quadrantes"
    );
    assert!(quadrants(&cart) <= 2, "a diagonal fica num canto");
}

/// **NENHUMA CÉLULA INVADE A VIZINHA** — a lei de layout das cenas irmãs, aqui em
/// dois eixos porque a cena é uma grelha.
#[test]
fn no_cell_climbs_into_its_neighbour() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_util_demo_document(&mut doc, &reg).expect("constroi");
    for (k, sink) in sinks.iter().enumerate() {
        let p = points(&doc, &reg, *sink, k);
        let (mut mx, mut my) = (0.0f32, 0.0f32);
        for q in &p {
            mx = mx.max(q[0].abs());
            my = my.max(q[1].abs());
        }
        assert!(
            my < ROW_GAP * 0.5,
            "celula {k} sobe {my}, meia linha e' {}",
            ROW_GAP * 0.5
        );
        assert!(
            mx < COL_X,
            "celula {k} alarga {mx}, a coluna vive a {COL_X}"
        );
    }
}
