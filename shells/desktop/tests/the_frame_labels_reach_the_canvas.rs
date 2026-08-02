//! **Arch-gate: a shell PUBLICA as etiquetas de moldura todo frame.**
//!
//! Enio, 2026-08-01: *"precisamos de uma pequena label no topo esquerdo dos frames"*. O produtor
//! (`vec_frame_labels`) e o pintor (`ph2d_editor::frame_label`) têm gates próprios; **nenhum dos
//! dois vê a ponte entre eles**, que mora no `render_loop` e exige `gfx` (janela + GPU). Sem esta
//! asserção os dois lados ficam verdes e a tela fica sem etiqueta nenhuma.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

#[test]
fn the_render_loop_publishes_the_frame_labels_it_computes() {
    assert!(
        SRC.contains("hero.gizmo.frame_labels = crate::vec_frame_labels::frame_labels("),
        "o `render_loop` deixou de publicar as etiquetas de moldura — o produtor e o pintor \
         continuam corretos e a tela fica muda"
    );
}

/// A lista publicada é a da SELEÇÃO viva — é isso que acende a etiqueta da moldura selecionada.
#[test]
fn the_published_labels_know_what_is_selected() {
    let i = SRC
        .find("hero.gizmo.frame_labels = crate::vec_frame_labels::frame_labels(")
        .expect("a publicação sumiu");
    let call = &SRC[i..(i + 400).min(SRC.len())];
    assert!(
        call.contains("&sel"),
        "a publicação deixou de receber a seleção — nenhuma etiqueta voltaria a acender"
    );
}
