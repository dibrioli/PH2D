//! ⭐⭐⭐ **O `Delete` da escultura DIZ por que recusou — e não come o da Hierarquia.**
//!
//! # O report
//!
//! Enio, 2026-09-04: *«corrija o deletar com a tecla del»*, sobre o canvas com a escultura na
//! tela. A tecla morria num de **três** guardas (`sculpt3d_keys_live`) e **nenhum deles dizia
//! nada** — nem ao artista, nem a quem foi diagnosticar. *Uma sessão inteira de leitura de
//! código responde ao que uma linha impressa responde à primeira.*
//!
//! # ⚠️ Por que este gate lê o FONTE
//!
//! As duas leis vivem num `impl App`, que pede janela, superfície e device — nenhum teste as
//! alcança. ⛔ E um gate textual **não distingue a chamada viva da chamada atrás de um
//! `if false`**: ele prende a FORMA, e a prova de que a forma corre é o smoke. É o mesmo
//! compromisso — declarado — do `the_hierarchy_has_a_delete_key`.

/// O teclado da escultura.
const KEYS: &str = include_str!("../src/sculpt3d_keys.rs");
/// Onde os guardas moram.
const DISPATCH: &str = include_str!("../src/input_dispatch.rs");

/// ⭐⭐⭐ **A razão é a FONTE do «está vivo?», e não uma segunda opinião.**
///
/// ⛔ Duas respostas à mesma pergunta divergem no dia em que alguém acrescenta um quarto guarda
/// a só uma delas — e o sintoma seria o report de origem outra vez: a tecla morre e a linha
/// impressa jura que está tudo bem.
#[test]
fn o_vivo_deriva_da_razao_e_nao_repete_os_guardas() {
    assert!(
        DISPATCH.contains("self.sculpt3d_keys_dead_reason().is_empty()"),
        "⛔ o `sculpt3d_keys_live` tem de DERIVAR da razao"
    );
    // Os três guardas aparecem UMA vez cada — dentro da razão.
    for guarda in [
        "self.sculpt3d_clay_on_screen()",
        "self.text_entry_focused()",
        "self.a_tool_owns_the_bare_keys()",
    ] {
        let n = DISPATCH.matches(guarda).count();
        assert!(
            n >= 1,
            "o guarda `{guarda}` desapareceu -- a razao deixou de o medir"
        );
    }
}

/// ⭐⭐⭐ **A recusa é IMPRESSA** — a lei que o `Delete` lá dentro já escrevia, aplicada ao
/// guarda.
#[test]
fn a_recusa_do_delete_e_reportada() {
    assert!(
        KEYS.contains("o Delete NAO foi para a escultura"),
        "⛔ a recusa do guarda voltou a ser MUDA"
    );
    assert!(
        KEYS.contains("sculpt3d_keys_dead_reason()"),
        "⛔ e ela tem de dizer QUAL guarda"
    );
}

/// ⭐⭐⭐ **O `Delete` sobre um PAINEL não é da escultura.**
///
/// ⛔ Este teclado corre **antes** de toda a cadeia (é o 2.º ramo do `key_input`); sem a cerca
/// de área ele comia o `Delete` da Hierarquia — onde o mesh agora tem linha —, do Flip, da
/// timeline e do Painter sempre que houvesse barro na tela.
#[test]
fn o_delete_sobre_um_painel_nao_e_da_escultura() {
    let arm = KEYS
        .split_once("if code == K::Delete\n")
        .map(|(_, resto)| resto)
        .unwrap_or_default();
    assert!(
        arm.contains("cursor_over_hero_panel"),
        "⛔ o `Delete` da escultura tem de ceder quando o ponteiro esta' sobre um painel"
    );
}
