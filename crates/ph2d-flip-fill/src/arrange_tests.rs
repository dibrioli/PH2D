//! Gates da R0. A pergunta que eles decidem é a do §8 do plano: **o percurso topológico
//! sobrevive a arte de mão em `f32`?**
//!
//! O oráculo de todos é o mesmo, e é o que a wave promete: **cada vértice do anel ou é um
//! vértice de uma linha, ou é um ponto que está EM CIMA de duas linhas** (uma interseção).
//! Nada mais pode aparecer ali — é isso que separa esta abordagem do contorno vetorizado.

use super::{Region, region_at};
use crate::Vec2;

type Line = (Vec<Vec2>, Vec<f32>, bool);

fn line(pts: &[(f32, f32)], closed: bool) -> Line {
    (
        pts.iter().map(|&(x, y)| Vec2::new(x, y)).collect(),
        vec![0.5; pts.len()],
        closed,
    )
}

/// Distância de `p` ao segmento `a→b`.
fn dist_seg(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = Vec2::new(b.x - a.x, b.y - a.y);
    let l2 = ab.x * ab.x + ab.y * ab.y;
    let t = if l2 <= 0.0 {
        0.0
    } else {
        (((p.x - a.x) * ab.x + (p.y - a.y) * ab.y) / l2).clamp(0.0, 1.0)
    };
    let (dx, dy) = (p.x - (a.x + t * ab.x), p.y - (a.y + t * ab.y));
    (dx * dx + dy * dy).sqrt()
}

/// **O ORÁCULO da wave:** todo ponto do anel está sobre alguma linha de entrada.
///
/// O contorno vetorizado de hoje NÃO passa nisto — ele chanfra as quinas e desliza nas
/// retas, então os pontos dele caem fora das linhas. É a diferença que o Enio viu entre o
/// balde e o Draw:Filled, transformada em asserção.
fn every_vertex_lies_on_a_line(r: &Region, lines: &[Line], tol: f32) {
    for (k, p) in r.outer.iter().enumerate() {
        let mut best = f32::MAX;
        for (pts, _, closed) in lines {
            let n = pts.len();
            let last = if *closed { n } else { n - 1 };
            for i in 0..last {
                best = best.min(dist_seg(*p, pts[i], pts[(i + 1) % n]));
            }
        }
        assert!(
            best <= tol,
            "vértice {k} do anel ({}, {}) está a {best} de QUALQUER linha — \
             a malha do fill inventou geometria, que é exatamente o defeito da wave",
            p.x,
            p.y
        );
    }
}

/// **A GRADE — a arte do smoke do Enio.** Duas linhas horizontais e duas verticais; o
/// clique cai no quadrado do meio.
///
/// É o caso que o `filled_shape_target` NÃO alcança (a região não é o interior de traço
/// nenhum) e que hoje cai obrigatoriamente na rota vetorizada.
#[test]
fn the_grid_cell_is_bounded_by_the_lines_own_vertices() {
    let lines = vec![
        line(&[(-10.0, 0.0), (110.0, 0.0)], false),
        line(&[(-10.0, 100.0), (110.0, 100.0)], false),
        line(&[(0.0, -10.0), (0.0, 110.0)], false),
        line(&[(100.0, -10.0), (100.0, 110.0)], false),
    ];
    let r = region_at(&lines, Vec2::new(50.0, 50.0)).expect("a célula central é limitada");
    every_vertex_lies_on_a_line(&r, &lines, 1e-3);

    let a = crate::signed_area(&r.outer).abs();
    assert!(
        (a - 10_000.0).abs() < 1.0,
        "a célula é 100x100 = 10.000 de área; veio {a}"
    );
    assert_eq!(
        r.outer.len(),
        4,
        "quatro linhas retas fazem um quadrado de QUATRO cantos, e cada canto é uma \
         interseção real — nenhum ponto a mais: {:?}",
        r.outer
    );
}

