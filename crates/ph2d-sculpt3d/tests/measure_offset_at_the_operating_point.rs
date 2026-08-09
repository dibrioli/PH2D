//! **SONDA: o deslocamento MOVE o carimbo no ponto de operação do PRODUTO?**
//!
//! O gate de kernel (`the_offset_moves_the_stamp_and_zero_is_byte_identical`)
//! usa meio ladrilho e uma fixture escolhida. Esta sonda pergunta a MESMA coisa
//! nos números que o artista tem em mãos: a escala SEMEADA de uma esfera do
//! produto, o eixo semeado (`elev = 90`), e o passo/faixa das rows.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_offset_at_the_operating_point -- --ignored --nocapture
//! ```

use ph2d_sculpt3d::{Alpha, AlphaImage, Brush};

/// Bandas diagonais 32² — um campo ESTRUTURADO, senão qualquer deslocamento
/// concorda consigo mesmo.
fn banded() -> AlphaImage {
    let n = 32u32;
    let mut rgba = vec![0u8; (n * n * 4) as usize];
    for y in 0..n {
        for x in 0..n {
            let v = if (x + y) % 8 < 4 { 255 } else { 0 };
            let i = ((y * n + x) * 4) as usize;
            rgba[i] = v;
            rgba[i + 1] = v;
            rgba[i + 2] = v;
            rgba[i + 3] = 255;
        }
    }
    AlphaImage::from_rgba(n, n, &rgba).expect("imagem valida")
}

fn brush_with(scale: f32, off: [f32; 2], img: &AlphaImage) -> Brush {
    Brush {
        alpha: Some(Alpha::Image(std::sync::Arc::new(img.clone()))),
        alpha_scale: scale,
        alpha_elev_deg: ph2d_sculpt3d::MAX_AXIS_ELEV_DEG,
        alpha_offset: off,
        ..Brush::default()
    }
}

/// **QUANTO CUSTA PEDIR O FRAME** — o número que decide se ele pode entrar numa
/// chave de cache consultada por quadro.
///
/// ⚠️ A conversão ângulo→vetor deste app é um rotor de UM grau **acumulado**
/// (`rotate_by_degrees`), então ela é `O(graus)` e a cadeia é serial — o pior
/// caso é `az = 359`, e é ele que a sonda mede.
#[test]
#[ignore = "sonda"]
fn measure_what_asking_for_the_frame_costs() {
    let b = Brush {
        alpha_az_deg: 359,
        alpha_elev_deg: ph2d_sculpt3d::MAX_AXIS_ELEV_DEG,
        ..Brush::default()
    };
    const N: u32 = 10_000;
    let t = std::time::Instant::now();
    let mut sink = 0.0f32;
    for _ in 0..N {
        sink += b.alpha_frame().axis()[0];
    }
    let per = t.elapsed().as_secs_f64() * 1e6 / f64::from(N);
    println!("alpha_frame() no pior azimute: {per:.3} us/chamada (sink {sink:.3})");
}

#[test]
#[ignore = "sonda"]
fn measure_offset_at_the_operating_point() {
    // A malha do smoke, e a escala que o produto SEMEIA para ela.
    let mesh = ph2d_mesh::shapes::uv_sphere(96, 144, 1.0);
    let s = ph2d_sculpt3d::recommended_scale(&mesh);
    let img = banded();
    println!("escala semeada = {s:.6}  (um ladrilho = {s:.6} unidades de objeto)");

    let base = brush_with(s, [0.0, 0.0], &img);
    let f0 = base.alpha_frame();
    let a0 = base.alpha.as_ref().expect("alpha");

    for off in [0.01f32, 0.05, 0.1, 0.5, 1.0] {
        let b = brush_with(s, [off, 0.0], &img);
        let f = b.alpha_frame();
        let a = b.alpha.as_ref().expect("alpha");
        let (mut moved, mut sum, mut worst) = (0usize, 0.0f64, 0.0f32);
        for p in mesh.positions() {
            let w0 = a0.weight_at(*p, s, &f0);
            let w1 = a.weight_at(*p, s, &f);
            let d = (w1 - w0).abs();
            if d > 1e-6 {
                moved += 1;
            }
            sum += f64::from(d);
            worst = worst.max(d);
        }
        let n = mesh.positions().len();
        println!(
            "offset {off:>5.2} ({:>6.2} ladrilhos): {moved:>6}/{n} vertices mudam · |d| medio \
             {:.4} · pior {worst:.4}",
            off / s,
            sum / n as f64
        );
    }
}
