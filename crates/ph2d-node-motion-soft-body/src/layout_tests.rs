//! Gates do ARRANJO — a camada que substituiu `rows`/`cols` por factos da forma
//! de repouso.
//!
//! Duas famílias, e a segunda é a que interessa:
//! - **a grelha não se mexeu**: cada resposta nova devolve, sobre uma malha
//!   autorada, a MESMA sequência de índices que o código anterior percorria;
//! - **a nuvem responde**: contorno, pino e regiões existem para um conjunto de
//!   pontos qualquer, e a grelha entregue pela porta volta a ser a grelha.

use super::*;
use crate::shape::ring_area;

/// O passeio que o `boundary_area` fazia à mão, escrito aqui como ORÁCULO —
/// literalmente o corpo daquela função, com `push` no lugar de `edge`.
fn walk_as_it_shipped(rows: usize, cols: usize) -> Vec<usize> {
    let at = |r: usize, c: usize| r * cols + c;
    let mut v = vec![at(0, 0)];
    for c in 1..cols {
        v.push(at(0, c));
    }
    for r in 1..rows {
        v.push(at(r, cols - 1));
    }
    for c in (0..cols - 1).rev() {
        v.push(at(rows - 1, c));
    }
    for r in (1..rows - 1).rev() {
        v.push(at(r, 0));
    }
    v
}

/// O laço `for cj { for rj { for r { for c } } } }` que o `cluster_goals_weighted`
/// tinha embutido, como oráculo das regiões da grelha.
fn nested_as_it_shipped(rows: usize, cols: usize, clusters: usize) -> Vec<Vec<usize>> {
    let (nr, nc) = counts(rows, cols, clusters);
    let mut out = Vec::new();
    for cj in 0..nc {
        let (c0, c1) = span(cj, nc, cols);
        for rj in 0..nr {
            let (r0, r1) = span(rj, nr, rows);
            let mut idx = Vec::new();
            for r in r0..r1 {
                for c in c0..c1 {
                    idx.push(r * cols + c);
                }
            }
            out.push(idx);
        }
    }
    out
}

const SIZES: &[(usize, usize)] = &[(2, 2), (4, 4), (8, 3), (3, 8), (5, 9), (16, 4), (32, 4)];

#[test]
fn the_grid_ring_is_the_walk_that_shipped() {
    for &(rows, cols) in SIZES {
        assert_eq!(
            grid_ring(rows, cols),
            walk_as_it_shipped(rows, cols),
            "{rows}x{cols}: o anel deixou de ser o passeio antigo"
        );
    }
}

#[test]
fn the_grid_buckets_are_the_nested_loop_that_shipped() {
    for &(rows, cols) in SIZES {
        for clusters in [1usize, 2, 3, 4, 8] {
            assert_eq!(
                grid_buckets(rows, cols, clusters),
                nested_as_it_shipped(rows, cols, clusters),
                "{rows}x{cols} @{clusters}: as regioes mudaram de ordem ou de membro"
            );
        }
    }
}

/// A linha de topo era `i < cols`; hoje é *o `y` máximo do repouso*. Sobre a
/// malha autorada as duas respostas são o MESMO conjunto — que é o que torna a
/// generalização uma reescrita e não uma mudança de produto.
#[test]
fn the_top_edge_of_a_grid_is_the_first_row() {
    for &(rows, cols) in SIZES {
        for spacing in [0.1f32, 0.7, 5.0] {
            let rest = grid_rest(rows, cols, spacing);
            // A fileira seguinte está a um `spacing` inteiro: meia fileira
            // apanha a 0 e deixa a 1 de fora, que é a lei antiga ao conjunto.
            let got = top_edge(&rest, spacing);
            let want: Vec<bool> = (0..rows * cols).map(|i| i < cols).collect();
            assert_eq!(got, want, "{rows}x{cols} @{spacing}: outra linha de topo");
        }
    }
}

