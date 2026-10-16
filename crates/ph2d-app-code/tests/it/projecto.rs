//! ⛔⛔ **Abrir OUTRO projecto esquece TODO o estado por `ScriptId`** (auditoria do fecho): os ids
//! recomeçam em 1 em cada projecto, e uma aba com ● do projecto A gravava o texto do bob por cima do
//! `#1 enemy.luau` do projecto B no Ctrl+S. Pela porta do load (`ProjectScripts::install` e as abas do
//! ficheiro a tomar o lugar das antigas, `document_tabs::load`).
//!
//! ⭐ **Regra das abas com ●**: nunca se perde texto em silêncio — cada texto por gravar do projecto
//! anterior (aberto ou fechado com ●) vira um FICHEIRO DE RECUPERAÇÃO e um aviso no ecrã diz onde.

use crate::{Mesa, ctrl};
use ph2d_editor_core::doc_surface::{DocumentSurface, Key};
use ph2d_script::{ProjectScripts, ScriptId, ScriptLibrary};
use std::path::PathBuf;

const BOB: &str = "-- bob do projecto A\n";
const ANA: &str = "-- ana do projecto A\n";
const ENEMY: &str = "-- enemy do projecto B\n";

fn pasta(nome: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ph2d_recupera_{nome}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

/// O que o Ctrl+O faz à família: a biblioteca nova entra e as abas do ficheiro tomam o lugar.
fn abre_o_projecto_b(m: &mut Mesa) -> ScriptId {
    let mut lib = ScriptLibrary::default();
    let id = lib.insert("enemy.luau", ENEMY);
    ProjectScripts::install(m.sim.world_mut(), lib);
    ph2d_editor_core::screens::hero::document_tabs::load(&mut m.hero, Default::default());
    m.quadro();
    id
}

fn ficheiros(d: &PathBuf) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(d)
        .map(|rd| {
            rd.filter_map(Result::ok)
                .filter_map(|e| std::fs::read_to_string(e.path()).ok())
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

#[test]
fn abrir_outro_projecto_nunca_grava_o_texto_do_anterior_por_cima() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", BOB), ("ana.luau", ANA)]);
    let dir = pasta("cima");
    m.docs().set_recovery_dir(dir.clone());
    // ana: editada e fechada com ● (estacionada); bob: salvo (selo) e editado de novo (aberto, ●).
    m.pede(ids[1]);
    m.escreve(ids[1], "x");
    m.docs().close(ids[1].0);
    m.quadro();
    m.pede(ids[0]);
    m.escreve(ids[0], "s");
    m.tecla(ids[0], Key::Char('s'), ctrl(), Some("s"));
    m.quadro();
    assert!(
        m.docs().badge_of(ids[0]).is_some(),
        "controlo: o bob tem selo"
    );
    m.escreve(ids[0], "y");
    assert!(m.docs().is_dirty(ids[0]) && m.docs().is_parked(ids[1]));

    let enemy = abre_o_projecto_b(&mut m);
    assert_eq!(enemy, ids[0], "o #1 do B tem o id do bob");
    assert!(m.docs().tabs().is_empty(), "as abas do projecto A fecharam");
    assert!(
        !m.docs().is_parked(ids[1]),
        "o estacionado do A foi esquecido"
    );
    assert!(!m.docs().is_dirty(enemy));
    // O File ▸ Save do projecto B não grava nada do A.
    m.hero.file_menu.save = true;
    m.quadro();
    assert_eq!(
        m.texto_salvo(enemy),
        ENEMY,
        "o texto do bob caiu sobre o enemy"
    );
    m.pede(enemy);
    assert_eq!(m.docs().text_of(enemy).as_deref(), Some(ENEMY));
    assert_eq!(
        m.docs().badge_of(enemy),
        None,
        "o selo do bob ficou no enemy"
    );
    assert!(m.docs().console_texts(enemy).is_empty());

    // Os textos por gravar do A ficaram em ficheiros, e o ecrã di-lo.
    let mut esperado = vec![format!("sy{BOB}"), format!("x{ANA}")];
    esperado.sort();
    assert_eq!(ficheiros(&dir), esperado, "os textos por gravar do A");
    assert!(
        m.toasts
            .iter()
            .any(|t| t.message.contains(&dir.display().to_string())),
        "o aviso diz onde ficaram"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// Controlo: sem textos por gravar, trocar de projecto não escreve ficheiro nenhum nem avisa.
#[test]
fn sem_textos_por_gravar_trocar_de_projecto_e_silencioso() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", BOB)]);
    let dir = pasta("limpo");
    m.docs().set_recovery_dir(dir.clone());
    m.pede(ids[0]);
    let enemy = abre_o_projecto_b(&mut m);
    assert!(m.docs().tabs().is_empty());
    assert!(ficheiros(&dir).is_empty());
    assert!(m.toasts.is_empty());
    m.pede(enemy);
    assert_eq!(m.docs().text_of(enemy).as_deref(), Some(ENEMY));
}
