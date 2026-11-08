//! As CAMADAS COM TINTA DO WET no arquivo (v218): a marca atravessa igual e um v217 abre sem ela, com os
//! campos depois do documento pintado no sítio deles.

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

/// ⚠️ Os bytes do v217 são escritos campo a campo (o documento sem o último campo): a partir do tipo vivo,
/// o teste provaria o braço com a hipótese dele.
#[test]
fn um_v217_abre_sem_a_marca_e_le_o_resto_no_sitio() {
    let d = documento();
    let velho = (
        (d.id, &d.layers, &d.canvas_rgba, &d.images),
        (&d.heights, &d.covers, &d.mats, d.size),
        (d.papel, &d.vidros),
    );
    let f = bare_file(empty_state());
    let v217 = (
        (&f.state, &f.assets, vec![velho], &f.motion),
        (&f.timeline, &f.physics, &f.tokens, &f.settings),
        (&f.player_tape, &f.sprite_pixels, &f.stable_id_counter),
        (&f.input_map, &f.pattern_art, &f.boards, &f.scripts),
        (
            &f.marcas_do_artista,
            &f.jogo,
            &f.mesa,
            &f.kanbans,
            &f.pixel_sprites,
            &f.sounds,
            vec![7u8],
        ),
    );
    let ver = crate::project_schema::PROJECT_SCHEMA_SEM_TINTAS_MOLHADAS;
    let path = written("tintas_v217", &(ver, v217));
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    let (f, _) = lido.expect("um v217 é recusado");
    let doc = &f.painted[0];
    assert!(doc.tintas_molhadas.is_empty(), "o v217 não tinha a marca");
    assert_eq!(
        (doc.id, doc.size, &doc.canvas_rgba),
        (d.id, d.size, &d.canvas_rgba)
    );
    assert_eq!(f.sfx_lab, [7], "o último campo do v217 lê-se no sítio dele");
}
