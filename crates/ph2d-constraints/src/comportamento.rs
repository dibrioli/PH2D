//! ***Behaviours* de um clique** (W5.4, [`docs/BlenderConstraints/06_W5_resto.md`](../../../docs/BlenderConstraints/06_W5_resto.md)
//! §3.2): Tremer, Flutuar, Orbitar e Atrair, com o MESMO movimento do nó do Motion de igual nome, e o
//! peso por distância a outro objecto (a lei do `motion.falloff`). Aqui só os números e o perfil de
//! fábrica — esta crate é folha; quem chama as leis dos nós é a fase (`ph2d-constraints-ecs`).

use serde::{Deserialize, Serialize};

/// O *behaviour* e o seu perfil de fábrica — a posição é a tag do submenu do Add Constraint.
/// ⚠️ **APPEND-ONLY** (o molde do `ph2d_shake::Perfil`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Perfil {
    /// `motion.wiggle`: ruído fractal, os dois eixos em linhas diferentes.
    #[default]
    Tremer,
    /// `motion.oscillator`: a onda (senóide) num canal.
    Flutuar,
    /// `motion.orbit`: gira em volta do alvo (sem alvo, de um pivô a `raio` à esquerda).
    Orbitar,
    /// `force.attractor` integrado quadro a quadro — o único com estado.
    Atrair,
}

impl Perfil {
    pub const ALL: [Self; 4] = [Self::Tremer, Self::Flutuar, Self::Orbitar, Self::Atrair];

    /// A chave i18n do nome.
    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Tremer => "panel.inspector.constraint.behaviour.shake",
            Self::Flutuar => "panel.inspector.constraint.behaviour.float",
            Self::Orbitar => "panel.inspector.constraint.behaviour.orbit",
            Self::Atrair => "panel.inspector.constraint.behaviour.attract",
        }
    }

    /// A posição em [`Self::ALL`].
    #[must_use]
    pub const fn tag(self) -> u8 {
        self as u8
    }

    /// O perfil desta posição, ou o de fábrica.
    #[must_use]
    pub fn from_tag(tag: u8) -> Self {
        Self::ALL.get(usize::from(tag)).copied().unwrap_or_default()
    }

    /// ⭐ **O clique:** os números de fábrica deste perfil (unidades do mundo — metros, como o Motion —,
    /// graus, segundos).
    #[must_use]
    pub const fn numeros(self) -> Comportamento {
        let base = Comportamento {
            perfil: self,
            onde: Onde::XY,
            amplitude: 0.1,
            frequencia: 3.0,
            semente: 0.0,
            velocidade: 90.0,
            raio: 1.5,
            forca: 40.0,
            amortece: 0.1,
            leva_rotacao: false,
            peso: None,
        };
        match self {
            Self::Tremer => base,
            Self::Flutuar => Comportamento {
                onde: Onde::Y,
                amplitude: 0.25,
                frequencia: 0.5,
                ..base
            },
            Self::Orbitar => base,
            // O atrito máximo da lei do `sim.step`: puxa, passa do alvo uma vez e assenta. O alcance é
            // largo: fora dele a força é ZERO e um objecto atirado para lá fica perdido (medido na cena
            // 10 em laço: a limalha saiu do raio de 6 m e congelou a 9,6 m do íman).
            Self::Atrair => Comportamento {
                raio: 30.0,
                amortece: 0.0,
                ..base
            },
        }
    }

    /// EXIGE alvo? O Atrair sim; o Orbitar mostra-o mas não o exige (sem ele, o pivô).
    #[must_use]
    pub const fn mostra_alvo(self) -> bool {
        matches!(self, Self::Orbitar | Self::Atrair)
    }

    /// Precisa de alvo? O Atrair sim; o Orbitar usa-o se houver (senão o pivô); os outros não.
    #[must_use]
    pub const fn precisa_de_alvo(self) -> bool {
        matches!(self, Self::Atrair)
    }
}

/// Onde o Tremer e o Flutuar mexem.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Onde {
    /// Os dois eixos (o Flutuar lê-o como Y — uma onda nos dois eixos é uma diagonal).
    #[default]
    XY,
    X,
    Y,
    /// A rotação; a amplitude em graus.
    Rot,
}

/// O peso por distância a outro objecto (a lei do `motion.falloff`, disco): `1` em cima dele, `0` a
/// `raio`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Peso {
    /// O `StableId` do objecto (esta crate não vê o ECS).
    pub objecto: u64,
    pub raio: f64,
    /// `0` Linear · `1` Quad · `2` Smooth · `3` Smoother (as do nó).
    pub curva: u8,
    /// `1 − peso`: forte longe, fraco perto.
    pub inverte: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Comportamento {
    pub perfil: Perfil,
    pub onde: Onde,
    /// Metros (ou graus em [`Onde::Rot`]) — Tremer e Flutuar.
    pub amplitude: f64,
    /// Hz — Tremer e Flutuar.
    pub frequencia: f64,
    /// A linha do ruído (Tremer) ou a fase em ciclos (Flutuar).
    pub semente: f64,
    /// Graus por segundo — Orbitar.
    pub velocidade: f64,
    /// Orbitar sem alvo: a distância ao pivô. Atrair: o alcance da força.
    pub raio: f64,
    /// Atrair: m/s² no centro.
    pub forca: f64,
    /// Atrair: a velocidade que fica ao fim de um segundo (`1` = sem atrito, a do `sim.step`).
    pub amortece: f64,
    /// Orbitar: o objecto vira com a volta.
    pub leva_rotacao: bool,
    pub peso: Option<Peso>,
}

impl Default for Comportamento {
    fn default() -> Self {
        Perfil::default().numeros()
    }
}
