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
    let mut lib = ScriptLibrary::from_bytes(bytes)
        .inspect_err(|e| eprintln!("[proj] scripts ilegiveis — load RECUSADO: {e}"))?;
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
    let cwd = r
        .imported
        .iter()
        .filter(|i| i.matched == Matched::Cwd)
        .count();
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

/// ⭐ **O degrau `190 → 191`** (ADR-0188) — `F` é o `ProjectFile` da shell, `world` o caminho até ao
/// snapshot dele. O campo novo (`scripts`) é o ÚLTIMO e um `Vec` vazio no postcard é o byte `0`: o
/// v190 (e o v189, que só difere no traço) lê-se com o tipo vivo acrescentando-o. Os blobs do `LuauScript` passam de v2 (o caminho) a
/// v3; o load importa os caminhos ([`open`]). Devolve o ficheiro e a frase do toast.
///
/// # Errors
/// Os bytes não são de um ficheiro anterior aos scripts (`ver` = a versão lida: 190 ou 189).
pub fn read_before_scripts<F: serde::de::DeserializeOwned>(
    ver: u32,
    bytes: &[u8],
    world: fn(&mut F) -> &mut WorldSnapshot,
    schema: u32,
) -> Result<(F, String), postcard::Error> {
    let (_, mut f) = postcard::from_bytes::<(u32, F)>(&[bytes, &[0]].concat())
        .inspect_err(|e| eprintln!("[proj] v{ver} ilegivel: {e}"))?;
    let m = ph2d_script::project_scripts::migrate_v2_blobs(world(&mut f));
    eprintln!(
        "[proj] migrado v{ver} -> v{schema} ({} script(s) v2 -> v3, {} ilegivel(eis))",
        m.migrated, m.unreadable
    );
    let aviso = tr_with(
        "shell.project_load.project_migrated_from_189",
        &[("ver", &ver), ("PROJECT_SCHEMA", &schema)],
    );
    Ok((f, aviso))
}

/// Põe os scripts abertos no mundo — substitui os do documento anterior.
pub fn install(world: &mut World, lib: ScriptLibrary) {
    ProjectScripts::install(world, lib);
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
        assert_eq!(
            notice(&ImportReport::default()),
            None,
            "nada importado: silêncio"
        );
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
