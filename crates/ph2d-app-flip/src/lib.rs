//! ⭐ **A metade de APLICAÇÃO do módulo Flip** (W2/L5, 2026-09-11).
//!
//! # O que está aqui
//!
//! As leis do editor de Flip que **não precisam da shell para existir**: a reamostragem e o
//! ajuste do traço autorado ([`smooth`]), a dilatação que fecha o vão do balde
//! ([`fill_dilate`]), a resolução de alvo da tira de quadros ([`strip_resolve`]), a política de
//! teclado do *peek* ([`peek`]), a composição multi-quadro ([`multiframe`]) e a cena de
//! demonstração ([`demo`]).
//!
//! # ⛔ A regra que faz isto funcionar
//!
//! **Nada aqui depende da `shells/desktop`.** Ela é um binário — uma dependência de volta seria
//! um ciclo — e é por isso que o corte é acíclico por construção: a shell chama aqui, e o que
//! precisa de `App` (janela, `AppGfx`, painéis, captura de undo) fica lá, nomeado.
//!
//! # ⚠️ O documento não é daqui
//!
//! [`ph2d_flip::FlipDoc`] é **partilhado** — a F8 dos Componentes fechou-o em 10/09 — e vive na
//! crate de módulo. Ele é o que se grava e o que o undo fotografa. O que sai da shell para aqui é
//! a metade de *aplicação*, nunca o modelo.

pub mod demo;
pub mod fill_dilate;
pub mod multiframe;
pub mod peek;
pub mod smooth;
pub mod strip_resolve;