/// ⭐ **O gate que faz a porta valer alguma coisa.** Entregar a malha autorada
/// pela porta `shape` tem de devolver o corpo autorado — e não um primo dele.
/// O contorno derivado (casco convexo com os colineares MANTIDOS) é o anel da
/// grelha, índice a índice, o que faz a área que a pressão defende ser o MESMO
/// `f32` nos dois caminhos.
#[test]
fn the_hull_of_a_grid_is_the_grid_ring() {
    for &(rows, cols) in SIZES {
        for spacing in [0.1f32, 0.7, 5.0] {
            let rest = grid_rest(rows, cols, spacing);
            assert_eq!(
                hull_ring(&rest),
                grid_ring(rows, cols),
                "{rows}x{cols} @{spacing}: o casco nao reproduz o anel da grelha"
            );
        }
    }
}

/// E a consequência que interessa ao produto: a ÁREA é o mesmo número.
///
/// ⚠️ **Não ao BIT, e a razão é a mesma que a wave do `falloff` já tinha
/// medido:** um corpo que entra pela porta **tem** de ser re-centrado (a
/// pressão escala os goals sobre o centro do quadro, e só pode tratar isso como
/// a mesma operação com o repouso na origem), e o centroide SOMADO de uma malha
/// que já está centrada não é exactamente zero — mede `−1,19e-7` numa 8×8.
/// Medido aqui: a pior diferença é de **2 ULP**. Um `assert_eq!` de bits
/// mediria a representação do centroide, não a costura.
#[test]
fn a_grid_through_the_port_defends_the_same_area() {
    let mut worst = 0.0f32;
    for &(rows, cols) in SIZES {
        let authored = BodyLayout::from_grid(rows, cols, 0.7);
        let through = BodyLayout::from_cloud(&grid_rest(rows, cols, 0.7));
        let a = ring_area(&authored.rest, authored.ring());
        let b = ring_area(&through.rest, through.ring());
        let rel = (a - b).abs() / a.abs();
        worst = worst.max(rel);
        assert!(rel < 1e-6, "{rows}x{cols}: area {a} vs {b} (rel {rel:e})");
        assert!(
            a < 0.0,
            "{rows}x{cols}: o anel devia enrolar no sentido horario"
        );
    }
    // CONTROLE de que a barra não é folgada: o erro real é ordens abaixo dela.
    assert!(worst < 1e-6, "pior erro relativo {worst:e}");
}

/// E o pino também: a nuvem que é uma grelha tem a mesma linha de topo.
#[test]
fn a_grid_through_the_port_pins_the_same_row() {
    for &(rows, cols) in SIZES {
        let through = BodyLayout::from_cloud(&grid_rest(rows, cols, 0.7));
        for i in 0..rows * cols {
            assert_eq!(
                through.is_pinned(i),
                i < cols,
                "{rows}x{cols}: particula {i} mudou de lado no pino"
            );
        }
    }
}

/// Um disco: a nuvem sem grelha nenhuma por trás. O contorno tem de fechar, com
/// área do sinal certo e da ordem de grandeza de um círculo.
#[test]
fn a_disc_gets_a_boundary_that_encloses_it() {
    let r = 3.0f32;
    let mut pts = Vec::new();
    for k in 0..64 {
        let t = k as f32 / 64.0 * core::f32::consts::TAU;
        pts.push([r * t.cos(), r * t.sin()]);
    }
    // Uns quantos pontos interiores, que não podem entrar no contorno.
    for k in 0..16 {
        let t = k as f32 / 16.0 * core::f32::consts::TAU;
        pts.push([0.5 * r * t.cos(), 0.5 * r * t.sin()]);
    }
    let lay = BodyLayout::from_cloud(&pts);
    assert_eq!(lay.ring().len(), 64, "o contorno apanhou pontos interiores");
    let area = ring_area(&lay.rest, lay.ring()).abs();
    let exact = core::f32::consts::PI * r * r;
    // Um polígono de 64 lados inscrito perde ~0,16% da área do círculo.
    let err = (area - exact).abs() / exact;
    assert!(err < 0.01, "area {area} contra {exact} (err {err})");
}

