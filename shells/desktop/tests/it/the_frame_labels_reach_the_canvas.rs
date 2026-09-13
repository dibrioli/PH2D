//! **Arch-gate: a shell PUBLICA as etiquetas de moldura todo frame.**
//!
//! Enio, 2026-08-01: *"precisamos de uma pequena label no topo esquerdo dos frames"*. O produtor
//! (`vec_frame_labels`) e o pintor (`ph2d_editor_core::frame_label`) têm gates próprios; **nenhum dos
//! dois vê a ponte entre eles**, que mora no `render_loop` e exige `gfx` (janela + GPU). Sem esta
//! asserção os dois lados ficam verdes e a tela fica sem etiqueta nenhuma.

/// O QUADRO pela ordem em que corre (`frame_text::render_frame`).
///
/// ⚠️ Desde a OBRA 2 da `line/render-loop` (2026-09-13) a publicação das etiquetas mora na fase
/// `fase_vector_tokens_and_labels`; lida só no `render_loop/mod.rs`, os dois testes reprovavam sobre produto
/// correcto.
fn src() -> &'static str {
    static FRAME: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    FRAME.get_or_init(crate::frame_text::render_frame)
}

#[test]
fn the_render_loop_publishes_the_frame_labels_it_computes() {
    assert!(
        src().contains("hero.gizmo.frame_labels = crate::vec_frame_labels::frame_labels("),
        "o `render_loop` deixou de publicar as etiquetas de moldura — o produtor e o pintor \
         continuam corretos e a tela fica muda"
    );
}

/// A lista publicada é a da SELEÇÃO viva — é isso que acende a etiqueta da moldura selecionada.
#[test]
fn the_published_labels_know_what_is_selected() {
    let src = src();
    let i = src
        .find("hero.gizmo.frame_labels = crate::vec_frame_labels::frame_labels(")
        .expect("a publicação sumiu");
    let call = &src[i..(i + 400).min(src.len())];
    assert!(
        call.contains("&sel"),
        "a publicação deixou de receber a seleção — nenhuma etiqueta voltaria a acender"
    );
}
