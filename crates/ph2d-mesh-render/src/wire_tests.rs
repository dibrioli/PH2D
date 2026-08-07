//! Gates da lista de arestas.

use super::*;
use ph2d_mesh::shapes;
use std::collections::BTreeSet;

/// O conjunto de arestas que a lista descreve, normalizado.
fn edge_set(out: &[u32]) -> BTreeSet<(u32, u32)> {
    out.chunks_exact(2)
        .map(|p| (p[0].min(p[1]), p[0].max(p[1])))
        .collect()
}

/// **Cada aresta aparece UMA vez** — o gate da regra de posse.
///
/// ⚠️ O oráculo não é a regra (`b > a`), é a CONTAGEM: uma lista com duplicatas
/// tem mais pares que arestas distintas, e é isso que se afirma. Escrever
/// *"todo par tem `b > a`"* seria o teste espelhando a implementação.
#[test]
fn every_edge_is_drawn_exactly_once() {
    let mesh = shapes::uv_sphere(16, 24, 1.0);
    let mut out = Vec::new();
    wire_indices(&mesh, &mut out);

    assert_eq!(out.len() % 2, 0, "a lista tem de ser pares de índices");
    assert_eq!(
        out.len() / 2,
        edge_set(&out).len(),
        "há arestas repetidas: {} pares para {} arestas distintas",
        out.len() / 2,
        edge_set(&out).len()
    );
}

/// **A lista contém toda aresta que as FACES usam** — o gate do outro lado.
///
/// ⚠️ Sem ele o gate acima é satisfeito por uma lista VAZIA: zero pares e zero
/// arestas distintas casam. O oráculo aqui é independente da porta — ele
/// reconstrói o conjunto varrendo as faces, que é a definição de *"as arestas
/// desta malha"*.
#[test]
fn no_edge_of_any_face_is_missing() {
    let mesh = shapes::uv_sphere(16, 24, 1.0);
    let mut out = Vec::new();
    wire_indices(&mesh, &mut out);
    let drawn = edge_set(&out);

    let mut from_faces = BTreeSet::new();
    for f in mesh.faces() {
        let vs = f.verts();
        for k in 0..vs.len() {
            let (a, b) = (vs[k], vs[(k + 1) % vs.len()]);
            from_faces.insert((a.min(b), a.max(b)));
        }
    }
    assert!(!from_faces.is_empty(), "a fixture não tem faces");
    assert_eq!(
        drawn,
        from_faces,
        "a lista de arestas e as faces discordam ({} contra {})",
        drawn.len(),
        from_faces.len()
    );
}

/// **A contagem obedece a EULER**, que é o que torna o custo previsível.
///
/// ⚠️ **A primeira versão deste gate afirmava `E = 3V − 6` e nasceu VERMELHA
/// sobre código correto** (744 arestas contra as 1080 que ela previa): aquela
/// fórmula vale para malha TRIANGULAR, e a esfera deste repo é **mista** —
/// quads no corpo, triângulos nos dois polos. O gate estava medindo uma
/// premissa que a fixture não tem, e quem estava errado era a minha aritmética.
///
/// O invariante honesto é o de Euler cru, `V − E + F = 2`, que não sabe o grau
/// das faces. E ele ainda sustenta a nota de custo do módulo, por outro
/// caminho: numa superfície fechada `2E = Σ grau ≥ 3F` ⇒ `E ≤ 3V − 6`, então
/// **24 bytes por vértice é o TETO**, e uma malha com quads paga menos.
#[test]
fn the_edge_count_obeys_euler_and_stays_under_three_per_vertex() {
    let mesh = shapes::uv_sphere(16, 24, 1.0);
    let mut out = Vec::new();
    wire_indices(&mesh, &mut out);
    let (v, f, e) = (mesh.vert_count(), mesh.face_count(), out.len() / 2);
    assert_eq!(
        v + f,
        e + 2,
        "V({v}) - E({e}) + F({f}) != 2 — a lista não descreve uma superfície fechada"
    );
    assert!(
        e <= 3 * v - 6,
        "{e} arestas passam do teto de 3V-6 ({}) que o custo do módulo declara",
        3 * v - 6
    );
}

/// **Um rascunho reusado não acumula.**
#[test]
fn the_scratch_is_cleared_between_calls() {
    let mesh = shapes::uv_sphere(8, 12, 1.0);
    let mut out = Vec::new();
    wire_indices(&mesh, &mut out);
    let once = out.len();
    wire_indices(&mesh, &mut out);
    assert_eq!(out.len(), once, "a segunda chamada somou à primeira");
}
