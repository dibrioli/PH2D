//! **Os SCRIPTS no arquivo** (CodeEditor T7, v190, ADR-0188 §2) — o texto viaja DENTRO do
//! `.ph2dproj`, e um v189 (o caminho de um ficheiro solto) importa o ficheiro no load. As peças são
//! da família (`ph2d_app_components::test_support::scripts`); aqui só o ficheiro da shell.

use super::*;
use ph2d_app_components::test_support::scripts::{ANDA, open_and_tick, snapshot};
use ph2d_script::{LuauScript, ProjectScripts, ScriptLibrary, ScriptSource};

const V189: &[u8] = include_bytes!("../tests/fixtures/projecto_scripts_v189.ph2dproj");
const V190: &[u8] = include_bytes!("../tests/fixtures/projecto_scripts_v190.ph2dproj");

fn estado(cfg: LuauScript) -> ProjectState {
    ProjectState {
        world: snapshot(cfg),
        ..empty_state()
    }
}

/// O `read_project_file` da shell e, depois dele, o resto do Ctrl+O sem janela.
fn abre_e_corre(path: &std::path::Path) -> (ScriptLibrary, LuauScript, f32) {
    let p = path.to_string_lossy();
    let (file, _) = headless_app().read_project_file(&p).expect("aceite");
    open_and_tick(file.state.world, &file.scripts, &p)
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
    // Outro nome que o dos braços antigos (`scripts_v190`): no mesmo processo os dois colidiam.
    let path = tmp_path("scripts_vivo");
    let bytes = postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).expect("serializa");
    std::fs::write(&path, bytes).expect("grava");
    let (back, cfg, x) = abre_e_corre(&path);
    let _ = std::fs::remove_file(&path);
    assert_eq!(back, lib, "a biblioteca atravessou o arquivo");
    assert_eq!(cfg.source, ScriptSource::Library(id));
    assert_eq!(x, 7.0, "o objecto correu o texto do projeto");
}

/// ⭐⭐⭐ **Um v189 com o CAMINHO de um ficheiro abre: o texto é importado e o objecto corre.**
#[test]
fn a_v189_project_with_a_script_path_imports_it_and_runs() {
    um_anterior_aos_scripts_importa_o_caminho_e_corre(189, V189);
}

/// ⭐ O mesmo para o **v190** — o degrau da `line/UIUX` (o traço em pares), o `main` de quando os
/// scripts entraram no projeto (`191`): sem `scripts`, com o `StrokeSpec` já na forma nova.
#[test]
fn a_v190_project_with_a_script_path_imports_it_and_runs() {
    um_anterior_aos_scripts_importa_o_caminho_e_corre(190, V190);
}

/// Os bytes foram gravados pelo código v193 (`c3662cd8e4`, numa worktree temporária) — ⛔ **nunca os regenere**:
/// o objecto aponta para `ph2d_gate_v<ver>.luau`, relativo à pasta do projeto.
fn um_anterior_aos_scripts_importa_o_caminho_e_corre(ver: u32, bytes: &[u8]) {
    let luau = std::env::temp_dir().join(format!("ph2d_gate_v{ver}.luau"));
    std::fs::write(&luau, ANDA).expect("o ficheiro solto");
    let path = tmp_path(&format!("scripts_v{ver}"));
    std::fs::write(&path, bytes).expect("grava o fixture");
    let (lib, cfg, x) = abre_e_corre(&path);
    let _ = (std::fs::remove_file(&path), std::fs::remove_file(&luau));
    let ScriptSource::Library(id) = cfg.source else {
        panic!("não importou: {:?}", cfg.source);
    };
    assert_eq!(lib.get(id).map(|d| d.text.as_str()), Some(ANDA));
    assert_eq!(x, 7.0, "o objecto correu o texto importado");
    assert!(lib.iter().count() == 1 && ProjectScripts::new(lib).len() == 1);
}
