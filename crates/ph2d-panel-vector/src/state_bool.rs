//! **A BOOLEANA VIVA** — o modo dos oito botões, e o fato que decide se o *Apply* é oferecido.
//!
//! Duas metades, e elas são de donos diferentes:
//!
//! - **`live_on`** é o que o ARTISTA escolheu (irmão do [`crate::state_expand`]): decide se os
//!   oito botões da seção Boolean CONSOMEM os operandos (o mundo de sempre) ou criam um GRUPO
//!   cujos filhos se combinam e continuam editáveis. Panel-local, e a shell o lê no clique.
//! - **`group_selected`** é um fato da CENA que só a shell enxerga (irmão do
//!   [`crate::state_cut`]): há um grupo booleano na seleção? É ele que decide se o **Apply**
//!   aparece — um *Apply* que não aplica nada é pior que *Apply* nenhum.
//!
//! ⚠️ A verdade da segunda mora no ECS (`ph2d_ecs::VecBoolGroup`), não aqui: isto é a projeção
//! que a shell publica por frame. O painel não alcança o mundo — e não deve: se alcançasse,
//! haveria duas respostas para *"esta seleção é uma booleana viva?"*.

use std::cell::Cell;

thread_local! {
    /// O modo dos oito botões. `false` = destrutivo, o comportamento de sempre.
    static LIVE_ON: Cell<bool> = const { Cell::new(false) };
    /// Há um grupo booleano vivo na seleção deste frame?
    static GROUP_SELECTED: Cell<bool> = const { Cell::new(false) };
    /// **O verbo do DIAGRAMA**, quando a seleção tem um. Ver [`bool_graph_op`].
    static GRAPH_OP: Cell<Option<Option<u8>>> = const { Cell::new(None) };
}

/// O modo escolhido pelo artista. **`false` por default, de propósito:** ligar a booleana viva
/// muda o que oito botões já existentes fazem, e um modo que se arma sozinho é uma ferramenta que
/// já mudou de comportamento quando você olha.
#[must_use]
pub fn bool_live_on() -> bool {
    LIVE_ON.with(Cell::get)
}

/// Os dois chips escrevem aqui (panel-local — a escolha é do painel, o efeito é da shell).
pub fn set_bool_live_on(on: bool) {
    LIVE_ON.with(|c| c.set(on));
}

/// Publica se a seleção deste frame está dentro de um grupo booleano (shell → painel).
pub fn set_bool_group_selected(sel: bool) {
    GROUP_SELECTED.with(|c| c.set(sel));
}

/// Há booleana viva selecionada? É isto que faz o **Apply** ser oferecido.
#[must_use]
pub(crate) fn bool_group_selected() -> bool {
    GROUP_SELECTED.with(Cell::get)
}

/// **O verbo que o DIAGRAMA usa**, publicado pela shell.
///
/// Três estados, e eles são coisas diferentes:
/// - `None` — a seleção não tem diagrama: os oito botões são a operação do grupo, como sempre.
/// - `Some(None)` — há diagrama e as ligações **discordam**: *misto*.
/// - `Some(Some(op))` — há diagrama e todas as ligações usam este verbo.
///
/// ⚠️ A distinção existe porque um botão que não muda nada é o pior defeito possível de um painel.
/// Com diagrama, quem manda é a operação de cada LIGAÇÃO — e o artista tem de VER isso antes de
/// clicar, senão ele clica *Subtract*, o clique reescreve as oito ligações de uma vez, e ele não
/// tinha como saber que era isso que ia acontecer.
#[must_use]
pub(crate) fn bool_graph_op() -> Option<Option<u8>> {
    GRAPH_OP.with(Cell::get)
}

/// A shell publica o verbo do diagrama (shell → painel).
pub fn set_bool_graph_op(op: Option<Option<u8>>) {
    GRAPH_OP.with(|c| c.set(op));
}
