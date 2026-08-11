//! **A esfera de escultura contra a que ela substituiu.**
//!
//! ```text
//! cargo run -p ph2d-mesh --release --example probe_sphere
//! ```
//!
//! ⚠️ **`--release`, e não é preferência:** sete subdivisões são aritmética por
//! vértice, e em debug isto mede o `opt-level=0` em vez do produto.
//!
//! ⚠️ **A razão de ARESTA é o número que decide a troca**, não o tempo. A esfera
//! UV concentra um leque de triângulos em cada polo e estica quads no equador,
//! então o mesmo pincel come áreas muito diferentes conforme onde o artista
//! toca; a subdividida não tem polo.

fn edge_stats(mesh: &ph2d_mesh::Mesh) -> (usize, f32, f32, f32) {
    let p = mesh.positions();
    let mut len: Vec<f32> = Vec::new();
    for f in mesh.faces() {
        let v = f.verts();
        for i in 0..v.len() {
            let (a, b) = (v[i] as usize, v[(i + 1) % v.len()] as usize);
            if a < b {
                let (x, y) = (p[a], p[b]);
                len.push(
                    ((x[0] - y[0]).powi(2) + (x[1] - y[1]).powi(2) + (x[2] - y[2]).powi(2)).sqrt(),
                );
            }
        }
    }
    len.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
    let n = len.len();
    (n, len[0], len[n / 2], len[n - 1])
}

fn radius_spread(mesh: &ph2d_mesh::Mesh) -> (f32, f32, f32) {
    let mut r: Vec<f32> = mesh
        .positions()
        .iter()
        .map(|p| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt())
        .collect();
    r.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
    (r[0], r[r.len() / 2], r[r.len() - 1])
}

fn main() {
    println!("A SUBDIVISAO, passo a passo (a lei e' `while faces < 50_000`):");
    println!("  n  faces   verts   tris  ms(passo)   r.min   r.med   r.max  desvio%");
    let t0 = std::time::Instant::now();
    let mut m = ph2d_mesh::shapes::cube(1.0);
    for n in 1..=7 {
        let t = std::time::Instant::now();
        m = ph2d_mesh::subdivide(&m);
        let ms = t.elapsed().as_secs_f64() * 1e3;
        let (lo, med, hi) = radius_spread(&m);
        println!(
            "{n:3} {:6} {:7} {:6} {ms:10.1} {lo:7.4} {med:7.4} {hi:7.4} {:7.2}",
            m.face_count(),
            m.vert_count(),
            m.triangle_count(),
            100.0 * (hi - lo) / med,
        );
    }
    println!("  cru, antes de normalizar: {:.1} ms\n", t0.elapsed().as_secs_f64() * 1e3);

    println!("O GESTO INTEIRO, pela porta do produto:");
    for (name, build) in [
        (
            "sculpt_sphere",
            (|| ph2d_mesh::shapes::sculpt_sphere(1.0)) as fn() -> ph2d_mesh::Mesh,
        ),
        ("uv_sphere(96,144)", || {
            ph2d_mesh::shapes::uv_sphere(96, 144, 1.0)
        }),
    ] {
        // Cada amostra faz o mesmo trabalho ⇒ o minimo e' o redutor certo.
        let mut ms = f64::MAX;
        let mut mesh = build();
        for _ in 0..5 {
            let t = std::time::Instant::now();
            mesh = build();
            ms = ms.min(t.elapsed().as_secs_f64() * 1e3);
        }
        let (n, lo, med, hi) = edge_stats(&mesh);
        let b = mesh.bounds();
        println!(
            "{name:18} {:6} verts · {:6} faces · {ms:6.1} ms · aresta max/min {:5.1}x \
             (min {lo:.5} med {med:.5} max {hi:.5}, {n} arestas) · caixa +-{:.4}",
            mesh.vert_count(),
            mesh.face_count(),
            hi / lo,
            b.max[0],
        );
    }
}
