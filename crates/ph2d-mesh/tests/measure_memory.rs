//! **A sonda de memória da malha** — o HR-13 emendado pelo ADR-0117:
//! *quem declara budget possui um gate que MEDE*.
//!
//! Ela existe **antes** de qualquer `MAX_TRIANGLES` ser escrito, e é dela que o
//! teto por tier (ADR-0104) sai. Escrever o teto primeiro e medir depois é o
//! erro que o `CLAUDE.md` §0.0 nomeia — o caso da sim de partículas, cujo teto
//! ficou 256× abaixo do que a máquina fazia.
//!
//! ```text
//! cargo test -p ph2d-mesh --release --test measure_memory -- --nocapture
//! ```
//!
//! ⚠️ Um `#[test]` por binário, de propósito: os contadores do dhat são globais
//! do processo e o `cargo test` roda os testes de um binário em threads. Dois
//! perfis no mesmo processo disputam.

use ph2d_mesh::{Mesh, QueryScratch, shapes};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const MB: f64 = 1_048_576.0;

/// A **propriedade**, que é o que um gate pode afirmar sem inventar um número:
/// a memória é LINEAR na contagem de triângulos. Se ela crescer mais rápido, há
/// uma estrutura quadrática escondida — e é isso que mata uma malha grande, não
/// o valor absoluto de bytes por triângulo.
///
/// A banda é generosa (±25%) porque a malha tem partes que **não** escalam
/// (nós do octree em degraus de 8, capacidade de `Vec` em potências de 2), e um
/// gate apertado sobre um número desses vira um gate que se silencia.
const LINEARITY_BAND: f64 = 0.25;

fn analytic_breakdown(m: &Mesh) -> Vec<(&'static str, usize)> {
    let v = m.vert_count();
    let f = m.face_count();
    let adj = m.adjacency();
    vec![
        ("posicoes", v * size_of::<[f32; 3]>()),
        ("normais", v * size_of::<[f32; 3]>()),
        ("faces", f * size_of::<[u32; 4]>()),
        ("normais de face", f * size_of::<[f32; 3]>()),
        (
            "csr vert->faces",
            adj.vert_faces.entry_count() * 4 + (v + 1) * 4,
        ),
        (
            "csr vert->verts",
            adj.vert_verts.entry_count() * 4 + (v + 1) * 4,
        ),
        ("octree", m.octree().memory_bytes()),
    ]
}

#[test]
fn measure_memory() {
    let _profiler = dhat::Profiler::builder().testing().build();

    // ⚠️ **Acorda o pool do `rayon` ANTES de medir qualquer malha.** Ele nasce
    // preguiçoso, na primeira chamada paralela — que é o build da primeira
    // malha —, e o dhat atribuía as alocações dele à malha de 10 k triângulos:
    // 54,3 → 77,9 B/triângulo, e o gate de linearidade disparava culpando uma
    // "estrutura super-linear" que não existe. O pool é custo de PROCESSO, fixo
    // e pago uma vez; contá-lo como custo de malha é a mesma classe de erro que
    // medir o primeiro traço junto com o *first-touch* dos buffers.
    let pool_before = dhat::HeapStats::get().curr_bytes;
    drop(shapes::sphere_with_triangles(20_000, 1.0));
    let pool = dhat::HeapStats::get()
        .curr_bytes
        .saturating_sub(pool_before);

    println!("\n=== ph2d-mesh :: memória residente por malha ===\n");
    println!(
        "  (pool do rayon, custo de processo pago uma vez: {:.2} MB)\n",
        pool as f64 / MB
    );
    println!(
        "{:>12} {:>10} {:>10} {:>12} {:>12}",
        "triangulos", "vertices", "faces", "MB (dhat)", "B/triangulo"
    );

    let mut per_tri: Vec<f64> = Vec::new();
    // 5 M é a escala que o kill-criterion K1 do `docs/3D/03.5` nomeia, então
    // ela é MEDIDA e não extrapolada de 1 M — extrapolar seria justamente
    // afirmar a linearidade que este teste existe para verificar.
    for target in [10_000usize, 100_000, 1_000_000, 5_000_000] {
        let base = dhat::HeapStats::get().curr_bytes;
        let mesh = shapes::sphere_with_triangles(target, 1.0);
        let held = dhat::HeapStats::get().curr_bytes.saturating_sub(base);
        let tris = mesh.triangle_count();

        println!(
            "{:>12} {:>10} {:>10} {:>12.2} {:>12.1}",
            tris,
            mesh.vert_count(),
            mesh.face_count(),
            held as f64 / MB,
            held as f64 / tris as f64
        );
        per_tri.push(held as f64 / tris as f64);

        if target == 5_000_000 {
            println!("\n  decomposição ANALÍTICA (o total acima é MEDIDO):");
            let parts = analytic_breakdown(&mesh);
            let sum: usize = parts.iter().map(|(_, b)| *b).sum();
            for (name, bytes) in &parts {
                println!(
                    "    {name:<18} {:>8.2} MB  ({:>4.1}%)",
                    *bytes as f64 / MB,
                    100.0 * *bytes as f64 / sum as f64
                );
            }
            println!("    {:<18} {:>8.2} MB", "soma", sum as f64 / MB);

            // Cor e máscara são preguiçosas — o que elas custam QUANDO tocadas.
            let mut painted = mesh.clone();
            let before = dhat::HeapStats::get().curr_bytes;
            painted.colors_mut()[0] = [1.0, 0.0, 0.0];
            painted.masks_mut()[0] = 1.0;
            let planes = dhat::HeapStats::get().curr_bytes.saturating_sub(before);
            println!(
                "\n  cor + máscara, materializadas ao primeiro toque: {:.2} MB \
                 ({:.1} B/vértice)",
                planes as f64 / MB,
                planes as f64 / mesh.vert_count() as f64
            );

            // O scratch de consulta: o custo do GESTO, que não pode ficar fora
            // da conta só por ser transitório.
            let mut scratch = QueryScratch::default();
            let mut out = Vec::new();
            mesh.verts_in_sphere([0.0, 1.0, 0.0], 0.2, &mut scratch, &mut out);
            println!(
                "  scratch de consulta (reusado entre dabs):        {:.2} MB",
                scratch.capacity_bytes() as f64 / MB
            );
        }
        drop(mesh);
    }

    println!();
    let lo = per_tri.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = per_tri.iter().cloned().fold(0.0, f64::max);
    println!(
        "B/triangulo: min {lo:.1}  max {hi:.1}  razao {:.3}\n",
        hi / lo
    );

    assert!(
        hi / lo <= 1.0 + LINEARITY_BAND,
        "a memória não é linear na contagem de triângulos: {lo:.1} a {hi:.1} B/tri \
         (razão {:.2}). Uma estrutura super-linear é o que mata a malha grande.",
        hi / lo
    );
}
