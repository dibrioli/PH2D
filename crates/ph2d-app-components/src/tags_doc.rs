//! ⭐⭐ **A ÁRVORE DE TAGS dentro do ficheiro de projecto** (TOP-20 #9,
//! `docs/Components/08_plano_tags.md` §2.1) — o formato, e a cache que o torna barato.
//!
//! # Onde ela mora, e porquê AQUI
//!
//! A árvore de tags é **autoria do projecto** (*«existe uma tag chamada Enemy»* é uma decisão do
//! trabalho), e apagar uma tag **desfaz-se** junto com a pertença que ela levou — logo ela viaja
//! dentro do `ProjectState`, que é a unidade do undo, como a biblioteca de assets
//! (`shells/desktop/src/project_library.rs`). A shell guarda os BYTES no estado e a árvore viva no
//! `AppGfx`; o formato e a cache vivem aqui, na família, e a shell só os chama.
//!
//! # O blob carrega a PRÓPRIA versão
//!
//! [`TAGS_DOC_VERSION`] mora dentro dos bytes, então a árvore pode ganhar campos (uma descrição —
//! a D3 do dono deixou-a de fora — uma cor, uma ordem manual) sem tocar no `PROJECT_SCHEMA`: o
//! precedente do `CATALOG_DOC_VERSION`. ⚠️ **O `PROJECT_SCHEMA` sobe UMA vez, quando o campo nasce**.
//!
//! # ⚠️ A cache é por REVISÃO, e a revisão é chave de cache, nunca identidade
//!
//! A captura do undo corre em **todo quadro com input**; codificar a árvore a cada um é o custo que
//! a biblioteca mediu em até `28 %` de um quadro. ⇒ codifica-se uma vez por mutação. Quem decide se
//! duas árvores são a mesma são os BYTES — é isso que faz um restauro não registar um passo espúrio
//! (a árvore restaurada nasce com revisão `0`, e re-codificar dá os mesmos bytes).
//!
//! # ⚠️ A `TagTree` não é serializável, e a razão é boa
//!
//! Ela vive numa folha **sem serde**, de propósito: o formato do ficheiro não pode herdar o layout
//! de um tipo de runtime. ⇒ o par de fio é o [`SavedTag`], e a conversão vive aqui.

use ph2d_tags::{Tag, TagId, TagTree};
use std::collections::BTreeMap;

/// A versão do blob. ⚠️ Mora **dentro** dos bytes — ver o cabeçalho.
pub const TAGS_DOC_VERSION: u32 = 1;

/// Uma tag, no fio.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SavedTag {
    /// A identidade durável — o que a pertença dos objectos guarda.
    pub id: u64,
    /// `"Enemy/Flying"`, com a grafia de quem o escreveu.
    pub path: String,
}

/// **Os bytes que o `.ph2dproj` guarda**: `(versão, tags pela ordem da árvore, next_id)`.
///
/// ⚠️ **O `next_id` viaja** — a lição que o `CATALOG_DOC_VERSION` 2 pagou: derivá-lo como
/// `max(id) + 1` recicla o id de uma tag apagada, e os objectos dela reapareceriam dentro da seguinte.
#[must_use]
pub fn collect(tree: &TagTree) -> Vec<u8> {
    let tags: Vec<SavedTag> = tree
        .tags()
        .map(|t| SavedTag {
            id: t.id.0,
            path: t.path.clone(),
        })
        .collect();
    // ⚠️ Um tuplo de `u32`, `Vec` e `u64` não falha a codificar; o vazio seria o degradado seguro.
    postcard::to_allocvec(&(TAGS_DOC_VERSION, tags, tree.next_id())).unwrap_or_default()
}

/// **A árvore que estes bytes descrevem**, e para onde foi cada gémeo que ela fundiu.
///
/// ⚠️ **Um blob ilegível ou de outra versão devolve uma árvore VAZIA e DIZ** — nunca estoura e nunca
/// fica em silêncio. Um blob VAZIO é um projecto que não tinha árvore, e abre vazio calado.
///
/// ⚠️ Quem restaura a pertença aplica o mapa (`ph2d_ecs::tags::remap`) no MESMO gesto — senão os
/// objectos do gémeo descartado perdiam a tag em silêncio.
#[must_use]
pub fn restore(blob: &[u8]) -> (TagTree, BTreeMap<TagId, TagId>) {
    if blob.is_empty() {
        return (TagTree::new(), BTreeMap::new());
    }
    let Ok((ver, tags, next_id)) = postcard::from_bytes::<(u32, Vec<SavedTag>, u64)>(blob) else {
        eprintln!("[proj] tags: blob ilegivel, o projecto abre sem arvore de tags");
        return (TagTree::new(), BTreeMap::new());
    };
    if ver != TAGS_DOC_VERSION {
        eprintln!("[proj] tags: versao {ver} desconhecida, ignorada");
        return (TagTree::new(), BTreeMap::new());
    }
    let (tree, remap) = TagTree::restore(
        tags.into_iter()
            .map(|t| Tag {
                id: TagId(t.id),
                path: t.path,
            })
            .collect(),
        next_id,
    );
    if !remap.is_empty() {
        eprintln!(
            "[proj] tags: {} gemeo(s) fundido(s) no menor id",
            remap.len()
        );
    }
    (tree, remap)
}

/// A cache que impede a árvore de ser re-codificada por quadro. Vive no `AppGfx`, ao lado dela.
#[derive(Default)]
pub struct TagsCache {
    /// A revisão de que estes bytes saíram. `None` = nunca codificada (ou invalidada).
    rev: Option<u64>,
    bytes: Vec<u8>,
    /// ⭐ Quantas vezes CODIFICOU — o instrumento da lei desta cache (a lição da biblioteca: sem ele
    /// o gate do *«uma vez por mutação»* media só os bytes, que são deterministas).
    #[cfg(test)]
    encodes: u32,
}

impl TagsCache {
    /// ⭐⭐ **Os bytes deste quadro** — re-codifica só se a árvore se mexeu.
    pub fn doc(&mut self, tree: &TagTree) -> &[u8] {
        if self.rev != Some(tree.revision()) {
            self.rev = Some(tree.revision());
            self.bytes = collect(tree);
            #[cfg(test)]
            {
                self.encodes += 1;
            }
        }
        &self.bytes
    }

    /// ⛔ **Obrigatório quando a árvore é SUBSTITUÍDA** (undo, load): a árvore nova nasce com
    /// revisão `0`, e colidir com a revisão que a cache já viu é o caso NORMAL.
    pub fn invalidate(&mut self) {
        self.rev = None;
    }
}

#[cfg(test)]
#[path = "tags_doc_tests.rs"]
mod tests;
