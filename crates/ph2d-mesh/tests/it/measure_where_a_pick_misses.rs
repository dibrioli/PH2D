//! **ONDE UM PICK ERRA** — a sonda que separa *o raio saiu da silhueta* de *o
//! teste de interseção tem furo*.
//!
//! Ela existe porque a atribuição fácil já falhou uma vez: a sonda do traço
//! (`ph2d-sculpt3d/tests/measure_stroke_ripple`, §3) mediu **~10%** dos dabs a
//! não serem aplicados, e o gate irmão
//! (`closed_mesh_never_leaks_a_ray`) mede o vazamento do Möller–Trumbore em
//! **0,016%**. Três ordens de grandeza separam os dois números, então **eles não
//! são o mesmo defeito**.
//!
//! O oráculo aqui é ANALÍTICO e não tem tolerância a escolher: um raio paralelo
//! a `-z` partindo de `(x, y, +d)` contra uma esfera de raio `R` centrada na
//! origem **tem** de acertar se `x² + y² < R²` e **não pode** acertar se
//! `> R²`. É a silhueta, e ela é conhecida em forma fechada.
//!
//! Rode com `-- --ignored --nocapture`.

use ph2d_mesh::{Mesh, Ray, shapes};

fn sweep(mesh: &Mesh, label: &str, radius: f32, n: usize) {
    // A grade cobre `[-1,2 R, +1,2 R]` para incluir o lado de fora da silhueta,
    // que é o CONTROLE: sem ele, um `raycast` que acertasse sempre passaria.
    let span = radius * 1.2;
    let (mut inside, mut inside_missed) = (0usize, 0usize);
    let (mut outside, mut outside_hit) = (0usize, 0usize);
    // A banda perto da silhueta é contada à parte: ali a esfera POLIGONAL é
    // sempre mais estreita que a analítica (uma corda corta o arco), então um
    // miss é honesto e não diz nada sobre estanqueidade.
    let mut near_rim = 0usize;
    let rim = 0.98 * radius;
    let mut first: Option<(f32, f32)> = None;

    for iy in 0..n {
        for ix in 0..n {
            let x = -span + 2.0 * span * (ix as f32 + 0.5) / n as f32;
            let y = -span + 2.0 * span * (iy as f32 + 0.5) / n as f32;
            let r = (x * x + y * y).sqrt();
            let hit = mesh
                .raycast(&Ray::new([x, y, radius * 3.0], [0.0, 0.0, -1.0]))
                .is_some();
            if r < radius {
                inside += 1;
                if !hit {
                    if r > rim {
                        near_rim += 1;
                    } else {
                        inside_missed += 1;
                        first.get_or_insert((x, y));
                    }
                }
            } else {
                outside += 1;
                if hit {
                    outside_hit += 1;
                }
            }
        }
    }
    println!(
        "{label:<22} dentro {inside:>6}  MISSOU {inside_missed:>5} ({:>6.3}%)  \
         perto da borda {near_rim:>4}  |  fora {outside:>6}  acertou {outside_hit}",
        100.0 * inside_missed as f32 / inside as f32
    );
    if let Some((x, y)) = first {
        println!("{:<22} 1º furo em ({x:.6}, {y:.6})", "");
    }
}

#[test]
#[ignore = "sonda: imprime, não afirma"]
fn measure_where_a_pick_misses() {
    println!("\n=== ONDE UM PICK ERRA (grade paralela, oráculo = silhueta) ===\n");
    sweep(&shapes::sculpt_sphere(1.0), "esfera de fábrica", 1.0, 400);
    sweep(&shapes::uv_sphere(48, 72, 1.0), "uv_sphere 48x72", 1.0, 400);
    sweep(&shapes::uv_sphere(12, 18, 1.0), "uv_sphere 12x18", 1.0, 400);

    // A MESMA esfera de fábrica depois de um deslocamento, pela porta que um
    // pincel usa (`positions_mut` + `refresh_region`) — o barro que subiu. Se o
    // número mudar aqui, a causa é a malha em MOVIMENTO (octree, caixas), e não
    // o teste de interseção.
    let mut moved = shapes::sculpt_sphere(1.0);
    let n = moved.positions().len();
    {
        let pos = moved.positions_mut();
        for (i, p) in pos.iter_mut().enumerate() {
            let s = 1.0 + 0.15 * ((i as f32 * 0.37).sin());
            *p = [p[0] * s, p[1] * s, p[2] * s];
        }
    }
    let all: Vec<u32> = (0..n as u32).collect();
    let mut scratch = ph2d_mesh::RegionScratch::default();
    moved.refresh_region(&all, &mut scratch);
    println!("(malha deslocada pela porta do pincel: {n} vértices)");
    sweep(&moved, "fábrica deslocada", 1.15, 400);
}
