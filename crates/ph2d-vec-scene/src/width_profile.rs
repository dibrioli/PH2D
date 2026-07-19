//! **O perfil de largura** de um traço — o que separa um desenho de um diagrama.
//!
//! Hoje um traço tem uma largura só. Um artista quer que ela VARIE ao longo do caminho: a
//! linha que afina na ponta, o traço de nanquim que engrossa na curva, a caligrafia. É o
//! *Power Stroke* do Inkscape e o *Width Tool* do Illustrator.
//!
//! # Três valores, não uma lista de alças — e é uma escolha, não uma limitação de hoje
//!
//! O Inkscape guarda pares `(posição, largura)` arbitrários e dá alças arrastáveis na linha.
//! O Illustrator faz o mesmo **e** vende *perfis* salvos — e são os perfis que o artista usa
//! em 90% dos casos: afina-no-fim, afina-nos-dois, engrossa-no-meio. Um perfil desses é
//! exatamente `(início, meio, fim)` mais **onde o meio senta**.
//!
//! Começar pelos perfis dá a capacidade inteira com quatro números, que é o que a tabela de
//! parâmetros de um efeito ou de um comando já sabe desenhar. As alças na linha são um GESTO
//! de canvas (um modo próprio, hit-test, undo por-alça) — outra wave, e que consome este
//! perfil em vez de o substituir.
//!
//! # A interpolação é SUAVE, e sem transcendental
//!
//! Ligar os três com retas deixa um vinco no meio: a largura tem derivada descontínua e a
//! silhueta ganha uma quina que ninguém desenhou. O `smoothstep` (`u²(3−2u)`) chega em cada
//! ponto de controle com derivada zero, então os três trechos se encontram lisos — e é
//! polinomial, o que mantém o HR-5 (nada de `sin`/`exp` em geometria de documento).

use serde::{Deserialize, Serialize};

/// Largura mínima que ainda é tinta. Abaixo disto o traço não tem o que desenhar.
pub const MIN_WIDTH_FACTOR: f64 = 1e-9;

/// O perfil, como MULTIPLICADORES da largura do traço.
///
/// Multiplicadores e não medidas absolutas: o artista já escolheu a largura no slider de
/// Width, e o perfil diz o que acontece com ELA ao longo do caminho. `1.0` em toda parte é o
/// traço uniforme de sempre — o que torna [`Self::UNIFORM`] um ponto neutro de verdade, e não
/// um valor que por acaso não faz nada.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WidthProfile {
    /// Multiplicador no começo do caminho.
    pub start: f64,
    /// Multiplicador no ponto de controle do meio.
    pub mid: f64,
    /// Multiplicador no fim.
    pub end: f64,
    /// Onde o meio senta, em fração do comprimento de ARCO (`0.5` = na metade).
    ///
    /// Por arco e não por segmento: é a mesma lei que o Zig Zag e o Blend já seguem — duas
    /// formas que se veem iguais têm de se comportar igual, e picar uma aresta em vinte
    /// pedaços não pode mover o ponto mais grosso do traço.
    pub position: f64,
}

impl Default for WidthProfile {
    fn default() -> Self {
        Self::UNIFORM
    }
}

impl WidthProfile {
    /// O traço uniforme — a largura que o `StrokeSpec` já diz, do começo ao fim.
    pub const UNIFORM: Self = Self {
        start: 1.0,
        mid: 1.0,
        end: 1.0,
        position: 0.5,
    };

    /// O multiplicador em `t`, a fração de ARCO percorrida (`0` = começo, `1` = fim).
    ///
    /// Fora de `[0,1]` o valor é clampado nas pontas — um chamador que amostre ligeiramente
    /// além da borda por erro de arredondamento recebe a ponta, não uma extrapolação.
    #[must_use]
    pub fn at(&self, t: f64) -> f64 {
        // ⚠️ Sem `t.clamp` aqui, de propósito: o [`smoothstep`] já clampa o `u` que ele
        // recebe, e um `t` fora do domínio cai no ramo cuja ponta é a resposta certa de
        // qualquer forma. Um clamp a mais seria uma SEGUNDA defesa da mesma propriedade — e
        // duas defesas com um gate só significam que mutar uma delas não sangra, que foi
        // exatamente o que aconteceu. [[feedback_layered_defenses_need_per_layer_gates]]
        let p = self.position.clamp(0.0, 1.0);
        // Nas bordas degeneradas (meio colado numa ponta) o trecho vazio não existe: a
        // resposta é o outro trecho inteiro, sem divisão por zero.
        let (a, b, u) = if t <= p {
            if p <= 0.0 {
                return self.mid;
            }
            (self.start, self.mid, t / p)
        } else {
            if p >= 1.0 {
                return self.mid;
            }
            (self.mid, self.end, (t - p) / (1.0 - p))
        };
        a + (b - a) * smoothstep(u)
    }

    /// Este perfil é o traço uniforme? Então quem o consome pode tomar o caminho barato — e,
    /// mais importante, um comando pode recusar-se a existir em vez de produzir a mesma coisa
    /// com outro nome.
    #[must_use]
    pub fn is_uniform(&self) -> bool {
        (self.start - 1.0).abs() < MIN_WIDTH_FACTOR
            && (self.mid - 1.0).abs() < MIN_WIDTH_FACTOR
            && (self.end - 1.0).abs() < MIN_WIDTH_FACTOR
    }

    /// O maior multiplicador do perfil — quanto o traço pode engrossar no pior ponto.
    #[must_use]
    pub fn peak(&self) -> f64 {
        self.start.max(self.mid).max(self.end)
    }
}

/// `u²(3−2u)` em `[0,1]`: sobe de 0 a 1 chegando com derivada ZERO nas duas pontas.
#[must_use]
fn smoothstep(u: f64) -> f64 {
    let u = u.clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

#[cfg(test)]
#[path = "width_profile_tests.rs"]
mod tests;
