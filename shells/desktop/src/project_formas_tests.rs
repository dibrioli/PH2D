//! A forma de colisão no arquivo (`219 → 220`, GEF02 B2): um v219 da cena 120 abre com as filhas «Collision
//! Shape», os mesmos nomes e camadas, a trilha da forma na filha e a mesma simulação; um v220 abre como está.

use super::sounds::written;
use super::*;
use ph2d_ecs::{Name, StableId};
use ph2d_physics_ecs::{Collider, RigidBody, main_shape};

fn cena_120() -> (ProjectFile, u64) {
    let reg = crate::init::build_component_registry();
    let mut sim = ph2d_ecs::SimWorld::new();
    ph2d_physics_scene::ficha::monta(sim.world_mut());
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

/// Os corpos por `StableId`: (id, nome, camada da forma principal).
fn corpos(sim: &mut ph2d_ecs::SimWorld) -> Vec<(u64, String, u8, bool)> {
    let w = sim.world_mut();
    let es: Vec<_> = w
        .query_filtered::<bevy_ecs::entity::Entity, bevy_ecs::query::With<RigidBody>>()
        .iter(w)
        .collect();
    let mut v: Vec<_> = es
        .into_iter()
        .filter_map(|e| {
            let m = main_shape(w, e)?;
            Some((
                w.get::<StableId>(e).unwrap().0,
                w.get::<Name>(e).map_or_else(String::new, |n| n.0.clone()),
                w.get::<Collider>(m).unwrap().layer,
                m != e,
            ))
        })
        .collect();
    v.sort_by_key(|c| c.0);
    v
}

fn hash(sim: &mut ph2d_ecs::SimWorld) -> u64 {
    let mut bridge = ph2d_physics_ecs::PhysicsBridge::new();
    let mut h = 0xcbf2_9ce4_8422_2325_u64;
    for t in 1..=240 {
        bridge.dispatch(sim, true, t);
        let w = sim.world_mut();
        let mut q: Vec<_> = w
            .query_filtered::<(&StableId, &ph2d_ecs::Transform), bevy_ecs::query::With<RigidBody>>()
            .iter(w)
            .map(|(s, t)| (s.0, *t))
            .collect();
        q.sort_by_key(|p| p.0);
        for (_, tr) in q {
            for v in [tr.translation.x, tr.translation.y, tr.rotation] {
                h = (h ^ u64::from(v.to_bits())).wrapping_mul(0x0100_0000_01b3);
            }
        }
    }
    h
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
    let (antes, depois) = (corpos(&mut s220), corpos(&mut s219));
    assert!(
        antes.len() >= 5 && antes.iter().all(|c| !c.3),
        "controlo: {antes:?}"
    );
    assert!(
        depois.iter().all(|c| c.3),
        "toda forma passou a filha: {depois:?}"
    );
    let sem_flag = |v: &[(u64, String, u8, bool)]| {
        v.iter()
            .map(|c| (c.0, c.1.clone(), c.2))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        sem_flag(&antes),
        sem_flag(&depois),
        "mesmos corpos, nomes e camadas"
    );
    let doc = ph2d_timeline::TimelineDoc::from_bytes(&f219.timeline).unwrap();
    assert_ne!(
        doc.bindings()[0].wire_id.0,
        chao,
        "a trilha da camada segue a filha"
    );
    assert!(f219.stable_id_counter > doc.bindings()[0].wire_id.0);
    assert_eq!(
        hash(&mut s220),
        hash(&mut s219),
        "a migração não muda a simulação"
    );
}
