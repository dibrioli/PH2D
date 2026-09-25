//! ⭐⭐⭐ **A BARRA DE VIDA** (plano 28, W4) — o componente de CONFIG.
//!
//! Uma entidade com [`HealthBar`] desenha, por cima de si, a vida de alguém: a dela própria (o
//! inimigo com a barra sobre a cabeça) ou a de um objecto com NOME (a barra do herói no placar, filha
//! do `UiCanvas`). A lei do rasto vive na folha `ph2d_hud::barra`; quem a corre e desenha é a ponte
//! `ph2d_app_components::health_bar_bridge`.
//!
//! # ⚠️ Os três CAMINHOS de uma barra, e porque é um componente só
//!
//! Sobre a cabeça e no placar são a MESMA coisa desenhada em sítios diferentes, e o sítio é o
//! `Transform` da entidade que a carrega: um filho do inimigo anda com ele, um filho do canvas anda
//! com a vista. ⇒ um componente, e o [`HealthBar::target`] diz DE QUEM é a vida — com a regra da
//! tabela de acções: **vazio = este objecto**.
//!
//! # ⚠️ O que ela NÃO guarda
//!
//! O rasto muda por quadro ⇒ vive na ponte, fora do mundo (a lei *«rebobinar é renascer»*, como os
//! emissores de partículas). ⛔ Aqui faria o undo ver cada quadro como um passo.
//!
//! # ⛔ A barra NÃO roda
//!
//! Ela segue a POSIÇÃO e a ESCALA de quem a carrega e fica sempre na horizontal: uma barra que rodasse
//! com um inimigo a cair deixaria de se ler. A escala é a média geométrica (`√|det|`), a mesma lei
//! que o traço vectorial desta casa usa para não virar caneta elíptica.

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

/// **Uma barra de vida.** Ver o cabeçalho do módulo.
///
/// ⚠️ **Os valores de fábrica são de PRODUTO, não limites:** uma barra de `1,0 × 0,14` meio metro
/// acima do centro cabe sobre o quadrado de `1 m` que toda cena desta casa usa; o rasto segura
/// `0,4 s` e escorre uma barra inteira por segundo. Nenhum deles é um tecto — o painel aceita
/// qualquer número finito.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HealthBar {
    /// O NOME do objecto cuja vida ela mostra. **Vazio = este objecto.**
    ///
    /// ⚠️ **O nome, nunca bits** — a referência durável desta casa (o undo respawna tudo com bits
    /// novos).
    pub target: String,
    /// A largura, em metros do mundo (antes da escala de quem a carrega).
    pub width: f32,
    /// A altura.
    pub height: f32,
    /// O deslocamento do CENTRO da barra em relação à pose de quem a carrega — `x`.
    pub offset_x: f32,
    /// E `y` (de fábrica, acima).
    pub offset_y: f32,
    /// A cor da vida.
    pub fill: [f32; 4],
    /// A cor do rasto — o pedaço que acabou de se perder.
    pub trail: [f32; 4],
    /// A cor do fundo — a vida inteira.
    pub back: [f32; 4],
    /// Quanto o rasto SEGURA depois de um golpe, em segundos.
    pub trail_delay_s: f32,
    /// Quantas barras inteiras o rasto escorre por segundo.
    pub trail_speed: f32,
    /// Esconde a barra enquanto a vida está CHEIA — dez inimigos intactos não enchem a tela de
    /// barras iguais.
    pub hide_when_full: bool,
}

impl Default for HealthBar {
    fn default() -> Self {
        Self {
            target: String::new(),
            width: 1.0,
            height: 0.14,
            offset_x: 0.0,
            offset_y: 0.75,
            fill: [0.36, 0.84, 0.42, 1.0],
            trail: [1.0, 1.0, 1.0, 1.0],
            back: [0.08, 0.08, 0.1, 0.8],
            trail_delay_s: 0.4,
            trail_speed: 1.0,
            hide_when_full: false,
        }
    }
}
