//! Os gates do **GESTO** da rolagem.
//!
//! ⚠️ Eles são ARCH-GATES sobre o fonte, e não testes de comportamento, pela razão de sempre nesta
//! shell: a decisão mora dentro do `on_mouse_wheel`, que precisa de `gfx` (janela + GPU) para
//! sequer converter o cursor para mundo. Nenhum teste de unidade a alcança — e um gate que
//! afirmasse a mesma coisa chamando a função por dentro estaria a testar a função, não a ORDEM em
//! que ela é perguntada, que é o que decide se a roda dá zoom ou rola.

/// O fonte do despachante de entrada.
fn dispatch_src() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/input_dispatch.rs"
    ))
    .expect("o despachante existe")
}

/// **A moldura é perguntada ANTES do zoom.**
///
/// ⚠️ Depois dele a câmera já se mexeu quando a moldura for consultada, e o artista veria a lista
/// rolar *e* a cena aproximar no mesmo tique — o defeito que nenhum gate de unidade pode ver,
/// porque as duas metades funcionam isoladas.
#[test]
fn the_frame_is_asked_before_the_camera_zooms() {
    let s = dispatch_src();
    let ask = s
        .find("wheel_scrolls_a_frame")
        .expect("a roda pergunta pela moldura");
    let zoom = s.find("camera.zoom(factor)").expect("o zoom da camera");
    assert!(
        ask < zoom,
        "a moldura tem de ser perguntada ANTES do zoom (pergunta em {ask}, zoom em {zoom})"
    );
}

/// **Um painel continua a ganhar da moldura.**
///
/// ⚠️ Controle POSITIVO da ordem acima: o `over_panel` é decidido no topo do handler e as duas
/// perguntas seguintes (escultura e moldura) são guardadas por ele. Sem esta afirmação, mover a
/// pergunta da moldura para o topo passaria no gate anterior e faria a roda sobre um painel rolar
/// a arte por baixo dele.
#[test]
fn a_panel_still_wins_the_wheel() {
    let s = dispatch_src();
    let i = s
        .find("wheel_scrolls_a_frame")
        .expect("a roda pergunta pela moldura");
    // A linha da pergunta tem de trazer a guarda do painel consigo.
    let line_start = s[..i].rfind('\n').map_or(0, |n| n + 1);
    let line = &s[line_start..i];
    assert!(
        line.contains("!over_panel"),
        "a pergunta da moldura tem de ser guardada por `!over_panel`, e a linha diz: {line:?}"
    );
}
