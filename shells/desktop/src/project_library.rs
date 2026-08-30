//! ⭐⭐⭐ **O DOCUMENTO DA BIBLIOTECA** — o que o navegador de assets tem de autorado, e a razão de
//! ele viver dentro da unidade do undo.
//!
//! # Porque ele está no `ProjectState` e os irmãos não
//!
//! O `motion`, a `timeline` e a `physics` ficam **fora** do [`crate::undo::ProjectState`] com um
//! motivo cada: os dois primeiros têm undo PRÓPRIO, e a terceira é *setting* de mundo. ⚠️ **Nenhum
//! dos dois motivos vale aqui:** o painel não tem undo próprio, e *«existe uma gaveta chamada
//! Personagens»* é autoria do trabalho, não uma preferência de quem o abre. ⇒ pedido do Enio,
//! 2026-08-30: *«deveria ter undo/redo no painel inclusive em del»*.
//!
//! ⛔ **E a nota que o mantinha fora estava ERRADA no mecanismo.** Ela dizia que metê-lo na captura
//! *«faria toda renomeação de gaveta reescrever o snapshot do mundo inteiro»* — falso desde a F2: a
//! captura do mundo é **incremental** e custa o tamanho da edição. O custo real é outro, e foi
//! medido (`measure_catalog_capture_cost`):
//!
//! | catálogos | atribuições | bytes | `collect` | % de um quadro de 16,7 ms |
//! |---|---|---|---|---|
//! | 4 | 20 | 827 | 8,9 µs | 0,05 % |
//! | 20 | 200 | 7 502 | 87,8 µs | 0,53 % |
//! | 50 | 2 000 | 71 132 | 802 µs | **4,8 %** |
//! | 200 | 10 000 | 358 514 | 4 680 µs | **28 %** |
//!
//! ⇒ **codificar por quadro é que era caro**, e a captura corre em **todo quadro com input**. A
//! cura é a cache por [`ph2d_asset_index::CatalogTree::revision`]: codifica-se **uma vez por
//! mutação**, e o quadro paga só o `clone` dos bytes.
//!
//! # ⚠️ A revisão é chave de CACHE, nunca identidade
//!
//! Quem decide se duas taxonomias são a mesma são **os bytes** — é isso que faz um `restore` de
//! undo não registar um passo espúrio contra a árvore que o produziu (a árvore restaurada nasce com
//! revisão `0`, e re-codificar dá exactamente os mesmos bytes).

use ph2d_asset_index::CatalogTree;

/// O que a biblioteca tem de AUTORADO — e que por isso desfaz.
#[derive(Clone, Default, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct LibraryDoc {
    /// A taxonomia, no blob versionado do [`crate::project_catalogs`].
    ///
    /// ⚠️ **Bytes e não a árvore**: a `CatalogTree` vive numa crate-folha **sem serde**, de
    /// propósito (o `AssetRef` é tipo de runtime, e o formato do ficheiro não pode herdar o layout
    /// dele). O blob carrega a própria versão, então a taxonomia evolui sem mover o
    /// `PROJECT_SCHEMA`.
    pub(crate) catalogs: Vec<u8>,
    /// ⭐ As imagens que o artista mandou SAIR da biblioteca — ver
    /// [`crate::asset_index_build::forgotten_textures`].
    ///
    /// ⚠️ **Ordenado** (vem de um `BTreeSet`), senão duas capturas do mesmo estado dariam bytes
    /// diferentes e o diff registaria um passo por quadro.
    pub(crate) forgotten: Vec<[u8; 32]>,
}

/// A cache que impede a re-codificação por quadro. Vive no `App`.
#[derive(Default)]
pub(crate) struct LibraryCache {
    /// A revisão da árvore de que estes bytes saíram. ⚠️ `None` = nunca codificada.
    rev: Option<u64>,
    doc: LibraryDoc,
}

impl LibraryCache {
    /// ⭐⭐ **O documento deste quadro** — re-codifica só se a árvore se mexeu.
    ///
    /// ⚠️ As lápides são lidas **sempre**: elas são um punhado de ids e não têm revisão própria; a
    /// cache existe para o custo que a medição achou, que é o da taxonomia.
    pub(crate) fn doc(&mut self, tree: &CatalogTree) -> &LibraryDoc {
        if self.rev != Some(tree.revision()) {
            self.rev = Some(tree.revision());
            self.doc.catalogs = crate::project_catalogs::collect(tree);
        }
        self.doc.forgotten = crate::asset_index_build::forgotten_textures();
        &self.doc
    }

    /// Invalida — usada quando a árvore é substituída por baixo (undo, `Open Project`).
    ///
    /// ⛔ Sem ela, restaurar uma árvore cuja revisão calhe igual à da cache devolveria os bytes
    /// ANTIGOS. A revisão é por-árvore e nasce em `0` a cada `restore`, então a colisão é o caso
    /// **normal**, não o raro.
    pub(crate) fn invalidate(&mut self) {
        self.rev = None;
    }
}

/// ⭐ **Aplica um documento restaurado** — a outra metade do undo.
///
/// ⚠️ Ela devolve a árvore em vez de a escrever: quem a possui é o `App`, e uma função que
/// escrevesse num campo global teria de conhecer a estrutura dele.
pub(crate) fn apply(doc: &LibraryDoc) -> CatalogTree {
    crate::asset_index_build::set_forgotten_textures(&doc.forgotten);
    crate::project_catalogs::restore(&doc.catalogs)
}

#[cfg(test)]
#[path = "project_library_tests.rs"]
mod tests;
