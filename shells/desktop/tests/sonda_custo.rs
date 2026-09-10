//! SONDA TEMPORARIA -- o preco por quadro da alternativa `Smooth`, em CPU.
use ph2d_poly2d::{GridOptions, RefineOptions, grid_mesh_of, refine_posed};
use ph2d_skeleton::{Skin, SkinBone, Xform};
use std::time::Instant;

const W: usize = 320;
const H: usize = 96;
const L: f64 = 320.0 / 3.0;

fn capsula() -> Vec<u8> {
    let (raio, ax, bx) = (
        H as f64 / 2.0 - 3.0,
        H as f64 / 2.0,
        W as f64 - H as f64 / 2.0,
    );
    let mut a = vec![0u8; W * H];
    for y in 0..H {
        for x in 0..W {
            let px = x as f64 + 0.5;
            let d = (px - px.clamp(ax, bx)).hypot(y as f64 + 0.5 - H as f64 / 2.0);
            a[y * W + x] = ((raio + 1.0 - d).clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    a
}
fn p2l(p: [f64; 2]) -> [f64; 2] {
    [p[0], H as f64 / 2.0 - p[1]]
}
fn rot(c: [f64; 2], t: f64) -> Xform {
    let (s, k) = (t.sin(), t.cos());
    Xform([
        k,
        -s,
        s,
        k,
        c[0] - (k * c[0] - s * c[1]),
        c[1] - (s * c[0] + k * c[1]),
    ])
}
fn pele(th: f64) -> Skin {
    let p = [[0.0, 0.0], [L, 0.0], [2.0 * L, 0.0], [3.0 * L, 0.0]];
    let m1 = rot(p[1], th);
    let m2 = m1.then(&rot(m1.apply(p[2]), th));
    let m = [Xform::IDENTITY, m1, m2];
    Skin::new(
        (0..3)
            .map(|i| SkinBone {
                rest_a: p[i],
                rest_b: p[i + 1],
                radius: L,
                pose: m[i],
            })
            .collect(),
    )
    .expect("3 ossos")
}

#[test]
fn custo() {
    println!(
        "\ncarga NO INICIO: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let alfa = capsula();
    let focos: Vec<[f64; 2]> = (0..=3)
        .map(|k| [f64::from(k) * L, H as f64 / 2.0])
        .collect();
    let m0 =
        grid_mesh_of(&alfa, W as u32, H as u32, &focos, GridOptions::default()).expect("malha");
    println!(
        "malha guardada: {} vert, {} tris\n",
        m0.rest.len(),
        m0.tris.len()
    );
    println!("=== (a) DEFORMAR -- CPU por quadro, por imagem presa ===");
    for graus in [20.0_f64, 35.0, 50.0] {
        let s = pele(graus.to_radians());
        for (nome, opts) in [("Fast  ", None), ("Smooth", Some(RefineOptions::default()))] {
            let (mut melhor, mut tris, mut kk) = (f64::INFINITY, 0, 0);
            for _ in 0..60 {
                let t0 = Instant::now();
                let mut w = s.scratch();
                let mut campo = |p: [f64; 2]| s.point(p2l(p), &mut w);
                let (m, posed, k) = match opts {
                    None => {
                        let p: Vec<_> = m0.rest.iter().map(|&q| campo(q)).collect();
                        (m0.clone(), p, 1)
                    }
                    Some(o) => refine_posed(&m0, &mut campo, o),
                };
                std::hint::black_box((&m, &posed));
                melhor = melhor.min(t0.elapsed().as_secs_f64() * 1000.0);
                (tris, kk) = (m.tris.len(), k);
            }
            println!(
                "  dobra {:>3}gr/junta | {nome} | k={kk} | {tris:5} tris | {melhor:6.3} ms ({:4.1}% de um quadro)",
                graus * 3.0,
                melhor / 16.67 * 100.0
            );
        }
    }
    println!("\n=== (b) ENCODAR -- recorte + imagem + pop, por triangulo ===");
    let rgba = std::sync::Arc::new(vec![200u8; W * H * 4]);
    for n in [216usize, 1944, 3456, 5400, 7776] {
        let mut melhor = f64::INFINITY;
        for _ in 0..15 {
            let t0 = Instant::now();
            let mut cena = ph2d_vector::VectorScene::new();
            for k in 0..n {
                let a = k as f64 * 0.37;
                let mut p = ph2d_vector::BezPath::new();
                p.move_to(ph2d_vector::Point::new(a, a));
                p.line_to(ph2d_vector::Point::new(a + 12.0, a));
                p.line_to(ph2d_vector::Point::new(a, a + 12.0));
                p.close_path();
                cena.push_clip(&p);
                cena.draw_image_rgba_transformed(
                    &rgba,
                    W as u32,
                    H as u32,
                    ph2d_vector::Affine::new([1.0, 0.0, 0.0, 1.0, a, a]),
                    ph2d_vector::ImageQuality::Medium,
                );
                cena.pop_layer();
            }
            std::hint::black_box(&cena);
            melhor = melhor.min(t0.elapsed().as_secs_f64() * 1000.0);
        }
        println!(
            "  {n:6} tris -> {melhor:7.3} ms ({:5.1}% de um quadro de 16,67 ms)",
            melhor / 16.67 * 100.0
        );
    }
    println!(
        "\ncarga NO FIM: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}
