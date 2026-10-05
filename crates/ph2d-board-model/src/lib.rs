//! **O documento do Quadro** (MiroClone) — os quadros de um projecto e o que há dentro deles.
//!
//! Um quadro é um DOCUMENTO numa aba da barra superior, nunca um objecto da cena (ordem do dono,
//! 2026-10-05; plano em `docs/MiroClone/02_plano.md` §1). Por isso este crate não conhece ECS,
//! render nem UI: o editor muda-o só por [`BoardOp`] e grava-o só por [`BoardSet::to_bytes`].
//!
//! Cada [`Element`] já nasce com o que a colaboração ao vivo (Etapa 2) precisa — `version`,
//! `nonce`, lápide `deleted` e ordem por [`FracKey`] —, para essa etapa não pedir migração.

mod board;
mod camera;
mod element;
mod frac;
mod ops;

pub use board::{Board, BoardId, BoardSet, Camera, FORMAT_VERSION};
pub use camera::{Area, ZOOM_RANGE};
pub use element::{BoardDoc, Element, ElementId, ElementKind, Rgba};
pub use frac::FracKey;
pub use ops::{BoardOp, apply_batch};
