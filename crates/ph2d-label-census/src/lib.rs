//! **O CENSO DOS RÓTULOS** — que texto de interface chega ao ecrã sem passar pela tabela de strings
//! (HR-15), e se as chaves da tabela existem dos dois lados.
//!
//! # Por que uma crate, e por que uma FOLHA
//!
//! O gate molde (`ph2d-editor-core/tests/it/no_label_of_this_crate_is_written_in_the_painter.rs`)
//! dizia *«cada linha dona de uma crate copia-o com a lista dela»*. Copiar ~700 linhas de leitor é
//! fazer N réguas da mesma grandeza, na mesma linguagem — e a segunda cópia é a que diverge. O
//! helper [`cfg_test`] já tinha pago essa lei dentro da própria `editor-core` (a `hr15` e a `hr12`
//! faziam a mesma pergunta de duas maneiras), e por isso mora aqui também.
//!
//! Zero dependências: um gate que use isto não recompila quando o produto muda.
//!
//! # ⛔⛔ A régua anterior via 44 % de um painel
//!
//! Medido em 2026-09-13, sobre a mesma árvore:
//!
//! | população | régua anterior (ponto fixo de pintores) | esta régua (lexical) |
//! |---|---:|---:|
//! | `ph2d-panel-painter-layers` | 166 | 378 |
//! | `ph2d-editor-core` | 32 | 593 |
//! | as crates de UI inteiras (painéis, `app-*`, shell, `editor-core`) | 445 | 5 533 |
//!
//! A régua anterior seguia o texto até um PINTOR (`paint_*`/`draw_*` com parâmetro `&str`). O texto
//! de um painel entra sobretudo por **construtores de widget** (`Button::new(id, "Apply")`), por
//! **tabelas** (`["Paint", "Erase"]`, `(id, "Reset")`) e por **`format!`** — as três formas que ela
//! declarava não ver, e que são a MAIORIA. *Declarar um ponto cego não o torna pequeno.*
//!
//! # A régua desta crate é LEXICAL, e erra para o lado ALTO
//!
//! Todo literal com cara de língua ([`is_language`]), fora de código de teste, fora dos contextos
//! que nunca pintam (mensagens de `panic!`/`expect`/log, padrões de `match`, comparações,
//! atributos, chaves). ⭐ **Um falso positivo é um nome numa lista**, lido e declarado com o
//! mecanismo; um falso negativo seria um rótulo que ninguém vê. *O erro ALTO é o barato.*
//!
//! # O que ela NÃO vê, declarado
//!
//! Texto que chega de fora do fonte (ficheiros, dados do documento) e texto montado só de pedaços
//! sem palavra (`format!("{a}{b}")` com `a` e `b` vindos de outro sítio).

pub mod cfg_test;
pub mod keys;
mod lexical;
mod source;

pub use lexical::{Literal, is_language, language_literals, language_literals_in};
