//! **O load de um projeto e os scripts** (ADR-0188 §2) — os dois passos que correm sobre o
//! `WorldSnapshot` ANTES de ele virar mundo:
//!
//! 1. [`migrate_v2_blobs`] — só num ficheiro de schema antigo: o blob v2 do `LuauScript` (o caminho)
//!    passa a v3 (`Unimported(caminho)`). O blob não leva versão (é postcard posicional): quem sabe a
//!    forma é o schema do ficheiro.
//! 2. [`import_unimported`] — em todo load: cada `Unimported(caminho)` cujo ficheiro se lê entra na
//!    biblioteca e passa a `Library(id)`. O que não se lê FICA `Unimported` — o cartão recusa-o.
//!
//! Ambos correm no snapshot e não no mundo vivo: o load é o ponto de partida do undo, e uma
//! importação feita depois dele seria o primeiro passo de `Ctrl+Z` de um projeto acabado de abrir.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ph2d_ecs::scene::{WorldSnapshot, stable_type_id};
use ph2d_script_doc::{ScriptId, ScriptLibrary};

use crate::component::{LuauScript, LuauScriptV2, ScriptSource};

/// O nome canónico com que o `LuauScript` é registado — a chave do blob.
pub const CANONICAL_NAME: &str = "ph2d::script::LuauScript";

/// O que o degrau v2 → v3 fez.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct V2Migration {
    pub migrated: usize,
    pub unreadable: usize,
}

/// Reescreve cada blob v2 do `LuauScript` como v3. Um blob ilegível fica como estava (o restauro do
/// mundo recusa-o em voz alta).
pub fn migrate_v2_blobs(world: &mut WorldSnapshot) -> V2Migration {
    let id = stable_type_id(CANONICAL_NAME);
    let mut out = V2Migration::default();
    for row in &mut world.entities {
        let Some(slot) = row.components.iter().position(|b| b.type_id == id) else {
            continue;
        };
        let v3 = match postcard::from_bytes::<LuauScriptV2>(&row.components[slot].data) {
            Ok(v2) => LuauScript::from(v2),
            Err(_) => {
                out.unreadable += 1;
                continue;
            }
        };
        match postcard::to_allocvec(&v3) {
            Ok(bytes) => {
                std::sync::Arc::make_mut(row).components[slot].data = bytes;
                out.migrated += 1;
            }
            Err(_) => out.unreadable += 1,
        }
    }
    out
}

/// Onde um caminho relativo se resolveu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Matched {
    Absolute,
    ProjectDir,
    Cwd,
}

/// Um ficheiro importado.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Imported {
    pub path: String,
    pub matched: Matched,
    pub id: ScriptId,
}

/// O que a importação fez.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImportReport {
    /// Um por FICHEIRO (dois objectos no mesmo ficheiro partilham o script).
    pub imported: Vec<Imported>,
    /// Objectos que passaram a apontar a biblioteca.
    pub objects: usize,
    /// Caminhos que não se deixaram ler — ficam `Unimported`.
    pub missing: Vec<String>,
}

/// Resolve `path`: absoluto; senão relativo à pasta do PROJETO; senão ao directório corrente.
#[must_use]
pub fn resolve(path: &str, project_dir: Option<&Path>, cwd: Option<&Path>) -> Option<(PathBuf, Matched)> {
    let p = Path::new(path);
    if p.is_absolute() {
        return p.is_file().then(|| (p.to_path_buf(), Matched::Absolute));
    }
    [(project_dir, Matched::ProjectDir), (cwd, Matched::Cwd)]
        .into_iter()
        .find_map(|(dir, m)| dir.map(|d| d.join(p)).filter(|c| c.is_file()).map(|c| (c, m)))
}

/// Lê um ficheiro e põe-no na biblioteca com o nome do ficheiro (único).
///
/// # Errors
/// O ficheiro não se lê como texto.
pub fn import_file(lib: &mut ScriptLibrary, path: &Path) -> std::io::Result<ScriptId> {
    let text = std::fs::read_to_string(path)?;
    let name = path
        .file_name()
        .map_or_else(|| "script.luau".to_owned(), |n| n.to_string_lossy().into_owned());
    Ok(lib.insert(&name, text))
}

/// Importa cada `Unimported(caminho)` legível e reaponta o objecto. Um ficheiro, um script: a chave
/// é o caminho CANÓNICO.
pub fn import_unimported(
    world: &mut WorldSnapshot,
    lib: &mut ScriptLibrary,
    project_dir: Option<&Path>,
    cwd: Option<&Path>,
) -> ImportReport {
    let id = stable_type_id(CANONICAL_NAME);
    let mut out = ImportReport::default();
    let mut by_file: BTreeMap<PathBuf, ScriptId> = BTreeMap::new();
    for row in &mut world.entities {
        let Some(slot) = row.components.iter().position(|b| b.type_id == id) else {
            continue;
        };
        let Ok(mut cfg) = postcard::from_bytes::<LuauScript>(&row.components[slot].data) else {
            continue; // o restauro do mundo recusa-o
        };
        let ScriptSource::Unimported(path) = &cfg.source else {
            continue;
        };
        let Some((file, matched)) = resolve(path, project_dir, cwd) else {
            out.missing.push(path.clone());
            continue;
        };
        let key = file.canonicalize().unwrap_or_else(|_| file.clone());
        let sid = match by_file.get(&key) {
            Some(sid) => *sid,
            None => match import_file(lib, &file) {
                Ok(sid) => {
                    out.imported.push(Imported {
                        path: path.clone(),
                        matched,
                        id: sid,
                    });
                    by_file.insert(key, sid);
                    sid
                }
                Err(_) => {
                    out.missing.push(path.clone());
                    continue;
                }
            },
        };
        cfg.source = ScriptSource::Library(sid);
        if let Ok(bytes) = postcard::to_allocvec(&cfg) {
            std::sync::Arc::make_mut(row).components[slot].data = bytes;
            out.objects += 1;
        }
    }
    out
}

#[cfg(test)]
#[path = "project_scripts_tests.rs"]
mod tests;
