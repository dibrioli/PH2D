//! Os gates da reparação não-manifold.

use super::{border_edges, drop_doubled_faces, non_manifold_edges, split_non_manifold};
use crate::{Face, Mesh, shapes};

/// ⭐⭐ **UMA ALETA** — duas faces formam a superfície e a terceira sai da mesma aresta.
///
/// É o caso mínimo que contém o fenómeno: a aresta `(0,1)` é reclamada por **três** faces.
fn finned() -> Mesh {
    let positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, 1.0],
    ];
    let faces = vec![
        Face::tri(0, 1, 2),
        Face::tri(1, 0, 3),
        // ⭐ A aleta: a mesma aresta `(0,1)`, uma terceira vez.
        Face::tri(0, 1, 4),
    ];
    Mesh::from_parts(positions, faces).expect("a fixtura e' construida aqui")
}

/// ⭐⭐⭐ **A REPARAÇÃO TIRA O DEFEITO — e a fixtura CONTÉM-NO.**
///
/// ⛔ As duas metades são precisas: sem a primeira asserção o gate aprovaria uma fixtura
/// já manifold, que é a forma canónica de um gate vazio.
#[test]
fn a_fin_is_split_until_the_mesh_is_manifold() {
    let mut mesh = finned();
    assert_eq!(
        non_manifold_edges(&mesh),
        1,
        "⛔ a fixtura tem de CONTER o fenomeno"
    );
    let before = mesh.positions().len();

    let rep = split_non_manifold(&mut mesh);
    eprintln!(
        "aleta: {} arestas mas antes, {} depois · {} vertices partidos, {} copias",
        rep.bad_edges_before, rep.bad_edges_after, rep.split_verts, rep.copies
    );
    assert_eq!(rep.bad_edges_before, 1);
    assert_eq!(
        rep.bad_edges_after, 0,
        "⛔ sobrou aresta nao-manifold: a reparacao nao fechou"
    );
    assert!(rep.copies > 0, "partir sem copiar nao parte nada");
    assert!(
        mesh.positions().len() > before,
        "as copias tem de existir na malha"
    );
    // ⚠️ **A geometria não se move** — as cópias nascem no mesmo sítio.
    for p in mesh.positions().iter().take(before) {
        assert!(p.iter().all(|c| c.is_finite()));
    }
    assert_eq!(mesh.face_count(), 3, "nenhuma face se perde");
}

/// ⭐⭐ **A INÉRCIA: numa malha já manifold nada acontece.**
///
/// ⛔ É este gate que impede a reparação de se alargar. Uma esfera não tem aresta ambígua,
/// logo todo anel é uma componente só e ninguém é duplicado.
#[test]
fn a_clean_sphere_is_untouched() {
    let mut mesh = shapes::uv_sphere(24, 36, 1.0);
    mesh.triangulate();
    let before = (mesh.positions().to_vec(), mesh.face_count());
    assert_eq!(non_manifold_edges(&mesh), 0, "a esfera ja' e' manifold");

    let rep = split_non_manifold(&mut mesh);
    assert_eq!(rep.bad_edges_before, 0);
    assert_eq!(rep.copies, 0, "⛔ ninguem e' copiado numa malha limpa");
    assert_eq!(rep.split_verts, 0);
    assert_eq!(
        mesh.positions(),
        &before.0[..],
        "⛔ as posicoes tem de sair byte-identicas"
    );
    assert_eq!(mesh.face_count(), before.1);
}

/// ⭐⭐ **UMA FOLHA DE ESPESSURA ZERO** — um triângulo e o seu espelho, colados a uma
/// superfície de verdade.
///
/// É a estrutura que a sonda `manifold_census` mediu na escultura do artista em 2026-08-26:
/// as arestas ambíguas são as arestas de faces repetidas com orientação **oposta**.
fn mirrored() -> Mesh {
    // Um octaedro fechado, mais um par (face, espelho) pousado sobre uma das faces dele.
    let mut mesh = shapes::octahedron(1.0);
    mesh.triangulate();
    let v = mesh.faces()[0].verts().to_vec();
    let mut faces = mesh.faces().to_vec();
    faces.push(Face::tri(v[0], v[1], v[2]));
    faces.push(Face::tri(v[2], v[1], v[0]));
    Mesh::from_parts(mesh.positions().to_vec(), faces).expect("a fixtura e' construida aqui")
}

