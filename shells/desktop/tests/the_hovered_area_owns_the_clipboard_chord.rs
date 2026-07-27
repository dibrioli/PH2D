//! **ARCH-GATE: a ÁREA SOB O MOUSE é dona do atalho de clipboard (regra do Blender).**
//!
//! Enio, 2026-07-19: *"se estou no modo vector e quero copiar/colar keyframes com o
//! teclado, as formas vetoriais são copiadas e não os keyframes … o atalho funciona onde
//! o mouse se encontra: mouse na timeline ⇒ keys, no canvas ⇒ formas."*
//!
//! A decisão mora em `key_input`, e ela depende de `cursor_over_timeline()`, que lê o
//! `store` DENTRO de `gfx` (janela + GPU) — **inalcançável por um teste headless**, o mesmo
//! muro que `the_undo_preserves_the_vector_selection.rs` documenta. Então o gate lê o
//! FONTE: os blocos de atalho do VETOR (clipboard e Delete) têm de CEDER quando o mouse
//! está sobre a timeline (`!self.cursor_over_timeline()`), e os blocos da TIMELINE têm de
//! vir DEPOIS, para pegar a tecla no fall-through.
//!
//! Um mutante que tire o `!self.cursor_over_timeline()` (o vetor volta a roubar sempre) ou
//! que troque o sinal (`!` some) muda a substring exata e este gate fica VERMELHO.

use std::fs;

fn keyboard_src() -> String {
    fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/input_dispatch/keyboard.rs"
    ))
    .expect("keyboard.rs legível")
}

/// A janela de guarda que PRECEDE uma chamada — o `if … {` acima dela.
fn guard_before<'a>(src: &'a str, needle: &str, window: usize) -> &'a str {
    let at = src
        .find(needle)
        .unwrap_or_else(|| panic!("`{needle}` existe em keyboard.rs"));
    &src[at.saturating_sub(window)..at]
}

/// O bloco de clipboard do VETOR (Ctrl+C/X/V/D/G) cede quando o mouse está na timeline.
#[test]
fn the_vector_clipboard_block_yields_when_the_cursor_is_over_the_timeline() {
    let src = keyboard_src();
    let guard = guard_before(&src, "self.vec_copy()", 900);
    assert!(
        guard.contains("self.vector_tool_active()"),
        "âncora perdida: o bloco de clipboard do vetor não é mais o que precede `vec_copy`"
    );
    assert!(
        guard.contains("!self.cursor_over_timeline()"),
        "o Ctrl+C/X/V/D do VETOR não cede à timeline: copiar keyframes com o mouse na \
         timeline copia as FORMAS (o bug do Enio). Adicione `&& !self.cursor_over_timeline()` \
         à guarda do bloco vetorial de clipboard."
    );
}

/// O Delete do VETOR (apaga forma/vértice) cede quando o mouse está na timeline.
#[test]
fn the_vector_delete_yields_when_the_cursor_is_over_the_timeline() {
    let src = keyboard_src();
    let guard = guard_before(&src, "self.vec_delete_selected_vertex_or_path()", 220);
    assert!(
        guard.contains("!self.cursor_over_timeline()"),
        "o Delete do VETOR não cede à timeline: apagar um keyframe com o mouse na timeline \
         apagaria a FORMA. Adicione `&& !self.cursor_over_timeline()` antes do \
         `vec_delete_selected_vertex_or_path()`."
    );
}

/// Os blocos da TIMELINE vêm DEPOIS dos do vetor — é o fall-through que pega a tecla
/// quando o vetor cede. Se o de clipboard da timeline subisse para antes do vetorial, o
/// vetor (que retorna cedo) nunca cederia a vez.
///
/// ⚠️ **As travas da timeline MUDARAM DE ARQUIVO em 2026-07-27** (integração `line/anim` +
/// `line/physics`: o `keyboard.rs` cruzou o cap de 600 LOC na árvore combinada). Elas agora
/// moram em `keyboard_timeline.rs`, atrás de UMA chamada — e é por isso que a asserção não
/// pode mais ser *"a substring X vem antes da substring Y no mesmo arquivo"*. A PROPRIEDADE
/// é a mesma e continua podendo falhar: no `keyboard.rs`, os blocos do vetor têm de vir
/// antes da **chamada** `self.timeline_key(…)`; mover a chamada para cima dos blocos
/// vetoriais reinstala exatamente o bug do Enio e deixa este gate VERMELHO.
#[test]
fn the_timeline_clipboard_and_delete_blocks_run_after_the_vector_ones() {
    let src = keyboard_src();
    let call = src
        .find("self.timeline_key(")
        .expect("a chamada das travas da timeline mora em keyboard.rs");

    let vec_copy = src
        .find("self.vec_copy()")
        .expect("bloco de clipboard do vetor");
    assert!(
        vec_copy < call,
        "o bloco de clipboard da timeline tem de correr DEPOIS do vetorial (fall-through)"
    );

    let vec_del = src
        .find("self.vec_delete_selected_vertex_or_path()")
        .expect("Delete do vetor");
    assert!(
        vec_del < call,
        "o Delete da timeline tem de correr DEPOIS do do vetor (fall-through)"
    );

    // …e as travas de fato estão do outro lado da chamada — sem isto o gate acima passaria
    // apontando para uma função vazia (a chamada existe, o bloco não).
    let sibling = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/input_dispatch/keyboard_timeline.rs"
    ))
    .expect("keyboard_timeline.rs legível");
    assert!(
        sibling.contains("I::CopySelection"),
        "o bloco de clipboard da timeline sumiu do irmão"
    );
    assert!(
        sibling.contains("TimelineIntent::DeleteSelection"),
        "o Delete da timeline sumiu do irmão"
    );
}
