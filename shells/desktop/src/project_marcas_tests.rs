//! **As MARCAS DO ARTISTA no arquivo** (Macros W9, v193, `docs/Macros/07_plano_W9.md`): o blob atravessa
//! o `.ph2dproj` igual, e um v191/v192 (sem o campo) abre sem marcas.

use super::*;

const V191: &[u8] = include_bytes!("../tests/fixtures/projecto_marcas_v191.ph2dproj");
const V192: &[u8] = include_bytes!("../tests/fixtures/projecto_marcas_v192.ph2dproj");
/// **A MESA DE SOM no arquivo** (F30 §B2, v194) — neta pela mesma razão: as fixtures são as portas.
#[path = "project_mesa_tests.rs"]
mod mesa;

/// ⭐ **As marcas atravessam o arquivo** — os bytes que o palco deu voltam iguais ao abrir.
#[test]
fn as_marcas_do_artista_atravessam_o_arquivo() {
    let marcas = vec![1u8, 1, 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0, 7, 7, 7];
    let file = ProjectFile {
        marcas_do_artista: marcas.clone(),
        ..bare_file(empty_state())
    };
    let path = tmp_path("marcas_v193");
    let bytes = postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).expect("serializa");
    std::fs::write(&path, bytes).expect("grava");
    let (back, _) = headless_app()
        .read_project_file(&path.to_string_lossy())
        .expect("aceite");
    let _ = std::fs::remove_file(&path);
    assert_eq!(back.marcas_do_artista, marcas);
}

/// ⭐ **Um v191 e um v192 abrem, sem marcas** — bytes gravados pelo código v193 (`c3662cd8e4`), ⛔ nunca os regenere.
#[test]
fn um_v191_e_um_v192_abrem_sem_marcas() {
    for (ver, bytes) in [(191u32, V191), (192, V192)] {
        let path = tmp_path(&format!("marcas_v{ver}"));
        std::fs::write(&path, bytes).expect("grava o fixture");
        let lido = headless_app().read_project_file(&path.to_string_lossy());
        let _ = std::fs::remove_file(&path);
        let (back, _) = lido.expect("um v191/v192 é recusado");
        assert!(back.marcas_do_artista.is_empty());
    }
}
