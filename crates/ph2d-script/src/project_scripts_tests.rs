use std::sync::atomic::{AtomicU32, Ordering};

use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components, snapshot_to_world};
use ph2d_ecs::{SimWorld, StableId, Transform};

use super::*;
use crate::props::ScriptValue;
use crate::register_script_components;

fn reg() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    register_script_components(&mut r);
    r
}

fn snapshot(sim: &mut SimWorld) -> WorldSnapshot {
    let mut prop = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
    let mut work = ph2d_ecs::WorklistBuf::default();
    let mut snap = WorldSnapshot::new();
    ph2d_ecs::scene::world_to_snapshot(sim.world_mut(), &mut prop, &mut work, &reg(), &mut snap)
        .expect("captura");
    snap
}

/// Os `LuauScript` do snapshot, pela ordem das linhas.
fn scripts(snap: &WorldSnapshot) -> Vec<LuauScript> {
    let mut sim = SimWorld::new();
    let es = snapshot_to_world(sim.world_mut(), snap, &reg()).expect("restaura");
    es.into_iter()
        .filter_map(|e| sim.world().get::<LuauScript>(e).cloned())
        .collect()
}

/// Uma pasta só deste teste, com ficheiros dentro.
fn pasta(ficheiros: &[(&str, &str)]) -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "ph2d_proj_scripts_{}_{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).expect("temp");
    for (n, t) in ficheiros {
        std::fs::write(dir.join(n), t).expect("escreve");
    }
    dir
}

/// ⭐ O blob v2 (o caminho) vira v3: caminho → por importar, vazio → sem script, números intactos.
#[test]
fn v2_blobs_become_v3_and_keep_the_artists_numbers() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    w.spawn((Transform::IDENTITY, StableId(1), LuauScript::default()));
    w.spawn((Transform::IDENTITY, StableId(2), LuauScript::default()));
    let mut snap = snapshot(&mut sim);
    let id = stable_type_id(CANONICAL_NAME);
    let mut own = BTreeMap::new();
    own.insert("speed".to_owned(), ScriptValue::Number(6.0));
    for (row, v2) in snap.entities.iter_mut().zip([
        LuauScriptV2 {
            source: "scripts/bob.luau".into(),
            own: own.clone(),
        },
        LuauScriptV2::default(),
    ]) {
        let row = std::sync::Arc::make_mut(row);
        let b = row.components.iter_mut().find(|b| b.type_id == id).expect("blob");
        b.data = postcard::to_allocvec(&v2).expect("v2");
    }
    let m = migrate_v2_blobs(&mut snap);
    assert_eq!(m, V2Migration { migrated: 2, unreadable: 0 });
    let s = scripts(&snap);
    assert_eq!(s[0].source, ScriptSource::Unimported("scripts/bob.luau".into()));
    assert_eq!(s[0].own, own);
    assert_eq!(s[1].source, ScriptSource::None);
}

/// ⭐⭐ Um caminho relativo resolve-se na pasta do PROJETO; dois objectos no mesmo ficheiro partilham
/// UM script; um caminho que não se lê fica por importar (o cartão recusa-o).
#[test]
fn unimported_paths_are_imported_once_per_file_and_missing_ones_stay() {
    let dir = pasta(&[("bob.luau", "-- bob\n")]);
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let un = |p: &str| LuauScript {
        source: ScriptSource::Unimported(p.into()),
        ..LuauScript::default()
    };
    w.spawn((Transform::IDENTITY, StableId(1), un("bob.luau")));
    w.spawn((Transform::IDENTITY, StableId(2), un(&dir.join("bob.luau").to_string_lossy())));
    w.spawn((Transform::IDENTITY, StableId(3), un("sumiu.luau")));
    let mut snap = snapshot(&mut sim);
    let mut lib = ScriptLibrary::new();
    let r = import_unimported(&mut snap, &mut lib, Some(&dir), None);
    assert_eq!(r.imported.len(), 1, "{r:?}");
    assert_eq!(r.imported[0].matched, Matched::ProjectDir);
    assert_eq!((r.objects, r.missing.clone()), (2, vec!["sumiu.luau".to_owned()]));
    let id = r.imported[0].id;
    assert_eq!(lib.get(id).map(|d| (d.name.as_str(), d.text.as_str())), Some(("bob.luau", "-- bob\n")));
    let s = scripts(&snap);
    assert_eq!(s[0].source, ScriptSource::Library(id));
    assert_eq!(s[1].source, ScriptSource::Library(id), "o mesmo ficheiro, o mesmo script");
    assert_eq!(s[2].source, ScriptSource::Unimported("sumiu.luau".into()));
}

/// Sem pasta de projeto que o tenha, o directório corrente é o último recurso — e diz-se qual.
#[test]
fn the_cwd_is_the_last_resort_and_is_recorded() {
    let proj = pasta(&[]);
    let cwd = pasta(&[("x.luau", "-- x\n")]);
    assert_eq!(resolve("x.luau", Some(&proj), Some(&cwd)).map(|r| r.1), Some(Matched::Cwd));
    assert_eq!(resolve("x.luau", Some(&cwd), Some(&proj)).map(|r| r.1), Some(Matched::ProjectDir));
    assert_eq!(resolve("x.luau", Some(&proj), None), None);
    let abs = cwd.join("x.luau");
    assert_eq!(
        resolve(&abs.to_string_lossy(), None, None).map(|r| r.1),
        Some(Matched::Absolute)
    );
}
