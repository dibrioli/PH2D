//! ⭐ As 32 cores das camadas de colisão (GEF02 onda 9c): o índice pelo número da camada. Os valores e a medida
//! da distinção: `docs/design/tokens.json` (`obj-layer-*`) e o gate `as_cores_das_camadas_sao_unicas_e_legiveis`.

use crate::color::ColorToken;

/// ⭐ O contorno da camada num quadro DESLIGADO (não colide; onda 9f): a mesma cor a esta opacidade — a cor continua
/// a dizer qual camada é, o estado lê-se pela caixa de marcar. O teste dos contornos mede os dois estados.
pub const LAYER_OUTLINE_OFF_ALPHA: f32 = 0.4; // LITERAL-PX-OK: opacidade do contorno desligado

impl ColorToken {
    /// As 32 cores das camadas de colisão, pelo número da camada.
    pub const LAYERS: [Self; 32] = [
        Self::ObjLayer0,
        Self::ObjLayer1,
        Self::ObjLayer2,
        Self::ObjLayer3,
        Self::ObjLayer4,
        Self::ObjLayer5,
        Self::ObjLayer6,
        Self::ObjLayer7,
        Self::ObjLayer8,
        Self::ObjLayer9,
        Self::ObjLayer10,
        Self::ObjLayer11,
        Self::ObjLayer12,
        Self::ObjLayer13,
        Self::ObjLayer14,
        Self::ObjLayer15,
        Self::ObjLayer16,
        Self::ObjLayer17,
        Self::ObjLayer18,
        Self::ObjLayer19,
        Self::ObjLayer20,
        Self::ObjLayer21,
        Self::ObjLayer22,
        Self::ObjLayer23,
        Self::ObjLayer24,
        Self::ObjLayer25,
        Self::ObjLayer26,
        Self::ObjLayer27,
        Self::ObjLayer28,
        Self::ObjLayer29,
        Self::ObjLayer30,
        Self::ObjLayer31,
    ];

    /// A cor da camada de colisão `n` (as 32 do motor; dá a volta por segurança).
    #[must_use]
    pub const fn layer(n: usize) -> Self {
        Self::LAYERS[n % 32]
    }
}
