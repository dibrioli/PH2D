//! **É este ficheiro um módulo de TESTE inteiro?** — a pergunta feita ao PAI, que é quem o compila.
//!
//! ⚠️ **A resposta mudou-se para [`ph2d_label_census::cfg_test`] (2026-09-13), e o motivo é o que
//! este ficheiro já escrevia:** *«ela vive AQUI porque tem dois donos, e a segunda cópia é a que
//! diverge»*. A `hr15` e a `hr12` eram os dois donos; a régua lexical do texto pintado e os gates por
//! crate dos painéis são o terceiro e o quarto, e nenhum deles vê `tests/common/` desta crate. O
//! código foi movido byte a byte — este ficheiro só lhe dá o nome antigo, para os três leitores de
//! `crate::cfg_test_modules` não mudarem.
//!
//! Não é um alvo de teste: vive em `tests/common/`, que o cargo não compila como binário.

#![allow(dead_code)]

pub use ph2d_label_census::cfg_test::is_declared_under_cfg_test;
