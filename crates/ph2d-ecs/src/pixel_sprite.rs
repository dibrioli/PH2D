//! `PixelSprite` — a entidade é um objecto **Pixel Sprite** (ADR-0192): o marcador do tipo
//! `ObjectKind::PixelSprite` e a identidade estável do documento de pixels dela.
//!
//! O documento (camadas, frames, cels, paleta) vive na família `ph2d-app-pixel`, chaveado por este
//! `u32`, e grava-se como blob com versão própria no projecto; esta componente só liga a entidade a
//! ele. Viaja no `WorldSnapshot`, logo o undo do mundo e o save preservam a ligação.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// A chave do documento de pixels desta entidade (a família aloca a próxima livre).
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelSprite(pub u32);

impl SimComponent for PixelSprite {}
