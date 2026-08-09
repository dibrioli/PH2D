//! **QUANTO CUSTA AVALIAR O PADRÃO SOBRE A MALHA INTEIRA** — a sonda que decide
//! a forma do preview no barro.
//!
//! ⚠️ **Ela mede pela porta do PRODUTO** (`Brush::alpha_weight` sobre
//! `Mesh::positions`), e não por um laço próprio. O número da W12 saiu de
//! PIXELS, e um preview no objeto amostra VÉRTICES: extrapolar de um para o
//! outro seria a inferência de segunda ordem que este repo já pagou caro.
//!
//! Rodar: `cargo test -p ph2d-sculpt3d --release --test measure_alpha_over_a_whole_mesh -- --ignored --nocapture`

use ph2d_sculpt3d::{Alpha, AlphaImage, Brush};

/// Uma imagem de teste do lado `n`: um xadrez suave, para a amostra bilinear ter
/// o que interpolar (uma chapa uniforme deixaria o compilador dobrar a leitura).
fn image(n: u32) -> Alpha {
    let mut rgba = Vec::with_capacity((n * n * 4) as usize);
    for y in 0..n {
        for x in 0..n {
            let v = u8::try_from((x * 7 + y * 13) % 256).unwrap_or(255);
            rgba.extend_from_slice(&[v, v, v, 255]);
        }
    }
    Alpha::Image(std::sync::Arc::new(
        AlphaImage::from_rgba(n, n, &rgba).expect("a fixture descreve o buffer"),
    ))
}

fn cost(mesh: &ph2d_mesh::Mesh, alpha: Alpha, scale: f32) -> f64 {
    let brush = Brush {
        alpha: Some(alpha),
        alpha_scale: scale,
        ..Brush::default()
    };
    let frame = brush.alpha_frame();
    let pos = mesh.positions();
    let mut best = f64::INFINITY;
    for _ in 0..3 {
        let t = std::time::Instant::now();
        let mut sink = 0.0f32;
        for p in pos {
            sink += brush.alpha_weight(*p, &frame);
        }
        let ms = t.elapsed().as_secs_f64() * 1e3;
        assert!(sink.is_finite());
        best = best.min(ms);
    }
    best
}

#[test]
#[ignore = "sonda de medição"]
fn measure_the_alpha_over_a_whole_mesh() {
    for (name, mesh) in [
        (
            "esfera 96x144 (13k)",
            ph2d_mesh::shapes::uv_sphere(96, 144, 1.0),
        ),
        (
            "esfera 533x800 (426k) = a cena =21",
            ph2d_mesh::shapes::uv_sphere(533, 800, 1.0),
        ),
    ] {
        println!("\n== {name} — {} vértices ==", mesh.vert_count());
        println!("{:<12} {:>10}  {:>10}", "padrão", "ms", "ns/vért");
        for a in &Alpha::ALL {
            let ms = cost(&mesh, a.clone(), 0.06);
            let ns = ms * 1e6 / mesh.vert_count() as f64;
            println!("{:<12} {ms:>10.3}  {ns:>10.1}", a.label());
        }
        // ⚠️ **A IMAGEM entra na MESMA tabela**, e em três tamanhos: o custo dela
        // é uma consulta a uma tabela, então o que pode variar não é a
        // aritmética — é o CACHE. Um número só esconderia exatamente isso.
        for n in [64u32, 512, 2048] {
            let a = image(n);
            let ms = cost(&mesh, a, 0.06);
            let ns = ms * 1e6 / mesh.vert_count() as f64;
            let mb = f64::from(n) * f64::from(n) / (1024.0 * 1024.0);
            println!(
                "{:<12} {ms:>10.3}  {ns:>10.1}   ({mb:.2} MB)",
                format!("Image {n}")
            );
        }
    }
}
