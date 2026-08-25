//! **A AÇÃO e as suas LIGAÇÕES** — o vocabulário que o artista autora.
//!
//! ⭐ As ligações apontam para os tipos de dispositivo que **esta crate já tinha** desde a M8
//! ([`GamepadButton`], [`GamepadAxis`]) mais o teclado que ela ganhou com o Input Map
//! ([`Key`]). *Um segundo vocabulário de botões seria duas respostas para a mesma pergunta.*

use serde::{Deserialize, Serialize};

use crate::gamepad::{GamepadAxis, GamepadButton};
use crate::keyboard::Key;

/// **O que produz o valor de uma acção.**
///
/// ⚠️ Um eixo entra **por metades** (`positive: true` = o lado direito/cima). É o que permite que
/// `move_left` e `move_right` sejam duas acções sobre o **mesmo** eixo físico, e é o que faz a
/// subtracção do [`crate::Input::axis`] funcionar sem um tipo novo.
///
/// ⛔ **Não há rato aqui, e é escopo, não esquecimento:** esta crate nunca modelou o rato — o
/// editor trata dele pelo seu próprio despacho. Acrescentá-lo é uma decisão à parte, com o seu
/// próprio `Event`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Binding {
    Key(Key),
    PadButton(GamepadButton),
    PadAxis { axis: GamepadAxis, positive: bool },
}

/// **O identificador ESTÁVEL de uma acção.**
///
/// ⚠️⚠️ **Não é um índice, e não é um hash do nome** — é um contador guardado no
/// [`crate::InputMap`], que **nunca reutiliza** um valor. As duas alternativas óbvias falham na
/// mesma pergunta, que é *o que acontece à gravação de ontem*:
///
/// | candidato | o que parte |
/// |---|---|
/// | **índice** na lista | **reordenar** o painel reescreve o significado de toda fita gravada |
/// | **hash do nome** | **renomear** `jump` para `pular` invalida toda fita gravada |
/// | ⭐ **contador estável** | nem reordenar nem renomear mexem no id — a fita sobrevive às duas |
///
/// É o mesmo raciocínio que fez o undo deste repo referir objectos por nome estável em vez de bits
/// de alocação, um domínio ao lado — e aqui a conclusão é a **oposta** (nome não serve), porque a
/// pergunta é outra.
///
/// ⛔ **O contador viaja com o mapa** ([`crate::InputMap`]). Um mapa recarregado que recomeçasse o
/// contador do zero **reutilizaria ids já gravados** — a armadilha clássica desta família.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ActionId(pub u32);

/// **Uma AÇÃO**: o nome que o jogo lê, e as ligações que a produzem.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputAction {
    /// O id estável. Atribuído pelo mapa, nunca pelo autor.
    pub id: ActionId,
    /// O nome que o código lê — `"jump"`, `"move_left"`.
    pub name: String,
    /// ⚠️ **N ligações, e zero é válido.** Teclado + comando + a segunda tecla do jogador canhoto
    /// são **a mesma acção**; e uma acção **declarada e por atribuir** tem de poder existir, senão
    /// o painel não consegue oferecer o passo *"agora escolha a tecla"*.
    pub bindings: Vec<Binding>,
    /// Abaixo disto a **força** é `0` (ruído do analógico). Ver o cabeçalho de [`crate`].
    pub dead_zone: f32,
    /// Acima disto `pressed` é `true`. Ver o cabeçalho de [`crate`].
    pub press_point: f32,
}

/// Ruído zero: uma tecla não treme.
const DEFAULT_DEAD_ZONE: f32 = 0.0;
/// Metade do curso — o ponto em que um gatilho analógico *"conta"*, e o que as referências usam
/// quando não há nada melhor a dizer.
const DEFAULT_PRESS_POINT: f32 = 0.5;

impl InputAction {
    /// **A porta única de uma acção nova.**
    #[must_use]
    pub fn new(id: ActionId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            bindings: Vec::new(),
            dead_zone: DEFAULT_DEAD_ZONE,
            press_point: DEFAULT_PRESS_POINT,
        }
    }

    /// Acrescenta uma ligação (encadeável).
    #[must_use]
    pub fn with(mut self, b: Binding) -> Self {
        self.bindings.push(b);
        self
    }

    /// Define os dois números, **já coagidos ao invariante** — ver [`InputAction::set_zone`].
    #[must_use]
    pub fn with_zone(mut self, dead_zone: f32, press_point: f32) -> Self {
        self.set_zone(dead_zone, press_point);
        self
    }

    /// **A porta que impõe a coerência dos dois números.**
    ///
    /// ⚠️ **`press_point` nunca fica abaixo de `dead_zone`.** Abaixo dela existiria um intervalo em
    /// que a acção diz `pressed == true` e entrega **força zero** — um estado que nenhum painel
    /// sabe desenhar e que nenhuma lei sabe consumir. *Impor o invariante na derivação, e não em
    /// cada gesto*, é o que impede que ele exista sequer.
    ///
    /// Os dois são presos a `0..1`, e a `dead_zone` a um épsilon abaixo de `1`: a normalização
    /// divide por `1 - dead_zone`, e uma `dead_zone` de exactamente `1` seria uma divisão por zero
    /// com cara de configuração inocente.
    ///
    /// ⚠️ **Uma porta só**: o [`InputAction::with_zone`] chama esta. Duas funções a impor o mesmo
    /// invariante seriam duas que divergem quando um terceiro número nascer.
    pub fn set_zone(&mut self, dead_zone: f32, press_point: f32) {
        /// O maior valor que a `dead_zone` pode ter sem que `1 - dead_zone` deixe de ser um divisor
        /// útil.
        const MAX_DEAD_ZONE: f32 = 0.99;
        let dz = clamp01(dead_zone).min(MAX_DEAD_ZONE);
        self.dead_zone = dz;
        self.press_point = clamp01(press_point).max(dz);
    }
}

/// `x` preso a `0..1`, com `NaN` a virar `0` — um `f32::clamp` devolve `NaN` para `NaN`, e um `NaN`
/// a viajar até à força de uma acção envenenaria toda subtracção que a lesse.
#[inline]
#[must_use]
fn clamp01(x: f32) -> f32 {
    if x.is_finite() {
        x.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[cfg(test)]
#[path = "action_tests.rs"]
mod tests;
