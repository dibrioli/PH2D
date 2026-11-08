//! A forma de colisão no arquivo (`219 → 220`, GEF02 B2): um v219 da cena 120 abre com as filhas «Collision
//! Shape» e a trilha da forma na filha; um v220 abre como está. A cena: `ph2d-physics-scene` `ficha_migra_tests`.

use super::sounds::written;
use super::*;
use ph2d_ecs::Name;

fn cena_120() -> (ProjectFile, u64) {
    let reg = crate::init::build_component_registry();
    let mut sim = ph2d_ecs::SimWorld::new();
    ph2d_physics_scene::ficha::monta_v219(sim.world_mut());
    let (mut st, w) = (empty_state(), sim.world_mut());
    let mut prop = ph2d_ecs::TransformPropagationState::new(w);
    let mut work = ph2d_ecs::WorklistBuf::default();
    ph2d_ecs::scene::world_to_snapshot(w, &mut prop, &mut work, &reg, &mut st.world).unwrap();
    let chao = st
        .world
        .entities
        .iter()
        .find(|r| r.parent.is_none())
        .unwrap()
        .id
        .0;
    let mut tl = ph2d_timeline::TimelineDoc::default();
    let k = ph2d_ecs::FieldKey::new(
        ph2d_ecs::canal::familia::FISICA,
        ph2d_ecs::canal_fisica::CAMADA,
        0,
        0,
    );
    let t = tl.bind(1, ph2d_timeline::PropKind::Field(k));
    tl.bindings_mut()
        .iter_mut()
        .find(|b| b.target == t)
        .unwrap()
        .wire_id = ph2d_timeline::WireId(chao);
    let counter = w.resource::<ph2d_ecs::StableIdCounter>().next_free();
    let f = ProjectFile {
        stable_id_counter: counter,
        timeline: tl.to_bytes().unwrap(),
        ..bare_file(st)
    };
    (f, chao)
}

fn abre(ver: u32, f: &ProjectFile) -> (ProjectFile, ph2d_ecs::SimWorld) {
    let path = written(&format!("formas_v{ver}"), &(ver, f));
    let lido = headless_app().read_project_file(&path.to_string_lossy());
    let _ = std::fs::remove_file(&path);
    let f = lido.expect("aceite").0;
    let mut sim = ph2d_ecs::SimWorld::new();
    let reg = crate::init::build_component_registry();
    ph2d_ecs::scene::snapshot_to_world(sim.world_mut(), &f.state.world, &reg).unwrap();
    (f, sim)
}

#[test]
fn um_v219_abre_com_as_formas_filhas_e_um_v220_abre_como_esta() {
    let _fio = ph2d_physics_ecs::LayerRowsV218::enter();
    let (f, chao) = cena_120();
    let (f220, mut s220) = abre(PROJECT_SCHEMA, &f);
    let (f219, mut s219) = abre(crate::project_schema::PROJECT_SCHEMA_FORMA_NO_CORPO, &f);
    assert_eq!(
        f220.state.world.entities.len(),
        f.state.world.entities.len(),
        "o v220 não migra"
    );
    let filhas = f219.state.world.entities.len() - f.state.world.entities.len();
    assert!(filhas >= 5, "toda forma passou a filha: {filhas}");
    let nomeadas = |s: &mut ph2d_ecs::SimWorld| {
        let mut q = s.world_mut().query::<&Name>();
        q.iter(s.world())
            .filter(|n| n.0 == ph2d_physics_ecs::NOME_DA_FORMA)
            .count()
    };
    assert_eq!(nomeadas(&mut s219), nomeadas(&mut s220) + filhas);
    let doc = ph2d_timeline::TimelineDoc::from_bytes(&f219.timeline).unwrap();
    assert_ne!(
        doc.bindings()[0].wire_id.0,
        chao,
        "a trilha da camada segue a filha"
    );
    assert!(f219.stable_id_counter > doc.bindings()[0].wire_id.0);
}
