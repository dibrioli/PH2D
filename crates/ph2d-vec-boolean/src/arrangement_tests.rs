//! Gates do arranjo planar — as faces que o Shape Builder pinta.
//!
//! O que estes provam, em ordem de importância:
//!
//! 1. **A face sob o cursor é a face certa.** É o único fato que o usuário percebe: ele
//!    aponta uma região e espera que aquela pisque.
//! 2. **Duas ilhas da MESMA pertinência são faces DIFERENTES.** É o caso que separa este
//!    desenho de um predicado sobre winding number — e é a razão de `FaceId` carregar uma
//!    componente além do bitmask.
//! 3. **A soma das faces é o todo, e elas não se sobrepõem.** É a definição de arranjo, e
//!    se ela quebra, o Shape Builder produz geometria dupla ou buraco.

use super::*;
use ph2d_vec_scene::{VecPath, VecVertex, contains_point};

/// Um quadrado eixo-alinhado, fechado, em MUNDO.
fn square(x0: f64, y0: f64, x1: f64, y1: f64) -> VecPath {
    VecPath {
        verts: [[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

/// Duas caixas que se sobrepõem no meio: A = [0,10]², B = [6,16]².
/// Faces: só-A · A∩B · só-B.
fn two_overlapping() -> Arrangement {
    Arrangement::new(vec![
        square(0.0, 0.0, 10.0, 10.0),
        square(6.0, 0.0, 16.0, 10.0),
    ])
}

#[test]
fn the_membership_of_a_point_is_which_shapes_cover_it() {
    let a = two_overlapping();
    assert_eq!(a.membership_at([3.0, 5.0]), 0b01, "só na A");
    assert_eq!(a.membership_at([8.0, 5.0]), 0b11, "nas duas");
    assert_eq!(a.membership_at([13.0, 5.0]), 0b10, "só na B");
    assert_eq!(a.membership_at([50.0, 50.0]), 0, "fora de tudo");
}

/// **O fato que o usuário percebe:** apontar uma região devolve AQUELA face, e a geometria
/// dela contém o ponto apontado. Se isto quebra, o realce pisca no lugar errado.
#[test]
fn the_face_under_the_cursor_is_the_region_that_contains_the_cursor() {
    let mut a = two_overlapping();
    for p in [[3.0, 5.0], [8.0, 5.0], [13.0, 5.0]] {
        let f = a
            .face_at(p)
            .unwrap_or_else(|| panic!("{p:?} está dentro de algo"));
        let path = a.face_path(f).expect("a face tem geometria").clone();
        assert!(
            contains_point(&path, p),
            "a face devolvida para {p:?} não contém {p:?}"
        );
    }
    assert!(a.face_at([50.0, 50.0]).is_none(), "fora de tudo não é face");
}

/// A face da INTERSEÇÃO é exatamente a caixa [6,10]×[0,10] — nem a mais, nem a menos.
/// (Um erro de sinal no `compute_region` daria a UNIÃO, e o teste acima ainda passaria:
/// a união também contém o ponto.)
#[test]
fn the_intersection_face_is_the_overlap_and_nothing_more() {
    let mut a = two_overlapping();
    let f = a.face_at([8.0, 5.0]).unwrap();
    let path = a.face_path(f).unwrap().clone();
    // Dentro da sobreposição: sim. Fora dela (mas dentro de A ou de B): NÃO.
    assert!(contains_point(&path, [8.0, 5.0]), "o miolo da sobreposição");
    assert!(
        !contains_point(&path, [3.0, 5.0]),
        "a parte só-A não é da face"
    );
    assert!(!contains_point(&path, [13.0, 5.0]), "a parte só-B tampouco");
}

/// **O caso que justifica o `component` no `FaceId`.**
///
/// Três caixas em fila: A e C mordem as pontas de B, e não se tocam. A região "dentro de B
/// e de mais ninguém" fica partida em **DUAS ILHAS** — uma de cada lado do meio de B.
///
/// Elas têm a MESMA pertinência (`{B}`). Um predicado sobre o winding number pegaria as
/// duas de uma vez, e o usuário não conseguiria pintar uma sem a outra. Aqui elas são faces
/// distintas, e apontar uma devolve só aquela.
#[test]
fn two_islands_of_the_same_membership_are_two_different_faces() {
    // Uma fita larga, atravessada de lado a lado por uma barra vertical. A região "só na
    // fita" fica cortada em DUAS ilhas — uma de cada lado da barra.
    let b = square(0.0, 0.0, 30.0, 10.0); // a fita
    let cut = square(12.0, -5.0, 18.0, 15.0); // a barra que a atravessa
    let mut a = Arrangement::new(vec![b, cut]);

    let left = a.face_at([5.0, 5.0]).expect("ilha esquerda");
    let right = a.face_at([25.0, 5.0]).expect("ilha direita");

    assert_eq!(
        left.membership, right.membership,
        "as duas ilhas têm a MESMA pertinência (só a fita)"
    );
    assert_ne!(
        left, right,
        "…e mesmo assim são faces DIFERENTES — é para isso que serve o `component`"
    );

    // E a geometria de cada uma contém só o seu lado.
    let lp = a.face_path(left).unwrap().clone();
    let rp = a.face_path(right).unwrap().clone();
    assert!(contains_point(&lp, [5.0, 5.0]) && !contains_point(&lp, [25.0, 5.0]));
    assert!(contains_point(&rp, [25.0, 5.0]) && !contains_point(&rp, [5.0, 5.0]));
}

/// **As faces PARTICIONAM o todo:** todo ponto dentro de alguma forma pertence a
/// exatamente UMA face. Se duas faces se sobrepusessem, o Shape Builder somaria geometria
/// duas vezes; se deixassem lacuna, ele abriria buraco.
///
/// A varredura é uma grade densa sobre três caixas que se cruzam de todo jeito — o tipo de
/// combinação que um teste de duas formas nunca alcança.
#[test]
fn every_covered_point_belongs_to_exactly_one_face() {
    let mut arr = Arrangement::new(vec![
        square(0.0, 0.0, 12.0, 12.0),
        square(6.0, 6.0, 18.0, 18.0),
        square(4.0, 9.0, 20.0, 13.0),
    ]);
    // Os pontos da varredura, desalinhados das bordas (um ponto EM cima de uma borda é
    // ambíguo por definição — o dedo nunca acerta uma borda exatamente).
    let pts: Vec<[f64; 2]> = (0..60)
        .flat_map(|i| (0..60).map(move |j| [i as f64 * 0.37 + 0.13, j as f64 * 0.37 + 0.11]))
        .collect();

    // Primeiro descobre TODAS as faces que a varredura alcança.
    let mut faces: Vec<FaceId> = Vec::new();
    for &p in &pts {
        if let Some(f) = arr.face_at(p)
            && !faces.contains(&f)
        {
            faces.push(f);
        }
    }
    assert!(
        faces.len() >= 6,
        "3 caixas cruzadas dão ≥6 faces, deu {}",
        faces.len()
    );

    // A geometria de cada uma, de uma vez (o `arr` é `&mut` no `face_path`).
    let geom: Vec<(FaceId, VecPath)> = faces
        .iter()
        .map(|&f| (f, arr.face_path(f).expect("a face tem geometria").clone()))
        .collect();

    // **EXATAMENTE UMA.** É esta a asserção que o nome do teste promete, e sem ela o teste
    // passa com o `compute_region` devolvendo a forma INTEIRA em vez da face (aí o ponto da
    // sobreposição estaria dentro de duas faces ao mesmo tempo, e ninguém veria).
    let mut covered = 0;
    for &p in &pts {
        let hits: Vec<FaceId> = geom
            .iter()
            .filter(|(_, path)| contains_point(path, p))
            .map(|(f, _)| *f)
            .collect();
        let Some(f) = arr.face_at(p) else {
            assert!(
                hits.is_empty(),
                "{p:?} está fora de tudo, mas caiu em {hits:?}"
            );
            continue;
        };
        covered += 1;
        assert_eq!(
            hits.as_slice(),
            [f].as_slice(),
            "{p:?} devia estar em EXATAMENTE uma face ({f:?}), e está em {hits:?}"
        );
    }
    assert!(
        covered > 500,
        "a varredura cobriu {covered} pontos — poucos?"
    );
}

/// **A face "só na A" EXCLUI a sobreposição** — é a subtração do `compute_region`, e sem
/// ela a face de uma forma seria a forma INTEIRA.
///
/// Este gate faltava, e a falta era invisível: eu testei que a face da INTERSEÇÃO está
/// certa (e ela não precisa de subtração nenhuma — não há nada "fora" dela para subtrair).
/// Removi a subtração de propósito e cinco dos seis testes seguiram verdes. É o gate que
/// morde o miolo do arranjo.
#[test]
fn the_face_of_one_shape_alone_excludes_the_part_the_other_covers() {
    let mut a = two_overlapping();
    let f = a.face_at([3.0, 5.0]).expect("dentro da A, fora da B");
    assert_eq!(f.membership, 0b01);
    let path = a.face_path(f).unwrap().clone();
    assert!(contains_point(&path, [3.0, 5.0]), "a parte só-A é da face");
    assert!(
        !contains_point(&path, [8.0, 5.0]),
        "a SOBREPOSIÇÃO não é da face só-A — a face é a forma MENOS o que a outra cobre"
    );
    assert!(
        !contains_point(&path, [13.0, 5.0]),
        "e a parte só-B tampouco"
    );
}

/// Uma forma SOZINHA é uma face só (ela mesma). O `apply_many` exige dois paths, e sem esta
/// guarda o `compute_region` devolveria vazio — e o Shape Builder não realçaria nada.
#[test]
fn a_lone_shape_is_a_single_face_that_is_the_shape_itself() {
    let mut a = Arrangement::new(vec![square(0.0, 0.0, 10.0, 10.0)]);
    let f = a.face_at([5.0, 5.0]).expect("a forma sozinha é uma face");
    assert_eq!(f.membership, 0b1);
    assert_eq!(f.component, 0);
    assert_eq!(a.region(0b1).len(), 1);
}
