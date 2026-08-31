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
    /// ⚠️ **`catalog_bytes`, e não `catalogs`, de propósito** (auditoria de 2026-08-30): o campo
    /// vizinho do `AppGfx` chama-se `catalogs` e é a **árvore viva**. Com o mesmo nome, o censo que
    /// exige `invalidate()` em quem substitui a árvore acusava esta cache — e ensinar-lhe a
    /// excepção seria pedir a um gate que adivinhasse a diferença. *Duas coisas diferentes com o
    /// mesmo nome são um gate cego à espera de acontecer.*
    ///
    /// ⚠️ **Bytes e não a árvore**: a `CatalogTree` vive numa crate-folha **sem serde**, de
    /// propósito (o `AssetRef` é tipo de runtime, e o formato do ficheiro não pode herdar o layout
    /// dele). O blob carrega a própria versão, então a taxonomia evolui sem mover o
    /// `PROJECT_SCHEMA`.
    pub(crate) catalog_bytes: Vec<u8>,
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
    /// ⭐ Quantas vezes ela CODIFICOU — o instrumento da lei desta cache.
    ///
    /// ⛔⛔ Sem ele o gate do *«e só então»* não media nada (auditoria de 2026-08-30): as
    /// asserções eram todas sobre os **bytes de saída**, e o `collect` é determinístico, logo
    /// apagar a guarda deixava o teste **verde** com a cache a re-codificar por quadro. *A lei que
    /// paga o desenho inteiro não tinha instrumento.*
    #[cfg(test)]
    encodes: u32,
}

impl LibraryCache {
    /// ⭐⭐ **O documento deste quadro** — re-codifica só se a árvore se mexeu.
    ///
    /// ⚠️ As lápides são lidas **sempre**: elas são um punhado de ids e não têm revisão própria; a
    /// cache existe para o custo que a medição achou, que é o da taxonomia.
    pub(crate) fn doc(&mut self, tree: &CatalogTree) -> &LibraryDoc {
        if self.rev != Some(tree.revision()) {
            self.rev = Some(tree.revision());
            self.doc.catalog_bytes = crate::project_catalogs::collect(tree);
            #[cfg(test)]
            {
                self.encodes += 1;
            }
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

    /// **Só para os gates:** quantas vezes ela codificou. Ver o campo.
    #[cfg(test)]
    pub(crate) fn probe_encodes(&self) -> u32 {
        self.encodes
    }
}

/// ⭐ **A metade GLOBAL do restauro** — as lápides, que vivem num `thread_local`.
///
/// ⛔⛔ **Ela é uma função à parte de propósito** (auditoria de 2026-08-30). Enquanto as duas
/// metades vinham juntas, o valor de retorno obrigava a chamada a viver dentro do `if let
/// Some(gfx)` dos dois chamadores — e **sem `gfx` (headless, ou a GPU ainda por subir) abrir um
/// projecto B deixava as lápides do A vivas.** *Uma função que devolve um valor e escreve num
/// global é governada pela guarda do valor.*
pub(crate) fn apply_forgotten(doc: &LibraryDoc) {
    crate::asset_index_build::set_forgotten_textures(&doc.forgotten);
}

/// ⭐ **A taxonomia restaurada** — a metade que tem dono.
///
/// ⚠️ Ela devolve a árvore em vez de a escrever: quem a possui é o `AppGfx`, e uma função que
/// escrevesse no campo teria de conhecer a estrutura dele.
#[must_use]
pub(crate) fn apply_catalogs(doc: &LibraryDoc) -> CatalogTree {
    crate::project_catalogs::restore(&doc.catalog_bytes)
}

#[cfg(test)]
#[path = "project_library_tests.rs"]
mod tests;
