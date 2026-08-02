//! **A sonda da subdivisão** — quanto custa, do que é feito, e onde está o teto.
//!
//! Ela existe porque o `subdivide` **não tem cap escrito**, e o §0 do
//! `CLAUDE.md` diz que um limite se MEDE antes de se escrever. As três perguntas:
//!
//! 1. o custo é linear na malha, ou há um passo que explode?
//! 2. **de que ele é feito** — o grafo de arestas, os canais, ou o `rebuild`
//!    (adjacência + octree + normais) da malha nova?
//! 3. qual é a maior malha que ainda subdivide num tempo que um artista aceita?
//!
//! ```text
//! cargo test -p ph2d-mesh --release --test measure_subdivide -- --ignored --nocapture
//! ```

use ph2d_mesh::{Mesh, shapes, subdivide};
use std::time::Instant;

/// O tamanho analítico de uma malha — os mesmos termos que a `measure_memory`
/// soma, para os dois números serem comparáveis.
fn mesh_bytes(m: &Mesh) -> usize {
    let (v, f) = (m.vert_count(), m.face_count());
    let adj = m.adjacency();
    v * 24
        + f * 16
        + f * 12
        + adj.vert_faces.entry_count() * 4
        + adj.vert_verts.entry_count() * 4
        + (v + 1) * 8
        + m.octree().memory_bytes()
}

fn ms(f: impl FnOnce()) -> f64 {
    let t = Instant::now();
    f();
    t.elapsed().as_secs_f64() * 1000.0
}

#[test]
#[ignore = "sonda: mede, não afirma"]
fn measure_the_cost_of_one_subdivision() {
    println!("\n  SUBDIVIDIR — o custo por tamanho de malha");
    println!("   entrada V      faces       saída V      faces       ms      ns/vértice");
    for (rings, segs) in [(16, 24), (32, 48), (64, 96), (128, 192), (192, 288)] {
        let mesh = shapes::uv_sphere(rings, segs, 1.0);
        // Duas corridas; a primeira aquece o alocador.
        let _ = subdivide(&mesh);
        let mut out = None;
        let t = ms(|| out = Some(subdivide(&mesh)));
        let out = out.expect("subdividiu");
        println!(
            "  {:>10}  {:>9}   {:>10}  {:>9}  {:>7.2}  {:>10.1}",
            mesh.vert_count(),
            mesh.face_count(),
            out.vert_count(),
            out.face_count(),
            t,
            t * 1e6 / out.vert_count() as f64
        );
    }
    println!();
}

#[test]
#[ignore = "sonda: mede, não afirma"]
fn measure_what_a_subdivision_is_made_of() {
    println!("\n  DE QUE ELE É FEITO — a malha de 128x192");
    let mesh = shapes::uv_sphere(128, 192, 1.0);
    let _ = subdivide(&mesh);

    let mut edges = None;
    let t_edges = ms(|| edges = Some(mesh.edges()));
    let edges = edges.expect("grafo");

    let mut whole = None;
    let t_whole = ms(|| whole = Some(subdivide(&mesh)));
    let out = whole.expect("saída");

    // O `rebuild` da malha de SAÍDA: adjacência + octree + normais, sobre 4× a
    // contagem. Medido pelo mesmo caminho que o `from_parts` toma.
    let positions = out.positions().to_vec();
    let faces = out.faces().to_vec();
    let mut rebuilt = None;
    let t_rebuild = ms(|| rebuilt = Mesh::from_parts(positions, faces).ok());
    assert!(rebuilt.is_some());

    println!(
        "  grafo de arestas  {t_edges:>8.2} ms  ({} arestas)",
        edges.len()
    );
    println!("  o rebuild da saída{t_rebuild:>8.2} ms  (adjacência + octree + normais)");
    println!("  a subdivisão toda {t_whole:>8.2} ms");
    println!(
        "  ⇒ o que sobra (plano + canais): {:.2} ms\n",
        t_whole - t_edges - t_rebuild
    );
}

#[test]
#[ignore = "sonda: mede, não afirma"]
fn measure_the_memory_a_subdivision_costs() {
    println!("\n  MEMÓRIA — o preço de cada nível");
    println!("   nível         V       faces    triângulos      MB (malha)");
    let mut mesh = shapes::uv_sphere(32, 48, 1.0);
    for level in 0..4 {
        let bytes = mesh_bytes(&mesh);
        println!(
            "  {level:>6}  {:>9}  {:>10}  {:>12}  {:>14.1}",
            mesh.vert_count(),
            mesh.face_count(),
            mesh.triangle_count(),
            bytes as f64 / (1024.0 * 1024.0)
        );
        if level < 3 {
            mesh = subdivide(&mesh);
        }
    }
    println!();
}
