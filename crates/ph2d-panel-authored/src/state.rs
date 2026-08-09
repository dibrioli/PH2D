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
    /// **Uma das N opções foi escolhida** (`Tabs`, `RadioGroup`, `SegmentedAdaptive`, `Dropdown`).
    ///
    /// ⚠️ **O ÍNDICE, e aqui ele é o dado certo — ao contrário da `key`.** O doc do enum diz que a
    /// chave viaja porque o índice de uma ROW muda quando o artista reordena a Hierarquia; a opção
    /// é o caso oposto, porque a posição dela **é** o que o controle marca (`selected_index`), e o
    /// rótulo pode repetir-se entre filhos irmãos. Quem quiser o nome tem-no: ele é o `Name` do
    /// filho, e a chave da row mais o índice o alcançam.
    ///
    /// ⚠️ **E este variante fecha um vão que a família de LISTA shipou em 2026-08-09:** as três
    /// primeiras caíam em [`Self::Fired`], que diz *"alguém mexeu neste controle"* e **não diz
    /// QUAL opção** — o ouvinte recebia um gesto sem a informação inteira que ele carrega.
    Choice { key: String, index: usize },
    /// Um gesto sem valor (`Button`, `Tag`, `ListItem`).
    Fired { key: String },
}

thread_local! {
    static INTENTS: RefCell<Vec<AuthoredIntent>> = const { RefCell::new(Vec::new()) };
}

pub(crate) fn push_intent(i: AuthoredIntent) {
    INTENTS.with(|q| q.borrow_mut().push(i));
}

/// **Drena** os pedidos.
///
/// ⚠️ **Esta linha dizia *"a shell chama uma vez por frame"*, e era FALSO** — a varredura do repo
/// (2026-08-08) mostra que **ninguém fora dos testes desta crate a chama**. A ausência é a cerca
/// declarada no cabeçalho, mas a nota dela envelheceu: a **W8b.3 ligou a row à ARTE** por outra
/// rota (`WidgetStore` → `VecViewState`), então uma das três metades que a cerca escalonava já foi
/// feita por fora.
///
/// ⚠️ **E o preço não é só higiene:** uma fila que só é empurrada **cresce sem teto** enquanto o
/// painel está aberto — um arrasto de slider empurra um intent com duas `String` por quadro.
/// Ligar o ouvinte **ou** parar de empurrar são as duas curas; escolher é da wave que o fizer.
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
