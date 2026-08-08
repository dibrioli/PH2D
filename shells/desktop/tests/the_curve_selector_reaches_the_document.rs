//! **A fiação do SELETOR DE CURVA** (plano UI/UX W7) — arch-gate sobre a costura que nenhum
//! teste de unidade alcança.
//!
//! # Por que este arquivo existe
//!
//! As duas metades do seletor têm gates próprios e headless: a porta que resolve o chip
//! (`easing_pick_for_id`), a que compõe o pick sobre a curva do documento (`easing_with`), e a
//! que publica a curva para o painel (`publish`). O que nenhuma delas vê é se o **render loop**
//! as chama — e ele vive dentro de um laço que exige janela.
//!
//! ⚠️ **O modo de falha é exatamente o que esta wave veio curar, uma volta acima:** o
//! `set_easing` existia, era público, estava testado, e **nenhum caminho de produto o chamava**.
//! Apagar o bloco que o honra devolveria o produto a esse estado com a bateria inteira VERDE —
//! porque todo gate de unidade continuaria a provar as peças.
//!
//! ⚠️ **Cada asserção é sobre uma PROPRIEDADE, nunca sobre distância em bytes** — a cicatriz que
//! esta linha já pagou duas vezes (2026-07-23), e o motivo pelo qual não há aqui nenhuma janela
//! de N caracteres depois de uma âncora.

use std::fs;

/// O `render_loop` inteiro. Um `expect` em vez de `unwrap` porque um caminho errado aqui é um
/// gate que passa a medir o vazio — o controle positivo abaixo é a outra metade dessa defesa.
fn render_loop() -> String {
    fs::read_to_string("src/render_loop/mod.rs").expect("o render_loop mudou de sitio")
}

/// **CONTROLE POSITIVO** — se o ficheiro deixar de conter a seção de estados, todo gate deste
/// arquivo passaria a afirmar coisas sobre um texto que não é o produto.
#[test]
fn the_scan_is_looking_at_the_states_wiring() {
    let src = render_loop();
    assert!(
        src.contains("pending_ui_state_duration"),
        "o render_loop nao tem mais a seccao de estados — os gates deste ficheiro ficaram cegos"
    );
}

/// **O clique no chip é CAPTURADO.** Sem isto o chip pinta, acende sob o rato, o Click atravessa
/// o barramento do painel e morre na shell — com o log a dizer `unhandled event`, que é
/// literalmente o que aconteceu no primeiro smoke da simetria.
#[test]
fn the_click_on_a_curve_chip_is_captured() {
    let src = render_loop();
    assert!(
        src.contains("easing_pick_for_id"),
        "ninguem no render_loop pergunta se o id clicado e' um chip de curva"
    );
    assert!(
        src.contains("pending_ui_easing = Some(p)"),
        "o pick e' resolvido e nao e' guardado — ele seria descartado no mesmo frame"
    );
}

/// **O pick é HONRADO no documento, e composto sobre a curva que ele já tem.**
///
/// ⚠️ As duas metades são separadas de propósito. `set_easing` sozinho provaria que *alguma* curva
/// é escrita; o `easing_with` é o que prova que a metade **não clicada** vem do documento em vez
/// de ser inventada — que é a diferença entre trocar a família e apagar a direção do artista.
#[test]
fn the_pick_reaches_the_document_composed_over_what_is_there() {
    let src = render_loop();
    // ⚠️ A âncora é a CHAMADA (`ui_states.set_easing(`), nunca o nome nu. A primeira versão deste
    // gate procurava `set_easing` e passava sob a mutação — porque o comentário que explica o
    // bloco, oito linhas acima, contém a palavra. *Um oráculo que casa com a documentação de si
    // mesmo não está a olhar para o produto* (a cicatriz do `stamps: media`, `line/Painter`).
    assert!(
        src.contains("ui_states.set_easing("),
        "o pick nunca chega ao documento — o seletor seria decorativo, que e' o estado \
         que esta wave veio curar"
    );
    assert!(
        src.contains("easing_with(cur, pick)"),
        "a curva nova nao e' composta sobre a atual — a metade nao clicada estaria a ser inventada"
    );
}

/// **A curva é escrita no hospedeiro ÚNICO**, pela mesma guarda da duração ao lado.
///
/// Sem ela, uma seleção múltipla escreveria a curva em… qual? Escolher em silêncio é como um
/// ajuste acaba pendurado no objeto errado — a razão escrita no `host()`.
#[test]
fn the_curve_is_written_to_the_single_host() {
    let src = render_loop();
    let honour = src
        .split_once("if let Some(pick) = pending_ui_easing")
        .expect("o bloco que honra o pick desapareceu")
        .1;
    let head: String = honour.chars().take_while(|c| *c != '{').collect();
    assert!(
        head.contains("[host] = self.vec_pen.selected_paths()"),
        "o bloco do pick nao e' guardado pelo hospedeiro unico: {head:?}"
    );
}
