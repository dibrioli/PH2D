//! As RECEITAS DE SOM no arquivo (v217): o blob atravessa igual e um v216 abre sem elas. O blob: `ph2d_sfx_recipe`.

use super::sounds::written;
use super::*;

#[test]
fn as_receitas_atravessam_o_arquivo_byte_a_byte() {
    let file = ProjectFile {
        sfx_lab: vec![1, 2, 3],
        ..bare_file(empty_state())
    };
    let path = written("sfx_v217", &(PROJECT_SCHEMA, &file));
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    assert_eq!(lido.expect("aceite").0.sfx_lab, [1, 2, 3]);
}
