//! ⭐ **F7: o QuickShape tem DUAS portas na shell, e as duas são chamadas.** O quadro pergunta o
//! segurar (`ph2d_app_vec::quickshape_frame::frame`) com o relógio dos carimbos do ponteiro e o
//! `Shift`, ANTES do perfil de largura (que a partir da forma pede a largura do estilo), e confirma a
//! forma que nasce pelo ouvido; o `Esc` chega ao lápis (`quickshape_frame::escape`).
//!
//! Mutações que têm de sangrar: a chamada do quadro apagada ou depois do `profile_live::arm`; outro
//! relógio; o `Shift` esquecido; o clique apagado; o braço do `Esc` apagado.

use std::fs;

fn src(p: &str) -> String {
    fs::read_to_string(p).unwrap_or_else(|_| panic!("{p}"))
}

#[test]
fn the_frame_asks_the_hold_before_the_width_profile() {
    let s = src("src/render_loop/fase_vector_live_geometry.rs");
    let call = s
        .find("quickshape_frame::frame(")
        .expect("o quadro não pergunta o segurar");
    let arm = s.find("crate::profile_live::arm(").expect("o perfil vivo");
    assert!(
        call < arm,
        "o segurar tem de vir ANTES do perfil de largura"
    );
    let w = &s[call..(call + 340).min(s.len())];
    assert!(
        w.contains("Self::timestamp_ns()"),
        "o relógio do segurar tem de ser o dos carimbos do ponteiro"
    );
    assert!(w.contains("shift_key()"), "o Shift é o regular");
    assert!(
        w.contains("UiSound::Click"),
        "a forma que nasce confirma-se pelo ouvido"
    );
}

#[test]
fn esc_reaches_the_pencil() {
    let s = src("src/input_dispatch/keyboard_escapes.rs");
    let esc = s
        .find("quickshape_frame::escape(")
        .expect("o Esc não chega ao lápis — a dica dele promete-o");
    let pen = s
        .find("self.vec.pen.is_drawing()")
        .expect("o Esc da caneta");
    assert!(esc < pen, "o braço do lápis vem antes do da caneta");
}
