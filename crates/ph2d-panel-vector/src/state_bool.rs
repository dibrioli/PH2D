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

use std::cell::{Cell, RefCell};

thread_local! {
    /// O modo dos oito botões. `false` = destrutivo, o comportamento de sempre.
    static LIVE_ON: Cell<bool> = const { Cell::new(false) };
    /// Há um grupo booleano vivo na seleção deste frame?
    static GROUP_SELECTED: Cell<bool> = const { Cell::new(false) };
    /// A fileira do verbo por forma: *(verbo aceso, nome da forma)*.
    static SHAPE_ROW: RefCell<Option<(u8, String)>> = const { RefCell::new(None) };
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

/// **O verbo da forma selecionada** dentro de uma booleana viva (shell → painel).
///
/// `None` faz a fileira **não existir**, e ele carrega a regra inteira — irmão exacto do
/// `frame_clip`, e pela mesma razão: quem alcança o mundo é a shell, e uma segunda resposta a
/// *"esta forma pode escolher um verbo?"* divergiria da primeira no dia seguinte.
///
/// A shell devolve `None` em quatro casos, e cada um é uma decisão:
///
/// - não há **primário** na seleção — o verbo é de UMA forma, e o primário é a que o dedo
///   apontou (tocar um filho seleciona o grupo inteiro, e é por isso que a contagem não serve:
///   ⚠️ exigi-la tornava a fileira **inalcançável por clique**, que foi o defeito de 22/08);
/// - a forma não está numa booleana viva;
/// - ela é a **BASE** — o verbo dela é inerte, e um controlo inerte pintado como vivo é pior que
///   controlo nenhum;
/// - o grupo está numa **RECEITA** (`Trim`/`Crop`/`Merge`/`MinusBack`), que é verbo da pilha
///   inteira e ignora os das formas.
pub fn set_bool_shape_row(row: Option<(u8, String)>) {
    SHAPE_ROW.with(|c| *c.borrow_mut() = row);
}

/// O verbo da forma selecionada — `None` = a fileira não é oferecida.
#[must_use]
pub(crate) fn bool_shape_row() -> Option<(u8, String)> {
    SHAPE_ROW.with(|c| c.borrow().clone())
}
