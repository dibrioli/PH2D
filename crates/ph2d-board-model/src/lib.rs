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
mod history;
mod legacy;
mod ops;

pub use board::{Board, BoardId, BoardSet, Camera, FORMAT_VERSION};
pub use camera::{Area, ZOOM_RANGE};
pub use element::{
    BoardDoc, Dash, Element, ElementId, ElementKind, Rgba, Shape, ShapeType, Style, rotate_about,
};
pub use frac::FracKey;
pub use history::{History, MAX_STEPS};
pub use ops::{BoardOp, apply_batch};

/// Espessura do contorno de uma forma nova, em unidades do mundo — a do Excalidraw (medida no
/// oráculo: `strokeWidth: 2`).
pub const DEFAULT_STROKE_WIDTH: f64 = 2.0;
/// Tamanho do texto de uma forma nova, em unidades do mundo — o «M» do Excalidraw (oráculo:
/// `fontSize: 20`).
pub const DEFAULT_FONT_SIZE: f64 = 20.0;
/// Tinta do texto quando o documento não diz outra (dado do documento, não da UI).
pub const DEFAULT_INK: [u8; 4] = [30, 30, 30, 255];
/// A tinta clara (texto sobre preenchimentos escuros).
pub const DEFAULT_PAPER: [u8; 4] = [250, 250, 250, 255];
