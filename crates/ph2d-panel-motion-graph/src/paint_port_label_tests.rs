//! As provas do rótulo de porta — a derivação, e o facto de alguém a chamar.

use super::PortLabel;

fn label(name: &str) -> String {
    PortLabel::of(name).as_str().to_string()
}

/// **A DERIVAÇÃO, caso a caso** — as formas que o catálogo de facto usa.
///
/// ⚠️ O `in0` é o que obriga a regra do dígito: sem ela lê-se `In0`, que é pior que o nome cru
/// porque parece uma palavra. E o `target_x` é o caso da foto do Enio — as duas portas que eram
/// dois círculos idênticos.
#[test]
fn a_port_name_becomes_a_human_label() {
    assert_eq!(label("target_x"), "Target X");
    assert_eq!(label("target_y"), "Target Y");
    assert_eq!(label("state"), "State");
    assert_eq!(label("obstacle"), "Obstacle");
    assert_eq!(label("forces"), "Forces");
    assert_eq!(label("in0"), "In 0");
    assert_eq!(label("in3"), "In 3");
    assert_eq!(label("select"), "Select");
    assert_eq!(label("anchor_x"), "Anchor X");
    assert_eq!(label("sample_hold"), "Sample Hold");
    assert_eq!(label("shape_w"), "Shape W");
}

/// **NADA ENTRA E NADA REBENTA** — os casos de borda de um caminho de DESENHO, que não pode
/// entrar em pânico por causa de um nome.
#[test]
fn a_degenerate_name_is_never_a_panic() {
    assert_eq!(label(""), "");
    // ⚠️ Um separador NO INÍCIO não abre espaço nenhum (a guarda `len > 0`), então um nome só
    // de separadores dá a string vazia — e não uma fileira de espaços que o painel desenharia.
    assert_eq!(label("_"), "");
    assert_eq!(label("__"), "");
    assert_eq!(label("_x"), "X");
    assert_eq!(label("9"), "9");
    // Mais comprido que o buffer: TRUNCA, e o que sai continua a ser um `&str` válido.
    let long = "a".repeat(120);
    let got = label(&long);
    assert!(got.len() <= 24, "truncou: {}", got.len());
    assert!(got.starts_with('A'));
    // Um byte não-ASCII passa verbatim sem partir o `&str` (o recuo é a string vazia).
    let _ = label("café");
}

/// ⛔⛔ **ALGUÉM TEM DE CHAMAR ISTO.**
///
/// A régua é o CALL SITE, e não a existência da função: sem a chamada em `draw_card` o rótulo
/// não é desenhado, os testes desta unidade continuam todos verdes, e o defeito é exactamente o
/// que o report descrevia — *"em nenhum lugar diz o que é"*. Um censo textual é fraco como
/// oráculo e **exacto** contra esta mutação: apagar a chamada apaga o nome.
///
/// ⚠️ Não há API que deixe um teste perguntar à cena *"que texto ficou aqui?"* (o
/// `paint_text_title` desenha glifos direto no `VectorScene`), então esta é a metade que se
/// consegue provar sem inventar uma camada de inspecção de texto.
#[test]
fn the_card_painter_actually_calls_it() {
    let src = include_str!("paint.rs");
    assert!(
        src.contains("draw_port_labels(ctx, n, view, theme);"),
        "o `draw_card` deixou de escrever os nomes das portas — o cartao volta a ter uma \
         fileira de sockets sem identificacao nenhuma"
    );
}
