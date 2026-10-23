//! ⭐ **O NOME de uma imagem da biblioteca** — o que o painel Assets mostra E o que um script escreve
//! (`ph2d.set(e, "sprite.image", "door_open")`, doc CodeEditor 08 §5 d4). Uma função, dois leitores.
//!
//! O nome é o do ficheiro SEM a extensão, e é ÚNICO: dois `door.png` em pastas diferentes são `door`
//! e `door (2)`, pela ordem (nome, caminho, blake3) — a mesma em toda sessão, porque nenhum dos três
//! muda. ⛔ Até ao d4 era o `file_name()` com a extensão, e dois ficheiros iguais colidiam.

use crate::asset_index_build::{TextureLibrary, texture_of};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_asset_index::AssetRef;
use ph2d_ecs::{Entity, Name, SimWorld, SpritePixels, StableId};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// O nome-base de um ficheiro: sem a pasta e sem a extensão.
#[must_use]
pub fn stem(path: &Path) -> Option<String> {
    path.file_stem().map(|n| n.to_string_lossy().into_owned())
}

/// O 1.º caminho (a ordem do `AssetDb`) de cada imagem — uma passagem, não uma busca por id.
pub(crate) fn paths_of(db: &AssetDb) -> BTreeMap<AssetId, PathBuf> {
    let mut out = BTreeMap::new();
    for p in db.tracked_paths() {
        if let Some(id) = db.id_for_path(&p) {
            out.entry(id).or_insert(p);
        }
    }
    out
}

/// ⭐ **Os nomes únicos.** Cada imagem: o nome do ficheiro, senão o `fallback` (o `Name` de quem a usa),
/// senão os 12 primeiros hex do blake3. Um nome repetido ganha ` (2)`, ` (3)`… pela ordem (nome,
/// caminho, id), saltando o nome-base de outra imagem (um `door (2).png` verdadeiro).
#[must_use]
pub fn unique(
    paths: &BTreeMap<AssetId, PathBuf>,
    ids: &BTreeMap<AssetId, Option<String>>,
) -> BTreeMap<AssetId, String> {
    let mut rows: Vec<(String, String, AssetId)> = ids
        .iter()
        .map(|(id, fallback)| {
            let path = paths.get(id);
            let base = path
                .and_then(|p| stem(p))
                .or_else(|| fallback.clone())
                .unwrap_or_else(|| id.to_hex()[..12].to_string());
            let at = path.map(|p| p.to_string_lossy().into_owned());
            (base, at.unwrap_or_default(), *id)
        })
        .collect();
    rows.sort_by(|a, b| (&a.0, &a.1, a.2.as_bytes()).cmp(&(&b.0, &b.1, b.2.as_bytes())));
    let bases: BTreeSet<String> = rows.iter().map(|r| r.0.clone()).collect();
    let mut used = BTreeSet::new();
    let mut out = BTreeMap::new();
    for (base, _, id) in rows {
        let mut name = base.clone();
        let mut n = 2;
        while used.contains(&name) || (name != base && bases.contains(&name)) {
            name = format!("{base} ({n})");
            n += 1;
        }
        used.insert(name.clone());
        out.insert(id, name);
    }
    out
}

/// ⭐⭐ **A biblioteca inteira, com nome** — o universo é o MESMO para o painel e para o script: o que o
/// mundo usa agora, o que o artista trouxe (`atlas_assets`: import, tela nova, abrir) e o que a
/// biblioteca lembra, menos o que o artista mandou sair e ninguém usa.
pub(crate) fn library(
    sim: &mut SimWorld,
    db: &AssetDb,
    atlas_assets: &BTreeMap<u32, AssetId>,
    remembered: &TextureLibrary,
) -> BTreeMap<AssetId, String> {
    let mut users: Vec<(u64, AssetId, Option<String>)> = {
        let w = sim.world_mut();
        let mut q = w.query::<(
            Entity,
            Option<&SpritePixels>,
            Option<&ph2d_render::Sprite>,
            Option<&StableId>,
            Option<&Name>,
        )>();
        q.iter(w)
            .filter_map(|(_, px, spr, sid, name)| {
                let id = texture_of(px, spr, atlas_assets)?;
                Some((sid.map_or(u64::MAX, |s| s.0), id, name.map(|n| n.0.clone())))
            })
            .collect()
    };
    users.sort_by_key(|(order, _, _)| *order);
    let live: BTreeSet<AssetId> = users.iter().map(|(_, id, _)| *id).collect();
    let mut ids: BTreeMap<AssetId, Option<String>> = BTreeMap::new();
    for (_, id, name) in users {
        ids.entry(id).or_insert(name);
    }
    for e in remembered.entries(&live) {
        if let AssetRef::Texture { asset } = e.key {
            ids.entry(AssetId::from_digest(asset))
                .or_insert_with(|| Some(e.name.clone()));
        }
    }
    for id in atlas_assets.values() {
        if live.contains(id) || !remembered.is_forgotten(id) {
            ids.entry(*id).or_insert(None);
        }
    }
    unique(&paths_of(db), &ids)
}

/// A biblioteca da sessão, com nome — a porta do script ([`crate::script_image`]).
pub fn session_library(
    sim: &mut SimWorld,
    db: &AssetDb,
    atlas_assets: &BTreeMap<u32, AssetId>,
) -> BTreeMap<AssetId, String> {
    crate::asset_index_build::with_library(|lib| library(sim, db, atlas_assets, lib))
}

#[cfg(test)]
#[path = "library_names_tests.rs"]
mod tests;
