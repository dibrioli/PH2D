//! ⭐⭐ **Os sons do projeto** (GEF 21 E2) — importar um ficheiro, gravar e abrir o `ProjectFile.sounds`.
//!
//! O som é asset do projeto por referência, como a imagem: os bytes vivem na
//! [`ProjectSounds`] do mundo e viajam no blob `PH2DSONS`; o `AudioSource2D.sound` guarda o NOME.
//! ⭐ **Todo ficheiro que chega ao campo vira som do projeto** — o Browse, um caminho escrito, a
//! queda de um ficheiro no app e o caminho antigo de um projeto `<= 214` (importado no load).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ph2d_ecs::scene::{WorldSnapshot, stable_type_id};
use ph2d_ecs::{AudioSource2D, ProjectSounds, SoundsBlobError, World};
use ph2d_i18n::tr_with;
use ph2d_script::project_scripts::resolve;

const SOURCE: &str = "ph2d::ecs::AudioSource2D";

/// O ficheiro não entra como som do projeto.
#[derive(Debug)]
pub enum ImportError {
    Read(std::io::Error),
    /// Os bytes não são áudio que este app descodifique.
    NotAudio(String),
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read(e) => write!(f, "{e}"),
            Self::NotAudio(e) => f.write_str(e),
        }
    }
}

/// **Importa** um ficheiro para os sons do projeto e devolve o nome com que ficou.
///
/// ⚠️ Descodifica uma vez ANTES de aceitar: um ficheiro que não é som não entra na biblioteca, e o
/// artista ouve a recusa em vez de um campo que nunca toca.
///
/// # Errors
/// O ficheiro não se lê, ou não é áudio.
pub fn import_into(lib: &mut ProjectSounds, path: &Path) -> Result<String, ImportError> {
    let bytes = std::fs::read(path).map_err(ImportError::Read)?;
    ph2d_audio_decode::decode_any::decode(&bytes).map_err(ImportError::NotAudio)?;
    let name = path
        .file_name()
        .map_or_else(|| "Sound".to_owned(), |n| n.to_string_lossy().into_owned());
    Ok(lib.add(&name, bytes))
}

/// [`import_into`] sobre a biblioteca do mundo (criada se faltar).
///
/// # Errors
/// Ver [`import_into`].
pub fn import(world: &mut World, path: &Path) -> Result<String, ImportError> {
    import_into(&mut world.get_resource_or_init::<ProjectSounds>(), path)
}

/// **Importa uma leva** (o diálogo Import… ou a queda de ficheiros na janela) e devolve uma frase por
/// ficheiro: onde ficou (Assets), ou porque não entrou.
pub fn import_batch(world: &mut World, paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|p| {
            let file = p.file_name().map(|n| n.to_string_lossy().into_owned());
            let file = file.unwrap_or_default();
            match import(world, p) {
                Ok(name) => tr_with(
                    "app.components.sound_project.added_to_the_project_sounds",
                    &[("name", &name)],
                ),
                Err(e) => tr_with(
                    "app.components.audio_inspector.not_a_sound",
                    &[("file", &file), ("why", &e)],
                ),
            }
        })
        .collect()
}

/// [`import`] na forma da porta `ph2d_app_game::smoke::Portas::importa_som`.
///
/// # Errors
/// Ver [`import_into`].
pub fn import_for_scene(sim: &mut ph2d_ecs::SimWorld, path: &Path) -> Result<String, String> {
    import(sim.world_mut(), path).map_err(|e| e.to_string())
}

/// Um ficheiro deste app é som? — a MESMA lista do diálogo de importação (`decode_any`).
#[must_use]
pub fn is_sound_file(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
        ph2d_audio_decode::decode_any::AUDIO_IMPORT_EXTS
            .iter()
            .any(|x| x.eq_ignore_ascii_case(e))
    })
}

/// O blob a gravar — vazio quando o projeto não tem sons.
#[must_use]
pub fn save(world: &World) -> Vec<u8> {
    world
        .get_resource::<ProjectSounds>()
        .map(ProjectSounds::to_bytes)
        .unwrap_or_default()
}

