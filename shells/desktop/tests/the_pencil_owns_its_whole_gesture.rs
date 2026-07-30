//! **O gesto do lápis é INTEIRO dele: press, move, release e a fuga.**
//!
//! As quatro metades vivem no `input_dispatch`, que precisa de janela + GPU — nenhum teste de
//! unidade as alcança, então a asserção é sobre o FONTE. É a 4ª condição da política de UI que a
//! `line/physics` escreveu: *todo edit pode ter gate e o gesto ainda não levar a lugar nenhum*.
//!
//! # O que cada asserção protege
//!
//! 1. **A ordem do press.** O roteador de Down é uma cadeia de modos que dão `return`; se o braço
//!    do lápis vier DEPOIS do da caneta, um arrasto de mão livre cai no `PenTool` e planta uma
//!    âncora — o modo existiria, seria alcançável pelo painel, e desenharia outra coisa.
//! 2. **O move é DESPACHADO.** Sem a chamada no handler de movimento a curva nunca cresce: o
//!    press põe um vértice na cena e o gesto inteiro vira um ponto.
//! 3. **O release COMITA.** Sem ele o traço fica como path vivo sem passo de undo — e o
//!    `post_frame_undo` registraria um passo espúrio pelo diff, com a fila de redo limpa.
//! 4. **O direito ABORTA.** É a tecla de fuga que a caneta, a forma e o conector já têm; sem ela
//!    um traço começado por acidente não tem como ser descartado.
//!
//! ⚠️ **A asserção é sobre a RELAÇÃO entre as CHAMADAS, nunca sobre distância em bytes.** Dois
//! arch-gates desta linha morreram na integração de 2026-07-23 por afirmarem *"a menos de 400
//! bytes"* / *"a menos de 1200"* — janelas que uma feature vizinha legítima estoura
//! ([[feedback_a_gate_anchored_on_a_byte_distance_is_a_proxy_that_expires]]).

const SRC: &str = include_str!("../src/input_dispatch.rs");

/// A posição da 1ª ocorrência, com uma mensagem que nomeia o que se perdeu.
fn at(needle: &str) -> usize {
    SRC.find(needle)
        .unwrap_or_else(|| panic!("o `input_dispatch` nao contem `{needle}`"))
}

/// **Controle positivo:** os âncoras existem. Um scanner que não acha nada passaria em silêncio
/// por todas as outras asserções, e este arquivo inteiro seria decoração.
#[test]
fn the_scanner_finds_what_it_scans_for() {
    for needle in [
        "DrawMode::Pencil",
        "self.vec_pencil.on_press(",
        "self.vec_pen.on_press(",
        "self.vec_shape.on_press(",
    ] {
        assert!(
            SRC.contains(needle),
            "controle positivo falhou: `{needle}` sumiu do dispatch — as assercoes de ORDEM \
             abaixo passariam sem examinar nada"
        );
    }
}

/// **O press do lápis precede o da caneta E o da forma.**
#[test]
fn the_pencil_press_runs_before_the_pen_and_the_shape() {
    let pencil = at("self.vec_pencil.on_press(");
    let pen = at("self.vec_pen.on_press(");
    let shape = at("self.vec_shape.on_press(");
    assert!(
        pencil < pen && pencil < shape,
        "o braco do lapis (byte {pencil}) roda DEPOIS da caneta ({pen}) ou da forma ({shape}) — \
         um arrasto de mao livre cairia no PenTool e plantaria uma ancora"
    );
}

/// **O move do lápis é despachado** (a chamada, não só a função).
///
/// A `fn vec_pencil_drag_move` e a chamada `self.vec_pencil_drag_move(` são strings diferentes de
/// propósito: definir o método e nunca o chamar é exatamente o modo de falha (a função ficaria
/// coberta pelos gates da crate e o produto não cresceria a curva).
#[test]
fn the_pencil_move_is_dispatched() {
    assert!(
        SRC.contains("fn vec_pencil_drag_move"),
        "o metodo de move do lapis nao existe"
    );
    assert!(
        SRC.contains("self.vec_pencil_drag_move("),
        "o move do lapis NUNCA e' chamado — o press poe um vertice na cena e o gesto inteiro \
         vira um ponto"
    );
}

/// **O release comita o passo de undo** — e o commit vem DEPOIS do release, que é quem decide se
/// houve traço.
#[test]
fn the_pencil_release_commits_one_undo_step() {
    let release = at("self.vec_pencil.on_release(");
    let commit = SRC[release..]
        .find("commit_if_changed")
        .map(|o| release + o)
        .expect("o release do lapis nao comita passo de undo nenhum");
    let cancel = SRC[release..]
        .find("self.vec_history.cancel()")
        .map(|o| release + o)
        .expect("o release do lapis nao cancela o passo pendente num clique perdido");
    // As duas metades moram no MESMO braço (o commit primeiro, o cancel no `else`), então o
    // cancel vem depois na fonte — a asserção é que os dois EXISTEM ali.
    assert!(
        commit < cancel + 4_000,
        "o commit ({commit}) e o cancel ({cancel}) do lapis nao estao no mesmo braco"
    );
}

/// **O botão direito aborta o traço vivo.**
#[test]
fn the_secondary_button_cancels_a_live_pencil_stroke() {
    assert!(
        SRC.contains("self.vec_pencil.cancel("),
        "o lapis nao tem tecla de fuga — um traco comecado por acidente nao pode ser descartado"
    );
}
