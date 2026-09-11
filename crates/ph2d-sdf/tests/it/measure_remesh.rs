//! **De que é feito um remesh, e quanto custa por resolução.**
//!
//! `cargo test -p ph2d-sdf --release --test measure_remesh -- --ignored --nocapture`
//!
//! O `DEFAULT_RESOLUTION = 150` é o número da REFERÊNCIA, e o §0 do `CLAUDE.md`
//! diz que um limite herdado vale um palpite até alguém medi-lo. Esta sonda é a
//! tabela que decide se ele fica, sobe ou desce — e ela roda o produto pela
//! porta do produto (`remesh`), não um laço próprio.

use std::time::Instant;

use ph2d_mesh::{Mesh, shapes};
use ph2d_sdf::{VoxelField, remesh, surface_nets};

fn stamp(label: &str, t: Instant) -> Instant {
    eprintln!(
        "    {label:<14} {:>8.1} ms",
        t.elapsed().as_secs_f64() * 1e3
    );
    Instant::now()
}

#[test]
#[ignore = "sonda de medição"]
fn what_a_remesh_costs_by_resolution() {
    let m = shapes::uv_sphere(96, 144, 1.0);
    eprintln!(
        "\nmalha de entrada: {} vértices / {} faces\n",
        m.vert_count(),
        m.face_count()
    );

    for res in [16u32, 32, 150, 512, 640, 768] {
        let t0 = Instant::now();
        // ⚠️ A RECUSA é dado, não erro da sonda: uma resolução em que o campo
        // vaza é exatamente o que esta tabela precisa mostrar. Um `expect` aqui
        // mataria a varredura no primeiro vazamento e esconderia o resto dela.
        let (out, report) = match remesh(&m, res) {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "resolução {res:>4}: {:>8.1} ms | RECUSA -- {e}",
                    t0.elapsed().as_secs_f64() * 1e3
                );
                continue;
            }
        };
        let total = t0.elapsed().as_secs_f64() * 1e3;

        // A memória do campo: 4 B de distância + 3 B de aresta atravessada.
        let mb = report.cells as f64 * 7.0 / (1024.0 * 1024.0);
        eprintln!(
            "resolução {res:>4}: {total:>8.1} ms | {:>9} células ({mb:>6.1} MB) | saída {:>7} v / {:>7} f",
            report.cells,
            out.vert_count(),
            out.face_count()
        );
    }
}

#[test]
#[ignore = "sonda de medição"]
fn what_a_remesh_is_made_of() {
    let m = shapes::uv_sphere(96, 144, 1.0);
    for res in [64u32, 150] {
        eprintln!("\n--- resolução {res} ---");
        let mut t = Instant::now();

        let mut closed = Mesh::from_parts(m.positions().to_vec(), m.faces().to_vec()).unwrap();
        ph2d_mesh::fill_holes(&mut closed);
        t = stamp("tapar", t);

        let mut field = VoxelField::for_bounds(closed.bounds(), res);
        t = stamp("alocar", t);

        field.voxelize(&closed);
        t = stamp("voxelizar", t);

        field.flood_fill();
        t = stamp("flood fill", t);

        let out = surface_nets(&field).unwrap();
        stamp("extrair", t);
        eprintln!("    saída: {} v / {} f", out.vert_count(), out.face_count());
    }
}
