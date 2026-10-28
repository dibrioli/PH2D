//! ⭐ **O formato do jogo** ([plano GEF 17](../../../docs/GameEngineFuncional/17_camera_no_canvas_plano.md) D-F):
//! a resolução que o jogo mostra, do PROJECTO. A janela do jogo, a moldura da câmera no canvas e o «Look through»
//! leem-na daqui — uma conta só ([`GameFormat::aspect`]).

/// As predefinições do dono (27/10). ⚠️ A posição é a tag gravada ([`GamePreset::from_u8`]): acrescentar no FIM.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GamePreset {
    #[default]
    Wide16x9,
    Wide16x10,
    Classic4x3,
    Photo3x2,
    Cinema21x9,
    Phone9x16,
    IpadLandscape,
    IpadPortrait,
    GameBoy,
    Nes,
    Snes,
    Gba,
    MegaDrive,
    Pico8,
    Custom,
}

impl GamePreset {
    pub const ALL: [Self; 15] = [
        Self::Wide16x9,
        Self::Wide16x10,
        Self::Classic4x3,
        Self::Photo3x2,
        Self::Cinema21x9,
        Self::Phone9x16,
        Self::IpadLandscape,
        Self::IpadPortrait,
        Self::GameBoy,
        Self::Nes,
        Self::Snes,
        Self::Gba,
        Self::MegaDrive,
        Self::Pico8,
        Self::Custom,
    ];

    /// A resolução da predefinição; `None` = personalizada.
    #[must_use]
    pub const fn resolution(self) -> Option<[u32; 2]> {
        Some(match self {
            Self::Wide16x9 => [1920, 1080],
            Self::Wide16x10 => [1920, 1200],
            Self::Classic4x3 => [1440, 1080],
            Self::Photo3x2 => [1620, 1080],
            Self::Cinema21x9 => [2520, 1080],
            Self::Phone9x16 => [1080, 1920],
            Self::IpadLandscape => [2048, 1536],
            Self::IpadPortrait => [1536, 2048],
            Self::GameBoy => [160, 144],
            Self::Nes => [256, 240],
            Self::Snes => [256, 224],
            Self::Gba => [240, 160],
            Self::MegaDrive => [320, 224],
            Self::Pico8 => [128, 128],
            Self::Custom => return None,
        })
    }

    /// Consola antiga: a janela do jogo amplia-a por um inteiro (píxeis nítidos).
    #[must_use]
    pub const fn is_retro(self) -> bool {
        matches!(
            self,
            Self::GameBoy | Self::Nes | Self::Snes | Self::Gba | Self::MegaDrive | Self::Pico8
        )
    }

    /// A chave i18n do nome na lista.
    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Wide16x9 => "game_format.wide_16_9",
            Self::Wide16x10 => "game_format.wide_16_10",
            Self::Classic4x3 => "game_format.classic_4_3",
            Self::Photo3x2 => "game_format.photo_3_2",
            Self::Cinema21x9 => "game_format.cinema_21_9",
            Self::Phone9x16 => "game_format.phone_9_16",
            Self::IpadLandscape => "game_format.ipad_landscape",
            Self::IpadPortrait => "game_format.ipad_portrait",
            Self::GameBoy => "game_format.game_boy",
            Self::Nes => "game_format.nes",
            Self::Snes => "game_format.snes",
            Self::Gba => "game_format.gba",
            Self::MegaDrive => "game_format.mega_drive",
            Self::Pico8 => "game_format.pico_8",
            Self::Custom => "game_format.custom",
        }
    }

    #[must_use]
    pub fn to_u8(self) -> u8 {
        Self::ALL.iter().position(|p| *p == self).unwrap_or(0) as u8
    }

    /// Um byte desconhecido cai no padrão em vez de recusar o projecto.
    #[must_use]
    pub fn from_u8(b: u8) -> Self {
        Self::ALL.get(usize::from(b)).copied().unwrap_or_default()
    }
}

/// O menor e o maior lado de uma resolução personalizada, em píxeis. O piso é o do PICO-8 dividido por 8
/// (um ladrilho); o tecto é o maior lado de textura que o wgpu garante (`max_texture_dimension_2d` = 8192).
pub const CUSTOM_MIN_PX: u32 = 16;
pub const CUSTOM_MAX_PX: u32 = 8192;

/// O formato do jogo: a predefinição e, para a personalizada, o tamanho escrito.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameFormat {
    pub preset: GamePreset,
    pub custom: [u32; 2],
}

impl Default for GameFormat {
    fn default() -> Self {
        Self {
            preset: GamePreset::Wide16x9,
            custom: [1280, 720],
        }
    }
}

impl GameFormat {
    /// A resolução que o jogo mostra, em píxeis.
    #[must_use]
    pub fn resolution(self) -> [u32; 2] {
        self.preset.resolution().unwrap_or_else(|| {
            self.custom.map(|v| v.clamp(CUSTOM_MIN_PX, CUSTOM_MAX_PX)) // CLAMP-OK: tecto = textura do wgpu
        })
    }

    /// ⭐ **A conta única**: largura ÷ altura do que o jogo mostra.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn aspect(self) -> f32 {
        let [w, h] = self.resolution();
        w as f32 / h.max(1) as f32
    }
}

/// Quando a moldura da câmera se vê no canvas (D-V).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CameraFrameShow {
    /// Sempre — ténue sem a câmera selecionada.
    #[default]
    Always,
    /// Só com a câmera selecionada.
    Selected,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tag_gravada_volta_a_mesma_predefinicao() {
        for p in GamePreset::ALL {
            assert_eq!(GamePreset::from_u8(p.to_u8()), p);
        }
        assert_eq!(GamePreset::from_u8(200), GamePreset::Wide16x9);
    }

    #[test]
    fn o_personalizado_fica_dentro_do_que_a_placa_desenha() {
        let f = GameFormat {
            preset: GamePreset::Custom,
            custom: [0, 99_999],
        };
        assert_eq!(f.resolution(), [CUSTOM_MIN_PX, CUSTOM_MAX_PX]);
    }
}
