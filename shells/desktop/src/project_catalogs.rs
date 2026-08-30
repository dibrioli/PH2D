//! ⭐⭐ **A TAXONOMIA dentro do ficheiro de projecto** (plano 07, wave A3) — irmão do
//! [`crate::project_texture_pattern`], e construído no molde dele.
//!
//! # Onde ela mora, e porquê AQUI
//!
//! Um catálogo é **autoria do projecto**, não preferência de utilizador: *«existe uma gaveta
//! chamada Personagens»* é uma decisão do trabalho, e não do teclado de quem o abre. É a mesma
//! divisão que o `input_map` declara — o que é do jogo vai no `.ph2dproj`, o que é do jogador vai
//! em `~/.ph2d/`.
//!
//! ⛔ **E não vai no `ProjectState`**, que é a unidade do undo. Ela viaja **ao lado** dele, como o
//! `motion`, a `timeline`, o `physics` e o `input_map` — ⚠️ com a consequência declarada: **um
//! gesto de catálogo não se desfaz com Ctrl+Z**. É a mesma dívida que o Input Map ship-ou, e a
//! alternativa (metê-la na captura do undo) faria toda renomeação de gaveta reescrever o snapshot
//! do mundo inteiro.
//!
//! # O blob carrega a PRÓPRIA versão
//!
//! [`CATALOG_DOC_VERSION`] mora dentro dos bytes, então esta taxonomia pode evoluir muitas waves
//! sem tocar no `PROJECT_SCHEMA` — o precedente exacto do `TimelineDoc`, do `sculpt` e da arte de
//! padrão. ⚠️ **O `PROJECT_SCHEMA` bumpa UMA vez, quando o campo nasce**, e é isso: um campo novo
//! no fim faz o postcard de um ficheiro anterior chegar ao fim dos bytes (`Hit the end of buffer`),
//! e o número é o que transforma isso num erro de versão em vez de num postcard a falhar longe da
//! causa.
//!
//! # ⚠️ O `AssetRef` não é serializável, e a razão é boa
//!
//! Ele vive numa crate **folha sem serde** ([`ph2d_asset_index`]), de propósito. ⇒ o formato tem o
//! par de fio [`SavedAssignment`], e a conversão vive aqui — que é onde a decisão *«como isto se
//! grava»* pertence. ⛔ Pôr `serde` naquela crate faria o formato do ficheiro herdar o layout de um
//! tipo de runtime, que é o que o `project_settings` declara como o erro a não repetir.

use ph2d_asset_index::{AssetRef, Catalog, CatalogId, CatalogTree};
use std::collections::BTreeMap;

/// A versão do blob. ⚠️ Ela mora **dentro** dos bytes — ver o cabeçalho.
const CATALOG_DOC_VERSION: u32 = 1;

/// Um catálogo, no fio.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct SavedCatalog {
    /// A identidade durável.
    pub(crate) id: u128,
    /// `"Personagens/Heróis"` — a hierarquia vive aqui.
    pub(crate) path: String,
}

/// `asset → catálogo`, no fio.
///
/// ⚠️ **As duas famílias têm endereços de tipos diferentes** (um `StableId` de 64 bits para um
/// prefab; 32 bytes de blake3 para uma imagem), e o fio guarda os dois em campos separados em vez
/// de os colapsar: colapsá-los obrigaria um dos dois a mentir sobre a própria identidade, que é a
/// nota que o `AssetRef` já carrega.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct SavedAssignment {
    /// `Some` para um prefab.
    pub(crate) prefab: Option<u64>,
    /// `Some` para uma imagem.
    pub(crate) image: Option<[u8; 32]>,
    /// O catálogo a que ele pertence.
    pub(crate) catalog: u128,
}

impl SavedAssignment {
    fn of(asset: &AssetRef, catalog: CatalogId) -> Self {
        match asset {
            AssetRef::Component { stable_id } => Self {
                prefab: Some(*stable_id),
                image: None,
                catalog: catalog.0,
            },
            AssetRef::Texture { asset } => Self {
                prefab: None,
                image: Some(*asset),
                catalog: catalog.0,
            },
        }
    }

    /// ⚠️ `None` para uma linha que não nomeia nenhuma das duas famílias — um ficheiro de uma
    /// versão futura pode ter uma terceira, e **ignorá-la é melhor que adivinhar**.
    fn to_pair(&self) -> Option<(AssetRef, CatalogId)> {
        let key = match (self.prefab, self.image) {
            (Some(stable_id), None) => AssetRef::Component { stable_id },
            (None, Some(asset)) => AssetRef::Texture { asset },
            _ => return None,
        };
        Some((key, CatalogId(self.catalog)))
    }
}

/// **Os bytes que o `.ph2dproj` guarda.**
///
/// ⚠️ **Ordenado e determinístico** (HR-5): a lista de catálogos já sai ordenada da árvore, e as
/// atribuições vêm de um `BTreeMap`. Dois saves da mesma taxonomia produzem os mesmos bytes.
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
    postcard::to_allocvec(&(CATALOG_DOC_VERSION, catalogs, assignments)).unwrap_or_default()
}

/// **A taxonomia que estes bytes descrevem.**
///
/// ⚠️ **Um blob ilegível ou de outra versão devolve uma taxonomia VAZIA e DIZ** — nunca estoura, e
/// nunca fica em silêncio. Um projecto que abrisse sem catálogos e sem uma linha de log faria o
/// artista concluir que o trabalho de arrumação se perdeu sem nada a que agarrar.
pub(crate) fn restore(blob: &[u8]) -> CatalogTree {
    if blob.is_empty() {
        return CatalogTree::new();
    }
    let Ok((ver, catalogs, assignments)) =
        postcard::from_bytes::<(u32, Vec<SavedCatalog>, Vec<SavedAssignment>)>(blob)
    else {
        eprintln!("[proj] catalogos: blob ilegivel, a biblioteca abre sem taxonomia");
        return CatalogTree::new();
    };
    if ver != CATALOG_DOC_VERSION {
        eprintln!("[proj] catalogos: versao {ver} desconhecida, ignorada");
        return CatalogTree::new();
    }
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
    )
}

#[cfg(test)]
#[path = "project_catalogs_tests.rs"]
mod tests;
