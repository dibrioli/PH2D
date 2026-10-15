//! **Os scripts do projeto no ficheiro** (ADR-0188 §2) — a metade do save/load que a shell só compõe.
//!
//! O texto vive no recurso [`ProjectScripts`] do mundo (fora do `ProjectState`: nem o undo global nem
//! o diff por quadro o veem) e viaja no `ProjectFile.scripts` como o blob `PH2DSCRIPTS`. Gravar nunca
//! escreve ficheiros soltos.

use std::path::Path;

use ph2d_ecs::World;
use ph2d_ecs::scene::WorldSnapshot;
use ph2d_i18n::tr_with;
use ph2d_script::project_scripts::{ImportReport, Matched, import_unimported};
use ph2d_script::{DocError, ProjectScripts, ScriptLibrary};

/// O blob a gravar — vazio quando o mundo não tem scripts do projeto.
#[must_use]
pub fn save(world: &World) -> Vec<u8> {
    world
        .get_resource::<ProjectScripts>()
        .map(|l| l.to_bytes())
        .unwrap_or_default()
}

/// Os scripts abertos e o que a importação tem de dizer ao artista.
pub struct Opened {
    pub lib: ScriptLibrary,
    /// A frase do toast quando o load importou ou não achou ficheiros soltos (`None` = nada a dizer).
    pub notice: Option<String>,
}

/// **Abre** os scripts de um ficheiro e importa cada caminho antigo (`Unimported`) do `snapshot`:
/// absoluto, senão na pasta do PROJETO, senão no directório corrente. O que não se lê fica por
/// importar — o cartão recusa-o, e o [`Opened::notice`] di-lo no load.
///
/// # Errors
/// O blob não é uma biblioteca legível (o load recusa o ficheiro inteiro, como os quadros).
pub fn open(
    snapshot: &mut WorldSnapshot,
    bytes: &[u8],
    project_path: &str,
) -> Result<Opened, DocError> {
    let mut lib = ScriptLibrary::from_bytes(bytes)?;
    let cwd = std::env::current_dir().ok();
    let r = import_unimported(
        snapshot,
        &mut lib,
        Path::new(project_path).parent(),
        cwd.as_deref(),
    );
    for i in &r.imported {
        eprintln!(
            "[proj] script importado: {} ({:?}) -> #{}",
            i.path, i.matched, i.id.0
        );
    }
    for m in &r.missing {
        eprintln!("[proj] script por importar (ficheiro nao encontrado): {m}");
    }
    let notice = notice(&r);
    Ok(Opened { lib, notice })
}

/// ⭐ Importar MUDA o documento: o próximo Ctrl+S grava o texto dentro do projeto e o ficheiro solto
/// deixa de mandar. O artista ouve-o no load — e ouve à parte o que veio do directório corrente
/// (o recurso que pode apanhar um homónimo que não é o do projeto).
fn notice(r: &ImportReport) -> Option<String> {
    let cwd = r.imported.iter().filter(|i| i.matched == Matched::Cwd).count();
    let mut parts = Vec::new();
    if !r.imported.is_empty() {
        let n = r.imported.len();
        parts.push(tr_with(
            "app.components.script_project.copied_script_file_s_into_the_project",
            &[("n", &n)],
        ));
    }
    if cwd > 0 {
        parts.push(tr_with(
            "app.components.script_project.found_in_the_app_working_folder_not_next_to_the_project",
            &[("n", &cwd)],
        ));
    }
    if !r.missing.is_empty() {
        let n = r.missing.len();
        parts.push(tr_with(
            "app.components.script_project.script_file_s_not_found_the_script_card_names_them",
            &[("n", &n)],
        ));
    }
    (!parts.is_empty()).then(|| parts.join(" \u{00b7} "))
}

/// Põe os scripts abertos no mundo — substitui os do documento anterior.
pub fn install(world: &mut World, lib: ScriptLibrary) {
    world.insert_resource(ProjectScripts(lib));
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_script::ScriptId;
    use ph2d_script::project_scripts::Imported;

    fn importado(matched: Matched) -> Imported {
        Imported {
            path: "bob.luau".into(),
            matched,
            id: ScriptId(1),
        }
    }

    /// ⭐ O load que importa DIZ-o no ecrã — e o que veio do directório corrente e o que faltou à parte.
    #[test]
    fn an_import_on_load_is_told_on_screen() {
        assert_eq!(notice(&ImportReport::default()), None, "nada importado: silêncio");
        let r = ImportReport {
            imported: vec![importado(Matched::ProjectDir), importado(Matched::Cwd)],
            objects: 2,
            missing: vec!["x.luau".into()],
        };
        let n = notice(&r).expect("frase");
        for k in [
            "app.components.script_project.copied_script_file_s_into_the_project",
            "app.components.script_project.found_in_the_app_working_folder_not_next_to_the_project",
            "app.components.script_project.script_file_s_not_found_the_script_card_names_them",
        ] {
            assert_ne!(ph2d_i18n::tr(k), k, "{k} sem tradução");
        }
        assert!(n.starts_with("Copied 2 script file(s)"), "{n}");
        assert!(n.contains("1 found in the app"), "{n}");
        assert!(n.contains("1 script file(s) not found"), "{n}");
    }
}