/// ⚠️ **A cobertura é o que impede uma partícula de ficar sem lei nenhuma.**
/// Toda partícula tem de cair em pelo menos uma região, e a sobreposição —
/// o mecanismo inteiro do §4.3 — tem de existir: alguém tem de estar em duas.
#[test]
fn the_cloud_regions_cover_the_body_and_overlap() {
    let mut pts = Vec::new();
    for r in 0..9 {
        for c in 0..17 {
            // Uma grelha DEFORMADA (passo irregular): nuvem de verdade, não a
            // malha disfarçada — uma fixtura regular não conteria o fenómeno.
            let x = c as f32 * 0.6 + (r % 3) as f32 * 0.17;
            let y = r as f32 * 0.5 + (c % 4) as f32 * 0.09;
            pts.push([x, y]);
        }
    }
    let lay = BodyLayout::from_cloud(&pts);
    for clusters in [1usize, 2, 3, 4, 6] {
        let buckets = lay.buckets(clusters);
        let mut hits = vec![0u32; pts.len()];
        for b in &buckets {
            assert!(
                b.windows(2).all(|w| w[0] < w[1]),
                "@{clusters}: uma regiao saiu fora de ordem crescente"
            );
            for &i in b {
                hits[i] += 1;
            }
        }
        assert!(
            hits.iter().all(|&h| h >= 1),
            "@{clusters}: particula sem regiao nenhuma"
        );
        if clusters > 1 {
            assert!(
                hits.iter().any(|&h| h >= 2),
                "@{clusters}: nenhuma sobreposicao — as bandas so ladrilham"
            );
        }
    }
}

/// O controle da contagem de bandas: uma nuvem com a forma e a densidade de uma
/// grelha tem de aceitar o mesmo `clusters` que ela — senão o mesmo knob
/// significaria duas coisas conforme a porta estivesse ligada.
#[test]
fn the_cloud_takes_the_same_cluster_count_as_the_grid_it_mimics() {
    for &(rows, cols) in &[(32usize, 4usize), (16, 8), (8, 8)] {
        let lay = BodyLayout::from_cloud(&grid_rest(rows, cols, 0.7));
        for clusters in [2usize, 3, 4] {
            let n_cloud = lay.buckets(clusters).len();
            let n_grid = BodyLayout::from_grid(rows, cols, 0.7)
                .buckets(clusters)
                .len();
            assert_eq!(
                n_cloud, n_grid,
                "{rows}x{cols} @{clusters}: {n_cloud} regioes na nuvem contra {n_grid} na grelha"
            );
        }
    }
}

/// Formas degeneradas não podem entrar em pânico nem inventar uma área: menos de
/// três pontos, e pontos todos numa linha, não fecham nada.
#[test]
fn a_degenerate_cloud_encloses_nothing() {
    for pts in [
        vec![],
        vec![[0.0f32, 0.0]],
        vec![[0.0f32, 0.0], [1.0, 1.0]],
        (0..8).map(|k| [k as f32, 2.0 * k as f32]).collect(),
    ] {
        let lay = BodyLayout::from_cloud(&pts);
        assert_eq!(
            ring_area(&lay.rest, lay.ring()),
            0.0,
            "uma nuvem degenerada devolveu area"
        );
    }
}

/// ⚠️ **A guarda dos `MIN_SPAN` membros NUNCA dispara sobre a malha autorada**, e
/// isto é o gate que o diz — sem ele a afirmação vive só num comentário, e uma
/// mutação que a removesse sobreviveria com toda a suíte verde (é o caso: o
/// caminho da grelha é byte-idêntico com ou sem ela).
///
/// A razão é estrutural: o `span` cresce cada banda de meia banda para cada
/// lado e o `counts` corta em `len / MIN_SPAN`, então nenhuma banda tem menos de
/// `MIN_SPAN` índices por eixo — logo nenhuma região tem menos de `MIN_SPAN²`
/// membros. A guarda existe para a NUVEM, onde uma banda pode calhar vazia.
#[test]
fn no_region_of_an_authored_mesh_is_too_small_to_fit() {
    for &(rows, cols) in SIZES {
        for clusters in [1usize, 2, 3, 4, 8, 16, 64] {
            for (j, b) in grid_buckets(rows, cols, clusters).iter().enumerate() {
                assert!(
                    b.len() >= 4,
                    "{rows}x{cols} @{clusters}: a regiao {j} tem {} membro(s)",
                    b.len()
                );
            }
        }
    }
}
