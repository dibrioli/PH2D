//! Gates da JUNÇÃO ([`super`]) — o inverso do corte, e a metade REVERSE que anda com ela.

use super::*;
use crate::{Contour, VecPath, VecVertex, VertexKind};

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x - 0.3, y - 0.7],
        out_handle: [x + 0.5, y + 0.2],
        kind: VertexKind::Smooth,
        corner_radius: 0.0,
    }
}

fn open(pts: &[[f64; 2]]) -> VecPath {
    VecPath {
        verts: pts.iter().map(|p| v(p[0], p[1])).collect(),
        closed: false,
        ..VecPath::default()
    }
}

// ── FECHAR ──────────────────────────────────────────────────────────────────

#[test]
fn closing_a_path_whose_ends_are_apart_keeps_both_and_adds_the_closing_edge() {
    let mut scene = VecScene::default();
    let id = scene.push_path(open(&[[0.0, 0.0], [4.0, 0.0], [4.0, 4.0]]));

    assert!(scene.close_path(id, 0.01));

    let p = &scene.paths()[0];
    assert!(p.closed);
    assert_eq!(
        p.verts.len(),
        3,
        "ninguém foi fundido: as pontas eram longe"
    );
}

/// Pontas COINCIDENTES fundem num vértice só, e o sobrevivente herda a tangente de SAÍDA da
/// primeira — um `pop()` cego apagaria a ponta da última curva.
#[test]
fn closing_a_loop_welds_the_coincident_ends_into_one_vertex() {
    let mut scene = VecScene::default();
    let mut loop_path = open(&[[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 0.0]]);
    // ⚠️ As duas pontas COINCIDEM em anchor mas têm tangentes DIFERENTES — que é o caso real (o
    // traço sai numa direção e a última curva chega noutra). Com o helper `v()` cru elas saem
    // idênticas, e aí *herdar a saída* e *não herdar* dão o mesmo resultado: a fixture não
    // conteria o fenômeno, e a mutação que apaga a herança passaria.
    loop_path.verts[0].out_handle = [0.0, -9.0];
    let id = scene.push_path(loop_path);
    let first_out = scene.paths()[0].verts[0].out_handle;

    assert!(scene.close_path(id, 0.01));

    let p = &scene.paths()[0];
    assert!(p.closed);
    assert_eq!(p.verts.len(), 3, "a duplicata some");
    assert_eq!(
        p.verts.last().unwrap().out_handle,
        first_out,
        "a costura herda a tangente de saída do lado que sumiu"
    );
    assert_eq!(p.verts[0].anchor, [4.0, 0.0], "o 1º é o que era o 2º");
}

/// A regra dos ≥ 2 vértices mora no `set_path_closed`, e é ELE quem o `close_path` chama — este
/// gate existe para que a porta nova não invente uma segunda regra de "quando dá para fechar".
#[test]
fn closing_delegates_the_flag_rule_and_refuses_what_that_rule_refuses() {
    let mut scene = VecScene::default();
    let two = scene.push_path(open(&[[0.0, 0.0], [1.0, 0.0]]));
    assert!(scene.close_path(two, 0.01), "2 vértices: o flag permite");
    assert!(scene.paths()[0].closed);
    let one = scene.push_path(VecPath {
        verts: vec![v(5.0, 5.0)],
        closed: false,
        ..VecPath::default()
    });
    assert!(!scene.close_path(one, 0.01), "1 vértice não fecha");
    let shut = scene.push_path(VecPath {
        verts: vec![v(0.0, 0.0), v(1.0, 0.0), v(1.0, 1.0)],
        closed: true,
        ..VecPath::default()
    });
    assert!(!scene.close_path(shut, 0.01), "já estava fechado");
    assert!(!scene.close_path(999, 0.01), "id inexistente");
}

// ── SOLDAR DOIS ─────────────────────────────────────────────────────────────

#[test]
fn joining_two_paths_leaves_one_object_and_the_lower_one_survives() {
    let mut scene = VecScene::default();
    let low = scene.push_path(open(&[[0.0, 0.0], [1.0, 0.0]]));
    let high = scene.push_path(open(&[[2.0, 0.0], [3.0, 0.0]]));

    // Chamado na ordem INVERSA de propósito: quem manda é o z, não a ordem do argumento.
    let kept = scene.join_paths(high, low, &VecXforms::default(), 0.01);

    assert_eq!(kept, Some(low), "o de BAIXO sobrevive");
    assert_eq!(scene.paths().len(), 1);
    assert_eq!(scene.paths()[0].verts.len(), 4);
}

