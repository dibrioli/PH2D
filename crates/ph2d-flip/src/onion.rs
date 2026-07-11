//! Ghost Frames (onion skin) — configuração por-objeto.
//!
//! Nome de produto: **Ghost Frames** (não "onion skin"). A renderização vem no
//! W3; aqui é só o dado. Defaults canônicos do Grease Pencil 5.2
//! (`DNA_grease_pencil_types.h:409-436`, ver `02_referencia §1`).

use crate::color::Rgba;
use serde::{Deserialize, Serialize};

/// Como os deltas de vizinhança são contados.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum OnionMode {
    /// Δ em número absoluto de frame.
    Absolute,
    /// Δ em número de keyframe (o default do GP).
    #[default]
    Relative,
    /// Só os keyframes selecionados.
    Selected,
}

/// Configuração de Ghost Frames de um objeto. Defaults = GP 5.2.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OnionSettings {
    /// Liga/desliga o desenho dos fantasmas. Default `false` na storage
    /// (objeto recém-criado não tem vizinhos a mostrar); a UI do W3 liga por
    /// padrão ("Ghost Frames on by default").
    pub enabled: bool,
    /// Opacidade global dos fantasmas (`0.5` no GP).
    pub opacity: f32,
    /// Quantos quadros antes mostrar (`1`).
    pub frames_before: u32,
    /// Quantos quadros depois mostrar (`1`).
    pub frames_after: u32,
    /// Tint dos quadros anteriores (verde do GP: `0.145, 0.420, 0.137`). Os
    /// valores são os do GP (display); o espaço de cor exato é resolvido no
    /// render (W3).
    pub color_before: Rgba,
    /// Tint dos quadros seguintes (azul do GP: `0.125, 0.082, 0.529`).
    pub color_after: Rgba,
    /// Esmaecer com a distância (`alpha ∝ 1/|Δ|`) — `USE_FADE` ligado no GP.
    pub fade: bool,
    /// Como contar os deltas.
    pub mode: OnionMode,
}

impl Default for OnionSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            opacity: 0.5,
            frames_before: 1,
            frames_after: 1,
            color_before: Rgba::new(0.145, 0.420, 0.137, 1.0),
            color_after: Rgba::new(0.125, 0.082, 0.529, 1.0),
            fade: true,
            mode: OnionMode::Relative,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_mirror_grease_pencil() {
        let o = OnionSettings::default();
        assert_eq!(o.opacity, 0.5);
        assert_eq!((o.frames_before, o.frames_after), (1, 1));
        assert!(o.fade);
        assert_eq!(o.mode, OnionMode::Relative);
        assert_eq!(o.color_before, Rgba::new(0.145, 0.420, 0.137, 1.0));
    }

    #[test]
    fn round_trips_through_postcard() {
        let o = OnionSettings::default();
        let bytes = postcard::to_allocvec(&o).unwrap();
        let back: OnionSettings = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(back, o);
    }
}
