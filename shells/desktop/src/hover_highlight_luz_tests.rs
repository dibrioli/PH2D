//! ⭐ **O ícone da LÂMPADA ganha o clique** (smoke do dono, 2026-10-10: *«o gizmo da lâmpada deve
//! ter prioridade quando o mouse seleciona objetos sobrepostos»*) — pelo `pick_objects_at`, a porta
//! dos três sítios do clique.

use super::{PickWorld, pick_objects_at};

fn pick(sim: &ph2d_ecs::SimWorld, tela: (f32, f32)) -> Vec<u64> {
    let (camera, janela) = (
        ph2d_render::Camera2d::default(),
        ph2d_host::WindowSize::new(1280, 720),
    );
    let (vec_scene, flip) = (ph2d_vec_scene::VecScene::new(), ph2d_flip::FlipDoc::new());
    let mut present = ph2d_ecs::PresentWorld::new();
    let mut w = PickWorld {
        sim,
        vec_scene: &vec_scene,
        flip: &flip,
        present: &mut present,
        camera: &camera,
        window_size: janela,
        pixels_per_meter: 100.0,
    };
    pick_objects_at(
        &mut w,
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
        tela,
    )
}

#[test]
fn o_icone_da_lampada_ganha_o_clique_sobre_um_vazio_no_mesmo_ponto() {
    let centro = ph2d_render::Camera2d::default()
        .world_to_screen([0.0, 0.0], ph2d_host::WindowSize::new(1280, 720));
    let mut sim = ph2d_ecs::SimWorld::new();
    let vazio = ph2d_app_components::object_add::spawn_empty_root(&mut sim, "Vazio");
    assert_eq!(
        pick(&sim, centro),
        vec![vazio],
        "controlo: o vazio sozinho é pegado"
    );
    let luz = ph2d_app_light2d::object_add::add(ph2d_app_light2d::object_add::LUZ, &mut sim)
        .expect("é da luz");
    assert_eq!(
        pick(&sim, centro),
        vec![luz],
        "o ícone da lâmpada tem prioridade"
    );
    let longe = (centro.0 + 200.0, centro.1);
    assert!(
        !pick(&sim, longe).contains(&luz),
        "fora do ícone a lâmpada não rouba o clique"
    );
}
