//! **Os SCRIPTS no arquivo** (CodeEditor T7, v190, ADR-0188 §2) — o texto viaja DENTRO do
//! `.ph2dproj`, e um v189 (o caminho de um ficheiro solto) importa o ficheiro no load.

use super::*;
use ph2d_script::project_scripts::CANONICAL_NAME;
use ph2d_script::{LuauScript, ProjectScripts, ScriptHost, ScriptLibrary, ScriptSource};

const ANDA: &str = "function update(self, dt) ph2d.set(self.id, \"x\", 7) end\n";

fn registo() -> ph2d_ecs::scene::ComponentRegistry {
    let mut r = ph2d_ecs::scene::ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut r);
    ph2d_script::register_script_components(&mut r);
    r
}

/// Um projeto com UM objecto que corre `cfg`.
fn estado(cfg: LuauScript) -> ProjectState {
    let mut sim = ph2d_ecs::SimWorld::new();
    sim.world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, ph2d_ecs::StableId(1), cfg));
    let mut prop = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
    let mut work = ph2d_ecs::WorklistBuf::default();
    let mut st = empty_state();
    ph2d_ecs::scene::world_to_snapshot(
        sim.world_mut(),
        &mut prop,
        &mut work,
        &registo(),
        &mut st.world,
    )
    .expect("captura");
    st
}

/// O caminho do Ctrl+O sem janela: ler o ficheiro, abrir os scripts, restaurar o mundo, um tique.
fn abre_e_corre(path: &std::path::Path) -> (ScriptLibrary, LuauScript, f32) {
    let mut app = headless_app();
    let p = path.to_string_lossy();
    let (mut file, _) = app.read_project_file(&p).expect("aceite");
    let lib = ph2d_app_components::script_project::open(&mut file.state.world, &file.scripts, &p)
        .expect("scripts legíveis");
    let mut sim = ph2d_ecs::SimWorld::new();
    let e = ph2d_ecs::scene::snapshot_to_world(sim.world_mut(), &file.state.world, &registo())
        .expect("restaura")[0];
    ph2d_app_components::script_project::install(sim.world_mut(), lib.clone());
    let mut host = ScriptHost::new().expect("vm");
    host.scene_sync(sim.world_mut());
    let r = host.scene_tick(sim.world_mut(), 0.1);
    assert!(r.failed.is_empty(), "{:?}", r.failed);
    let w = sim.world();
    let cfg = w.get::<LuauScript>(e).expect("cfg").clone();
    (
        lib,
        cfg,
        w.get::<ph2d_ecs::Transform>(e).expect("pose").translation.x,
    )
}

/// ⭐⭐ **Gravar → abrir guarda o TEXTO e a REFERÊNCIA** — sem ficheiro solto nenhum, e o objecto corre.
#[test]
fn a_v190_project_keeps_the_script_text_and_the_reference() {
    let mut lib = ScriptLibrary::new();
    let id = lib.insert("bob.luau", ANDA);
    let file = ProjectFile {
        scripts: lib.to_bytes(),
        ..bare_file(estado(LuauScript::library(id)))
    };
    let path = tmp_path("scripts_v190");
    std::fs::write(
        &path,
        postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).expect("serializa"),
    )
    .expect("grava");
    let (back, cfg, x) = abre_e_corre(&path);
    let _ = std::fs::remove_file(&path);
    assert_eq!(back, lib, "a biblioteca atravessou o arquivo");
    assert_eq!(cfg.source, ScriptSource::Library(id));
    assert_eq!(x, 7.0, "o objecto correu o texto do projeto");
}

/// ⭐⭐⭐ **Um v189 com o CAMINHO de um ficheiro abre: o texto é importado e o objecto corre.**
///
/// ⚠️ Os bytes v189 são escritos campo a campo (a forma de então, sem `scripts`) — não a partir do
/// tipo vivo, senão o teste provaria o braço com a hipótese dele.
#[test]
fn a_v189_project_with_a_script_path_imports_it_and_runs() {
    let luau = std::env::temp_dir().join(format!("ph2d_gate_v189_{}.luau", std::process::id()));
    std::fs::write(&luau, ANDA).expect("o ficheiro solto");
    let nome = luau
        .file_name()
        .expect("nome")
        .to_string_lossy()
        .into_owned();
    let mut f = bare_file(estado(LuauScript::default()));
    let tid = ph2d_ecs::scene::stable_type_id(CANONICAL_NAME);
    let row = std::sync::Arc::make_mut(&mut f.state.world.entities[0]);
    let blob = row
        .components
        .iter_mut()
        .find(|b| b.type_id == tid)
        .expect("blob");
    // v2: o caminho RELATIVO à pasta do projeto (o ficheiro de projeto mora no mesmo temp).
    blob.data = postcard::to_allocvec(&ph2d_script::component::LuauScriptV2 {
        source: nome,
        own: Default::default(),
    })
    .expect("v2");
    let v189 = (
        &f.state,
        &f.assets,
        &f.painted,
        &f.motion,
        &f.timeline,
        &f.physics,
        &f.tokens,
        &f.settings,
        &f.player_tape,
        &f.sprite_pixels,
        &f.stable_id_counter,
        &f.input_map,
        &f.pattern_art,
        &f.boards,
    );
    let path = tmp_path("scripts_v189");
    std::fs::write(
        &path,
        postcard::to_allocvec(&(189u32, v189)).expect("serializa"),
    )
    .expect("grava");
    let (lib, cfg, x) = abre_e_corre(&path);
    let _ = (std::fs::remove_file(&path), std::fs::remove_file(&luau));
    let ScriptSource::Library(id) = cfg.source else {
        panic!("não importou: {:?}", cfg.source);
    };
    assert_eq!(lib.get(id).map(|d| d.text.as_str()), Some(ANDA));
    assert_eq!(x, 7.0, "o objecto correu o texto importado");
    assert!(lib.iter().count() == 1 && ProjectScripts(lib).len() == 1);
}
