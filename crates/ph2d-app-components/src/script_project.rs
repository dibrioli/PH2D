//! **Os scripts do projeto no ficheiro** (ADR-0188 §2) — a metade do save/load que a shell só compõe.
//!
//! O texto vive no recurso [`ProjectScripts`] do mundo (fora do `ProjectState`: nem o undo global nem
//! o diff por quadro o veem) e viaja no `ProjectFile.scripts` como o blob `PH2DSCRIPTS`. Gravar nunca
//! escreve ficheiros soltos.

use std::path::Path;

use ph2d_ecs::World;
use ph2d_ecs::scene::WorldSnapshot;
use ph2d_script::project_scripts::{Matched, import_unimported};
use ph2d_script::{DocError, ProjectScripts, ScriptLibrary};

/// O blob a gravar — vazio quando o mundo não tem scripts do projeto.
#[must_use]
pub fn save(world: &World) -> Vec<u8> {
    world
        .get_resource::<ProjectScripts>()
        .map(|l| l.to_bytes())
        .unwrap_or_default()
}

/// **Abre** os scripts de um ficheiro e importa cada caminho antigo (`Unimported`) do `snapshot`:
/// absoluto, senão na pasta do PROJETO, senão no directório corrente. O que não se lê fica por
/// importar — o cartão recusa-o.
///
/// # Errors
/// O blob não é uma biblioteca legível (o load recusa o ficheiro inteiro, como os quadros).
pub fn open(
    snapshot: &mut WorldSnapshot,
    bytes: &[u8],
    project_path: &str,
) -> Result<ScriptLibrary, DocError> {
    let mut lib = ScriptLibrary::from_bytes(bytes)?;
    let cwd = std::env::current_dir().ok();
    let r = import_unimported(
        snapshot,
        &mut lib,
        Path::new(project_path).parent(),
        cwd.as_deref(),
    );
    for i in &r.imported {
        let onde = match i.matched {
            Matched::Absolute => "absoluto",
            Matched::ProjectDir => "na pasta do projeto",
            Matched::Cwd => "no directorio corrente",
        };
        eprintln!(
            "[proj] script importado: {} ({onde}) -> #{}",
            i.path, i.id.0
        );
    }
    for m in &r.missing {
        eprintln!("[proj] script por importar (ficheiro nao encontrado): {m}");
    }
    Ok(lib)
}

/// Põe os scripts abertos no mundo — substitui os do documento anterior.
pub fn install(world: &mut World, lib: ScriptLibrary) {
    world.insert_resource(ProjectScripts(lib));
}
