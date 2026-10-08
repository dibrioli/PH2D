//! Os três componentes da luz 2D (plano §2.3).

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

/// **Uma fonte de luz** — o objeto do menu Add.
///
/// ⭐ `intensidade` é a **fluência a 1 m da luz, no vazio** (para `Ponto` pequeno; no limite
/// `tamanho_m → 0` a fluência é exatamente `intensidade / r`). Por isso a radiância do disco é
/// `cor·intensidade / (2·tamanho_m)`: crescer a lâmpada amacia a sombra **sem** mudar o brilho.
/// Para `Ambiente`, `intensidade` é a fluência num ponto desobstruído (o céu inteiro).
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Luz {
    /// Cor LINEAR.
    pub cor: [f32; 3],
    pub intensidade: f32,
    /// Raio da FONTE, em metros do mundo (não escala com o `Transform`). Decide a penumbra.
    pub tamanho_m: f32,
    /// Alcance da queda autoral (`queda`, janela de Karis); `f32::INFINITY` = física pura.
    pub alcance_m: f32,
    pub tipo: TipoDeLuz,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TipoDeLuz {
    /// Disco de raio `tamanho_m` na posição do objeto (ponto exato se `tamanho_m == 0`).
    #[default]
    Ponto,
    /// Radiância que chega do infinito por toda direção desobstruída. Posição, tamanho e
    /// alcance não se aplicam.
    Ambiente,
}

/// **Este objeto bloqueia a luz.** A forma é a do objeto (o quad do sprite, na coleta da W1).
///
/// `opacidade` é a fração bloqueada **por entrada** do raio no objeto: atravessá-lo uma vez
/// multiplica a luz por `1 − opacidade`, seja qual for a espessura.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BloqueiaLuz {
    pub opacidade: f32,
}

impl Default for BloqueiaLuz {
    fn default() -> Self {
        Self { opacidade: 1.0 }
    }
}

/// **Opt-out de receber luz** (HUD, fundo pintado com luz própria). Por omissão TUDO recebe.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NaoRecebeLuz;
