//! Quanto custa CONSTRUIR o plano — a medição que decide a fiação.
//!
//!     cargo run -p ph2d-mesh-colors --release --example mede_o_plano

use std::time::Instant;

fn grelha(n: u32) -> (usize, Vec<[u32; 3]>) {
    let verts = ((n + 1) * (n + 1)) as usize;
    let mut f = Vec::with_capacity((2 * n * n) as usize);
    for y in 0..n {
        for x in 0..n {
            let a = y * (n + 1) + x;
            f.push([a, a + 1, a + n + 1]);
            f.push([a + 1, a + n + 2, a + n + 1]);
        }
    }
    (verts, f)
}

fn main() {
    println!(
        "{:>10} {:>10} {:>12} {:>12} {:>14}",
        "V", "F", "topologia", "amostras", "plano (MB)"
    );
    for n in [128u32, 256, 314, 512] {
        let (v, f) = grelha(n);
        let it = || f.iter().map(|t| &t[..]);
        for nivel in [0u8, 2] {
            let t0 = Instant::now();
            let tinta = ph2d_mesh_colors::Tinta::nova(v, it(), nivel);
            let dt = t0.elapsed();
            println!(
                "{v:>10} {:>10} {:>10.2?} {:>12} {:>11.1} MB   (nível {nivel})",
                f.len(),
                dt,
                tinta.amostras().len(),
                tinta.amostras().len() as f64 * 12.0 / 1e6,
            );
        }
    }
}
