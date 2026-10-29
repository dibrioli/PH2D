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
        input_map: ph2d_input::InputMap::new(),
        pattern_art: Vec::new(),
        boards: Vec::new(),
        scripts: Vec::new(),
        marcas_do_artista: Vec::new(),
        jogo,
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

/// ⭐ **Um v197 abre, com 16:9**. ⚠️ Os bytes são escritos campo a campo (a forma de então, sem o campo).
#[test]
fn um_v197_abre_com_o_formato_de_fabrica() {
    let f = arquivo(Vec::new());
    let v197 = (
        &f.assets,
        &f.painted,
        &f.motion,
        &f.timeline,
        &f.physics,
        &f.tokens,
        &f.settings,
        &f.player_tape,
        &f.sprite_pixels,
        &f.stable_id_counter,
        &f.input_map,
        &f.pattern_art,
        &f.boards,
        &f.scripts,
        &f.marcas_do_artista,
    );
    let path = tmp_path("jogo_v197");
    // A cena vetorial de um v197 é a v23 (sem as regras de CAD, que nasceram no 201): o postcard é a
    // concatenação dos campos, e a vazia viva acaba nos dois zeros das regras.
    let st = &f.state;
    let vec_vivo = postcard::to_allocvec(&*st.vec).expect("serializa");
    assert_eq!(
        &vec_vivo[vec_vivo.len() - 2..],
        &[0, 0],
        "a cena vazia acaba nas regras vazias"
    );
    let mut bytes = postcard::to_allocvec(&(197u32, &st.world)).expect("serializa");
    bytes.extend_from_slice(&vec_vivo[..vec_vivo.len() - 2]);
    let resto = (&st.flip, &st.guides, &st.ui_states, &st.library, &st.tags);
    bytes.extend(postcard::to_allocvec(&resto).expect("serializa"));
    bytes.extend(postcard::to_allocvec(&v197).expect("serializa"));
    std::fs::write(&path, bytes).expect("grava");
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    let (back, _) = lido.expect("um v197 é recusado");
    assert!(back.jogo.is_empty());
    let mut s = ph2d_editor_core::ProjectSettings::default();
    s.game_format.preset = ph2d_editor_core::GamePreset::Nes;
    crate::project_settings::install_jogo(&mut s, &back.jogo);
    assert_eq!(s.game_format, ph2d_editor_core::GameFormat::default());
}

/// ⭐ **Um blob `jogo` v1 (a onda 17, sem o ícone) abre com o formato dele e o ícone de fábrica** (17b).
#[test]
fn um_jogo_v1_abre_com_o_icone_de_fabrica() {
    let v1 = postcard::to_allocvec(&(1u8, 8u8, [1280u32, 720u32], 1u8)).expect("serializa");
    let mut s = ph2d_editor_core::ProjectSettings {
        camera_icon_px: 100.0,
        ..Default::default()
    };
    crate::project_settings::install_jogo(&mut s, &v1);
    assert_eq!(s.game_format.preset, ph2d_editor_core::GamePreset::GameBoy);
    assert_eq!(s.camera_frame, ph2d_editor_core::CameraFrameShow::Selected);
    assert_eq!(
        s.camera_icon_px,
        ph2d_editor_core::game_format::CAMERA_ICON_PX
    );
}