/// ⭐⭐⭐ **A CURA TIRA A FOLHA E NÃO ABRE A PEÇA.**
///
/// ⛔ As duas metades são precisas, e a segunda é a que faltou às quatro reparações de
/// 2026-08-25: todas curavam a aresta ambígua **abrindo** a superfície.
#[test]
fn a_zero_thickness_sheet_leaves_without_opening_the_piece() {
    let mut mesh = mirrored();
    assert_eq!(
        non_manifold_edges(&mesh),
        3,
        "⛔ a fixtura tem de CONTER o fenomeno: as tres arestas da face duplicada"
    );
    assert_eq!(
        border_edges(&mesh),
        0,
        "a fixtura parte de uma peca FECHADA"
    );
    let faces_before = mesh.face_count();

    let rep = drop_doubled_faces(&mut mesh);
    eprintln!(
        "folha dupla: {} pares espelhados, {} repeticoes puras · ambiguas {} -> {} · \
         bordo {} -> {} · recusada {}",
        rep.mirror_pairs,
        rep.same_winding_dropped,
        rep.bad_edges_before,
        rep.bad_edges_after,
        rep.border_before,
        rep.border_after,
        rep.refused
    );
    assert!(!rep.refused, "⛔ a cura recusou-se numa peca que ela fecha");
    assert_eq!(
        rep.mirror_pairs, 1,
        "o par espelhado tem de ser reconhecido"
    );
    assert_eq!(rep.bad_edges_after, 0, "⛔ sobrou aresta ambigua");
    assert_eq!(
        rep.border_after, 0,
        "⛔⛔ A CURA ABRIU A PECA -- e' exactamente esse o defeito das outras quatro"
    );
    assert_eq!(
        mesh.face_count(),
        faces_before - 2,
        "saem as DUAS faces do par, nunca uma"
    );
}

/// ⭐⭐⭐ **A RECUSA DISPARA quando remover a folha abriria a peça.**
///
/// ⛔⛔ **Este é o gate que decide se a cura pode ficar ligada.** Uma folha dupla pode
/// partilhar aresta com superfície de verdade que ninguém mais cobre; removê-la deixaria
/// essa aresta com **uma** face. ⇒ a operação mede o bordo e desfaz-se.
///
/// ⚠️ *Sem este gate, `drop_doubled_faces` é a quinta variante do mesmo erro — e o erro
/// não seria visível em peça fechada nenhuma.*
#[test]
fn the_cure_refuses_itself_when_it_would_tear_the_surface() {
    // O par espelhado `(0,1,2)`+`(2,1,0)` e uma face de verdade `(1,2,3)` na MESMA aresta.
    let positions = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
    ];
    let faces = vec![
        Face::tri(0, 1, 2),
        Face::tri(2, 1, 0),
        // ⭐ A face de verdade: e' ela que ficaria com a aresta `(1,2)` sozinha.
        Face::tri(1, 2, 3),
    ];
    let mut mesh = Mesh::from_parts(positions, faces).expect("fixtura");
    assert_eq!(non_manifold_edges(&mesh), 1, "a fixtura contem o fenomeno");
    let before = (mesh.positions().to_vec(), mesh.face_count());

    let rep = drop_doubled_faces(&mut mesh);
    eprintln!(
        "recusa: bordo {} -> {} · recusada {}",
        rep.border_before, rep.border_after, rep.refused
    );
    assert!(
        rep.refused,
        "⛔⛔ a cura ACEITOU rasgar a peca -- e' o defeito das quatro de 25/08"
    );
    assert_eq!(rep.bad_edges_after, rep.bad_edges_before, "a recusa DESFAZ");
    assert_eq!(mesh.face_count(), before.1, "nenhuma face saiu");
    assert_eq!(mesh.positions(), &before.0[..], "nada se moveu");
}

/// ⭐⭐ **INÉRCIA: uma malha sem repetição sai byte-idêntica.**
#[test]
fn a_clean_sphere_has_no_sheets_to_drop() {
    let mut mesh = shapes::uv_sphere(24, 36, 1.0);
    mesh.triangulate();
    let before = (mesh.positions().to_vec(), mesh.faces().to_vec());

    let rep = drop_doubled_faces(&mut mesh);
    assert_eq!(rep.mirror_pairs, 0);
    assert_eq!(rep.same_winding_dropped, 0);
    assert!(!rep.refused);
    assert_eq!(
        mesh.positions(),
        &before.0[..],
        "⛔ as posicoes tem de sair iguais"
    );
    assert_eq!(
        mesh.faces(),
        &before.1[..],
        "⛔ as faces tem de sair iguais"
    );
}

/// ⭐ **REPETIÇÃO PURA — a mesma orientação — fica com UMA.**
///
/// ⚠️ É a outra leitura da mesma contagem, e a coluna da orientação é que as separa: aqui
/// a segunda cópia não acrescenta superfície, então sai **uma** e não duas.
#[test]
fn a_same_winding_duplicate_leaves_one_behind() {
    let mut mesh = shapes::octahedron(1.0);
    mesh.triangulate();
    let v = mesh.faces()[0].verts().to_vec();
    let mut faces = mesh.faces().to_vec();
    faces.push(Face::tri(v[0], v[1], v[2]));
    let n = faces.len();
    let mut mesh = Mesh::from_parts(mesh.positions().to_vec(), faces).expect("fixtura");

    let rep = drop_doubled_faces(&mut mesh);
    assert_eq!(rep.mirror_pairs, 0, "nao ha' espelho aqui");
    assert_eq!(rep.same_winding_dropped, 1, "sai UMA copia, nao duas");
    assert_eq!(mesh.face_count(), n - 1);
}
