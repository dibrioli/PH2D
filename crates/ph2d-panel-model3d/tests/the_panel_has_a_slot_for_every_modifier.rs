//! ⛔⛔ **O `MAX_MODES` ERA UMA MINA, e ela quase disparou** (2026-08-30).
//!
//! Ele estava em `8`, e o `ph2d_field::UnaryKind::ALL` tinha **exactamente 8**. O modificador
//! seguinte nasceria com o chip **não pintado** (o `paint` faz `.take(MAX_MODES)`) e **sem id
//! registado** — e `apply_click` faz `match store.get_mut(id)`, que devolve `None`: *o evento nunca
//! nasce*. Um chip bonito e morto, exactamente o report de 2026-08-23 (*"nenhum botão funcionou"*).
//!
//! ⚠️ **Nenhum portão o via.** O `every_painted_button_answers_a_real_click` varre **o que o painel
//! registou ao pintar**, e um chip que nunca é pintado não entra na varredura. O
//! `rows_beyond_the_family_are_reported_not_dropped` é do `MAX_ROWS`, que é outra coisa — o
//! `MAX_MODES` não tinha rodapé nem gate.
//!
//! ⭐ E a fileira **WRAPPA** (`segmented_row_counts`), então o teto nunca foi visual: é só registo.

// ⚠️ Pela porta que a crate já publicava — `pub use populate::{MAX_MODES, MAX_ROWS}`. *Antes de
// abrir um módulo, confira o que já está aberto.*
use ph2d_panel_model3d::MAX_MODES;

/// ⭐ Cada natureza de modificador tem um slot registado — senão ela é inalcançável pelo ponteiro.
#[test]
fn the_panel_has_a_slot_for_every_modifier() {
    let n = ph2d_field::UnaryKind::ALL.len();
    assert!(
        n <= MAX_MODES as usize,
        "há {n} naturezas de modificador e o painel regista {MAX_MODES} slots — os de mais nascem \
         sem id no store, e o clique deles NÃO EXISTE"
    );
}

/// ⛔ **O CONTROLE**: um teto que já esteja no limite não é folga, é a mina armada outra vez.
///
/// ⚠️ Ele afirma **folga**, e não igualdade, porque a igualdade é o estado em que o gate acima passa
/// e o modificador seguinte morre em silêncio. *A cerca fica no lado perigoso.*
#[test]
fn the_slot_budget_still_has_room_for_the_next_modifier() {
    let n = ph2d_field::UnaryKind::ALL.len();
    assert!(
        n < MAX_MODES as usize,
        "o painel regista exactamente {MAX_MODES} slots para {n} modificadores — não há folga, e o \
         próximo a nascer fica sem chip e sem clique"
    );
}
