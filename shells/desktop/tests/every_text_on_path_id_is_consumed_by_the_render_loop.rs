//! **Arch-gate: os ids do Text on Path são CONSUMIDOS pela `render_loop`.**
//!
//! O gate de seam do painel (`ph2d-panel-vector/tests/seam.rs`) prova que o clique chega ao
//! **barramento**. Isso é metade: um id pode chegar ao barramento e **morrer lá**, porque
//! ninguém do outro lado o lê — o botão pinta, arma, despacha, e não acontece nada.
//!
//! É exatamente a classe de bug que o repo já pagou com os botões Undo/Redo da barra (*"o Redo
//! não despachava coisa alguma — pintado, clicável, órfão, com um gate ao lado afirmando que
//! ele estava no store: **registrado ≠ despachado**"*).
//!
//! Nenhum teste de unidade alcança a `render_loop` (ela precisa de janela e GPU), então a
//! prova é sobre o FONTE — o mesmo recurso que a linha de física usou para pinar que o Join
//! não faz fan-out, e que o load da física instala as settings depois do `rebuild`.
//!
//! ⚠️ **Contador de controle positivo:** se o scanner deixar de encontrar os ids por uma
//! mudança de forma (um `match` no lugar do `else if`, um `use` que encurta o caminho), ele
//! passa a guardar nada — e um gate que não vê nada passa sempre. Por isso ele exige encontrar
//! os CINCO, e falha nomeando qual faltou.

use std::fs;

/// Os ids que o painel manda para o barramento, e que a `render_loop` tem de ler.
///
/// O Offset entra: ele é um `ValueChanged`, não um `Click`, e o modo de falha é o mesmo (o
/// slider anda na tela e o documento não muda).
const CONSUMED: &[&str] = &[
    "VECTOR_TEXTPATH_LINK",
    "VECTOR_TEXTPATH_DETACH",
    "VECTOR_TEXTPATH_FLIP",
    "VECTOR_TEXTPATH_FLIP_OFF",
    "VECTOR_TEXTPATH_OFFSET",
];

#[test]
fn every_text_on_path_id_is_read_by_the_render_loop() {
    let src = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/render_loop/mod.rs"
    ))
    .expect("render_loop/mod.rs");

    let missing: Vec<&str> = CONSUMED
        .iter()
        .copied()
        .filter(|id| !src.contains(id))
        .collect();
    assert!(
        missing.is_empty(),
        "estes ids do Text on Path chegam ao barramento e MORREM lá — a `render_loop` nunca os \
         lê, então o controle pinta, arma, despacha e não faz nada: {missing:?}"
    );

    // …e o outro lado: cada um tem de acabar numa PORTA do `vec_text_ride`, não numa ação
    // inventada no meio do laço. As quatro portas são a superfície inteira da feature.
    for door in [
        "vec_text_ride::link(",
        "vec_text_ride::detach(",
        "vec_text_ride::edit(",
    ] {
        assert!(
            src.contains(door),
            "a `render_loop` não chama `{door}` — o comando chegou e não tem quem o execute"
        );
    }
}
