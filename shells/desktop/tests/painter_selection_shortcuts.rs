//! **Os atalhos da seleção do Painter têm um dono, e ele é o MODO** — arch-gate sobre o fonte.
//!
//! Enio, 2026-08-07: a seleção não tinha atalho nenhum; o painel era a única porta. Ctrl+X/C/V/A/D e
//! Ctrl+Shift+I agora existem — e cada um deles **já tem outro dono no app**: Ctrl+A é *selecionar
//! todos os nós* do vetor, Ctrl+C/V são o clipboard do grafo de nós e da timeline.
//!
//! ⚠️ **O que torna isso seguro é a guarda `is_selection_mode`**, e ela não é observável por um teste
//! de unidade: a cadeia mora no `App`, que exige janela. Sem este gate, apagar a guarda deixaria a
//! workspace inteira VERDE e roubaria o Ctrl+A do vetor em silêncio.

use std::fs;

const CHAIN: &str = "shells/desktop/src/input_dispatch/keyboard_painter.rs";
const CALLER: &str = "shells/desktop/src/input_dispatch/keyboard.rs";

fn read(rel: &str) -> String {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("nao consegui ler {}: {e}", p.display()))
}

/// **A cadeia recusa antes de tocar em qualquer verbo quando o modo não é Selection.**
///
/// O oráculo é POSICIONAL, e é o que importa: a guarda tem de estar ANTES do `match` que despacha —
/// uma guarda depois dele já teria cortado a seleção do vetor.
///
/// **Mutação que sangra:** tirar o `if !painter.is_selection_mode()`.
#[test]
fn the_clipboard_chain_is_gated_on_the_selection_mode() {
    let src = read(CHAIN);
    let start = src
        .find("fn painter_selection_clipboard_chain")
        .expect("a cadeia de clipboard existe");
    let body = &src[start..];
    let guard = body
        .find("is_selection_mode()")
        .expect("a cadeia PERGUNTA pelo modo antes de agir");
    let dispatch = body
        .find("KeyCode::KeyX")
        .expect("a cadeia despacha o Ctrl+X");
    assert!(
        guard < dispatch,
        "a guarda de modo tem de correr ANTES do despacho, senao o Ctrl+A do vetor ja foi roubado"
    );
}

/// **Ctrl exigido, e Ctrl+I só com Shift.** Sem a primeira, um `C` nu cortaria; sem a segunda, o
/// Ctrl+I (que outros donos usam) seria engolido.
///
/// **Mutação que sangra:** trocar `if !ctrl { return false; }` por nada, ou tirar o `if shift`.
#[test]
fn the_chain_demands_ctrl_and_reserves_plain_ctrl_i() {
    let src = read(CHAIN);
    let start = src.find("fn painter_selection_clipboard_chain").unwrap();
    let body = &src[start..];
    assert!(
        body.contains("if !ctrl {"),
        "sem modificador a tecla nua tem de cair fora"
    );
    assert!(
        body.contains("KeyCode::KeyI if shift"),
        "o inverter e Ctrl+SHIFT+I; um Ctrl+I nu nao e nosso"
    );
}

/// **A cadeia é CHAMADA, e depois da cadeia do Delete.** Uma função que ninguém chama é um atalho que
/// não existe — o modo de falha exato que o `keyboard_painter` já documenta para o Delete.
///
/// **Mutação que sangra:** apagar a chamada, ou movê-la para antes do `painter_delete_chain`.
#[test]
fn the_chain_is_called_after_the_delete_chain() {
    let src = read(CALLER);
    let del = src
        .find("self.painter_delete_chain(")
        .expect("a cadeia do Delete e chamada");
    let clip = src
        .find("self.painter_selection_clipboard_chain(")
        .expect("a cadeia de clipboard e CHAMADA (senao o atalho nao existe)");
    assert!(
        del < clip,
        "a ordem declarada e Delete primeiro; e ela que impede a proxima tecla de nascer ambigua"
    );
}
