//! **O INSTRUMENTO** — corre a extracção sobre um `.mapa` e imprime o relatório.
//!
//! ⚠️ Ele corre **o mesmo caminho do produto**, e não uma cópia: o que ele imprime
//! é o que a crate faz.
//!
//! ```text
//! cargo run -p ph2d-quadextract --example extract_info -- <ficheiro.mapa> ...
//! ```

/// Arestas da saída usadas **uma** vez (bordo) e **três ou mais** (não-manifold).
fn edge_census(mesh: &ph2d_mesh::Mesh) -> (usize, usize) {
    use std::collections::BTreeMap;
    let mut n: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            *n.entry(if a < b { (a, b) } else { (b, a) }).or_default() += 1;
        }
    }
    (
        n.values().filter(|c| **c == 1).count(),
        n.values().filter(|c| **c >= 3).count(),
    )
}

/// Um histograma sem os baldes vazios — *um balde que ninguém enche lê-se como zero,
/// e imprimir dezassete zeros esconde o único que interessa.*
fn hist(h: &[usize]) -> String {
    h.iter()
        .enumerate()
        .filter(|(_, n)| **n > 0)
        .map(|(i, n)| format!("{i}:{n}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn main() {
    let mut any = false;
    for path in std::env::args().skip(1) {
        any = true;
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                println!("{path}: nao deu para ler: {e}");
                continue;
            }
        };
        let mapa = match ph2d_quadextract::mapa::Mapa::parse(&text) {
            Ok(m) => m,
            Err(e) => {
                println!("{path}: {e}");
                continue;
            }
        };
        let t = std::time::Instant::now();
        match ph2d_quadextract::extract(&mapa.as_map(), None) {
            Ok((mesh, r)) => {
                let ms = t.elapsed().as_secs_f64() * 1000.0;
                let shape = ph2d_quadfill::quad_shape(&mesh);
                let chi = ph2d_quadextract::euler_characteristic(&mesh);
                println!("{path}");
                println!(
                    "  entrada: {} faces, {} verts | grade 2^-{} | interiores {} bordo {} nao-manifold {}",
                    mapa.tris.len(),
                    mapa.pos.len(),
                    r.grid_exponent,
                    r.interior_edges,
                    r.boundary_edges,
                    r.non_manifold_edges
                );
                println!(
                    "  colapso: {} arestas (tardio {}), {} faces mortas | residuo rot {:.3e} trans {:.3e} | transicoes inexactas {}",
                    r.collapsed_edges,
                    r.late_collapsed,
                    r.dead_faces,
                    r.rot_residual,
                    r.shift_residual,
                    r.inexact_transitions
                );
                println!(
                    "  saneamento: ponto-fixo {} inteiro {} leques abertos {} holonomia partida {}",
                    r.pinned_fixed, r.pinned_integer, r.open_fans, r.holonomy_broken
                );
                let val: Vec<String> = r
                    .valence
                    .iter()
                    .enumerate()
                    .filter(|(_, n)| **n > 0)
                    .map(|(v, n)| format!("{v}:{n}"))
                    .collect();
                println!("  valencias: {}", val.join(" "));
                println!(
                    "  nos: vertice {} aresta {} face {} | faces degeneradas {} dobradas {}",
                    r.vertex_nodes, r.edge_nodes, r.face_nodes, r.degenerate_faces, r.folded_faces
                );
                println!(
                    "  saidas: {} emitidas, {} ligadas, {} pendentes-bordo, {} orfas, {} fugidas | {} passos, {} inversoes",
                    r.ports,
                    r.linked,
                    r.pending_boundary,
                    r.orphan,
                    r.runaway,
                    r.walk_steps,
                    r.walk_flips
                );
                println!(
                    "  ORDEM das saidas: limpas {:?} (tem de ser tudo balde 3) | dobradas {:?} (balde 1)",
                    r.port_step, r.port_step_folded
                );
                println!("  LADOS por percurso: {}", hist(&r.ring_len));
                println!("  CANTOS distintos pos-fusao: {}", hist(&r.ring_distinct));
                let (b, nm) = edge_census(&mesh);
                println!("  arestas da SAIDA: {} de bordo, {} nao-manifold", b, nm);
                println!(
                    "  celulas: {} fechadas, {} abandonadas, {} nao-fechadas | fusao {} grupos | leques colapsados {}",
                    r.cells_closed,
                    r.cells_abandoned,
                    r.cells_unclosed,
                    r.merged_groups,
                    r.collapsed_fans
                );
                let nf = mesh.face_count().max(1);
                #[allow(clippy::cast_precision_loss)]
                let pct = 100.0 * r.quads as f64 / nf as f64;
                println!(
                    "  SAIDA: {} verts, {} quads ({pct:.1}%), {} degeneradas, {} triangulos | X = {chi} | {ms:.0} ms",
                    mesh.vert_count(),
                    r.quads,
                    r.degenerate_cells,
                    r.triangles
                );
                println!(
                    "  FORMA: aspecto p50 {:.2} p99 {:.2} max {:.1} (>4x: {}) | enviesamento p50 {:.1} p99 {:.1} max {:.1} (>60: {}) | area spread {:.2}",
                    shape.aspect_p50,
                    shape.aspect_p99,
                    shape.aspect_max,
                    shape.aspect_over_4,
                    shape.skew_p50,
                    shape.skew_p99,
                    shape.skew_max,
                    shape.skew_over_60,
                    shape.area_spread
                );
            }
            Err(e) => println!("{path}: recusado: {e}"),
        }
    }
    if !any {
        println!("uso: extract_info <ficheiro.mapa> ...");
    }
}
