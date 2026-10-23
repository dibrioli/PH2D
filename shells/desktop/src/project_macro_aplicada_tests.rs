//! ⭐ **A imagem de uma macro aplicada volta do ficheiro** (W18) — filho de `project_tests`: aplicar ▸
//! Criar novo o Splash gravado, gravar e abrir pelas peças das portas do Ctrl+S / Ctrl+O (sem placa:
//! os píxeis são do documento do Painter, que no load escreve por cima do atlas).

use super::*;
use ph2d_app_macros::aplicar;
use ph2d_app_painter::documentos_no_projecto::{carimba_e_colhe, devolve};
use ph2d_ecs::{ChildOf, Entity, Macro, Name, SimWorld, StableId};
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::macro_track::{fim, roteiro};

fn aplica_o_splash(sim: &mut SimWorld, painter: &mut PainterTool) -> u64 {
    let mut doc =
        ph2d_macro::codec::read(&roteiro::grava_o_splash(64, 0).expect("grava").0).expect("lê");
    doc.tracks[0].nasce = Some(ph2d_macro::Nascimento {
        inicio: ph2d_macro::MacroTime(0),
        nome: "Tela".into(),
        local: [0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0],
        pai: None,
    });
    let fi = ph2d_macro::ficheiro::Ficheiro {
        doc,
        modo: 0,
        nome: "Splash".into(),
    };
    let novo = ph2d_editor_core::macro_edits::OndeAplicar::Novo { em: None };
    let mut motion = ph2d_app_motion::motion_state::MotionState::new();
    let raiz = ph2d_app_components::object_add::spawn_empty_root;
    aplicar::sem_placa(&fi, novo, sim, painter, &mut motion, &raiz).expect("aplica")
}

fn filho_de(sim: &SimWorld, raiz: u64) -> u64 {
    let pai = Some(Entity::from_bits(raiz));
    let w = sim.world();
    let f: Vec<u64> = w
        .iter_entities()
        .filter(|e| e.get::<ChildOf>().map(ChildOf::parent) == pai)
        .map(|e| e.id().to_bits())
        .collect();
    assert_eq!(f.len(), 1, "uma imagem dentro do objecto Macro");
    f[0]
}

#[test]
fn a_imagem_de_uma_macro_aplicada_volta_do_ficheiro_e_a_macro_toca_nela() {
    let reg = crate::init::build_component_registry();
    let (mut sim, mut painter, mut proximo) = (SimWorld::new(), PainterTool::default(), 1);
    let raiz = aplica_o_splash(&mut sim, &mut painter);
    let img = filho_de(&sim, raiz);
    let antes = painter.baked_document_pixels(img).expect("o editor tem-na");
    let painted = carimba_e_colhe(&mut sim, &mut painter, &mut proximo);
    let state = ProjectState::capture(
        &ph2d_preview_drive::PreviewDrive::default(),
        &mut sim,
        &VecScene::new(),
        &ph2d_flip::FlipDoc::new(),
        &ph2d_guides::GuideSet::default(),
        &ph2d_ui_state::StateSets::default(),
        &crate::project_library::LibraryDoc::default(),
        &[],
        &reg,
        &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
        None,
    );
    let file = ProjectFile {
        painted,
        ..super::writer::bare_file(state)
    };
    let path = tmp_path("macro_aplicada");
    std::fs::write(
        &path,
        postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).expect("serializa"),
    )
    .expect("grava");

    // Abrir noutro mundo, noutro Painter.
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    let (lido, _) = lido.expect("lê o ficheiro");
    let (mut sim, mut painter) = (SimWorld::new(), PainterTool::default());
    let _ = lido.state.restore(&mut sim, &reg);
    let reatados = devolve(&mut sim, &mut painter, &mut proximo, lido.painted);
    let w = sim.world_mut();
    let raizes: Vec<(u64, Vec<u8>)> = w
        .query::<(Entity, &Macro)>()
        .iter(w)
        .filter(|(_, m)| !m.doc.is_empty())
        .map(|(e, m)| (e.to_bits(), m.doc.clone()))
        .collect();
    assert_eq!(raizes.len(), 1, "o objecto Macro voltou com a gravação");
    let img = filho_de(&sim, raizes[0].0);
    let nome = sim
        .world()
        .get::<Name>(Entity::from_bits(img))
        .map(|n| n.0.clone());
    assert_eq!(nome.as_deref(), Some("Tela"));
    // O que a porta sobe para a textura do sprite (o `baked_document_pixels` dá-o uma vez).
    assert_eq!(reatados.len(), 1, "o documento voltou ao sprite");
    let (bits, px, w, h) = reatados[0].clone();
    assert_eq!(bits, img, "à imagem dentro do objecto Macro");
    assert_eq!((px, w, h), antes.clone(), "os mesmos píxeis");
    let trilha = ph2d_macro::codec::read(&raizes[0].1)
        .expect("lê")
        .tracks
        .remove(0);
    let alvo = ph2d_ecs::entity_of_stable_id(sim.world_mut(), StableId(trilha.target));
    assert_eq!(
        alvo.map(Entity::to_bits),
        Some(img),
        "a trilha resolve para a imagem reaberta"
    );
    let mut t = PainterTool::default();
    t.install_document(1, fim::documento_no_fim(&trilha, None).expect("toca"));
    assert_eq!(
        t.baked_document_pixels(1),
        Some(antes),
        "a Macro reaberta toca até ela"
    );
}
