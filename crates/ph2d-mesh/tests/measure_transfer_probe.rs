//! **De que a travessia é feita** — a decomposição, por ablação.
//! `cargo test -p ph2d-mesh --release --test measure_transfer_probe -- --ignored --nocapture`

use std::time::Instant;

#[test]
#[ignore = "sonda de medição"]
fn what_a_nearest_query_costs() {
    let mut from = ph2d_mesh::shapes::uv_sphere(48, 96, 1.0);
    let _ = from.masks_mut();
    let to = ph2d_mesh::shapes::uv_sphere(180, 360, 1.02);
    let n = to.vert_count();
    println!(
        "\n  fonte {} tris | destino {n} verts",
        from.triangle_count()
    );

    // (a) A CONSULTA do octree sozinha, no raio que a travessia usa.
    let seed = from.bounds().longest_edge() / (from.triangle_count() as f32).sqrt();
    let mut faces = Vec::new();
    let mut total = 0usize;
    let t = Instant::now();
    for &p in to.positions() {
        from.octree().faces_in_sphere(p, seed, &mut faces);
        total += faces.len();
    }
    let ms_query = t.elapsed().as_secs_f64() * 1e3;
    println!(
        "  (a) so' a consulta ...... {ms_query:8.1} ms  ({:.2} us/vertice, {:.1} faces/consulta)",
        ms_query * 1e3 / n as f64,
        total as f64 / n as f64
    );

    // (b) A travessia INTEIRA.
    let mut dst = to.clone();
    let t = Instant::now();
    ph2d_mesh::transfer_authored(&from, &mut dst);
    let ms_all = t.elapsed().as_secs_f64() * 1e3;
    println!(
        "  (b) a travessia ......... {ms_all:8.1} ms  ({:.2} us/vertice)",
        ms_all * 1e3 / n as f64
    );
    println!(
        "  => a consulta e' {:.0}% da travessia",
        ms_query / ms_all * 100.0
    );
}
