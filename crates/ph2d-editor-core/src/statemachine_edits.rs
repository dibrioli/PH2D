//! ⭐⭐⭐ **O VOCABULÁRIO DO CÉREBRO** (TOP-20 #15) — o instantâneo e a edição, num módulo abaixo
//! do [`crate::action_bus`] e do [`crate::screens`].
//!
//! # ⛔ Por que ele não está no `screens::hero`, com os irmãos
//!
//! Pela mesma razão do [`crate::tags_edits`], do [`crate::factory_edits`], do
//! [`crate::topdown_edits`] e do [`crate::projectile_edits`], e com o mesmo número atrás: a catraca
//! do DAG tolera a aresta `action_bus → screens` num **tecto** e escreve a cura ao lado dela.
//!
//! *É o quinto degrau da mesma migração, e cada um torna o resto mais barato.*
//!
//! # ⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! | aviso | o que se passa | como se cura |
//! |---|---|---|
//! | `no states yet` | a máquina está vazia e não pensa | `+ Add State` |
//! | `only thinks while the clock plays` | a ponte corre no dreno de sinais | carregar no play |
//! | `nothing listens to this state` | um estado sem seta de saída é um beco | acrescentar uma transição |
//!
//! ⭐ **E o readout do ESTADO CORRENTE é a razão de esta secção existir com o relógio a andar:** um
//! cérebro sem ele é uma tabela que o artista tem de simular de cabeça.
//!
//! # ⚠️ As setas são ÍNDICES; o sinal é um NOME
//!
//! Renomear um estado **não** parte uma seta (elas guardam o índice); o `on` é o contrato com o
//! resto do mundo e por isso é texto.

/// Uma linha da lista de ESTADOS.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InspectorStateRow {
    pub name: String,
    /// O sinal publicado ao ENTRAR. Vazio = calado.
    pub on_enter: String,
    /// O sinal publicado ao SAIR. Vazio = calado.
    pub on_exit: String,
    /// ⚠️ **Alguma seta SAI daqui?** — `false` é um beco, e o painel di-lo.
    pub has_exit: bool,
}

/// Uma linha da lista de TRANSIÇÕES.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InspectorTransitionRow {
    pub from: u8,
    /// O nome do sinal. **Vazio = nunca dispara**, e o painel di-lo.
    pub on: String,
    pub to: u8,
}

/// Snapshot da secção STATE MACHINE da entidade selecionada.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorStateMachineInfo {
    pub entity_bits: u64,
    pub states: Vec<InspectorStateRow>,
    pub transitions: Vec<InspectorTransitionRow>,
    /// Em que estado a cena abre.
    pub initial: u8,
    /// ⭐ **Em que estado ele está AGORA** — `None` enquanto o vivo não nasceu (a cena ainda não
    /// correu um quadro com esta máquina).
    ///
    /// ⚠️ **Só de leitura**: é estado vivo, não autoria. O painel mostra-o e não o deixa editar —
    /// escrevê-lo seria uma segunda porta para o que a ponte decide.
    pub current: Option<u8>,
    /// O relógio está a andar?
    pub clock_playing: bool,
    pub selected_count: usize,
}

/// Uma edição de um campo da secção STATE MACHINE.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateMachineFieldEdit {
    /// `+ Add State`.
    AddState,
    /// `x Remove State` — ⚠️ **apagar um estado reescreve as setas que o apontam** (é a shell que o
    /// faz, porque a lei é do documento).
    RemoveState(u8),
    StateName(u8, String),
    StateOnEnter(u8, String),
    StateOnExit(u8, String),
    /// `+ Add Transition`.
    AddTransition,
    RemoveTransition(u8),
    TransitionFrom(u8, u8),
    TransitionOn(u8, String),
    TransitionTo(u8, u8),
    /// Em que estado a cena abre.
    Initial(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠️ **Todo campo do modelo tem uma edição** — um campo sem variante é um knob que o painel
    /// mostra e que ninguém pode mexer, e ele lê-se exactamente como um controlo morto.
    ///
    /// ⭐ E o `current` **não** tem, de propósito: ele é vivo, e a ausência é a decisão.
    #[test]
    fn todo_campo_autorado_tem_uma_edicao() {
        let campos_de_estado = ["name", "on_enter", "on_exit"];
        let edicoes_de_estado = [
            StateMachineFieldEdit::StateName(0, String::new()),
            StateMachineFieldEdit::StateOnEnter(0, String::new()),
            StateMachineFieldEdit::StateOnExit(0, String::new()),
        ];
        assert_eq!(campos_de_estado.len(), edicoes_de_estado.len());

        let campos_de_seta = ["from", "on", "to"];
        let edicoes_de_seta = [
            StateMachineFieldEdit::TransitionFrom(0, 0),
            StateMachineFieldEdit::TransitionOn(0, String::new()),
            StateMachineFieldEdit::TransitionTo(0, 0),
        ];
        assert_eq!(campos_de_seta.len(), edicoes_de_seta.len());

        // E as listas têm as duas metades — sem `Remove`, o `+` é uma porta de sentido único.
        let _ = StateMachineFieldEdit::AddState;
        let _ = StateMachineFieldEdit::RemoveState(0);
        let _ = StateMachineFieldEdit::AddTransition;
        let _ = StateMachineFieldEdit::RemoveTransition(0);
        let _ = StateMachineFieldEdit::Initial(0);
    }
}
