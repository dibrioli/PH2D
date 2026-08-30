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

/// ⭐⭐ **E o cabeçalho de secção é DESENHADO** — o fio entre o campo e o pixel.
///
/// ⚠️ Um `ParamRow::section` preenchido que ninguém pinta é um facto publicado e invisível, que é
/// exactamente o estado que o report descreve. ⛔ O cabeçalho **não** entra no índice de acerto (é
/// rótulo, não controle), então nenhum gate de costura o vê — este é textual de propósito, e fixa a
/// INSTRUÇÃO INTEIRA: uma asserção só sobre o nome da função sobrevive a `if false &&`.
#[test]
fn the_row_loop_paints_the_section_header() {
    let src = std::fs::read_to_string("src/paint.rs").expect("paint.rs");
    assert!(
        src.contains("if let Some(key) = row.section {\n            y = paint_section(ctx, tr(key), x, w, y);\n        }"),
        "o laco das linhas deixou de pintar o cabecalho de seccao — os numeros de um modificador \
         voltam a ficar no meio dos da forma, sem dizer de quem sao"
    );
}
