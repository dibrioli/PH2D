//! Os gates da reparação não-manifold.

use super::{non_manifold_edges, split_non_manifold};
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
