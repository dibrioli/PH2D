//! ⭐ **A acção aberta da aba Input Map, para o Inspector** (onda 4, 7b). O tipo mora na folha do ASSUNTO: a aba
//! (`ph2d-input-map-editor`) escreve-o a cada quadro e o painel (`ph2d-panel-inspector`) lê-o, sem aresta entre os dois
//! (`architecture_no_dependency_climbs_a_layer`). Texto já do artista: a aba resolve teclas e leitores.

/// A acção aberta.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputMapAction {
    pub name: String,
    pub group: String,
    /// Vazio = ninguém lê.
    pub readers: Vec<String>,
    pub keys: Vec<String>,
    /// `(dead, press)` já formatados — só numa acção com manche.
    pub stick: Option<(String, String)>,
}

/// O que o Inspector mostra com a aba à frente.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputMapPane {
    /// Nenhuma acção aberta: a frase do que fazer.
    Pick,
    Action(InputMapAction),
}

thread_local! {
    static CURRENT: std::cell::RefCell<Option<InputMapPane>> = const { std::cell::RefCell::new(None) };
}

/// `None` = a aba não está à frente (o Inspector de sempre).
pub fn set_current_input_map(pane: Option<InputMapPane>) {
    CURRENT.with(|c| *c.borrow_mut() = pane);
}

#[must_use]
pub fn current_input_map() -> Option<InputMapPane> {
    CURRENT.with(|c| c.borrow().clone())
}
