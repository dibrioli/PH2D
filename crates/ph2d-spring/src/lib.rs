//! **A MOLA** — a alternativa ao par *duração + curva*, e o que ela de facto compra.
//!
//! # O que uma mola dá que uma curva não dá
//!
//! Não é a FORMA: `Elastic Out` mede pico **1,373** / assenta em **0,631** / **4** travessias
//! contra **1,309 / 0,600 / 3** de um oscilador real (`tests/measure_spring.rs`) — a mesma
//! animação. O que ela dá é **continuidade de VELOCIDADE sob interrupção**.
//!
//! Revertendo a 30% do caminho, a volta arranca a **1,34×** a velocidade com que a ida chegava sob
//! o `ph2d_ui_state::DEFAULT_EASING` — o olho não separa isso de 1,00×. Mas o seletor de curva existe
//! desde a W7c, e há dois regimes a um clique onde ele morde: **`Cubic InOut` a 0,00×** (a cena
//! **para e recomeça**) e **`Elastic Out` a 7,02×** (estalo). Uma mola não tem esse degrau: ela
//! carrega a velocidade que tinha para dentro do movimento novo.
//!
//! # ⚠️ Ela é uma OPÇÃO, e o sistema de easing fica INTACTO
//!
//! Nada aqui toca `ph2d_ui_state::Transition` nem o catálogo de curvas. Um hospedeiro sem mola é
//! **byte-idêntico** ao que já shipava — a `Option` ausente nem sequer entra no caminho.
//!
//! ⚠️ **E ela não tem duração nem curva**, o que é a razão de o painel trocar as duas linhas em
//! vez de as somar: *rigidez* e *amortecimento* respondem a mesma pergunta que *duração* e
//! *curva*, e oferecer as quatro seria pedir ao artista que mantivesse dois modelos de acordo.
//!
//! # O integrador é o do repo, não um segundo
//!
//! Euler semi-implícito — o mesmo do Motion e o mesmo que a sonda `measure_spring` usa para o
//! oscilador de referência. Um integrador próprio aqui daria números que a medição não prevê.

use serde::{Deserialize, Serialize};

/// Rigidez por omissão (`ω`, rad/s).
///
/// ⚠️ **Não é número de recurso** (§0): nada fica mais barato por ele ser pequeno. Ele descreve
/// PERCEPÇÃO, e é o `ω` que a sonda `measure_spring` usa para reproduzir o `Elastic Out` do
/// catálogo — o ponto em que a indústria converge para um controle de UI.
pub const DEFAULT_STIFFNESS: f64 = 12.0;

/// Amortecimento por omissão (`ζ`, adimensional).
///
/// ⚠️ **`1.0` é o CRÍTICO — chega ao alvo sem passar.** É o default porque o overshoot da pose
/// afim é **clampado** hoje (ver `ph2d_ui_state::Transition::at`), então uma mola sub-amortecida por
/// omissão pareceria *parar* no alvo antes de assentar. Quem quiser o salto baixa o valor e vê o
/// que o sistema de easing hoje mostra para `Back`/`Elastic` — que é a mesma coisa, pelo mesmo
/// clamp.
pub const DEFAULT_DAMPING: f64 = 1.0;

/// Os tetos que os SLIDERS oferecem — do mesmo tipo dos de duração: o que se esgota é a
/// legibilidade do controle, não um recurso.
pub const MIN_STIFFNESS: f64 = 1.0;
pub const MAX_STIFFNESS: f64 = 60.0;
pub const MIN_DAMPING: f64 = 0.1;
pub const MAX_DAMPING: f64 = 2.0;

/// Quão perto do alvo, e quão devagar, a mola tem de estar para se dar por chegada.
///
/// ⚠️ Uma mola **não termina sozinha** — ela converge assintoticamente. Sem um critério ela
/// animaria para sempre, e a máquina nunca chamaria `arrive`, que é quem põe a pose EXATA (a lei
/// que impede a cena de derivar a cada hover).
const SETTLE_X: f64 = 1.0e-3;
const SETTLE_V: f64 = 1.0e-3;

/// Um passo de integração fixo, em segundos.
///
/// ⚠️ **Fixo, e não o `dt` do quadro**: um integrador cujo passo é a taxa de quadros dá
/// trajetórias diferentes em máquinas diferentes — o mesmo motivo pelo qual o solver do Wet Paint
/// tem passo próprio. O `advance` consome o `dt` real em fatias deste tamanho.
const STEP: f64 = 1.0 / 240.0;

/// Rigidez e amortecimento de um hospedeiro.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spring {
    /// `ω` — quão forte ela puxa.
    pub stiffness: f64,
    /// `ζ` — `< 1` passa do alvo e volta, `= 1` chega sem passar, `> 1` chega devagar.
    pub damping: f64,
}

impl Default for Spring {
    fn default() -> Self {
        Self {
            stiffness: DEFAULT_STIFFNESS,
            damping: DEFAULT_DAMPING,
        }
    }
}

impl Spring {
    /// Os valores dentro da faixa que os sliders oferecem.
    #[must_use]
    pub fn clamped(self) -> Self {
        Self {
            stiffness: self.stiffness.clamp(MIN_STIFFNESS, MAX_STIFFNESS),
            damping: self.damping.clamp(MIN_DAMPING, MAX_DAMPING),
        }
    }
}

/// **O estado VIVO de uma mola em voo** — onde ela está no caminho, e com que velocidade.
///
/// ⚠️ **A velocidade é o campo inteiro da feature.** Uma reversão constrói um caminho novo e
/// **herda `v`**, que é exatamente o que uma curva não sabe fazer: ela recomeça em `t = 0`, e num
/// `InOut` isso é velocidade zero — a cena para e arranca de novo.
#[derive(Clone, Copy, Debug)]
pub struct SpringState {
    /// Posição no caminho: `0` é a origem, `1` o destino. Pode passar de 1 (sub-amortecida).
    pub x: f64,
    /// Velocidade, em unidades de caminho por segundo.
    pub v: f64,
}

impl SpringState {
    /// Uma mola parada na origem — o começo de um movimento que não interrompeu nada.
    #[must_use]
    pub fn at_rest() -> Self {
        Self { x: 0.0, v: 0.0 }
    }

    /// **A mola que RETOMA**: o caminho é novo, mas a velocidade é a que a cena já tinha.
    ///
    /// `x` volta a 0 porque o caminho foi re-medido a partir da pose viva (a lei (a) da
    /// `ph2d_ui_state::Machine`); `v` é reescalada para as unidades do caminho novo pelo chamador, que
    /// é quem sabe quanto ele mede.
    #[must_use]
    pub fn resuming(v: f64) -> Self {
        Self { x: 0.0, v }
    }

    /// Anda `dt` segundos. Devolve `true` quando assentou — e é o chamador que decide o que
    /// *assentar* significa para a pose (a `ph2d_ui_state::Machine` chama o `arrive`, que põe a pose
    /// EXATA).
    pub fn advance(&mut self, dt: f64, spring: Spring) -> bool {
        let s = spring.clamped();
        let (w, z) = (s.stiffness, s.damping);
        let mut left = dt.max(0.0);
        while left > 0.0 {
            let h = left.min(STEP);
            // Euler semi-implícito: a aceleração usa o `x` velho, e o `x` novo usa o `v` novo.
            let a = w * w * (1.0 - self.x) - 2.0 * z * w * self.v;
            self.v += a * h;
            self.x += self.v * h;
            left -= h;
        }
        (self.x - 1.0).abs() < SETTLE_X && self.v.abs() < SETTLE_V
    }
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod lib_tests;
