//! **A FORMA ESPACIAL do peso da força do pincel de tecido** (espec §4.4 e §8.1).
//!
//! ⚠️⚠️ **Ele existia na lei e não existia no painel.** A `ph2d-cloth` responde
//! às duas formas desde que a lei da referência nasceu, o corpus do oráculo tem
//! **quatro** traços que só a de PLANO exercita (`plano_agarrar_plano_local`,
//! `plano_arrastar_plano_local`, `plano_empurrar_plano_local`,
//! `plano_apertar_ponto_plano_local`) — e a tradução `Brush → Pincel` escrevia
//! `Radial` **literal**. *Uma lei medida na bancada e não ligada no produto é uma
//! lei que o artista não tem.*

use ph2d_cloth::verlet_gesto::FalloffForca;

/// As duas formas de queda da força (espec §8.1, omissão `Radial`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClothForceFalloff {
    /// A distância é ao **ponto** — uma esfera à volta do centro da queda.
    #[default]
    Radial,
    /// A distância é ao **PLANO** que passa pelo centro da área com a normal do
    /// movimento: o peso cai ao longo do traço e não a partir dele.
    Plane,
}

impl ClothForceFalloff {
    /// As duas, na ordem do painel do alvo (espec §8.4).
    pub const ALL: [Self; 2] = [Self::Radial, Self::Plane];

    /// O rótulo do chip (a UI da casa é inglesa).
    #[must_use]
    pub fn label(self) -> &'static str {
        ph2d_i18n::tr_em(ph2d_i18n::Idioma::Ingles, self.label_key())
    }

    /// ⭐⭐ **A CHAVE do rótulo** — `sculpt3d.cloth_force_falloff.<variante>`, e é ela que a interface passa ao
    /// [`ph2d_i18n::tr`]. O texto vive na tabela (`ph2d-i18n/src/sculpt_engine.rs`), com a
    /// [`label`](Self::label) acima a lê-lo em inglês: *uma lei só, com um acessório derivado.*
    #[must_use]
    pub fn label_key(self) -> &'static str {
        match self {
            Self::Radial => "sculpt3d.cloth_force_falloff.radial",
            Self::Plane => "sculpt3d.cloth_force_falloff.plane",
        }
    }

    /// A lei correspondente na `ph2d-cloth`.
    #[must_use]
    pub fn falloff(self) -> FalloffForca {
        match self {
            Self::Radial => FalloffForca::Radial,
            Self::Plane => FalloffForca::Plano,
        }
    }
}
