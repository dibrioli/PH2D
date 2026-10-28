//! **O FORMATO DO JOGO no arquivo** (plano GEF 17, v198): o blob atravessa o `.ph2dproj`, e um v197 (sem o campo)
//! abre com o formato de fábrica.

use super::*;

/// ⭐ Um projecto Game Boy com a moldura «Só selecionada» volta igual ao abrir.
#[test]
fn o_formato_do_jogo_atravessa_o_arquivo() {
    let mut autorado = ph2d_editor_core::ProjectSettings::default();
    autorado.game_format.preset = ph2d_editor_core::GamePreset::GameBoy;
    autorado.camera_frame = ph2d_editor_core::CameraFrameShow::Selected;
    let file = ProjectFile {
        jogo: crate::project_settings::collect_jogo(autorado),
        ..bare_file(empty_state())
    };
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
    let f = bare_file(empty_state());
    let v197 = (
        &f.state,
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
    let bytes = postcard::to_allocvec(&(197u32, v197)).expect("serializa");
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
