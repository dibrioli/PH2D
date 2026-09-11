//! Onde a faixa PARA — e se ela para.
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

fn grid(n: usize, half: f32) -> ph2d_mesh::Mesh {
    let mut pos = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            let f = |k: usize| (k as f32 / n as f32) * 2.0 * half - half;
            pos.push([f(i), f(j), 0.0]);
        }
    }
    let at = |i: usize, j: usize| (j * (n + 1) + i) as u32;
    let mut faces = Vec::new();
    for j in 0..n {
        for i in 0..n {
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j),
                at(i + 1, j + 1),
            ));
            faces.push(ph2d_mesh::Face::tri(
                at(i, j),
                at(i + 1, j + 1),
                at(i, j + 1),
            ));
        }
    }
    ph2d_mesh::Mesh::from_parts(pos, faces).expect("ok")
}

#[test]
#[ignore = "sonda"]
fn measure_where_the_strip_stops() {
    let r = 0.4f32;
    let plane = r * ph2d_sculpt3d::STRIP_PLANE_FRACTION;
    println!(
        "o plano congelado esta' em z = {plane:.4} (lift {:.2} r)",
        ph2d_sculpt3d::STRIP_PLANE_FRACTION
    );
    for acc in [false, true] {
        print!("accumulate {acc:<5}:");
        for dabs in [1usize, 3, 9, 27, 81, 243] {
            let mut mesh = grid(60, 1.5);
            let b = Brush {
                verb: Verb::ClayStrips,
                radius: r,
                strength: 1.0,
                accumulate: acc,
                ..Brush::default()
            };
            let mut s = SculptStroke::default();
            s.begin(&mesh);
            for k in 0..dabs {
                let x = (k as f32 - (dabs - 1) as f32 * 0.5) * 0.15 * r;
                s.dab(
                    &mut mesh,
                    &b,
                    &Dab::at([x, 0.0, 0.0], r, [0.0, 0.0, -1.0]),
                    Symmetry::default(),
                );
            }
            let p = mesh
                .positions()
                .iter()
                .map(|q| q[2])
                .fold(f32::NEG_INFINITY, f32::max);
            print!("  {dabs}:{p:.4}");
        }
        println!();
    }
}
