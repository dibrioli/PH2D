//! **Arch-gate: a largura chega à seleção quando é AUTORADA, não quando um slider é arrastado.**
//!
//! Enio, 2026-08-01: *"modificar o valor da caixa de texto ao lado do slider Width não muda o
//! stroke"*. O mecanismo era exato — o bridge perguntava ao store se o slider de Width estava em
//! `Dragging`, e digitar na caixa escreve o VALOR do slider sem nunca lhe mudar o ESTADO. A
//! largura chegava ao tool (o seam painel→tool está provado no `seam.rs` do painel) e morria ali:
//! a forma selecionada nunca era reescrita.
//!
//! # Por que um arch-gate, e não um teste de comportamento
//!
//! ⚠️ A decisão mora dentro do `vector_bridge::dispatch`, que exige `gfx` (janela + GPU) — nenhum
//! teste de unidade a alcança. Os gates de unidade provam a CANALIZAÇÃO (`restyle_selected_strokes`
//! honra o `Option` que recebe — `the_width_is_rewritten_only_when_the_caller_hands_one`) e o TOOL
//! (o one-shot acende no evento de largura e não no de cor — `tool_tests`); **nenhum dos dois vê
//! qual dos dois valores o bridge de facto entrega**, e era exatamente aí que o defeito vivia.
//!
//! # As quatro afirmações
//!
//! 1. o `width_authored` é drenado do **TOOL** (quem recebeu o evento sabe que houve autoria);
//! 2. é ele que decide a largura entregue ao `restyle_selected_strokes`;
//! 3. é ele, também, que faz o **detector** contar a diferença de largura — sem isso o
//!    `will_change` dá falso num traço em que só a largura mudou, e o restyle nem roda;
//! 4. ~~o `width_dragging` **continua** a decidir o agrupamento do undo~~ — ⛔ MORREU em 2026-09-12
//!    (`line/render-loop`, A9): o único leitor desse agrupamento era a `History` do vetor, uma pilha
//!    que o Ctrl+Z nunca leu. O passo de undo é o da fila global, por diff do quadro, e o
//!    `width_dragging` saiu do bridge com ela. As duas AUSÊNCIAS acima continuam a guardar a
//!    regressão (a largura e o detector não podem voltar a perguntar pelo arrasto).

const SRC: &str = include_str!("../../../../crates/ph2d-app-vec/src/vector_bridge.rs");

/// Onde `needle` aparece — falha nomeando quem sumiu (controle positivo: um `find` que não casa é
/// um gate que passaria por vácuo).
fn at(needle: &str) -> usize {
    SRC.find(needle)
        .unwrap_or_else(|| panic!("a âncora `{needle}` sumiu do `vector_bridge`"))
}

#[test]
fn the_authored_width_comes_from_the_tool_not_from_the_slider_state() {
    at("let width_authored = tool.take_width_authored();");
}

#[test]
fn the_width_handed_to_the_restyle_is_the_authored_one() {
    at("width_authored.then_some(new_w)");
    assert!(
        !SRC.contains("width_dragging.then_some("),
        "a largura entregue ao `restyle_selected_strokes` voltou a sair do ESTADO do slider — \
         digitar na caixa numérica escreve o valor do slider sem o pôr em `Dragging`, então a \
         forma selecionada deixa de ser reescrita (o defeito reportado em 2026-08-01)"
    );
}

#[test]
fn the_change_detector_counts_a_width_that_was_authored() {
    at("width_authored && (s.width - new_w).abs()");
    assert!(
        !SRC.contains("width_dragging && (s.width"),
        "o detector voltou a perguntar pelo ARRASTO: num traço em que só a largura mudou o \
         `will_change` daria falso e o restyle nem rodaria — a metade silenciosa do mesmo defeito"
    );
}

// ⛔ `the_undo_session_still_asks_whether_a_drag_is_in_flight` MORREU em 2026-09-12 (ver o item 4 do
// cabeçalho): o `session` que ele lia só agrupava passos da `History` do vetor, que nenhum Ctrl+Z lia.
