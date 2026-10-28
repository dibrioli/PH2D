//! **Os «Power-Ups» do PH2D** (Kanban W11) — o que o Trello não pode ter: o cartão que abre o
//! objecto. Duas leis puras:
//!
//! - [`resolve`]: uma [`ProjectLink`] contra o [`Catalog`] do projecto (o que existe AGORA e o nome
//!   de cada coisa) dá o nome actual, o tipo e se está viva; morta, mostra o nome de reserva
//!   («quebrada»), nunca some. [`refresh`] grava o nome actual como reserva.
//! - [`notes`]: nota adesiva ↔ cartão (título = 1.ª linha, o resto em markdown, cor ↔ etiqueta).
//!
//! O canal «abrir/seleccionar» vive no `ph2d-editor-core` (`kanban_links`), que publica o catálogo.

mod catalog;
pub mod notes;

pub use catalog::{BoardEntry, Catalog, Kind, Resolved, kind_of, link_op, refresh, resolve};
pub use ph2d_kanban_model::{ProjectLink, Target};