/// Os sons abertos e o que a importação tem de dizer ao artista.
pub struct Opened {
    pub lib: ProjectSounds,
    /// A frase do toast (`None` = nada a dizer).
    pub notice: Option<String>,
}

/// **Abre** os sons de um ficheiro e importa cada `sound` que a biblioteca não conhece e que se lê
/// como caminho (absoluto · pasta do PROJETO · directório corrente), reescrevendo-o para o nome.
/// O que não se acha fica como está — o cartão diz «not in the project».
///
/// # Errors
/// O blob não é uma biblioteca legível (o load recusa o ficheiro inteiro, como os scripts).
pub fn open(
    snapshot: &mut WorldSnapshot,
    bytes: &[u8],
    project_path: &str,
) -> Result<Opened, SoundsBlobError> {
    let mut lib = ProjectSounds::from_bytes(bytes)
        .inspect_err(|e| eprintln!("[proj] sons ilegiveis — load RECUSADO: {e}"))?;
    let cwd = std::env::current_dir().ok();
    let (copiados, faltam) = import_paths(
        snapshot,
        &mut lib,
        Path::new(project_path).parent(),
        cwd.as_deref(),
    );
    let mut parts = Vec::new();
    if copiados > 0 {
        parts.push(tr_with(
            "app.components.sound_project.copied_sound_files_into_the_project",
            &[("n", &copiados)],
        ));
    }
    if faltam > 0 {
        parts.push(tr_with(
            "app.components.sound_project.sound_files_not_found",
            &[("n", &faltam)],
        ));
    }
    let notice = (!parts.is_empty()).then(|| parts.join(" \u{00b7} "));
    Ok(Opened { lib, notice })
}

/// O passe do [`open`]: devolve `(ficheiros importados, caminhos que não se acharam)`.
pub(crate) fn import_paths(
    world: &mut WorldSnapshot,
    lib: &mut ProjectSounds,
    project_dir: Option<&Path>,
    cwd: Option<&Path>,
) -> (usize, usize) {
    let id = stable_type_id(SOURCE);
    let mut by_file: BTreeMap<PathBuf, String> = BTreeMap::new();
    let (mut copiados, mut faltam) = (0, 0);
    for row in &mut world.entities {
        let Some(slot) = row.components.iter().position(|b| b.type_id == id) else {
            continue;
        };
        let Ok(mut cfg) = postcard::from_bytes::<AudioSource2D>(&row.components[slot].data) else {
            continue; // o restauro do mundo recusa-o
        };
        let path = cfg.sound.trim().to_owned();
        if path.is_empty() || lib.get(&path).is_some() {
            continue;
        }
        let Some((file, _)) = resolve(&path, project_dir, cwd) else {
            eprintln!("[proj] som por importar (ficheiro nao encontrado): {path}");
            faltam += 1;
            continue;
        };
        let key = file.canonicalize().unwrap_or_else(|_| file.clone());
        let name = if let Some(n) = by_file.get(&key) {
            n.clone()
        } else {
            let antes = lib.len();
            match import_into(lib, &file) {
                Ok(n) => {
                    eprintln!("[proj] som importado: {path} -> {n}");
                    copiados += usize::from(lib.len() > antes);
                    by_file.insert(key, n.clone());
                    n
                }
                Err(e) => {
                    eprintln!("[proj] som por importar ({path}): {e}");
                    faltam += 1;
                    continue;
                }
            }
        };
        cfg.sound = name;
        if let Ok(b) = postcard::to_allocvec(&cfg) {
            std::sync::Arc::make_mut(row).components[slot].data = b;
        }
    }
    (copiados, faltam)
}

/// Põe os sons abertos no mundo — substitui os do documento anterior.
pub fn install(world: &mut World, lib: ProjectSounds) {
    world.insert_resource(lib);
}

#[cfg(test)]
#[path = "sound_project_tests.rs"]
mod tests;
