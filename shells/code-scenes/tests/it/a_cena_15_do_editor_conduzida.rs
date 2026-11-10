//! ⭐ **A cena 15 do editor de scripts** (`PH2D_CODE_SMOKE=15`, o pedido do dono de 10/11): o passo 1
//! do roteiro (`STEPS_15`) sobre a cena montada como a shell a monta, sem o `luau-lsp` (só o lint
//! nosso): a lista de problemas cheia e a grelha «Who has what» com catorze donos e as células âmbar.

use ph2d_app_code::smoke_drive::{Conductor, drive, steps};
use ph2d_app_code::{CodeDocs, IntelServer};
use ph2d_app_components::code_smoke::arena::{ARENA_NAME, DONOS};
use ph2d_app_components::code_smoke::monta_em;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::doc_surface::DocumentSurface;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::BandTab;
use ph2d_script::{ProjectScripts, ScriptHost};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vec_entities::entities::VecEntityMap;
use ph2d_vec_scene::VecScene;
use ph2d_vector::VectorScene;

const QUADRO_NS: u64 = 50_000_000;
const AREA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1400.0,
    h: 900.0,
};

#[test]
fn o_passo_1_da_cena_15_mostra_muitos_problemas_e_a_grelha_com_celulas_mudadas() {
    let mut sim = SimWorld::new();
    let formas = monta_em(&mut sim, &mut VecScene::new(), &mut VecEntityMap::new(), 15);
    assert_eq!(formas.len(), DONOS);
    let mut hero = HeroScreen::new(NodeId(1));
    let mut docs = CodeDocs::default();
    docs.set_intel_server(IntelServer::Off);
    hero.documents.install_surface(Box::new(docs));
    let mut host = ScriptHost::new().expect("vm");
    let mut toasts = ph2d_editor_core::ToastQueue::default();
    let mut ts = TextSystem::without_system_fonts();
    let id = sim
        .world()
        .resource::<ProjectScripts>()
        .by_name(ARENA_NAME)
        .expect("arena.luau no projecto");
    let mut c = Conductor::scene(steps(15).expect("o roteiro da 15"), 1);
    let mut agora = 1_000_000_000_000_u64;
    for _ in 0..400 {
        agora += QUADRO_NS;
        hero.store.note_clock(u128::from(agora));
        host.scene_sync(sim.world_mut());
        let _ = host.scene_tick(&mut sim, 1.0 / 60.0);
        ph2d_app_code::frame(&mut sim, Some(&mut host), &mut hero, &mut toasts, false);
        hero.documents.follow_surfaces();
        let d = hero
            .documents
            .surfaces_mut()
            .find_mut::<CodeDocs>()
            .unwrap();
        if d.text_of(id).is_some() {
            d.paint(
                id.0,
                AREA,
                &mut VectorScene::new(),
                &mut ts,
                &Theme::default(),
            );
        }
        if c.finished() {
            break;
        }
        drive(d, &mut c);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    let d = hero
        .documents
        .surfaces_mut()
        .find_mut::<CodeDocs>()
        .unwrap();
    assert!(c.finished() && !c.failed(), "o passo 1 chegou ao fim");
    assert_eq!(d.band_tab_of(id), BandTab::Problems);
    let lista = d.problem_list_of(id);
    assert!(lista.len() >= 8, "≥ 8 problemas listados: {lista:#?}");
    let (_, rows, _) = d.bench.grid_table().expect("a grelha pintada");
    assert!(rows.len() >= 8, "≥ 8 linhas na grelha: {}", rows.len());
    let mudadas = rows
        .iter()
        .flat_map(|r| &r.cells)
        .filter(|c| c.changed)
        .count();
    assert!(mudadas >= 2, "≥ 2 células âmbar: {mudadas}");
}
