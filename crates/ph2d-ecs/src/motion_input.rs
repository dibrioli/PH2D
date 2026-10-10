//! ⭐ **O componente `Motion Input`** (ciclo 19, doc 128 §3.6) — um objecto que publica uma entrada
//! para o grafo do Motion (o `value.input` com o mesmo nome a lê).
//!
//! É CONFIG (registado, gravado, com override por instância e variante pelo sistema de
//! componentes). O valor de cada quadro vai para o recurso [`crate::MotionInputs`], pela ponte da
//! `ph2d-app-components`.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

/// **O que o objecto publica.** ⚠️ A ordem é a do seletor do Inspector; acrescentar é no fim.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotionSource {
    /// O número do campo `value`.
    #[default]
    Value,
    /// A cor do campo `color`.
    Color,
    /// A vida do objecto, de `0` (morto) a `1` (cheio) — o objecto precisa de `Health`.
    Health,
    /// Um DISPARO cada vez que a vida desce (um gatilho).
    Damage,
}

impl MotionSource {
    pub const ALL: [Self; 4] = [Self::Value, Self::Color, Self::Health, Self::Damage];

    #[must_use]
    pub fn index(self) -> u8 {
        match self {
            Self::Value => 0,
            Self::Color => 1,
            Self::Health => 2,
            Self::Damage => 3,
        }
    }
}

#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MotionInput {
    /// O nome da entrada (o do `value.input`).
    pub name: String,
    pub source: MotionSource,
    pub value: f32,
    /// RGBA linear.
    pub color: [f32; 4],
}

impl Default for MotionInput {
    fn default() -> Self {
        Self {
            name: "input".to_string(),
            source: MotionSource::Value,
            value: 0.0,
            color: [1.0; 4],
        }
    }
}
