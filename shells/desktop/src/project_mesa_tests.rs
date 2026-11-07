//! **A MESA DE SOM no arquivo** (F30 §B2, v194): o blob atravessa o `.ph2dproj`, e um v193 (ou
//! anterior, sem o campo) abre com a mesa no padrão.

use super::*;

/// ⭐ **A mesa atravessa o arquivo** — o que o artista deixou na mesa volta igual ao abrir.
#[test]
fn a_mesa_viaja_no_projecto() {
    let mut mesa = ph2d_audio::Mesa::default();
    mesa.bus[1].ganho = 0.4;
    mesa.ducking_chave = 0;
    let file = ProjectFile {
        mesa: mesa.to_bytes(),
        ..bare_file(empty_state())
    };
    let path = tmp_path("mesa_v194");
    std::fs::write(
        &path,
        postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).expect("serializa"),
    )
    .expect("grava");
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    let (back, _) = lido.expect("aceite");
    assert_eq!(
        ph2d_audio::Mesa::from_bytes(&back.mesa).expect("mesa"),
        mesa
    );
}
