//! Gates de **SOLDAR** (plano 39) — a lei pura, sem cena e sem ponteiro.

use super::*;
use crate::VertexKind;

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

fn ponta(a: &(Vec<VecVertex>, bool), fim: bool) -> [f64; 2] {
    let v = &a.0;
    if fim {
        v[v.len() - 1].anchor
    } else {
        v[0].anchor
    }
}

fn quase(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-6 && (a[1] - b[1]).abs() < 1e-6
}

/// **Sem cruzamento, o contorno sai INTACTO** — e intacto quer dizer os mesmos vértices, não uma
/// reconstrução. Um caminho que não encontra ninguém não pode pagar o preço de ser cortado.
#[test]
fn a_contour_that_meets_nobody_comes_out_untouched() {
    let verts = vec![v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0)];
    let out = split_at(&verts, false, &[]);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].0, verts, "os vertices tem de ser os MESMOS");
    assert!(!out[0].1);
}

/// **ABERTO com `n` cruzamentos ⇒ `n + 1` arcos**, e as pontas vizinhas caem no MESMO sítio — que
/// é a coisa toda: o nó partilhado não é uma estrutura, é a coincidência das coordenadas.
#[test]
fn an_open_contour_splits_into_arcs_that_share_their_endpoints() {
    let verts = vec![v(0.0, 0.0), v(12.0, 0.0)];
    let out = split_at(&verts, false, &[0.25, 0.5]);
    assert_eq!(out.len(), 3, "dois cortes dao TRES arcos");
    for w in 0..out.len() - 1 {
        assert!(
            quase(ponta(&out[w], true), ponta(&out[w + 1], false)),
            "o arco {w} nao termina onde o {} comeca: {:?} contra {:?}",
            w + 1,
            ponta(&out[w], true),
            ponta(&out[w + 1], false)
        );
    }
    // …e o primeiro/último ainda são as pontas do original.
    assert!(quase(ponta(&out[0], false), [0.0, 0.0]));
    assert!(quase(ponta(&out[2], true), [12.0, 0.0]));
}

/// **FECHADO com `n` cruzamentos ⇒ `n` arcos**, e o último dá a volta pela emenda. ⚠️ Um anel com
/// **UM** corte vira **UM** arco aberto — não é degenerado, é um anel aberto.
#[test]
fn a_closed_contour_gives_one_arc_per_crossing_and_the_last_wraps() {
    let quadrado = vec![v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0), v(0.0, 10.0)];
    let dois = split_at(&quadrado, true, &[0.25, 0.75]);
    assert_eq!(dois.len(), 2, "dois cortes num anel dao DOIS arcos");
    assert!(dois.iter().all(|a| !a.1), "e os dois sao ABERTOS");
    // O fecho do ciclo: a ponta do último cai no começo do primeiro.
    assert!(quase(ponta(&dois[1], true), ponta(&dois[0], false)));

    let um = split_at(&quadrado, true, &[0.4]);
    assert_eq!(um.len(), 1, "um corte num anel da' UM arco");
    assert!(!um[0].1, "e ele e' ABERTO — o anel foi cortado");
    assert!(
        quase(ponta(&um[0], false), ponta(&um[0], true)),
        "as duas pontas do anel aberto caem no mesmo sitio"
    );
}

/// **Um corte em cima de uma ponta não conta** — ele não parte nada, e emitir um arco de
/// comprimento zero poria lixo na cena.
#[test]
fn a_cut_on_an_endpoint_produces_no_extra_arc() {
    let verts = vec![v(0.0, 0.0), v(10.0, 0.0)];
    assert_eq!(split_at(&verts, false, &[0.0, 1.0]).len(), 1);
    assert_eq!(split_at(&verts, false, &[1e-12]).len(), 1);
}

/// **Cortes repetidos contam UMA vez** — duas travessias no mesmo ponto (a mesma esquina vista por
/// duas arestas) não podem produzir um arco de comprimento zero entre elas.
#[test]
fn duplicate_cuts_collapse_into_one() {
    let verts = vec![v(0.0, 0.0), v(10.0, 0.0)];
    let out = split_at(&verts, false, &[0.5, 0.5 + 1e-12, 0.5 - 1e-12]);
    assert_eq!(out.len(), 2, "um corte so': {} arcos", out.len());
}
