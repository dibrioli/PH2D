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
///
/// ⚠️ **A fixtura tem de ser CURVA.** Com duas retas cruzando em coordenadas redondas os dois lados
/// já calculam o MESMO ponto por acaso, e o gate passaria com a fusão desligada — foi a primeira
/// redacção deste teste, e três mutações sobreviveram a ela. *A fixtura mais azarada possível é a
/// que aprova.*
#[test]
fn fusing_turns_two_near_points_into_one_and_a_lone_end_is_not_a_joint() {
    // Duas pontas PERTO mas diferentes — o que o corte de dois contornos curvos produz.
    let mut arcos = vec![
        (vec![v(0.0, 0.0), v(10.0, 0.0)], false),
        (vec![v(10.0, 0.03), v(20.0, 5.0)], false),
        (vec![v(80.0, 80.0), v(90.0, 80.0)], false), // sozinha, longe de tudo
    ];
    let antes = (arcos[0].0[1].anchor, arcos[1].0[0].anchor);
    assert_ne!(
        antes.0, antes.1,
        "a fixtura precisa de duas pontas DIFERENTES"
    );

    let juntas = fuse_endpoints(&mut arcos, 0.1);

    assert_eq!(juntas, 1, "uma junta, e a ponta solitaria NAO conta");
    assert_eq!(
        arcos[0].0[1].anchor, arcos[1].0[0].anchor,
        "as duas pontas tem de virar o MESMO ponto, bit a bit"
    );
    // …e é o CENTROIDE, não uma das duas: escolher uma faria a solda depender da ordem.
    assert!((arcos[0].0[1].anchor[1] - 0.015).abs() < 1e-12);
    // A solitária não se mexeu.
    assert_eq!(arcos[2].0[0].anchor, [80.0, 80.0]);
}

/// ⚠️ **A alça acompanha a âncora** — mover só a âncora mudaria a CURVA em vez de a deslocar, e o
/// arco descolaria da forma que tinha.
#[test]
fn fusing_carries_the_handles_with_the_anchor() {
    let mut a = v(10.0, 0.0);
    a.out_handle = [12.0, 3.0];
    let mut arcos = vec![
        (vec![v(0.0, 0.0), a], false),
        (vec![v(10.0, 0.04), v(20.0, 0.0)], false),
    ];
    fuse_endpoints(&mut arcos, 0.1);
    let f = &arcos[0].0[1];
    assert!(
        (f.out_handle[1] - (3.0 + 0.02)).abs() < 1e-12,
        "a alca nao andou com a ancora: {:?}",
        f.out_handle
    );
}
