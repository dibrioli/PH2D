//! O estado do painel — **vazio**, e a razão é a espinha da wave.
//!
//! # O valor de uma row mora no `WidgetStore`, e em lugar nenhum mais
//!
//! O store é quem o ponteiro escreve: arrastar um slider, focar um campo, apertar um toggle. Uma
//! cópia do valor aqui seria a **segunda resposta** a *"quanto vale este slider?"* — e a falha não
//! seria sutil, porque o arrasto moveria uma e o `paint` leria a outra: o controle não se mexeria
//! sob o dedo. O `WidgetEvent` do repo já diz isto na cara (*"value-bearing variants carry only
//! the `NodeId`; the caller re-reads from the store"*).
//!
//! # O que sobra é a FILA DE INTENTS, e ela é a fronteira nomeada da wave
//!
//! Um gesto numa row emite `(chave, valor)`. **Quem escuta ainda não existe** — ligar a row a um
//! token, a uma propriedade de arte ou a um evento de jogo é a W4b/W8a, e é o próprio plano que o
//! escalona. A fila existe para que essa fronteira seja uma **porta** em vez de um vão: o painel
//! diz o que mudou, e a wave seguinte encaixa o ouvinte sem mexer aqui.
//!
//! ⚠️ **Isto não é um botão que não faz nada.** Um slider que se arrasta e se move está a fazer
//! exactamente o que a wave promete — mostrar o painel que o artista desenhou, vivo. O que ele
//! ainda não faz é MEXER em algo, e a cena de smoke diz isso com todas as letras em vez de deixar
//! o artista descobrir.

use std::cell::RefCell;

/// O que um gesto numa row PEDE.
///
/// ⚠️ A chave viaja, não o índice: o índice muda quando o artista reordena os filhos, e um ouvinte
/// que guardasse *"a row 2"* passaria a controlar outra coisa depois de um arrasto na Hierarquia.
/// A chave é o que ele desenhou.
#[derive(Clone, Debug, PartialEq)]
pub enum AuthoredIntent {
    /// Um controle contínuo mudou (`Slider`) — o valor é a fração `0..=1` da trilha.
    Value { key: String, value: f32 },
    /// Um controle de dois estados mudou (`Toggle`, `Checkbox`).
    Flag { key: String, on: bool },
    /// Um campo de texto mudou.
    Text { key: String, text: String },
    /// Um gesto sem valor (`Button`, `Tag`, `ListItem`).
    Fired { key: String },
}

thread_local! {
    static INTENTS: RefCell<Vec<AuthoredIntent>> = const { RefCell::new(Vec::new()) };
}

pub(crate) fn push_intent(i: AuthoredIntent) {
    INTENTS.with(|q| q.borrow_mut().push(i));
}

/// **Drena** os pedidos — a shell chama uma vez por frame.
///
/// ⚠️ Drenar ESVAZIA: um segundo leitor veria a fila vazia e o gesto do artista viraria no-op num
/// dos dois caminhos. É a mesma lei do `wash_diag` do Painter e do `drain_intents` dos Tokens.
#[must_use]
pub fn drain_intents() -> Vec<AuthoredIntent> {
    INTENTS.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

/// Estado por-painel exigido pelo trait `Panel`. Vazio: ver o § do cabeçalho.
#[derive(Default)]
pub struct AuthoredPanelState;
