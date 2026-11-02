//! ⭐⭐ **A TAXONOMIA dentro do ficheiro de projecto** (plano 07, wave A3) — irmão do
//! [`crate::project_texture_pattern`]. Um catálogo é autoria do PROJECTO (vai no `.ph2dproj`, e no
//! `ProjectState` desde 30/08 — undo/redo —, composto pelo [`crate::project_library`]); este módulo
//! é só o **formato** (bytes ⇄ árvore).
//!
//! [`CATALOG_DOC_VERSION`] mora dentro dos bytes: a taxonomia evolui sem tocar no `PROJECT_SCHEMA`
//! (o precedente do `TimelineDoc`). ⚠️ O `AssetRef` vive numa crate folha SEM serde, de propósito:
//! o par de fio é [`SavedAssignment`] e a conversão vive aqui — o formato do ficheiro nunca herda o
//! layout de um tipo de runtime.

use ph2d_asset_index::{AssetRef, Catalog, CatalogId, CatalogTree};
use std::collections::BTreeMap;

/// A versão do blob. ⚠️ Ela mora **dentro** dos bytes — ver o cabeçalho.
///
/// * **2** — o `next_id` viaja (auditoria de 2026-08-30): a v1 derivava-o como `max(id) + 1` e
///   RECICLAVA o id de um catálogo apagado (a escolha da coluna sobrevive ao Ctrl+Z e apontaria
///   em silêncio para um catálogo criado depois).
/// * **3** — D17 (dono 30/10): um Kanban e um Quadro também se arrumam numa gaveta. A v2 lê-se
///   ([`SavedAssignmentV2`]); o postcard não se descreve, então cada versão tem o seu par de fio.
const CATALOG_DOC_VERSION: u32 = 3;

/// Um catálogo, no fio.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct SavedCatalog {
    /// A identidade durável.
    pub(crate) id: u128,
    /// `"Personagens/Heróis"` — a hierarquia vive aqui.
    pub(crate) path: String,
}

/// `asset → catálogo`, no fio. ⚠️ Um campo por família (os endereços são de tipos diferentes):
/// colapsá-los obrigaria um deles a mentir sobre a própria identidade.
#[derive(serde::Serialize, serde::Deserialize, Default)]
pub(crate) struct SavedAssignment {
    /// `Some` para um prefab.
    pub(crate) prefab: Option<u64>,
    /// `Some` para uma imagem.
    pub(crate) image: Option<[u8; 32]>,
    /// O catálogo a que ele pertence.
    pub(crate) catalog: u128,
    /// `Some` para um Kanban (v3).
    pub(crate) kanban: Option<u64>,
    /// `Some` para um Quadro (v3).
    pub(crate) board: Option<u64>,
}

/// A linha da v2 — só para ler ficheiros antigos.
#[derive(serde::Deserialize)]
pub(crate) struct SavedAssignmentV2 {
    prefab: Option<u64>,
    image: Option<[u8; 32]>,
    catalog: u128,
}

impl From<SavedAssignmentV2> for SavedAssignment {
    fn from(v: SavedAssignmentV2) -> Self {
        let (prefab, image, catalog) = (v.prefab, v.image, v.catalog);
        Self {
            prefab,
            image,
            catalog,
            ..Self::default()
        }
    }
}

impl SavedAssignment {
    fn of(asset: &AssetRef, catalog: CatalogId) -> Self {
        let mut s = Self {
            catalog: catalog.0,
            ..Self::default()
        };
        match *asset {
            AssetRef::Component { stable_id } => s.prefab = Some(stable_id),
            AssetRef::Texture { asset } => s.image = Some(asset),
            AssetRef::Kanban { id } => s.kanban = Some(id),
            AssetRef::Board { id } => s.board = Some(id),
            AssetRef::Sound { .. } => {} // a queda não o arruma (`asset_drop::resolve`)
        }
        s
    }

    /// ⚠️ `None` para uma linha que não nomeia exactamente UMA família — **ignorá-la é melhor
    /// que adivinhar**.
    fn to_pair(&self) -> Option<(AssetRef, CatalogId)> {
        let key = match (self.prefab, self.image, self.kanban, self.board) {
            (Some(stable_id), None, None, None) => AssetRef::Component { stable_id },
            (None, Some(asset), None, None) => AssetRef::Texture { asset },
            (None, None, Some(id), None) => AssetRef::Kanban { id },
            (None, None, None, Some(id)) => AssetRef::Board { id },
            _ => return None,
        };
        Some((key, CatalogId(self.catalog)))
    }
}

/// **Os bytes que o `.ph2dproj` guarda.** ⚠️ Ordenado e determinístico (HR-5): os catálogos saem
/// ordenados da árvore e as atribuições de um `BTreeMap`.
pub(crate) fn collect(tree: &CatalogTree) -> Vec<u8> {
    let catalogs: Vec<SavedCatalog> = tree
        .catalogs()
        .iter()
        .map(|c| SavedCatalog {
            id: c.id.0,
            path: c.path.clone(),
        })
        .collect();
    let assignments: Vec<SavedAssignment> = tree
        .assignments()
        .iter()
        .map(|(a, c)| SavedAssignment::of(a, *c))
        .collect();
    postcard::to_allocvec(&(CATALOG_DOC_VERSION, catalogs, assignments, tree.next_id()))
        .unwrap_or_default()
}

type Blob<A> = (u32, Vec<SavedCatalog>, Vec<A>, u128);

/// **A taxonomia que estes bytes descrevem.** ⚠️ Um blob ilegível ou de outra versão devolve uma
/// taxonomia VAZIA e DIZ — nunca estoura, e nunca fica em silêncio.
pub(crate) fn restore(blob: &[u8]) -> CatalogTree {
    if blob.is_empty() {
        return CatalogTree::new();
    }
    let parsed = match postcard::take_from_bytes::<u32>(blob) {
        Ok((CATALOG_DOC_VERSION, _)) => postcard::from_bytes::<Blob<SavedAssignment>>(blob).ok(),
        Ok((2, _)) => postcard::from_bytes::<Blob<SavedAssignmentV2>>(blob)
            .ok()
            .map(|(v, c, a, n)| (v, c, a.into_iter().map(Into::into).collect(), n)),
        Ok((ver, _)) => {
            eprintln!("[proj] catalogos: versao {ver} desconhecida, ignorada");
            return CatalogTree::new();
        }
        Err(_) => None,
    };
    let Some((_, catalogs, assignments, next_id)) = parsed else {
        eprintln!("[proj] catalogos: blob ilegivel, a biblioteca abre sem taxonomia");
        return CatalogTree::new();
    };
    CatalogTree::restore(
        catalogs
            .into_iter()
            .map(|c| Catalog {
                id: CatalogId(c.id),
                path: c.path,
            })
            .collect(),
        assignments
            .iter()
            .filter_map(SavedAssignment::to_pair)
            .collect::<BTreeMap<_, _>>(),
        next_id,
    )
}

#[cfg(test)]
#[path = "project_catalogs_tests.rs"]
mod tests;
