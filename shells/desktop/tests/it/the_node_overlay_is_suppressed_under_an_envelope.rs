//! **Arch-gate: sob um Envelope, o overlay de NÓS não é desenhado** — você edita a GAIOLA.
//!
//! ## O que este gate protege
//!
//! Quando a seleção é um container de Envelope (ADR-0129), a forma sob a gaiola é DERIVADA: os nós
//! dela são a SAÍDA do warp, não estado editável. Desenhar as alças de nó dela (a) confunde (não se
//! pode arrastá-las) e (b) expõe o handle longo que o refit de Fréchet deixa numa quina CÔNCAVA — o
//! "traço à deriva" que o Enio reportou numa estrela sob envelope (2026-07-24). O idioma de
//! referência (Illustrator/Affinity): com um envelope ativo, os nós do objeto SOMEM.
//!
//! ## Por que um gate de TEXTO
//!
//! A decisão mora no corpo do `render_frame` (`if !envelope_selected`), que exige janela + GPU —
//! nenhum unit test o alcança (irmão do `the_frame_draws_the_live_offset_geometry`). O
//! comportamento de `is_envelope` está gateado no `envelope_live_star_tests`; aqui prova-se a
//! COSTURA: que a chamada de `draw_overlays` de fato pende do gate. Removê-lo compila e devolve a
//! alça à deriva, sem nenhum teste de unidade notar.

/// O QUADRO pela ordem em que corre (`frame_text::render_frame`).
///
/// ⚠️ Desde a OBRA 2 da `line/render-loop` (2026-09-13) o overlay de edição mora na fase
/// `fase_vector_edit_overlay`; lido só no `render_loop/mod.rs`, este gate reprovava sobre produto correcto. A
/// relação guard → chamada é a do texto que corre.
fn src() -> &'static str {
    static FRAME: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    FRAME.get_or_init(crate::frame_text::render_frame)
}

fn at(needle: &str) -> usize {
    src().find(needle).unwrap_or_else(|| {
        panic!(
            "`{needle}` sumiu do render_loop — se foi renomeado, atualize este gate (e confira que \
             sob um envelope os nós da forma não voltaram: `PH2D_BUILD_SMOKE=27`)"
        )
    })
}

/// **O `draw_overlays` pende de `if !envelope_selected`, e `envelope_selected` sai de
/// `is_envelope`.** Tirar o guard devolve as alças da forma derivada à tela (a alça à deriva).
#[test]
fn the_node_overlay_hangs_off_the_envelope_gate() {
    let src = src();
    let sel = at("let envelope_selected");
    let sel_end = src[sel..].find(';').expect("fim do binding") + sel;
    assert!(
        src[sel..sel_end].contains("envelope_gesture::is_envelope"),
        "`envelope_selected` não pergunta a `is_envelope` — se ele deixar de ver o container como \
         envelope, o overlay de nós volta a desenhar a forma DERIVADA:\n{}",
        &src[sel..sel_end]
    );
    let guard = at("if !envelope_selected {");
    let call = at("ph2d_vec_render::draw_overlays(");
    assert!(
        sel < guard && guard < call,
        "o `draw_overlays` não está sob `if !envelope_selected` — as alças de nó da forma sob a \
         gaiola voltariam à tela (o handle longo do refit numa quina côncava = a alça à deriva)"
    );
    // Nada entre o guard e a chamada além de espaço/{ — a chamada é a PRIMEIRA coisa guardada.
    let between = &src[guard + "if !envelope_selected {".len()..call];
    assert!(
        between.trim().is_empty(),
        "há código entre `if !envelope_selected {{` e o `draw_overlays` — o guard pode não estar a \
         guardá-LO:\n{between}"
    );
}
