//! As CAMADAS COM TINTA DO WET no arquivo (v225): a marca atravessa igual.

use super::sounds::written;
use super::*;

/// Um documento pintado de uma camada, com a camada marcada como tinta do Wet.
fn documento() -> ph2d_tool_painter::PaintedDocument {
    let mut t = ph2d_tool_painter::PainterTool::default();
    t.bind_document(1, vec![255; 4 * 4 * 4], 4, 4);
    let mut doc = t
        .collect_documents(&[(1, 9)].into_iter().collect())
        .pop()
        .expect("o documento ligado");
    let camada = doc.layers.active().expect("uma camada");
    doc.tintas_molhadas.insert(camada);
    doc
}

#[test]
fn a_marca_da_tinta_do_wet_atravessa_o_arquivo() {
    let doc = documento();
    let file = ProjectFile {
        painted: vec![doc.clone()],
        ..bare_file(empty_state())
    };
    let path = written("tintas_v218", &(PROJECT_SCHEMA, &file));
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    assert_eq!(lido.expect("aceite").0.painted, [doc]);
}
