//! As páginas da LINGUAGEM: uma por biblioteca que o runtime abre (o censo `tests/it/censo.rs` lê-as
//! do `ScriptHost` real) e `globals` para as funções soltas. Aqui só o código — assinaturas (as do
//! `luau-lsp` 1.70.1, conferidas pelo gate `oraculo_lsp::a_assinatura_da_pagina_e_a_do_assistente` do
//! `ph2d-app-code`) e exemplos (corridos pelo censo); o texto é `code.help.<página>.<membro>` em
//! `ph2d-i18n`, com `.<parâmetro>` e `.returns`.

use crate::{Kind, Member, Page, Param};
use ph2d_i18n::tr;

pub(crate) struct Entry {
    pub name: &'static str,
    pub kind: Kind,
    pub sigs: &'static [&'static str],
    /// Os nomes dos argumentos com texto (`code.help.<p>.<m>.<nome>`), pela ordem da 1.ª assinatura.
    pub params: &'static [&'static str],
    /// Devolve alguma coisa (`code.help.<p>.<m>.returns`).
    pub returns: bool,
    pub example: &'static str,
}

pub(crate) struct Lib {
    pub id: &'static str,
    pub entries: &'static [Entry],
}

mod bit32;
mod buffer;
mod coroutine;
mod debug;
mod globals;
mod integer;
mod math;
mod os;
mod string;
mod table;
mod utf8;
mod vector;

/// A página das funções soltas (`print`, `typeof`, …).
pub(crate) const GLOBALS: &str = "globals";

/// Pela ordem do índice: as funções soltas, depois as bibliotecas por ordem alfabética.
pub(crate) const LIBS: &[Lib] = &[
    Lib {
        id: GLOBALS,
        entries: globals::ENTRIES,
    },
    Lib {
        id: "bit32",
        entries: bit32::ENTRIES,
    },
    Lib {
        id: "buffer",
        entries: buffer::ENTRIES,
    },
    Lib {
        id: "coroutine",
        entries: coroutine::ENTRIES,
    },
    Lib {
        id: "debug",
        entries: debug::ENTRIES,
    },
    Lib {
        id: "integer",
        entries: integer::ENTRIES,
    },
    Lib {
        id: "math",
        entries: math::ENTRIES,
    },
    Lib {
        id: "os",
        entries: os::ENTRIES,
    },
    Lib {
        id: "string",
        entries: string::ENTRIES,
    },
    Lib {
        id: "table",
        entries: table::ENTRIES,
    },
    Lib {
        id: "utf8",
        entries: utf8::ENTRIES,
    },
    Lib {
        id: "vector",
        entries: vector::ENTRIES,
    },
];

/// A chave do texto de um argumento (`...` não é um nome de chave).
fn param_key(name: &str) -> &str {
    if name == "..." { "vararg" } else { name }
}

fn text(key: &str) -> String {
    tr(key).to_owned()
}

pub(crate) fn pages() -> Vec<Page> {
    LIBS.iter()
        .map(|lib| {
            let base = format!("code.help.{}", lib.id);
            Page {
                id: lib.id,
                // o nome de uma biblioteca é código (`math`); só `globals` não é uma palavra do script
                title: if lib.id == GLOBALS {
                    text(&format!("{base}.title"))
                } else {
                    lib.id.to_owned()
                },
                summary: text(&base),
                about: text(&format!("{base}.about")),
                members: lib
                    .entries
                    .iter()
                    .map(|e| {
                        let key = format!("{base}.{}", e.name);
                        Member {
                            name: e.name,
                            kind: e.kind,
                            sigs: e.sigs.iter().map(|s| (*s).to_owned()).collect(),
                            text: text(&key),
                            params: e
                                .params
                                .iter()
                                .map(|p| Param {
                                    name: (*p).to_owned(),
                                    text: text(&format!("{key}.{}", param_key(p))),
                                })
                                .collect(),
                            returns: e.returns.then(|| text(&format!("{key}.returns"))),
                            example: (!e.example.is_empty()).then_some(e.example),
                        }
                    })
                    .collect(),
            }
        })
        .collect()
}
