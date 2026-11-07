//! **Os SONS do projeto no arquivo** (v215, GEF 21 E2): o blob atravessa o `.ph2dproj` igual e um v214
//! (sem o campo) abre sem eles. O import dos caminhos antigos é gateado em `ph2d_app_components::sound_project`.

use super::*;

pub(super) fn written(name: &str, v: &impl serde::Serialize) -> std::path::PathBuf {
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
