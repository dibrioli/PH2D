//! **As MARCAS DO ARTISTA no arquivo** (Macros W9, v193, `docs/Macros/07_plano_W9.md`): o blob atravessa
//! o `.ph2dproj` igual, e um v191/v192 (sem o campo) abre sem marcas; o dos KANBANS (v194, ADR-0195,
//! formato em `ph2d_app_kanban::project`) também, e um v191..v193 abre sem eles.

use super::*;

/// **A MESA DE SOM no arquivo** (F30 §B2, v194) — neta pela mesma razão: as fixtures são as portas.
#[path = "project_mesa_tests.rs"]
mod mesa;

/// ⭐ **As marcas e os Kanbans atravessam o arquivo** — os bytes que o palco deu voltam iguais ao abrir.
#[test]
fn as_marcas_do_artista_atravessam_o_arquivo() {
    let marcas = vec![1u8, 1, 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0, 7, 7, 7];
    let file = ProjectFile {
        marcas_do_artista: marcas.clone(),
        kanbans: vec![1, 0, 9],
        ..bare_file(empty_state())
    };
    let path = tmp_path("marcas_v193");
    let bytes = postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).expect("serializa");
    std::fs::write(&path, bytes).expect("grava");
    let (back, _) = headless_app()
        .read_project_file(&path.to_string_lossy())
        .expect("aceite");
    let _ = std::fs::remove_file(&path);
    assert_eq!(
        (back.marcas_do_artista, back.kanbans),
        (marcas, vec![1, 0, 9])
    );
}
