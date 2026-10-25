//! **A família Pixel Art** (ADR-0192, `docs/PixelArt/02_plano_modulo.md`).
//!
//! O objecto **Pixel Sprite** nasce no menu Add (o marcador [`ph2d_ecs::PixelSprite`] guarda a chave
//! do documento) e edita-se no modo Edit, no viewport, com as ferramentas e os atalhos do Aseprite:
//! `B` lápis, `E` borracha, `I` conta-gotas (`Alt` segurado = conta-gotas), `X` troca frente/fundo,
//! `Ctrl+Z`/`Ctrl+Shift+Z` no desfazer PRÓPRIO do sprite, a roda em zoom inteiro.
//!
//! O documento ([`ph2d_pixel_core::Sprite`]) vive em [`PixelApp`], fora do mundo e do undo do mundo,
//! e grava-se como blob com versão própria no projecto ([`persist`]). A shell só compõe: chama
//! [`frame`](PixelApp::frame) uma vez por quadro, encaminha o ponteiro/roda/teclado ([`input`]) e
//! pede o desenho ([`draw`]).

pub mod draw;
mod frame;
pub mod input;
pub mod mode;
pub mod object_add;
pub mod panel;
pub mod persist;
pub mod smoke;
mod state;

pub use state::{Doc, PixelApp, Tool};

/// A família declarada à shell.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "pixel",
    routers: smoke::ROUTERS,
};
