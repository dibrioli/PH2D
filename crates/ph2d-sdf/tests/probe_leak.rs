//! Quantas resolucoes vazam, antes e depois.
use ph2d_mesh::{Mesh, shapes, shapes_open};
use ph2d_sdf::VoxelField;

fn leaks(closed: &Mesh, res: u32) -> bool {
    let mut f = VoxelField::for_bounds(closed.bounds(), res);
    f.voxelize(closed);
    f.flood_fill() == 0
}

fn closed_of(m: &Mesh) -> Mesh {
    let mut c = Mesh::from_parts(m.positions().to_vec(), m.faces().to_vec()).unwrap();
    let _ = ph2d_mesh::fill_holes(&mut c);
    c
}

#[test]
#[ignore = "sonda"]
fn how_many_resolutions_leak() {
    for (name, m) in [
        ("esfera uv(96,144)", shapes::uv_sphere(96, 144, 1.0)),
        ("esfera uv(24,32)", shapes::uv_sphere(24, 32, 1.0)),
        ("cubo", shapes::cube(1.0)),
        ("tubo aberto", shapes_open::open_tube3()),
    ] {
        let c = closed_of(&m);
        let bad: Vec<u32> = (40u32..=400).filter(|r| leaks(&c, *r)).collect();
        eprintln!(
            "{name:20} vazam {:3} de 361  {}",
            bad.len(),
            if bad.len() <= 14 {
                format!("{bad:?}")
            } else {
                format!("[{:?} ...]", &bad[..14])
            }
        );
    }
}
