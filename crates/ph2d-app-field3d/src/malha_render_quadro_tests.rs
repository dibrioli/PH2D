//! A malha cai onde o traçado, o gizmo e o contorno a desenham.

use ph2d_field_render::{Lens, Orbit, Screen};

fn projeta(m: &[[f32; 4]; 4], p: [f32; 3], (w, h): (u32, u32)) -> ([f32; 2], f32) {
    let c = [0, 1, 2, 3].map(|i| m[0][i] * p[0] + m[1][i] * p[1] + m[2][i] * p[2] + m[3][i]);
    let (x, y, z) = (c[0] / c[3], c[1] / c[3], c[2] / c[3]);
    ([(x + 1.0) * 0.5 * w as f32, (1.0 - y) * 0.5 * h as f32], z)
}

/// ⭐ **A projeção é a do `Orbit`** — perspectiva e ortográfica, ecrã largo e alto, câmara girada
/// e afastada do centro: o pixel da matriz é o `Orbit::project`, e a profundidade cai em `0..1`.
#[test]
fn a_projecao_e_a_do_orbit() {
    let mut pior = 0.0f32;
    for lens in [Lens::Perspective { half_fov: 0.4 }, Lens::Ortho] {
        for (yaw, pitch) in [(0.72, 0.52), (-1.9, -0.3), (2.6, 1.1)] {
            for tamanho in [(640u32, 360u32), (300, 900)] {
                let mut cam = Orbit::from_yaw_pitch(yaw, pitch);
                cam.lens = lens;
                cam.target = [0.2, -0.1, 0.3];
                cam.half_extent = 1.3;
                let m = super::camera(&cam, tamanho).view_proj;
                let screen = Screen::new(tamanho.0, tamanho.1, cam.half_extent);
                for p in [
                    [0.0, 0.0, 0.0],
                    [0.5, 0.3, -0.4],
                    [-0.7, 0.1, 0.6],
                    [0.2, -0.6, 0.0],
                ] {
                    let Some((px, _)) = cam.project(p, screen) else {
                        continue;
                    };
                    let (q, z) = projeta(&m, p, tamanho);
                    pior = pior.max((q[0] - px[0]).abs()).max((q[1] - px[1]).abs());
                    assert!(
                        (0.0..=1.0).contains(&z),
                        "profundidade fora: {z} ({lens:?})"
                    );
                }
            }
        }
    }
    assert!(pior < 1.0e-2, "a malha cai {pior} px longe do traçado");
}
