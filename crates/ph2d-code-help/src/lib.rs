//! **A página de ajuda** do editor de scripts (CodeEditor Onda 2, item 5).
//!
//! Uma [`Page`] por biblioteca do Luau (`math`, `string`, …, e `globals` para as funções soltas) e
//! as nossas (`ph2d`, `motion`, `types`, `hooks`), geradas da `api_table`. O Ctrl+clique, o F12 e o
//! link dentro da página perguntam TODOS a [`resolve::resolve`] que página e que membro um sítio
//! nomeia. Plano, oráculo e sonda: `docs/CodeEditor/04_pagina_de_ajuda.md`.
//!
//! Texto: o da linguagem em `ph2d-i18n` (`code.help.<página>…`), escrito por nós (a licença do texto
//! do site do Luau não está confirmada); o nosso em `ph2d_script::api_help`. O censo
//! (`tests/it/censo.rs`) confere o catálogo contra o runtime REAL e corre cada exemplo.

mod luau;
mod ours;
pub mod resolve;

use std::sync::OnceLock;

/// A espécie de um membro.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Function,
    Constant,
    Type,
    /// Um gancho que o motor chama (`init`, `update`, `on_signal`).
    Hook,
}

/// Um argumento: o nome na assinatura e o que ele é.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub text: String,
}

/// Um membro de uma página.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    pub name: &'static str,
    pub kind: Kind,
    /// Cada forma de chamar (`(n: number): number`); um tipo dá a definição; uma constante o tipo.
    pub sigs: Vec<String>,
    /// O que faz: parágrafos separados por linha em branco; `` `código` `` e `[pagina.membro]`.
    pub text: String,
    pub params: Vec<Param>,
    /// O que devolve; `None` = nada.
    pub returns: Option<String>,
    /// Código Luau; cada `--> x` no fim de uma linha é o que o `print` dela escreve.
    pub example: Option<&'static str>,
}

/// Uma página.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Page {
    /// `math`, `globals`, `ph2d`, … (o nome com que o script a escreve, salvo `globals`/`types`/`hooks`).
    pub id: &'static str,
    pub title: String,
    /// Uma frase (a negrito no topo).
    pub summary: String,
    /// O resto da descrição (pode ser vazio).
    pub about: String,
    pub members: Vec<Member>,
}

impl Page {
    /// O índice do membro `name`.
    #[must_use]
    pub fn member(&self, name: &str) -> Option<usize> {
        self.members.iter().position(|m| m.name == name)
    }
}

/// Um sítio de uma página: o topo (`member: None`) ou um membro.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Anchor {
    pub page: &'static str,
    pub member: Option<&'static str>,
}

/// Todas as páginas, pela ordem do índice (as da linguagem, depois as nossas).
#[must_use]
pub fn catalog() -> &'static [Page] {
    static CATALOG: OnceLock<Vec<Page>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let mut pages = luau::pages();
        pages.extend(ours::pages());
        pages
    })
}

/// A página `id`.
#[must_use]
pub fn page(id: &str) -> Option<&'static Page> {
    catalog().iter().find(|p| p.id == id)
}

/// O sítio `page.member` se existe (`member` vazio = o topo).
#[must_use]
pub fn anchor(page_id: &str, member: Option<&str>) -> Option<Anchor> {
    let p = page(page_id)?;
    let member = match member {
        Some(m) => Some(p.members[p.member(m)?].name),
        None => None,
    };
    Some(Anchor { page: p.id, member })
}
