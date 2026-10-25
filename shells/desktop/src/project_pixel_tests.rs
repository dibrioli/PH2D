//! **Os PIXEL SPRITES no arquivo** (v194, ADR-0192): o blob atravessa o `.ph2dproj` igual, um v193
//! (sem o campo) abre sem eles, e um blob ilegível recusa o projecto inteiro.

use super::*;

/// Um blob com um Pixel Sprite desenhado (a chave `1`, a do objecto).
fn pixel_blob() -> Vec<u8> {
    let mut app = ph2d_app_pixel::PixelApp::default();
    let key = app.create(Default::default()).expect("sprite");
    let mut sim = ph2d_ecs::SimWorld::new();
    sim.world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, ph2d_ecs::PixelSprite(key)));
    let bytes = ph2d_app_pixel::persist::to_bytes(&app, sim.world());
    assert!(!bytes.is_empty(), "um objecto vivo grava o documento dele");
    bytes
}

#[test]
fn the_pixel_sprites_cross_the_file_byte_for_byte() {
    let blob = pixel_blob();
    let file = ProjectFile {
        pixel_sprites: blob.clone(),
        ..bare_file(empty_state())
    };
    let path = tmp_path("pixel_v194");
    let bytes = postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).expect("serializa");
    std::fs::write(&path, bytes).expect("grava");
    let (back, _) = headless_app()
        .read_project_file(&path.to_string_lossy())
        .expect("aceite");
    let _ = std::fs::remove_file(&path);
    assert_eq!(back.pixel_sprites, blob);
}

/// ⚠️ Os bytes do v193 são escritos campo a campo (a forma de então, sem o campo) — não a partir do
/// tipo vivo, senão o teste provaria o braço com a hipótese dele.
#[test]
fn a_v193_opens_without_pixel_sprites() {
    let f = bare_file(empty_state());
    let v193 = (
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
    let path = tmp_path("pixel_v193");
    let bytes = postcard::to_allocvec(&(193u32, v193)).expect("serializa");
    std::fs::write(&path, bytes).expect("grava");
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    let (back, _) = lido.expect("um v193 é recusado");
    assert!(back.pixel_sprites.is_empty());
}

#[test]
fn an_unreadable_pixel_blob_refuses_the_whole_file_and_leaves_the_session_alone() {
    let mut app = headless_app();
    app.undo
        .push_undo(empty_state(), crate::undo::SelectionMark::default());
    let file = ProjectFile {
        pixel_sprites: vec![0xff; 4],
        ..bare_file(empty_state())
    };
    let path = tmp_path("load_bad_pixel");
    let bytes = postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).expect("serializa");
    std::fs::write(&path, bytes).expect("grava");
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    assert!(
        app.undo.can_undo(),
        "o load mutou a sessão antes de recusar os pixel sprites ilegíveis"
    );
}

#[test]
fn a_well_formed_pixel_blob_opens_and_installs_the_documents() {
    let mut app = headless_app();
    app.undo
        .push_undo(empty_state(), crate::undo::SelectionMark::default());
    let file = ProjectFile {
        pixel_sprites: pixel_blob(),
        ..bare_file(empty_state())
    };
    let path = tmp_path("load_good_pixel");
    let bytes = postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).expect("serializa");
    std::fs::write(&path, bytes).expect("grava");
    app.project_load_from(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    assert!(!app.undo.can_undo(), "o arquivo foi RECUSADO");
    assert!(app.pixel.doc(1).is_some(), "o documento não foi instalado");
}
