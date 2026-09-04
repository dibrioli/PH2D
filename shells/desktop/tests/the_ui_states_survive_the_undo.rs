//! **ARCH-GATE: os ESTADOS de UI atravessam o undo nas DUAS direções** (plano UI/UX W7).
//!
//! O `ProjectState` é a unidade do undo E do save, e o `ui_states` é *plain data* nele — como o
//! `guides`. Plain data não tem ponte a reconstruir, então o preço dela é uma linha em cada
//! direção: o `capture` guarda, o `apply_project` copia de volta.
//!
//! ⚠️ **Uma linha esquecida aqui não falha em lado nenhum.** A mutação que apaga a cópia do
//! `apply_project` foi corrida contra a suíte inteira do shell — **2031 testes** — e **sobreviveu**:
//! o campo continua a viajar no arquivo, o round-trip continua verde, e o que quebra é só o
//! comportamento (um Ctrl+Z deixa a tabela obsoleta; gravar um estado deixa de desfazer). É o
//! buraco que este arquivo existe para tapar.
//!
//! ⚠️ **E por que um arch-gate e não um teste de comportamento:** `apply_project` lê `self.gfx`,
//! que é janela + GPU, então headless ele retorna no primeiro `let Some(gfx)` e nenhuma asserção
//! sobre o efeito é alcançável. É o mesmo muro que `the_hovered_area_owns_the_clipboard_chord.rs`
//! e `the_undo_preserves_the_vector_selection.rs` documentam.

use std::fs;

// ⚠️⚠️ **O PAR, não um ficheiro.** A `App` que opera a fila mudou-se para o irmão
// `undo_app.rs` na integração de 2026-09-04 (tecto de LOC estourado pela SOMA de duas
// linhas), e todo gate que lia só `undo.rs` ficou a afirmar sobre o ficheiro errado — em
// silêncio no dia seguinte, se a lei ainda lá estivesse. ⇒ *um gate que PARSEIA o fonte lê
// a família inteira, nunca um nome de ficheiro.*
fn undo_src() -> String {
    let mut s = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/undo.rs"))
        .expect("undo.rs legível");
    s.push_str(
        &fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/undo_app.rs"))
            .expect("undo_app.rs legível"),
    );
    s
}

/// **CONTROLE POSITIVO.** As duas âncoras que os gates abaixo procuram existem — sem isto, um
/// rename de campo deixaria os dois verdes por vácuo, a afirmar coisas sobre um arquivo que já
/// não fala delas.
#[test]
fn the_undo_still_carries_the_plain_data_of_the_document() {
    let src = undo_src();
    assert!(
        src.contains("pub(crate) guides: ph2d_guides::GuideSet,"),
        "o `guides` saiu do ProjectState — as ancoras destes gates descrevem outro arquivo"
    );
    assert!(
        src.contains("pub(crate) ui_states: ph2d_ui_state::StateSets,"),
        "o `ui_states` saiu do ProjectState"
    );
}

/// **O `capture` GUARDA os estados.** Sem isto o undo grava um passo que já perdeu a tabela, e o
/// primeiro Ctrl+Z a apaga.
#[test]
fn capturing_a_step_takes_the_ui_states_with_it() {
    let src = undo_src();
    assert!(
        src.contains("ui_states: ui_states.clone(),"),
        "o `ProjectState::capture` nao guarda os estados de UI: todo passo de undo nasce sem eles"
    );
}

/// **O `apply_project` DEVOLVE os estados.** É a metade que a mutação provou não estar coberta por
/// mais nada.
#[test]
fn restoring_a_step_puts_the_ui_states_back() {
    let src = undo_src();
    let at = src
        .find("fn apply_project")
        .expect("o `apply_project` existe em undo.rs");
    let body = &src[at..];
    assert!(
        body.contains("gfx.ui_states = state.ui_states.clone();"),
        "o `apply_project` nao devolve os estados de UI ao documento: um Ctrl+Z deixa a tabela \
         obsoleta, e gravar um estado deixa de desfazer"
    );
    // …e ele o faz ao lado do irmão que já estava certo, que é onde a próxima pessoa vai procurar.
    let guides_at = body.find("gfx.guides = state.guides.clone();");
    let states_at = body.find("gfx.ui_states = state.ui_states.clone();");
    assert!(
        matches!((guides_at, states_at), (Some(g), Some(s)) if s.abs_diff(g) < 200),
        "a copia dos estados desgarrou-se da do `guides` — o plain data do documento e' um \
         assunto so', e separa-lo e' como o proximo campo nasce sem uma das metades"
    );
}
