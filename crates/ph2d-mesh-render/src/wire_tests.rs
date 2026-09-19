//! Gates da lista de arestas.

use super::*;
use ph2d_mesh::shapes;
use std::collections::BTreeSet;

/// O conjunto de arestas que a lista descreve, normalizado.
fn edge_set(out: &[u32]) -> BTreeSet<(u32, u32)> {
    out.as_chunks::<2>()
        .0
        .iter()
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

/// Uma chapa `n × n` triangulada por leque de quadrado — **é** uma grade, e as
/// diagonais dela são as únicas arestas que não correm nos eixos.
fn grade(n: usize) -> ph2d_mesh::Mesh {
    let mut pos = Vec::new();
    for j in 0..n {
        for i in 0..n {
            pos.push([i as f32, j as f32, 0.0]);
        }
    }
    let mut faces = Vec::new();
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            let a = (j * n + i) as u32;
            let (b, c, d) = (a + 1, a + n as u32, a + n as u32 + 1);
            faces.push(ph2d_mesh::Face::tri(a, b, d));
            faces.push(ph2d_mesh::Face::tri(a, d, c));
        }
    }
    ph2d_mesh::Mesh::from_parts(pos, faces).expect("a grade é bem formada")
}

/// ⭐⭐⭐⭐ **GATE — A VISTA DA GRADE DEVOLVE A GRADE.**
///
/// Numa chapa `n × n` há `2·n·(n−1)` arestas nos eixos e `(n−1)²` diagonais. A
/// vista tem de esconder **exactamente** as diagonais — nem uma a mais (abriria
/// um buraco), nem uma a menos (a grade continuaria escondida).
///
/// ⛔ **A contagem é DERIVADA da geometria da fixtura, não escrita à mão:** um
/// número copiado aqui deixaria de descrever a chapa no dia em que `n` mudasse.
#[test]
fn a_vista_da_grade_esconde_exactamente_as_diagonais() {
    const N: usize = 9;
    let mesh = grade(N);
    let (mut cheio, mut so_grade) = (Vec::new(), Vec::new());
    wire_indices(&mesh, &mut cheio);
    wire_indices_com(&mesh, true, &mut so_grade);

    let (a, b) = (edge_set(&cheio), edge_set(&so_grade));
    let nos_eixos = 2 * N * (N - 1);
    let diagonais = (N - 1) * (N - 1);
    assert_eq!(a.len(), nos_eixos + diagonais, "a fixtura mudou de forma");
    assert_eq!(
        b.len(),
        nos_eixos,
        "a vista deixou {} arestas onde a grade tem {nos_eixos}",
        b.len()
    );
    // ⚠️ E o que ficou é SUBCONJUNTO do que havia: a vista **esconde**, ela
    // nunca inventa uma aresta que a malha não tem.
    assert!(b.is_subset(&a), "a vista inventou arestas");
    // ⭐ E as que saíram são todas diagonais — nenhuma corre num eixo.
    let pos = mesh.positions();
    for (u, v) in a.difference(&b) {
        let (p, q) = (pos[*u as usize], pos[*v as usize]);
        assert!(
            (p[0] - q[0]).abs() > 0.5 && (p[1] - q[1]).abs() > 0.5,
            "a vista escondeu uma aresta de EIXO ({u}, {v})"
        );
    }
}

/// ⚠️ **Com a vista desligada o caminho é o de sempre, AO BIT** — sem isto, a
/// vista nova mudaria o arame de todo artista que não a pediu.
#[test]
fn a_vista_desligada_e_o_arame_de_sempre() {
    let mesh = shapes::uv_sphere(12, 18, 1.0);
    let (mut antigo, mut novo) = (Vec::new(), Vec::new());
    wire_indices(&mesh, &mut antigo);
    wire_indices_com(&mesh, false, &mut novo);
    assert_eq!(antigo, novo);
}

/// ⛔⛔ **GATE — A BEIRA NUNCA SE ESCONDE.**
///
/// Uma aresta com **um** triângulo só é a silhueta de uma peça aberta; escondê-la
/// abriria a malha na tela. ⚠️ A fixtura é construída para a beira ser a aresta
/// **mais longa** do triângulo dela — senão o gate ficaria verde sem nunca
/// chegar ao caso.
#[test]
fn a_beira_de_uma_peca_aberta_nunca_se_esconde() {
    // Um triângulo só: as três arestas têm um triângulo, e a hipotenusa é a
    // mais longa.
    let mesh = ph2d_mesh::Mesh::from_parts(
        vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        vec![ph2d_mesh::Face::tri(0, 1, 2)],
    )
    .expect("o triângulo é bem formado");
    let (mut cheio, mut so_grade) = (Vec::new(), Vec::new());
    wire_indices(&mesh, &mut cheio);
    wire_indices_com(&mesh, true, &mut so_grade);
    assert_eq!(
        edge_set(&cheio),
        edge_set(&so_grade),
        "a vista comeu a beira de uma peça aberta"
    );
}

/// ⚠️ **Uma malha de QUADS não muda**, por construção — a diagonal de um
/// quadrilátero não é uma aresta, logo não está na lista de onde se tira.
#[test]
fn numa_malha_de_quads_a_vista_nao_muda_nada() {
    let mesh = shapes::cube(1.0);
    let (mut cheio, mut so_grade) = (Vec::new(), Vec::new());
    wire_indices(&mesh, &mut cheio);
    wire_indices_com(&mesh, true, &mut so_grade);
    assert_eq!(edge_set(&cheio), edge_set(&so_grade));
    assert!(!cheio.is_empty(), "a fixtura tem de ter arestas");
}
