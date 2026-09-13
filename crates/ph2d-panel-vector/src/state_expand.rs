//! Os knobs do **Offset Path** (a seção Expand) — irmão de `state.rs` pelo teto de 600 LOC.
//!
//! São estado PANEL-LOCAL, e é isso que os separa dos irmãos `state_*`: os outros publicam o que a
//! shell OBSERVOU na cena, estes guardam o que o artista ESCOLHEU e a shell vem ler no clique.
//!
//! ⚠️ Não confundir com o Corner/Side do **Contour**, que faz a mesma pergunta com os mesmos
//! códigos mas tem resposta por-FORMA (vive no componente, não aqui): lá o chip retuna um efeito
//! que já está na tela, aqui ele arma o PRÓXIMO offset.

use std::cell::Cell;

/// O par `(junção, lado)` com que os dois knobs NASCEM — `0` Miter e `2` Both.
///
/// `pub` porque a SHELL guarda o par que viu no quadro anterior para distinguir *"o artista clicou
/// um chip"* de *"o painel está no valor de sempre"*, e esse par tem de nascer IGUAL a este: com
/// outro, o 1.º quadro leria um clique que ninguém deu e retunaria os offsets vivos da seleção. Os
/// dois números eram escritos à mão aqui e no construtor da `App` (A9, 2026-09-12).
pub const EXPAND_KNOBS_AT_BIRTH: (u8, u8) = (0, 2);

thread_local! {
    /// Junção das quinas do **Offset Path** (`0` Miter · `1` Round · `2` Bevel).
    ///
    /// Panel-local e NÃO é o join do traço: um path pode não ter traço nenhum e ainda ser
    /// offsetado, e mesmo tendo, a quina que o traço desenha e a quina que o offset produz
    /// são escolhas diferentes. Reaproveitar `VECTOR_JOIN_*` faria um controle mexer em
    /// dois fatos.
    static EXPAND_JOIN: Cell<u8> = const { Cell::new(EXPAND_KNOBS_AT_BIRTH.0) };
    /// Qual contorno o **Offset Path** move (`0` Outer · `1` Inner · `2` Both). Default `2`
    /// (Both) — num caminho sem furo é o único contorno de qualquer jeito. Panel-local, como o
    /// `EXPAND_JOIN`: a shell o lê no clique/drag de offset e o motor o honra.
    static EXPAND_SIDE: Cell<u8> = const { Cell::new(EXPAND_KNOBS_AT_BIRTH.1) };
}

/// A junção escolhida para o **Offset Path** (`0` Miter · `1` Round · `2` Bevel).
///
/// `pub` porque a SHELL a lê no clique de Offset Path — o painel escolhe, o motor honra.
/// Uma cópia do lado da shell seria a segunda resposta a *"que quina o offset faz?"*, e
/// divergiria no dia em que alguém acrescentasse um estilo de junção.
#[must_use]
pub fn expand_join() -> u8 {
    EXPAND_JOIN.with(Cell::get)
}

/// `pub` porque o gate do RETUNE (shell) precisa armar a quina sem montar o painel — o
/// produto só escreve pelo clique nos chips.
pub fn set_expand_join(v: u8) {
    EXPAND_JOIN.with(|c| c.set(v));
}

/// Qual contorno o **Offset Path** move (`0` Outer · `1` Inner · `2` Both).
///
/// `pub` pela mesma razão do [`expand_join`]: a SHELL a lê no gesto de offset, e uma cópia
/// dela do outro lado seria a segunda resposta a *"que contorno o offset move?"*.
#[must_use]
pub fn expand_side() -> u8 {
    EXPAND_SIDE.with(Cell::get)
}

/// `pub` pela razão do [`set_expand_join`].
pub fn set_expand_side(v: u8) {
    EXPAND_SIDE.with(|c| c.set(v));
}