/// O par mais próximo decide a ORIENTAÇÃO. Aqui as pontas que se encaram são a PRIMEIRA de cada
/// um, então o dst tem de correr ao contrário — se a orientação for ignorada, a costura liga as
/// pontas erradas e a forma dá um nó.
#[test]
fn the_closest_endpoint_pair_decides_which_way_each_path_runs() {
    let mut scene = VecScene::default();
    let a = scene.push_path(open(&[[0.0, 0.0], [-5.0, 0.0]]));
    let b = scene.push_path(open(&[[0.1, 0.0], [5.0, 0.0]]));

    scene.join_paths(a, b, &VecXforms::default(), 0.5).unwrap();

    let xs: Vec<f64> = scene.paths()[0].verts.iter().map(|v| v.anchor[0]).collect();
    assert_eq!(
        xs,
        vec![-5.0, 0.0, 5.0],
        "a cadeia sai monotônica: as pontas que se encaravam fundiram no meio"
    );
}

/// Pontas SEPARADAS não fundem — o artista pediu para ligar, e ligar duas pontas separadas é uma
/// linha. É a distinção que o Illustrator faz, e o único parâmetro é a tolerância.
#[test]
fn ends_farther_than_the_tolerance_get_a_segment_instead_of_a_weld() {
    let mut scene = VecScene::default();
    let a = scene.push_path(open(&[[-5.0, 0.0], [0.0, 0.0]]));
    let b = scene.push_path(open(&[[2.0, 0.0], [7.0, 0.0]]));

    scene.join_paths(a, b, &VecXforms::default(), 0.01).unwrap();

    let xs: Vec<f64> = scene.paths()[0].verts.iter().map(|v| v.anchor[0]).collect();
    assert_eq!(xs, vec![-5.0, 0.0, 2.0, 7.0], "as DUAS pontas sobrevivem");
}

#[test]
fn joining_is_refused_for_a_closed_path_or_for_a_path_with_itself() {
    let mut scene = VecScene::default();
    let a = scene.push_path(open(&[[0.0, 0.0], [1.0, 0.0]]));
    let shut = scene.push_path(VecPath {
        verts: vec![v(2.0, 0.0), v(3.0, 0.0), v(3.0, 1.0)],
        closed: true,
        ..VecPath::default()
    });
    let x = VecXforms::default();
    assert_eq!(scene.join_paths(a, a, &x, 0.01), None, "consigo mesmo");
    assert_eq!(scene.join_paths(a, shut, &x, 0.01), None, "com um fechado");
    assert_eq!(scene.join_paths(a, 999, &x, 0.01), None, "com um fantasma");
    assert_eq!(scene.paths().len(), 2, "nenhuma recusa mexeu na cena");
}

// ── REVERTER ────────────────────────────────────────────────────────────────

/// A extensão do `reverse_path` a TODOS os contornos é byte-idêntica no contorno único — que é o
/// único caso que o `weld_new_shape`, seu chamador antigo, alcança.
#[test]
fn reversing_a_single_contour_path_is_what_it_always_was() {
    let mut scene = VecScene::default();
    let id = scene.push_path(open(&[[0.0, 0.0], [1.0, 2.0], [3.0, 4.0]]));
    let before = scene.paths()[0].verts.clone();

    assert!(scene.reverse_path(id));

    let after = &scene.paths()[0].verts;
    let mut expect = before;
    expect.reverse();
    for v in &mut expect {
        std::mem::swap(&mut v.in_handle, &mut v.out_handle);
    }
    assert_eq!(*after, expect);
}

/// Num COMPOUND, inverter metade dele deixaria contornos de sentidos misturados — sob `NonZero`
/// isso muda QUAL região é buraco, em silêncio.
#[test]
fn reversing_a_compound_turns_every_contour_not_just_the_primary() {
    let mut scene = VecScene::default();
    let mut p = VecPath {
        verts: vec![v(0.0, 0.0), v(4.0, 0.0), v(4.0, 4.0)],
        closed: true,
        ..VecPath::default()
    };
    p.subpaths.push(Contour::new_closed(vec![
        v(1.0, 1.0),
        v(2.0, 1.0),
        v(2.0, 2.0),
    ]));
    let id = scene.push_path(p);

    assert!(scene.reverse_path(id));

    let p = &scene.paths()[0];
    assert_eq!(p.verts[0].anchor, [4.0, 4.0], "o primário virou");
    assert_eq!(
        p.contour(1).unwrap().0[0].anchor,
        [2.0, 2.0],
        "o buraco virou junto"
    );
}
