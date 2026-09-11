//! **A sonda que decide a faixa do raio em pixels** — §0: meça antes de limitar.
//!
//! O raio do pincel deixou de ser fração do MODELO e passou a ser pixels de
//! TELA (item 6b). Os dois limites da faixa antiga tinham mecanismo escrito —
//! *"abaixo de uma aresta não pega vértice"* e *"acima de meio modelo é um
//! deformador global"* — e nenhum dos dois sobrevive à troca de unidade sem ser
//! re-medido, porque quantos vértices cabem num pixel é função do ZOOM.
//!
//! Rodar: `cargo test -p ph2d-mesh-render --test measure_screen_radius -- --ignored --nocapture`

use ph2d_mesh_render::Camera3d;

/// A cena do smoke: a mesma esfera e o mesmo enquadramento que o artista vê.
fn scene(size: (u32, u32)) -> (ph2d_mesh::Mesh, Camera3d) {
    let mesh = ph2d_mesh::shapes::uv_sphere(96, 144, 1.0);
    let aspect = size.0 as f32 / size.1 as f32;
    let mut cam = Camera3d {
        yaw: 0.6,
        pitch: 0.35,
        ..Camera3d::default()
    };
    cam.frame(mesh.bounds(), aspect);
    (mesh, cam)
}

/// Quantos vértices caem dentro de um disco de `px` pixels em torno do ponto da
/// frente do modelo — a pergunta que o piso da faixa responde.
fn verts_under(mesh: &ph2d_mesh::Mesh, cam: &Camera3d, size: (u32, u32), px: f32) -> usize {
    let front = front_point(mesh, cam);
    let Some((cx, cy)) = cam.project(front, size) else {
        return 0;
    };
    mesh.positions()
        .iter()
        .filter(|p| {
            cam.project(**p, size)
                .is_some_and(|(x, y)| (x - cx).hypot(y - cy) <= px)
        })
        .count()
}

/// O ponto da malha mais próximo do olho — onde um clique no centro da tela cai.
fn front_point(mesh: &ph2d_mesh::Mesh, cam: &Camera3d) -> [f32; 3] {
    let eye = cam.eye();
    *mesh
        .positions()
        .iter()
        .min_by(|a, b| {
            let da = (glam::Vec3::from(**a) - eye).length_squared();
            let db = (glam::Vec3::from(**b) - eye).length_squared();
            da.total_cmp(&db)
        })
        .expect("a esfera tem vértices")
}

/// Quantos pixels de ALTURA o modelo enquadrado ocupa — a régua do teto.
fn model_height_px(mesh: &ph2d_mesh::Mesh, cam: &Camera3d, size: (u32, u32)) -> f32 {
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for p in mesh.positions() {
        if let Some((_, y)) = cam.project(*p, size) {
            lo = lo.min(y);
            hi = hi.max(y);
        }
    }
    hi - lo
}

#[test]
#[ignore = "sonda de medição; roda com --ignored"]
fn what_a_screen_radius_buys_at_each_size() {
    for size in [(1280u32, 720u32), (2560, 1440)] {
        let (mesh, cam) = scene(size);
        let h = model_height_px(&mesh, &cam, size);
        println!(
            "\n=== viewport {}x{} · o modelo mede {h:.0} px de altura ===",
            size.0, size.1
        );
        println!(" raio px | vértices sob o dab | fração da altura do modelo");
        for px in [0.5f32, 1.0, 2.0, 4.0, 8.0, 24.0, 64.0, 160.0, 400.0] {
            let n = verts_under(&mesh, &cam, size, px);
            println!("{px:8.1} | {n:18} | {:.2}", 2.0 * px / h);
        }

        // O mesmo pincel, com a câmera APROXIMADA: é isto que a wave entrega —
        // o raio de mundo encolhe e o de tela fica, então o artista alcança
        // detalhe fino sem mexer no slider.
        let near = Camera3d {
            distance: cam.distance * 0.25,
            ..cam
        };
        let front = front_point(&mesh, &cam);
        for px in [4.0f32, 24.0] {
            let far_r = cam.world_radius_for_screen_px(front, px, size);
            let near_r = near.world_radius_for_screen_px(front, px, size);
            println!(
                "  {px:.0} px = {far_r:.4} de mundo enquadrado, {near_r:.4} a 1/4 da distância"
            );
        }
    }
}
