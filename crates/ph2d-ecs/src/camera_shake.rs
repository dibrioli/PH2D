//! ⭐⭐⭐ **O ABANÃO DA CÂMERA — *isto explodiu, e a vista tremeu*** (suplente #25 do TOP-20, o
//! último dos cinco).
//!
//! # Porque ele existe: a composição foi MEDIDA e não chega
//!
//! A sonda `mede_o_que_a_composicao_ja_da_ao_abanao` (`ph2d-app-components`, `--ignored`) mediu as
//! cinco perguntas antes da primeira linha desta wave:
//!
//! | a pergunta | a resposta |
//! |---|---|
//! | a tabela de acções sabe dizer «treme»? | ⛔ **não** — `9` verbos e nenhum é da câmera |
//! | um TWEEN sobre a própria câmera exprime-o? | ⛔⛔ **não**, e é o achado: o tween **escreveu** (`Transform.x = 2,0`) e o **centro da vista ficou em `[0,0]`** |
//! | um sinal sabe QUEM gritou? | ⭐ **sim**, em `3` das `5` origens amostradas |
//! | há gerador determinista? | ⭐ **sim** — *splitmix64*, **privado** da fábrica |
//! | rebobinar já renasce? | ⭐ **sim** — o censo existe, e um vivo NOVO tem de entrar nele |
//!
//! ⇒ **o buraco é um só: nada escreve na VISTA.** O concorrente mais forte — um tween de pose sobre
//! a câmera — move um [`crate::Transform`] que o enquadramento **não lê** (ele vem do
//! [`crate::CameraRuntime`]).
//!
//! # ⭐⭐ A arquitectura: a EXPLOSÃO não procura a câmera
//!
//! É o modelo *fonte + ouvinte* do Cinemachine Impulse, e ele cai **exactamente** no barramento que
//! o ADR-0075 já obriga: quem explode **publica** um sinal, e a câmera **lê** — ⛔ ninguém chama
//! ninguém, e não há um `Camera.Shake()` a atravessar a cena.
//!
//! * o [`ShakeEmitter`] mora em **quem explode** e diz *«ao ouvir isto, levanta tanto trauma»*;
//! * o [`CameraShake`] mora na **câmera** e diz **como** ela treme;
//! * a distância entre os dois **atenua** o impulso ([`ph2d_shake::atenuacao`]), que é o que faz
//!   uma explosão ao longe abanar menos — e é o que o `UCameraShakeSourceComponent` do Unreal
//!   exprime com os mesmos dois raios.
//!
//! # ⛔ E NÃO há um verbo `Shake` na tabela de acções
//!
//! Seria a **segunda** maneira de pedir a mesma coisa — e a pior das duas, porque um verbo não tem
//! de onde tirar a DISTÂNCIA (ele age sobre um alvo, não a partir de um sítio). *Duas superfícies
//! sobre um valor divergem no dia em que uma ganhar um cuidado*, e aqui elas já nasceriam
//! diferentes. Há gate.
//!
//! # ⛔ O que fica de fora, com o motivo
//!
//! **Abanão ANGULAR** (o *roll* do Cinemachine): a `CameraView` da shell tem **três** campos
//! (`center`, `height_world`, `cull_mask`) e **nenhum é um ângulo** — a vista deste app não roda, e
//! pô-la a rodar é uma wave de renderer, não um campo aqui. *Um limite legítimo diz de que recurso
//! ele é* (§0.0): este é da superfície da vista, e está contado.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{SignalFrom, SimComponent};

/// **Quantas fontes de abanão uma entidade pode ter.**
///
/// ⚠️ O número é o da SUPERFÍCIE e não um palpite: esta secção do Inspector é a lista + editor do
/// [`crate::CounterWatch`] com outros campos, logo o tecto é o mesmo [`crate::WATCHES_MAX`] — *um
/// modelo que aceita o que o painel não mostra produz estado inalcançável.*
pub const SHAKE_EMITTERS_MAX: usize = crate::WATCHES_MAX;

/// **COMO esta câmera treme** — os cinco números da lei, mais nada.
///
/// ⚠️ **A presença É o valor:** uma câmera sem este componente não treme, e é a ausência que o diz —
/// não um campo `enabled` a mais (a lei que o [`crate::CameraFollow`] já escreve ao lado).
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CameraShake {
    /// Metros de deslocamento no pico. O de fábrica é `0,25` — ver [`Self::default`].
    pub amplitude: f32,
    /// Hz. Quantas vezes por segundo a vista muda de direcção.
    pub frequencia: f32,
    /// Trauma por segundo que o decaimento leva ⇒ `1 / decaimento` é a duração de um abanão cheio.
    pub decaimento: f32,
    /// A potência do trauma — a faixa e o porquê dela vivem em [`ph2d_shake::EXPOENTE_MIN`] e
    /// [`ph2d_shake::EXPOENTE_MAX`].
    pub expoente: u8,
    /// A semente do ruído: duas câmeras com sementes diferentes tremem de maneiras diferentes.
    pub semente: u64,
}

impl SimComponent for CameraShake {}

