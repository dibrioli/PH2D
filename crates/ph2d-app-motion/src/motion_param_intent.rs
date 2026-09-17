//! ⭐⭐ **A CAIXA DE CORREIO DO CARTÃO** — as intenções de param que sobem para a ponte.
//!
//! ⚠️⚠️ **Ela viveu dentro da crate do painel lateral até 2026-09-17, e isso era o que tornava a
//! remoção dele impossível** (doc 114 §13): o TIPO e a FILA moravam lá, e quem os enchia era o
//! CARTÃO — cinco sítios em `motion_bridge_intents.rs` e `motion_bridge_choices.rs`, mais o dreno
//! do `motion_bridge_params_edit.rs`. ⛔ *Apagar o painel primeiro deixaria o cartão mudo.*
//!
//! ⭐ **A outra metade daquele ficheiro NÃO veio, e a ausência é a decisão:** o
//! `set_current_params`/`current_params` publicava o `ParamsSnapshot` **para o painel pintar**, e
//! sem painel não há quem leia. *O que se constrói para ninguém ver não se constrói.*

use std::cell::RefCell;

/// A param edit the panel asks the shell to apply (M1.P1). Tagged with the node
/// id + canonical param name so it is unambiguous even if the selection changed
/// between the edit and the drain.
#[derive(Clone, Debug, PartialEq)]
pub enum MotionParamIntent {
    SetParam {
        node: u32,
        param: &'static str,
        value: f64,
    },
    /// A **text** param edit (a formula) — carries a `String` (the `f64` `SetParam` cannot).
    /// The bridge applies it via `Graph::set_text_param`.
    SetTextParam {
        node: u32,
        param: &'static str,
        value: String,
    },
    /// **Devolve o param ao default do nó** — a ponte chama `Graph::clear_param` E
    /// `clear_text_param` para o mesmo nome.
    ///
    /// ⚠️ Os dois, de propósito: um nome viaja por UM dos canais, nunca pelos dois, e o painel
    /// não precisa saber por qual. Enumerar "este é de texto, aquele é de f32" no lado da UI é a
    /// lista que apodrece no dia em que um param muda de canal — e a §5 do CLAUDE.md registra
    /// exatamente essa migração acontecendo (o gradiente do `color_ramp`, a paleta do
    /// `color_array`). Limpar o que não existe é um no-op barato.
    ResetParam { node: u32, param: String },
    /// **Ask the shell to open a file dialog** for a [`FileRow`], and write the pick into
    /// `param` (a `Graph::set_text_param`).
    ///
    /// ⚠️ **It carries no filter, and that is the design.** A dialog is an OS window that
    /// freezes the loop, so only the shell may open one — and the shell resolves the
    /// filter from the `ParamUiHint` it published for `(node, param)`. Putting the
    /// extensions in the intent would make the panel the second place that answers *what
    /// is an audio file*, through a channel nothing gates.
    PickFile { node: u32, param: &'static str },
}

thread_local! {
    static INTENTS: RefCell<Vec<MotionParamIntent>> = const { RefCell::new(Vec::new()) };
}

/// Queue a param edit for the bridge to apply (panel → shell).
///
/// ⚠️ **`pub`, e a assimetria era o problema:** o `drain_param_intents` já é público (a shell
/// drena), e só esta metade não era — o que deixava o caminho REAL de uma edição de param
/// inalcançável de fora deste crate. Um gate da shell tinha então de chamar a função interna
/// que o `apply_param_edits` chama, em vez do `apply_param_edits`, e um gate assim fica verde
/// no dia em que o executor deixar de a chamar. *É a forma de gate vazio que a auditoria deste
/// módulo apanhou vinte e quatro vezes.*
pub fn push_param_intent(intent: MotionParamIntent) {
    INTENTS.with(|c| c.borrow_mut().push(intent));
}

/// Drain the queued param edits (shell bridge, each frame). Capacity-retaining.
pub fn drain_param_intents() -> Vec<MotionParamIntent> {
    INTENTS.with(|c| std::mem::take(&mut *c.borrow_mut()))
}
