//! **O FORMATO DO JOGO no arquivo** (plano GEF 17, v198): o blob atravessa o `.ph2dproj`, e um v197 (sem o campo)
//! abre com o formato de fábrica.

use super::super::tests::headless_app;
use super::*;

/// Um ficheiro vazio com este `jogo`.
fn arquivo(jogo: Vec<u8>) -> ProjectFile {
    ProjectFile {
        state: empty_state(),
        assets: Vec::new(),
        painted: Vec::new(),
        motion: String::new(),
        timeline: Vec::new(),
        physics: Default::default(),
        tokens: Vec::new(),
        settings: crate::project_settings::collect(Default::default()),
        player_tape: ph2d_physics_ecs::TapeWire::default(),
        sprite_pixels: Vec::new(),
        stable_id_counter: ph2d_ecs::StableId::FIRST,
        pattern_art: Vec::new(),
        boards: Vec::new(),
        scripts: Vec::new(),
        marcas_do_artista: Vec::new(),
        jogo,
        mesa: Vec::new(),
        kanbans: Vec::new(),
        pixel_sprites: Vec::new(),
        sounds: Vec::new(),
        sfx_lab: Vec::new(),
        physics_layers: Vec::new(),
        game_id: 1,
    }
}

/// ⭐ Um projecto Game Boy com a moldura «Só selecionada» volta igual ao abrir.
#[test]
fn o_formato_do_jogo_atravessa_o_arquivo() {
    let mut autorado = ph2d_editor_core::ProjectSettings::default();
    autorado.game_format.preset = ph2d_editor_core::GamePreset::GameBoy;
    autorado.camera_frame = ph2d_editor_core::CameraFrameShow::Selected;
    let file = arquivo(crate::project_settings::collect_jogo(autorado));
    let path = tmp_path("jogo_v198");
    let bytes = postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).expect("serializa");
    std::fs::write(&path, bytes).expect("grava");
    let (back, _) = headless_app()
        .read_project_file(&path.to_string_lossy())
        .expect("aceite");
    let _ = std::fs::remove_file(&path);
    let mut lido = ph2d_editor_core::ProjectSettings::default();
    crate::project_settings::install_jogo(&mut lido, &back.jogo);
    assert_eq!(lido, autorado);
}

/// ⭐ Uma forma «Outline» (anéis com buraco) sai e volta igual pelo `.ph2dproj` (W2b, schema 226).
#[test]
fn a_forma_outline_atravessa_o_arquivo() {
    use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components, snapshot_to_world};
    use ph2d_physics_ecs::{Collider, ColliderOutline, ColliderShape};
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    let o = ColliderOutline {
        rings: vec![
            vec![[0, 0], [64, 0], [64, 64], [0, 64]],
            vec![[16, 16], [16, 48], [48, 48], [48, 16]],
        ]
        .into(),
        picture_px: [4, 4],
        detail_q4: 5,
        custom: true,
        ..ColliderOutline::default()
    };
    let shape = ColliderShape::Outline {
        half_x: 0.5,
        half_y: 0.25,
    };
    let mut sim = ph2d_ecs::SimWorld::new();
    let c = Collider {
        shape,
        ..Collider::default()
    };
    let e = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::default(), c, o.clone()));
    let mut file = arquivo(Vec::new());
    // GEF02 09/11: os anéis à mão (`custom`) e a trilha dos pontos (51/52) atravessam o arquivo.
    let mut tl = ph2d_timeline::TimelineState::new();
    for (kk, v) in ph2d_physics_outline::canais_pontos::publicados(&o) {
        let (p, z) = (ph2d_timeline::PropKind::Field(kk), Default::default());
        tl.doc.upsert_key(
            e.id().to_bits(),
            p,
            z,
            ph2d_timeline::AnimValue::Float(v),
            ph2d_timeline::Interp::Linear,
        );
    }
    file.timeline = ph2d_timeline_persist::serialize(&mut tl, sim.world_mut()).expect("timeline");
    let mut prop = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
    let mut wl = ph2d_ecs::WorklistBuf::new();
    ph2d_ecs::scene::world_to_snapshot(
        sim.world_mut(),
        &mut prop,
        &mut wl,
        &reg,
        &mut file.state.world,
    )
    .expect("snapshot");
    let path = tmp_path("outline_v226");
    std::fs::write(
        &path,
        postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).expect("serializa"),
    )
    .expect("grava");
    let (back, _) = headless_app()
        .read_project_file(&path.to_string_lossy())
        .expect("aceite");
    let _ = std::fs::remove_file(&path);
    assert_eq!(back.state.world, file.state.world);
    let mut dst = ph2d_ecs::SimWorld::new();
    let e = snapshot_to_world(dst.world_mut(), &back.state.world, &reg).expect("restaura")[0];
    assert_eq!(dst.world().get::<ColliderOutline>(e), Some(&o));
    let tl = ph2d_timeline_persist::install_from_project(&back.timeline).expect("timeline");
    assert_eq!(tl.doc.bindings().len(), 16, "8 pontos × (X, Y)");
    assert_eq!(dst.world().get::<Collider>(e).map(|c| c.shape), Some(shape));
}