impl Default for CameraShake {
    /// ⚠️ **Os valores de fábrica são os de um abanão de IMPACTO**, e cada um tem uma razão:
    ///
    /// | campo | valor | porquê |
    /// |---|---|---|
    /// | `amplitude` | `0,25` m | `2,5 %` da altura da vista de fábrica (`10,0` m, o
    ///   [`crate::GameCamera::default`]): visível e sem enjoar. ⚠️ **Esta linha dizia `11,25` m e
    ///   `~2 %`, e o número não vinha de componente nenhum** — corrigido em 2026-09-20, ao medi-lo
    ///   para a escada dos perfis ([`ph2d_shake::Perfil`]) |
    /// | `frequencia` | `20` Hz | acima do que um olho segue e abaixo do que um ecrã a 60 Hz amostra mal (⚠️ a `30` já há dois quadros por período) |
    /// | `decaimento` | `2,0` | meio segundo de abanão cheio |
    /// | `expoente` | `2` | o que a referência ship (`bevy_trauma_shake`) |
    /// | `semente` | `0x5EED` | ⚠️ um valor **NÃO nulo**: `0` é o «por semear» do *splitmix* |
    fn default() -> Self {
        Self {
            amplitude: 0.25,
            frequencia: 20.0,
            decaimento: 2.0,
            expoente: 2,
            semente: 0x5EED,
        }
    }
}

impl CameraShake {
    /// **A PORTA para a lei.** ⚠️ Escrita duas vezes (aqui e na ponte), as duas divergiriam no dia
    /// do sexto número.
    #[must_use]
    pub fn lei(&self) -> ph2d_shake::Lei {
        ph2d_shake::Lei {
            amplitude: self.amplitude,
            frequencia: self.frequencia,
            decaimento: self.decaimento,
            expoente: self.expoente,
            semente: self.semente,
        }
    }
}

/// **O que a câmera está a fazer AGORA** — ⛔ e ele **não é registado**.
///
/// ⚠️ **A cerca é o TIPO:** ele não deriva `Serialize`, logo a linha do registo nem compila — o
/// precedente do [`crate::TimerRuntime`]. Registá-lo poria **cada quadro de um abanão** dentro do
/// ficheiro gravado e um passo na pilha de `Ctrl+Z`.
///
/// ⚠️ **Ele entra no [`crate::rewind_runtime`]**, e o «nascer» dele é `Default` — trauma zero e
/// relógio zero. Sem isso, rebobinar deixaria a vista a tremer o resto de um abanão da corrida
/// anterior.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct CameraShakeRuntime {
    /// `0..1`. Ver [`ph2d_shake::TRAUMA_MAX`].
    pub trauma: f32,
    /// ⭐ **Segundos COM TRAUMA acumulados**, e não o relógio do mundo. Ele só anda enquanto há o
    /// que abanar — é isso que mantém o argumento do ruído longe do tecto do `f32` (o cabeçalho da
    /// [`ph2d_shake`]) **e** que faz duas explosões seguidas começarem em fases diferentes.
    pub t: f32,
}

/// **Uma fonte:** *ao ouvir este sinal, levanta tanto trauma — e quanto chega depende de quão
/// longe a câmera está daqui.*
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShakeSource {
    /// O sinal que ela ouve. ⚠️ **Vazio = calada** — a lei dos contactos da física, e o que faz uma
    /// fonte acabada de anexar não abanar nada até alguém escrever o nome.
    pub on: String,
    /// ⭐ **A cerca de quem falou** — o `SignalFrom` do suplente #24, reutilizado inteiro.
    ///
    /// ⚠️ **É ela que faz dez bombas iguais não abanarem todas quando UMA explode:** com
    /// [`SignalFrom::Myself`] cada bomba só ouve o próprio estrondo. *Sem ela, a única cena que este
    /// componente serve bem é a que tem um objecto só.*
    pub de: SignalFrom,
    /// Quanto trauma este sinal levanta **à queima-roupa** (`0..1`).
    pub forca: f32,
    /// Até esta distância (metros) o impulso chega INTEIRO.
    pub dentro: f32,
    /// A partir desta distância ele não chega. ⚠️ `fora <= dentro` é um **corte duro** em `dentro` —
    /// a lei responde-o sozinha ([`ph2d_shake::atenuacao`]).
    pub fora: f32,
}

impl Default for ShakeSource {
    /// ⚠️ **`dentro`/`fora` de fábrica são `3` e `12` metros** — a `3` m a explosão está dentro da
    /// vista de fábrica (`11,25` m de altura, logo `±5,6`) e a `12` está fora dela: *uma fonte
    /// acabada de anexar abana quando a coisa está no ecrã e cala-se quando ela não está.*
    fn default() -> Self {
        Self {
            on: String::new(),
            de: SignalFrom::Anyone,
            forca: 0.6,
            dentro: 3.0,
            fora: 12.0,
        }
    }
}

/// **As fontes de uma entidade.** Lista, como os [`crate::Timers`] e as [`crate::CounterWatch`]:
/// um inimigo abana pouco ao levar um tiro e muito ao morrer, e isso são duas linhas.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ShakeEmitter(pub Vec<ShakeSource>);

impl SimComponent for ShakeEmitter {}

#[cfg(test)]
#[path = "camera_shake_tests.rs"]
mod tests;
