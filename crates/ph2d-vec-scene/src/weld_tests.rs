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

/// ⭐⭐⭐ **A FUSÃO é o que separa «dividir» de «soldar»** (report do Enio, 2026-08-31).
/// ⭐ **O aglomerado é o NÓ, e o nó é o CENTROIDE.**
#[test]
fn two_points_within_the_tolerance_become_one_node_at_their_middle() {
    let (de_quem, nos) = cluster_endpoints(&[[0.0, 0.0], [0.3, -0.2]], 1.0);
    assert_eq!(nos.len(), 1);
    assert_eq!(de_quem, vec![Some(0), Some(0)]);
    assert!((nos[0][0] - 0.15).abs() < 1e-12 && (nos[0][1] + 0.1).abs() < 1e-12);
}

/// ⛔ **Uma ponta sozinha não é junta de nada** — e é isso que impede a solda de mover uma ponta
/// livre para "o sítio dela".
#[test]
fn a_lone_point_belongs_to_no_node() {
    let (de_quem, nos) = cluster_endpoints(&[[0.0, 0.0], [50.0, 0.0]], 1.0);
    assert!(nos.is_empty());
    assert_eq!(de_quem, vec![None, None]);
}

/// ⚠️ **A folga é a cerca**: as mesmas duas pontas, com o ímã apertado, ficam duas.
#[test]
fn the_tolerance_is_a_fence_and_it_holds() {
    let (de_quem, nos) = cluster_endpoints(&[[0.0, 0.0], [0.3, -0.2]], 0.1);
    assert!(nos.is_empty());
    assert_eq!(de_quem, vec![None, None]);
}

/// ⚠️ **Dois nós distintos numa lista só** — o agrupamento não é global, é por vizinhança.
#[test]
fn two_separate_meetings_give_two_nodes() {
    let (de_quem, nos) =
        cluster_endpoints(&[[0.0, 0.0], [0.1, 0.0], [90.0, 0.0], [90.1, 0.0]], 1.0);
    assert_eq!(nos.len(), 2);
    assert_eq!(de_quem, vec![Some(0), Some(0), Some(1), Some(1)]);
}

/// ⚠️ **A alça acompanha a âncora.** Mover só a âncora mudaria a CURVA em vez de a deslocar.
#[test]
fn moving_an_endpoint_carries_its_handles() {
    let mut v = VecVertex {
        anchor: [10.0, 10.0],
        in_handle: [8.0, 10.0],
        out_handle: [12.0, 11.0],
        kind: crate::VertexKind::Smooth,
        corner_radius: 0.0,
    };
    mover_ponta(&mut v, [20.0, 30.0]);
    assert_eq!(v.anchor, [20.0, 30.0]);
    assert_eq!(v.in_handle, [18.0, 30.0]);
    assert_eq!(v.out_handle, [22.0, 31.0]);
}

/// ⛔⛔ **Um cruzamento na EMENDA de um anel é um nó, e não lixo.**
///
/// A fracção `0` de um contorno fechado é um ponto interior — a emenda não é fronteira de nada. O
/// filtro que serve a um contorno ABERTO (onde `0` e `1` são as PONTAS) apagava-o, e o anel saía
/// com **um** arco em vez de dois. Achado pelo balde (plano 40): a `ellipse` começa em
/// `(cx + r, cy)`, que é exactamente onde uma recta horizontal pelo centro a corta.
#[test]
fn a_crossing_on_the_seam_of_a_ring_still_cuts_it() {
    let anel = crate::ellipse([0.0, 0.0], 50.0, 50.0);
    let dois = split_at(&anel.verts, true, &[0.0, 0.5]);
    assert_eq!(
        dois.len(),
        2,
        "o corte na emenda foi descartado: {} arcos",
        dois.len()
    );
    assert!(dois.iter().all(|(v, c)| !c && v.len() >= 2));
    // E a mesma fracção escrita como `1.0` é o MESMO corte, não um segundo.
    let mesmo = split_at(&anel.verts, true, &[1.0, 0.5]);
    assert_eq!(mesmo.len(), 2, "`1.0` e `0.0` sao a mesma emenda");
}

/// ⚠️ **Num contorno ABERTO o filtro FICA** — ali `0` e `1` são as pontas, e cortar uma ponta
/// devolveria um arco de comprimento zero.
#[test]
fn a_cut_on_the_tip_of_an_open_path_still_splits_nothing() {
    let verts = vec![v(0.0, 0.0), v(10.0, 0.0)];
    assert_eq!(split_at(&verts, false, &[0.0, 1.0]).len(), 1);
}
