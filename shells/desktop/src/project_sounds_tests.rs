//! **Os SONS do projeto no arquivo** (v215, GEF 21 E2): o blob atravessa o `.ph2dproj` igual e um v214
//! (sem o campo) abre sem eles. O import dos caminhos antigos é gateado em `ph2d_app_components::sound_project`.

use super::*;

fn written(name: &str, v: &impl serde::Serialize) -> std::path::PathBuf {
    let path = tmp_path(name);
    std::fs::write(&path, postcard::to_allocvec(v).unwrap()).expect("grava");
    path
}

#[test]
fn the_project_sounds_cross_the_file_byte_for_byte() {
    let mut lib = ph2d_ecs::ProjectSounds::default();
    lib.add("moeda.ogg", vec![1, 2, 3]);
    let file = ProjectFile {
        sounds: lib.to_bytes(),
        ..bare_file(empty_state())
    };
    let path = written("sons_v215", &(PROJECT_SCHEMA, &file));
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    assert_eq!(lido.expect("aceite").0.sounds, lib.to_bytes());
}

/// ⚠️ Os bytes do v214 são escritos campo a campo (a forma de então, sem o campo) — não a partir do tipo
/// vivo, senão o teste provaria o braço com a hipótese dele.
#[test]
fn a_v214_opens_without_sounds() {
    let f = bare_file(empty_state());
    let v214 = (
        (&f.state, &f.assets, &f.painted, &f.motion),
        (&f.timeline, &f.physics, &f.tokens, &f.settings),
        (&f.player_tape, &f.sprite_pixels, &f.stable_id_counter),
        (&f.input_map, &f.pattern_art, &f.boards, &f.scripts),
        (
            &f.marcas_do_artista,
            &f.jogo,
            &f.mesa,
            &f.kanbans,
            vec![7u8],
        ),
    );
    let path = written(
        "sons_v214",
        &(crate::project_schema::PROJECT_SCHEMA_SEM_SONS, v214),
    );
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    let (f, _) = lido.expect("um v214 é recusado");
    assert_eq!(
        f.pixel_sprites,
        [7],
        "o último campo do v214 lê-se no sítio dele"
    );
    assert!(f.sounds.is_empty());
}
