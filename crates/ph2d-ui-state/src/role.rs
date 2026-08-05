//! **O PAPEL de um estado** — o que ele significa para quem o dispara.

use serde::{Deserialize, Serialize};

/// O que um estado É.
///
/// ⚠️ **É um PAPEL e não um nome livre, e a escolha decide a feature inteira.** Um nome livre
/// obrigaria a uma tabela de gatilhos — *"quando o rato entra, vá para o estado chamado assim"* —
/// e essa tabela é uma segunda autoria que o artista teria de manter em dia com a lista. Com o
/// papel, o gatilho é DERIVADO: entrar é [`Hover`](StateRole::Hover), apertar é
/// [`Pressed`](StateRole::Pressed), e não há nada a ligar.
///
/// ⚠️ **O que isto deliberadamente NÃO é:** um grafo de estados com transições autoradas (a
/// *state machine* do Rive). Aquilo é outra feature — tem condições, entradas nomeadas e um
/// editor próprio — e construí-la por dentro desta faria a lista de papéis virar uma lista de
/// nós com meia sintaxe.
///
/// ⚠️ **A ordem dos variants é a ordem em que a UI os oferece**, e o [`Default`](StateRole::Default)
/// é o primeiro porque é o único obrigatório: é para ele que tudo volta.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StateRole {
    /// A pose em repouso. **É a única obrigatória** — sem ela não há para onde voltar, e um
    /// hover que não volta é um botão que fica preso.
    Default,
    /// O rato está por cima.
    Hover,
    /// O botão está a ser apertado.
    Pressed,
    /// O controle não aceita o gesto.
    Disabled,
}

impl StateRole {
    /// Todos, na ordem em que a UI os oferece.
    pub const ALL: [StateRole; 4] = [
        StateRole::Default,
        StateRole::Hover,
        StateRole::Pressed,
        StateRole::Disabled,
    ];

    /// A chave i18n do rótulo.
    ///
    /// ⚠️ Uma chave e não um literal: a UI deste app é traduzida (HR-15), e um `&'static str` em
    /// inglês aqui seria a única string do módulo que não passa pelo catálogo.
    #[must_use]
    pub fn i18n_key(self) -> &'static str {
        match self {
            StateRole::Default => "panel.vector.states.role.default",
            StateRole::Hover => "panel.vector.states.role.hover",
            StateRole::Pressed => "panel.vector.states.role.pressed",
            StateRole::Disabled => "panel.vector.states.role.disabled",
        }
    }
}
