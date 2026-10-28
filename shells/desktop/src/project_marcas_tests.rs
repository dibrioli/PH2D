//! **As MARCAS DO ARTISTA no arquivo** (Macros W9, v193, `docs/Macros/07_plano_W9.md`): o blob atravessa
//! o `.ph2dproj` igual, e um v191/v192 (sem o campo) abre sem marcas; o dos KANBANS (v194, ADR-0195,
//! formato em `ph2d_app_kanban::project`) também, e um v191..v193 abre sem eles.

use super::*;

const V191: &[u8] = include_bytes!("../tests/fixtures/projecto_marcas_v191.ph2dproj");
const V192: &[u8] = include_bytes!("../tests/fixtures/projecto_marcas_v192.ph2dproj");
/// **A MESA DE SOM no arquivo** (F30 §B2, v194) — neta pela mesma razão: as fixtures são as portas.
#[path = "project_mesa_tests.rs"]
mod mesa;

/// **Os itens das listas no arquivo** (F30 §A1, v195) — neta pela mesma razão.
#[path = "project_itens_tests.rs"]
mod itens;

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

/// ⭐ **Um v191/v192 abre sem marcas e um v193 com elas, todos sem Kanbans** — bytes gravados pelo código v193
/// (`c3662cd8e4`), ⛔ nunca os regenere. O v193 é o fixture v192 com a versão trocada e o campo das marcas
/// acrescentado (um tuplo postcard é a concatenação).
#[test]
fn um_v191_um_v192_e_um_v193_abrem_sem_kanbans() {
    let marcas = vec![3u8, 1, 4];
    let mut v193 = V192.to_vec();
    assert_eq!(&v193[..2], &[0xC0, 0x01], "o fixture v192 começa pela versão 192");
    v193[0] = 0xC1;
    v193.extend(postcard::to_allocvec(&marcas).expect("serializa"));
    for (ver, bytes) in [(191u32, V191.to_vec()), (192, V192.to_vec()), (193, v193)] {
        let path = tmp_path(&format!("marcas_v{ver}"));
        std::fs::write(&path, bytes).expect("grava o fixture");
        let lido = headless_app().read_project_file(&path.to_string_lossy());
        let _ = std::fs::remove_file(&path);
        let (back, _) = lido.unwrap_or_else(|| panic!("o v{ver} foi recusado"));
        let esperadas: &[u8] = if ver == 193 { &marcas } else { &[] };
        assert_eq!(back.marcas_do_artista, esperadas, "v{ver}");
        assert!(back.kanbans.is_empty(), "v{ver}");
    }
}
