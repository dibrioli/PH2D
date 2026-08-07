//! **O CONTRATO da math** — o que um token numérico precisa de saber sobre fórmulas, que é o
//! mínimo possível (plano UI/UX W4c.3).
//!
//! # ⚠️ O parser NÃO mora aqui, e a razão é uma aresta MEDIDA
//!
//! O repo já tem **um** parser VEX-lite (`ph2d-expr-parse`, ADR-0144) e a lei desta casa é *um
//! parser, dois consumidores* — escrever um terceiro seria a terceira resposta a *"o que é uma
//! fórmula neste app?"*. Mas ligar esta crate a ele **não é grátis**: `ph2d-expr-parse` depende de
//! `ph2d-expr`, que depende de **`ph2d-nodegraph`**.
//!
//! `ph2d-tokens` é a folha de que **44 widgets e todo painel** dependem, e ela declara zero deps de
//! runtime. Pô-la a arrastar o substrato de GRAFO DE NÓS é a aresta ao contrário: um botão de
//! ícone passaria a compilar o motor de cozimento para saber de que cor é.
//!
//! ⇒ **O parser é INJECTADO.** Esta crate guarda a fórmula como TEXTO — que é o que o arquivo
//! guarda de qualquer maneira — e recebe as duas perguntas que sabe fazer sobre ela num
//! [`MathHost`] instalado por quem *pode* depender do parser (`ph2d-token-math`). É o padrão que o
//! `LutSpec` já usa nos nós: *o substrato fica agnóstico, quem sabe instala a capacidade*.
//!
//! # As DUAS perguntas, e por que são duas
//!
//! - [`MathHost::deps`] — *que tokens esta fórmula lê?* É a pergunta da **lei do ciclo** e da
//!   validação na porta; ela não precisa de números.
//! - [`MathHost::eval`] — *quanto ela vale, dados os valores?* É a pergunta da **leitura**.
//!
//! Colapsá-las num único `resolve` obrigaria a caminhada do ciclo a inventar valores para tokens
//! que ainda não têm nenhum — a pergunta do grafo seria respondida com aritmética.
//!
//! # ⚠️ Sem host instalado, uma fórmula é RECUSADA — nunca dobrada
//!
//! O contrato do `ph2d_expr::Bindings` diz que *nome desconhecido vale `0.0`, não pânico* —
//! correcto para um nó de partículas e **venenoso** para um design system. Aqui a política é a
//! oposta e vale em todo lado: o que não se consegue responder é recusado na **porta de escrita**,
//! e o que já está guardado e não se consegue ler cai na **fábrica**. Um número inventado seria
//! indistinguível de um número autorado.

use std::cell::RefCell;

use crate::num::NumToken;

/// *Quanto vale este token?* — a pergunta que a avaliação faz ao chamador, uma vez por referência.
///
/// ⚠️ Um alias de tipo, e não porque a assinatura é longa: ela é a **fronteira** desta camada, e um
/// nome torna dizível o que ela é — *o chamador é quem sabe o que um token vale; esta crate só sabe
/// perguntar*. É também o que o `clippy::type_complexity` estava a apontar.
pub type ValueOf<'a> = &'a dyn Fn(NumToken) -> f32;

/// As duas perguntas que esta crate sabe fazer sobre uma fórmula, respondidas por quem tem o parser.
///
/// ⚠️ `fn` ponteiros e não `Box<dyn Fn>`: o host é instalado uma vez no boot e nunca captura estado
/// — um closure com ambiente convidaria alguém a fechar sobre um modo ou um documento, e a fórmula
/// passaria a valer coisas diferentes conforme *quem* a instalou.
#[derive(Clone, Copy)]
pub struct MathHost {
    /// *Que tokens esta fórmula lê?* — na ordem de aparição, sem repetições. `Err` é a frase da
    /// recusa, pronta para um toast.
    pub deps: fn(&str) -> Result<Vec<NumToken>, String>,
    /// *Quanto ela vale?* — `value_of` responde por cada token referido.
    pub eval: fn(&str, ValueOf<'_>) -> Result<f32, String>,
}

thread_local! {
    static HOST: RefCell<Option<MathHost>> = const { RefCell::new(None) };
}

/// **A porta única de instalação.** Chamada uma vez, no boot, por quem pode depender do parser.
///
/// ⚠️ Instalar duas vezes é legal e a última vence — é o que torna um teste capaz de trocar o host
/// sem um `clear` ao lado. O que **não** é legal é haver dois hosts vivos, e não há: um slot.
pub fn install_math(host: MathHost) {
    HOST.with(|h| *h.borrow_mut() = Some(host));
}

/// Retira o host — só os gates precisam disto (para medir o que acontece **sem** math).
pub fn uninstall_math() {
    HOST.with(|h| *h.borrow_mut() = None);
}

/// *Há como responder sobre fórmulas?* — o painel pergunta isto para decidir se **oferece** o botão.
///
/// ⚠️ É o padrão do `set_ml_available` do AI Denoise: sem a capacidade, o controlo **não existe**,
/// em vez de existir e não fazer nada.
#[must_use]
pub fn math_available() -> bool {
    HOST.with(|h| h.borrow().is_some())
}

/// Os tokens que `src` lê — `Err` com a frase, ou `Err` genérico se ninguém instalou o host.
pub fn deps_of(src: &str) -> Result<Vec<NumToken>, String> {
    let host = HOST.with(|h| *h.borrow());
    match host {
        Some(h) => (h.deps)(src),
        None => Err("formulas are not available in this build".to_string()),
    }
}

/// Quanto `src` vale — `None` quando não há host, e o chamador cai na fábrica.
///
/// ⚠️ `None` e não `0.0`: zero é um comprimento legítimo, então devolvê-lo aqui tornaria *"não sei"*
/// indistinguível de *"vale zero"* — que é exactamente a rachura que o `Bindings` do IR tem e que
/// esta camada existe para não herdar.
#[must_use]
pub fn eval(src: &str, value_of: ValueOf<'_>) -> Option<f32> {
    let host = HOST.with(|h| *h.borrow())?;
    (host.eval)(src, value_of).ok()
}

#[cfg(test)]
#[path = "num_expr_tests.rs"]
mod tests;
