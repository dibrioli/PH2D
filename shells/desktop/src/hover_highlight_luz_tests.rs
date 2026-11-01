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
        frame_order: &ph2d_render::draw_bands::FrameOrder::default(),
        present: &mut present,
        camera: &camera,
        window_size: janela,
        pixels_per_meter: 100.0,
        camera_icon_px: ph2d_editor_core::game_format::CAMERA_ICON_PX,
        camera_frames: &[],
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
    // ⭐ À frente, sem cortar a lista (dono 01/11): o clique seguinte desce para o que está por baixo.
    assert_eq!(
        pick(&sim, centro),
        vec![luz, vazio],
        "o ícone da lâmpada tem prioridade e o vazio por baixo continua alcançável"
    );
    let longe = (centro.0 + 200.0, centro.1);
    assert!(
        !pick(&sim, longe).contains(&luz),
        "fora do ícone a lâmpada não rouba o clique"
    );
}

/// Uma imagem já desenhada, como o espelho do `present` a guarda: o pick de sprites lê isto.
fn imagem_desenhada(present: &mut ph2d_ecs::PresentWorld, sim: &mut ph2d_ecs::SimWorld) -> u64 {
    let e = sim.world_mut().spawn(ph2d_ecs::Transform::IDENTITY).id();
    let mut ri: ph2d_render::sprite::RenderInstance = bytemuck::Zeroable::zeroed();
    ri.size = [2.0, 2.0];
    ri.basis = ph2d_render::sprite::RenderInstance::IDENTITY_BASIS;
    let gt = ph2d_ecs::GlobalTransform::from_transform(ph2d_ecs::Transform::IDENTITY);
    present.world_mut().spawn((ph2d_ecs::SimRef(e), gt, ri));
    e.to_bits()
}

/// ⭐⭐ **O 1.º clique é do que o quadro DESENHOU por cima** (dono 01/11), e não da ordem em que o
/// pick de cada família os encontra: as duas imagens sobrepostas chegam do pick de sprites com a
/// mais NOVA à frente; a ordem do quadro põe a mais VELHA por cima ⇒ é ela a primeira.
///
/// Mutação que deve sangrar: tirar o `sort_by_key` do `pick_objects_at`.
#[test]
fn o_primeiro_clique_e_do_que_o_quadro_desenhou_por_cima() {
    let centro = ph2d_render::Camera2d::default()
        .world_to_screen([0.0, 0.0], ph2d_host::WindowSize::new(1280, 720));
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = ph2d_ecs::PresentWorld::new();
    let velha = imagem_desenhada(&mut present, &mut sim);
    let nova = imagem_desenhada(&mut present, &mut sim);
    let mut ordem = ph2d_render::draw_bands::FrameOrder::default();
    for (rank, bits) in [(0u32, nova), (1, velha)] {
        ordem.record(rank, None);
        ordem.enrol(bits, rank);
    }
    assert_eq!(
        pega(&sim, &mut present, &ordem, centro),
        vec![velha, nova],
        "o 1.º clique nao foi do que esta' desenhado por cima"
    );

    // ⛔ Travada, a de cima deixa de se pegar no canvas (como o vetor e o Flip já faziam).
    sim.world_mut()
        .entity_mut(ph2d_ecs::Entity::from_bits(velha))
        .insert(ph2d_ecs::Locked);
    assert_eq!(
        pega(&sim, &mut present, &ordem, centro),
        vec![nova],
        "uma imagem travada ainda se pega no canvas"
    );
}

fn pega(
    sim: &ph2d_ecs::SimWorld,
    present: &mut ph2d_ecs::PresentWorld,
    ordem: &ph2d_render::draw_bands::FrameOrder,
    tela: (f32, f32),
) -> Vec<u64> {
    let (vec_scene, flip) = (ph2d_vec_scene::VecScene::new(), ph2d_flip::FlipDoc::new());
    let camera = ph2d_render::Camera2d::default();
    let mut w = PickWorld {
        sim,
        vec_scene: &vec_scene,
        flip: &flip,
        frame_order: ordem,
        present,
        camera: &camera,
        window_size: ph2d_host::WindowSize::new(1280, 720),
        pixels_per_meter: 100.0,
        camera_icon_px: ph2d_editor_core::game_format::CAMERA_ICON_PX,
        camera_frames: &[],
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
