//! **A NEGATIVA NÃO CONSTRÓI O GRAFO DE ARESTAS** — o gate do piso do regime.
//!
//! Depois que a região alcança a densidade pedida, TODO evento de ponteiro de um
//! traço chega ao refino para ouvir *não há o que partir*. Essa resposta era
//! paga com um [`ph2d_mesh::Edges`] inteiro — `O(malha)` num gesto limitado pela
//! pegada, e **86% do custo de um dab em regime** (1,84 de 2,14 ms a 113k).
//!
//! # Por que o oráculo é ALOCAÇÃO e não relógio
//!
//! A propriedade é *"a negativa não constrói a estrutura de malha inteira"*, e
//! ela é observável **exatamente** em bytes: o grafo aloca três vetores
//! proporcionais a vértices, faces e arestas. Um kill de wall-clock mediria o
//! perfil do build e a carga da máquina; o dhat conta o que de fato foi pedido
//! ao alocador, e o número é o mesmo em toda corrida.
//!
//! ⚠️ **A barra é uma RAZÃO contra o grafo medido na mesma corrida**, não um
//! literal em bytes: ela se calibra sozinha quando a fixture muda de tamanho, e
//! é imune a eu ter chutado uma constante.
//!
//! ⚠️ **Um `#[test]` por binário, de propósito** — os contadores do dhat são
//! globais do processo, e o `cargo test` roda os testes de um binário em
//! threads. É a mesma razão que o `measure_memory` documenta.

use ph2d_mesh::{Mesh, Refine, refine_in_sphere, shapes};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// Quanto mais barata que o grafo a negativa tem de ser.
///
/// Medido no dia em que isto foi escrito: **147,8 KB contra 1,345 MB, ou 9,1×**.
///
/// ⚠️ **O que a negativa ainda aloca é a PEGADA**, não a malha — a lista de
/// faces candidatas que o octree devolve. É por isso que a razão depende de que
/// fração do modelo o pincel cobre, e é por isso que a barra tem folga: em 4× ela
/// tem margem de 2,3× sobre o medido, e ainda assim uma reconstrução do grafo
/// (razão ~0,9) a derruba com sobra.
const CHEAPER_BY: u64 = 4;

#[test]
fn the_refusal_does_not_build_the_edge_graph() {
    let _profiler = dhat::Profiler::builder().testing().build();

    // Acorda o pool do `rayon` antes de medir — ele nasce preguiçoso na
    // primeira chamada paralela, e atribuí-lo ao dab seria a mesma classe de
    // erro que o `measure_memory` já pagou.
    drop(shapes::sphere_with_triangles(2_000, 1.0));

    let mut m = shapes::uv_sphere(128, 192, 1.0);
    m.triangulate();
    m.rebuild();
    let centre = [0.0, 1.0, 0.0];
    let radius = 0.25f32;
    let target = 0.6 * mean_edge(&m);
    let mut births = Vec::new();

    // Leva a região à densidade pedida. ⚠️ **Sem isto a fixture não contém o
    // fenômeno** — ela mediria o PRIMEIRO dab, que é o outro regime e que de
    // fato constrói o grafo, porque de fato tem o que partir.
    let first = refine_in_sphere(&mut m, centre, radius, target, &mut births);
    assert!(
        matches!(first, Refine::Done { .. }),
        "o controle: a fixture tem de ter algo a refinar, e nao teve ({first:?})"
    );

    let before = dhat::HeapStats::get().total_bytes;
    let r = refine_in_sphere(&mut m, centre, radius, target, &mut births);
    let refusal = dhat::HeapStats::get().total_bytes - before;
    assert!(
        matches!(r, Refine::Enough),
        "a regiao ja' esta' na densidade pedida: {r:?}"
    );

    let before = dhat::HeapStats::get().total_bytes;
    drop(std::hint::black_box(m.edges()));
    let graph = dhat::HeapStats::get().total_bytes - before;

    assert!(
        refusal * CHEAPER_BY < graph,
        "a negativa alocou {refusal} B contra {graph} B do grafo de arestas — \
         ela esta' construindo a estrutura de malha inteira para dizer nao"
    );
}

/// O comprimento médio de aresta — a régua que faz o alvo acompanhar a malha,
/// para o refino de fato acontecer na primeira chamada.
fn mean_edge(m: &Mesh) -> f32 {
    let pos = m.positions();
    let mut tris = Vec::new();
    m.triangle_indices(&mut tris);
    let mut sum = 0.0f32;
    for t in &tris {
        for k in 0..3 {
            let (a, b) = (pos[t[k] as usize], pos[t[(k + 1) % 3] as usize]);
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            sum += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        }
    }
    sum / (tris.len() * 3).max(1) as f32
}
