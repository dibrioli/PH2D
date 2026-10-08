//! **As GOTAS do papel no arquivo de projeto** (`line/PainterWatercolor`, degrau `187 → 188`) —
//! filho de `project_tests`: as fixtures (`headless_app`, `tmp_path`, `empty_state`) são as portas.
//!
//! ⚠️ Os bytes do `187` montam-se **campo a campo, sem passar pelo tipo congelado** — a cerca do
//! `the_frozen_v95_bytes_still_load`: o `ProjectFileV187` referencia os tipos VIVOS, e um gate que
//! serializasse por ele mudaria junto com ele.

use super::*;
use ph2d_tool_painter::DripPaper;
use std::collections::BTreeMap;

/// Um documento pintado no layout `186`/`187`: com o papel e o vidro, SEM as gotas. Os mapas vazios
/// codificam igual seja qual for o tipo dos elementos.
fn doc_187(id: u32) -> impl serde::Serialize {
    (
        id,
        ph2d_tool_painter::layers::LayerStack::default(),
        vec![9u8; 4],              // canvas_rgba
        BTreeMap::<u8, u8>::new(), // images
        BTreeMap::<u8, u8>::new(), // heights
        BTreeMap::<u8, u8>::new(), // covers
        BTreeMap::<u8, u8>::new(), // mats
        (1u32, 1u32),              // size
        Some([10u8, 20, 30]),      // papel
        BTreeMap::<u8, u8>::new(), // vidros
    )
}

/// **Um ficheiro `187` com um documento pintado**, na ordem exacta do `ProjectFile` de então.
fn v187_bytes() -> Vec<u8> {
    let body = (
        empty_state(),
        Vec::<u8>::new(),                                     // assets
        vec![doc_187(4)],                                     // painted
        String::new(),                                        // motion
        Vec::<u8>::new(),                                     // timeline
        ph2d_physics_ecs::PhysicsSettings::default(),         // physics
        Vec::<u8>::new(),                                     // tokens
        crate::project_settings::collect(Default::default()), // settings
        ph2d_physics_ecs::TapeWire::default(),                // player_tape
        Vec::<u8>::new(),                                     // sprite_pixels
        7u64,                                                 // stable_id_counter
        ph2d_input::InputMap::default(),                      // input_map
        Vec::<u8>::new(),                                     // pattern_art
        Vec::<u8>::new(),                                     // boards
    );
    postcard::to_allocvec(&(187u32, body)).expect("serializa um 187")
}

/// ⭐ **Um projecto `187` com pintura ABRE, e cada documento nasce com as gotas default** — o papel e
/// os campos depois da pintura leem-se no sítio certo.
///
/// **Mutações que devem sangrar:** o braço `187 =>` a sair (o ficheiro é recusado) · o
/// `ProjectFileV187` a ler o documento pelo tipo vivo (os bytes deslizam).
#[test]
fn a_v187_project_opens_and_its_paintings_get_the_default_drips() {
    let mut app = headless_app();
    let path = tmp_path("load_v187_gotas");
    std::fs::write(&path, v187_bytes()).expect("grava o 187");
    let lido = app.read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    let (file, counter) = lido.expect("um projecto 187 abre (migrado), nao e' recusado");
    assert_eq!(
        counter, None,
        "o contador viaja no proprio ficheiro desde a v95"
    );
    assert_eq!(
        file.stable_id_counter, 7,
        "os campos depois da pintura leram-se no sitio"
    );
    assert_eq!(file.painted.len(), 1);
    let doc = &file.painted[0];
    assert_eq!(doc.id, 4);
    assert_eq!(
        doc.papel,
        Some([10, 20, 30]),
        "o papel atravessa a migracao"
    );
    assert_eq!(
        doc.gotas,
        DripPaper::DEFAULT,
        "um 187 nao tem gotas a dizer"
    );
}

/// ⛔ **O CONTROLO do degrau: o tipo VIVO não lê um `187` com pintura** — senão o degrau não seria
/// preciso. (Um `187` SEM pintura lê-se igual: a lista vazia codifica igual.)
#[test]
fn the_live_type_cannot_read_a_v187_with_a_painting() {
    let vivo = postcard::from_bytes::<(u32, ProjectFile)>(&v187_bytes());
    assert!(
        vivo.is_err() || vivo.is_ok_and(|(_, f)| f.stable_id_counter != 7),
        "o tipo VIVO leu um 187 com pintura inteiro e correcto"
    );
}

/// ⭐ **As gotas autoradas atravessam o ficheiro de hoje**: gravar e reabrir pelo leitor do load.
#[test]
fn the_paper_drips_round_trip_through_the_project_file() {
    let gotas = DripPaper {
        create: false,
        size: 9,
        length: 2,
        easy: true,
    };
    let doc = ph2d_tool_painter::PaintedDocument {
        id: 2,
        layers: Default::default(),
        canvas_rgba: vec![1, 2, 3, 4],
        images: Default::default(),
        heights: Default::default(),
        covers: Default::default(),
        mats: Default::default(),
        size: (1, 1),
        papel: None,
        vidros: Default::default(),
        gotas,
    };
    let file = ProjectFile {
        state: empty_state(),
        assets: Vec::new(),
        painted: vec![doc],
        motion: String::new(),
        timeline: Vec::new(),
        physics: Default::default(),
        tokens: Vec::new(),
        settings: crate::project_settings::collect(Default::default()),
        player_tape: ph2d_physics_ecs::TapeWire::default(),
        sprite_pixels: Vec::new(),
        stable_id_counter: ph2d_ecs::StableId::FIRST,
        input_map: ph2d_input::InputMap::new(),
        pattern_art: Vec::new(),
        boards: Vec::new(),
    };
    let path = tmp_path("roundtrip_gotas");
    let bytes = postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).expect("serializa");
    std::fs::write(&path, bytes).expect("grava");
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    let (back, _) = lido.expect("o ficheiro de hoje abre");
    assert_eq!(
        back.painted[0].gotas, gotas,
        "as gotas voltaram como foram gravadas"
    );
}