/// **A célula do meio, com as linhas TRÊMULAS** — o caso de verdade, e o que o §8 do plano
/// usa como critério de morte: o percurso tem de sobreviver a `f32` sobre arte de mão.
#[test]
fn a_hand_drawn_grid_still_closes_a_ring_of_the_lines_own_vertices() {
    let wobble = |i: usize| ((i as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
    let h = |y: f32, s: usize| -> Line {
        line(
            &(0..40)
                .map(|i| (-10.0 + i as f32 * 3.0, y + wobble(i + s) * 3.0))
                .collect::<Vec<_>>(),
            false,
        )
    };
    let v = |x: f32, s: usize| -> Line {
        line(
            &(0..40)
                .map(|i| (x + wobble(i + s) * 3.0, -10.0 + i as f32 * 3.0))
                .collect::<Vec<_>>(),
            false,
        )
    };
    let lines = vec![h(0.0, 0), h(100.0, 7), v(0.0, 13), v(100.0, 29)];

    let r = region_at(&lines, Vec2::new(50.0, 50.0)).expect("a célula trêmula é limitada");
    every_vertex_lies_on_a_line(&r, &lines, 1e-2);

    let a = crate::signed_area(&r.outer).abs();
    assert!(
        (7_000.0..13_000.0).contains(&a),
        "a célula trêmula deveria ter ~10.000 de área; veio {a}"
    );
    // E o ponto todo da wave: o anel tem de carregar os vértices DA MÃO, não 4 cantos.
    assert!(
        r.outer.len() > 40,
        "o anel deveria carregar os vértices trêmulos das linhas; veio {} pontos",
        r.outer.len()
    );
}

/// **Arte com VÃO ⇒ `None`.** É a divisão de trabalho do §3: o arranjo não fecha uma face
/// que não está fechada, e a resposta honesta é entregar o caso ao raster.
///
/// Sem isto o motor devolveria a face EXTERNA (um anel que engloba o desenho inteiro) e a
/// cor vazaria para a tela toda — o modo de falha mais caro possível.
#[test]
fn an_open_region_is_refused_so_the_raster_can_take_it() {
    let lines = vec![
        line(&[(-10.0, 0.0), (110.0, 0.0)], false),
        line(&[(-10.0, 100.0), (110.0, 100.0)], false),
        line(&[(0.0, -10.0), (0.0, 110.0)], false),
        // A 4ª parede PARA antes de encostar: o vão.
        line(&[(100.0, -10.0), (100.0, 40.0)], false),
    ];
    assert!(
        region_at(&lines, Vec2::new(50.0, 50.0)).is_none(),
        "região aberta tem de ser RECUSADA — devolver a face externa pintaria a tela toda"
    );
}

/// **Uma forma fechada sozinha** — o caso que o `filled_shape_target` já resolve hoje.
/// Aqui ele tem de sair do MESMO motor, senão a R3 (aposentar o ramo especial) não fecha.
#[test]
fn a_single_closed_shape_comes_out_of_the_same_engine() {
    let n = 24;
    let ring: Vec<(f32, f32)> = (0..n)
        .map(|i| {
            let a = i as f32 / n as f32 * std::f32::consts::TAU;
            (50.0 + 40.0 * a.cos(), 50.0 + 40.0 * a.sin())
        })
        .collect();
    let lines = vec![line(&ring, true)];
    let r = region_at(&lines, Vec2::new(50.0, 50.0)).expect("o interior do círculo");
    every_vertex_lies_on_a_line(&r, &lines, 1e-3);
    assert_eq!(
        r.outer.len(),
        n,
        "o anel tem de ser os {n} vértices do próprio traço — nem um a mais"
    );
}

/// **A QUINA AGUDA** — a fixture que matou a abordagem anterior (BUGS #16: *"os dois lados
/// do bico estão à mesma distância, a projeção alterna entre eles"*).
///
/// Aqui ela tem de passar, e por uma razão estrutural: o percurso nunca pergunta
/// distância. Se este gate falhar, a wave morre com a mesma causa da tentativa de 2026-07-13.
#[test]
fn a_sharp_spike_does_not_collapse_the_ring() {
    // Um "V" muito agudo fechado por uma tampa: a região é fina e tem um bico.
    let lines = vec![
        line(
            &[(0.0, 0.0), (50.0, 100.0), (52.0, 100.0), (2.0, 0.0)],
            true,
        ),
        line(&[(-10.0, 5.0), (60.0, 5.0)], false),
    ];
    let r = region_at(&lines, Vec2::new(26.0, 50.0)).expect("a região do bico");
    every_vertex_lies_on_a_line(&r, &lines, 1e-2);
    let a = crate::signed_area(&r.outer).abs();
    assert!(
        a > 1.0,
        "o anel colapsou num nó de área ~zero ({a}) — é o defeito do BUGS #16 de volta"
    );
}

/// **Formas ANINHADAS: a face certa é a MENOR que contém o clique.**
///
/// Um quadrado dentro de outro. Um clique no miolo está dentro dos DOIS anéis, e a região
/// que o usuário quis é a interna — a maior seria "o quadrado grande inteiro", pintando
/// por cima do pequeno.
///
/// ⚠️ Este gate nasceu de uma **mutação sobrevivente**: trocar `menor área` por `maior`
/// não derrubava nenhum dos 5 primeiros, porque em todos eles **uma única face** continha
/// a semente — a regra do mínimo nunca era exercida. Fixture que não contém o fenômeno não
/// prova a regra que fala dele.
#[test]
fn nested_shapes_pick_the_innermost_face() {
    let outer = line(
        &[(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)],
        true,
    );
    let inner = line(
        &[(30.0, 30.0), (70.0, 30.0), (70.0, 70.0), (30.0, 70.0)],
        true,
    );
    let lines = vec![outer, inner];

    let r = region_at(&lines, Vec2::new(50.0, 50.0)).expect("o miolo do quadrado interno");
    every_vertex_lies_on_a_line(&r, &lines, 1e-3);
    let a = crate::signed_area(&r.outer).abs();
    assert!(
        (a - 1_600.0).abs() < 1.0,
        "o clique no miolo tem de dar o quadrado INTERNO (40x40 = 1.600); veio {a} \
         (10.000 = o externo, que pintaria por cima do interno)"
    );

    // E o clique ENTRE os dois dá o anel — a face do meio, que também é a menor que o
    // contém.
    let r2 = region_at(&lines, Vec2::new(15.0, 50.0)).expect("a faixa entre os quadrados");
    let a2 = crate::signed_area(&r2.outer).abs();
    assert!(
        (a2 - 10_000.0).abs() < 1.0,
        "entre os dois, a face é delimitada pelo quadrado externo; veio {a2}"
    );
}
