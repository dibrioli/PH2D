//! **SONDA — o pixel que produziu o raio é o pixel de volta?**
//!
//! O report é *"o lugar onde o mouse toca não corresponde ao local na malha"*, e
//! a fiação tem duas metades que podem discordar: a que converte um CLIQUE em
//! raio (`ray_through`) e a que desenha (`view_proj`, de que o `project` sai).
//! Se as duas usarem molduras diferentes o barro é esculpido num lugar e
//! aparece noutro — e nenhum gate de nenhuma das metades enxerga isso.

use ph2d_mesh_render::Camera3d;

#[test]
#[ignore = "sonda"]
fn a_click_comes_back_to_the_pixel_it_left_from() {
    // Duas molduras: quadrada e a larga de uma janela real. Um erro de aspecto
    // é INVISÍVEL num viewport quadrado (`w/h == 1`), que é exatamente a fixture
    // em que ele passaria.
    for size in [(1000u32, 1000u32), (2560, 1440), (1440, 2560)] {
        let mut worst = 0.0f32;
        for cam in [
            Camera3d {
                yaw: 0.6,
                pitch: 0.35,
                ..Camera3d::default()
            },
            Camera3d {
                yaw: -2.1,
                pitch: -0.8,
                ..Camera3d::default()
            },
        ] {
            for px in [0.1f32, 0.3, 0.5, 0.7, 0.9] {
                for py in [0.1f32, 0.35, 0.5, 0.8] {
                    let (x, y) = (px * size.0 as f32, py * size.1 as f32);
                    let ray = cam.ray_through(x, y, size);
                    // Um ponto a uma distância qualquer ao longo do raio TEM de
                    // projetar de volta no mesmo pixel — é a definição de as
                    // duas metades concordarem.
                    let o = ray.origin();
                    let d = ray.dir();
                    let at = [o[0] + d[0] * 3.0, o[1] + d[1] * 3.0, o[2] + d[2] * 3.0];
                    let Some((bx, by)) = cam.project(at, size) else {
                        println!("  {size:?} ({x:.0},{y:.0}) -> SEM PROJECAO");
                        continue;
                    };
                    worst = worst.max((bx - x).hypot(by - y));
                }
            }
        }
        println!("viewport {size:?}: pior erro de volta = {worst:.4} px");
    }
}
