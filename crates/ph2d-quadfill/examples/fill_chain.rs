//! ⭐⭐⭐ **A ROTA DO *FILL*, EM DISCO** — o irmão do `chain_info`, e o A/B da QUANTIZAÇÃO.
//!
//! ```text
//! cargo run --release -p ph2d-quadfill --example fill_chain -- <peca.obj> <saida.obj>
//! ```
//!
//! # Por que ele existe
//!
//! A rota que **shipa** é a da extracção
//! (`shells/desktop/src/sculpt3d_history_retopo_extract.rs`), e ela **não chama o F4**:
//! vai `F1 → F2 → F3 → corte → pente → G3/G5 → extracção`. A rota do *fill*
//! (`…_retopo_global.rs`) chama-o — `ph2d_quantize::quantize_within` — e é a **única**
//! diferença de fase entre as duas antes do preenchimento.
//!
//! ⚠️ **Para o A/B ser sobre UMA variável**, este instrumento repete a montante do
//! `chain_info` **passo a passo**: o mesmo F1 (`remesh_isotropic` com
//! [`ph2d_remesh_iso::ALPHA`]), o mesmo `h` (a aresta **mediana** depois do F1) e o
//! campo **liso** (`solve_miq`). ⇒ corra o irmão com `PH2D_ALIGN_WEIGHT=0`, senão
//! compara-se também o campo. *Duas fases a mudar de uma vez não respondem a pergunta
//! nenhuma.*

use ph2d_crossfield::{Dual, solve_miq};
use ph2d_quadfill::{SMOOTHING_ROUNDS, fill};
use ph2d_quantize::{Budget, quantize_within};
use ph2d_trace::trace_patches;

/// A aresta mediana — ⚠️ **a mesma conta do `chain_info`**, e é de propósito que ela
/// esteja escrita igual: é ela que fixa a densidade da grade nos dois lados.
fn median_edge(mesh: &ph2d_mesh::Mesh) -> f32 {
    let pos = mesh.positions();
    let mut e: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (pos[v[k] as usize], pos[v[(k + 1) % v.len()] as usize]);
            let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
            e.push(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
        }
    }
    e.sort_by(f32::total_cmp);
    e[e.len() / 2]
}

fn main() {
    let mut args = std::env::args().skip(1);
    let piece = args.next().expect("uso: fill_chain <peca.obj> [saida.obj]");
    let out_path = args.next();

    let text =
        std::fs::read_to_string(&piece).unwrap_or_else(|e| panic!("nao consegui ler {piece}: {e}"));
    let pieces = ph2d_mesh::import_obj(&text)
        .unwrap_or_else(|e| panic!("{piece} nao e' um OBJ que este leitor entenda: {e:?}"));
    let mut mesh = pieces
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{piece} nao tem uma peca dentro"))
        .mesh;

    let alpha = std::env::var("PH2D_ALPHA")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(ph2d_remesh_iso::ALPHA);
    let f1 = ph2d_remesh_iso::remesh_isotropic(&mut mesh, alpha);
    mesh.triangulate();
    let h = median_edge(&mesh);
    println!(
        "  F1: {} -> {} vertices, {} faces | h (aresta mediana) = {h:.5}",
        f1.verts_before,
        f1.verts_after,
        mesh.face_count()
    );

    let dual = Dual::build(&mesh);
    let (field, _) = solve_miq(&dual);
    let layout = trace_patches(&mesh, &dual, &field);
    let tr = &layout.report;
    println!(
        "  F3: {} patches, {} separatrizes, {} soltas, {} nao-disco, {} arcos",
        tr.patches, tr.separatrices, tr.dangling, tr.non_disk, tr.arcs
    );

    let l = match layout.to_layout(h) {
        Ok(l) => l,
        Err(e) => {
            println!("  ⛔ o layout nao passa a porta do F4: {e:?}");
            return;
        }
    };
    // ⚠️ O orçamento é o mesmo da sonda de cadeia que já existe (`tests/pipeline.rs`):
    // *o relógio do F4 é do LAYOUT e não do tamanho da malha.*
    let (q, qr) = match quantize_within(&l, Budget::new(256, 512)) {
        Ok(v) => v,
        Err(e) => {
            println!("  ⛔ o F4 RECUSOU: {e:?}");
            return;
        }
    };
    println!("  ⭐ F4 (a fase que a rota da extraccao NAO tem): {qr:?}");

    match fill(&mesh, &mesh, &layout, &q, SMOOTHING_ROUNDS) {
        Ok((out, r)) => {
            #[allow(clippy::cast_precision_loss)]
            let pct = if r.verts == 0 {
                0.0
            } else {
                100.0 * r.irregular as f64 / r.verts as f64
            };
            println!(
                "  F5: {} quads, {} nao-quads, {} verts, {} IRREGULARES ({pct:.1} %), bordo {}",
                r.quads, r.non_quads, r.verts, r.irregular, r.boundary_edges
            );
            if let Some(path) = out_path {
                let text = ph2d_mesh::write_obj(&[ph2d_mesh::ExportPiece {
                    mesh: &out,
                    name: Some("Piece"),
                    pose: ph2d_mesh::Pose::default(),
                }]);
                std::fs::write(&path, text).unwrap_or_else(|e| panic!("{path}: {e}"));
                println!("  (peca gravada em {path})");
            }
        }
        Err(e) => println!("  ⛔ o F5 recusou: {e:?}"),
    }
}
