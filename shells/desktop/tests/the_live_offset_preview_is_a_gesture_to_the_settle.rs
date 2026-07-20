//! **Arch-gate: o preview VIVO do Offset entra no `drawing` do `settle_origins`.**
//!
//! O preview do arrasto reescreve geometria de MUNDO a cada frame — como a caneta — e o
//! `clone_from(&pre)` restaura o `next_id` da cena, então o resultado renasce com o MESMO id
//! (e a MESMA entidade) todo frame. Se o `settle` o assentar no frame 1, o frame 2 desenha
//! mundo × centro = translação DOBRADA (*"pula para o canto direito"*, Enio 2026-07-20).
//!
//! O gate de unidade (`the_live_preview_draws_in_the_same_place_every_frame`) prova o
//! MECANISMO, mas ele espelha a sequência do frame — e espelho não vê a `render_loop` real
//! ([[feedback_harness_reproduces_mechanism_not_context]]). Este irmão cobra o SÍTIO: a
//! lista `drawing` que a `render_loop` passa ao `settle_origins` tem de encadear os
//! `live_paths` da sessão de Offset. Tirar o chain de lá deixa o unit gate verde e o
//! produto vermelho — é exatamente a mutação que só este arquivo sangra.

use std::fs;

#[test]
fn the_render_loops_settle_drawing_includes_the_offset_session() {
    let src = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/render_loop/mod.rs"
    ))
    .expect("render_loop/mod.rs");

    // O trecho entre a construção do `drawing` e a chamada do settle — a lista mora ali.
    let (before_settle, _) = src
        .split_once("crate::vec_transform::settle_origins(sim, vec_scene,")
        .expect("a render_loop chama o settle_origins do vetor");
    let tail = &before_settle[before_settle.len().saturating_sub(1500)..];

    assert!(
        tail.contains("vec_offset_session") && tail.contains("live_paths"),
        "o `drawing` do `settle_origins` não encadeia os `live_paths` da `vec_offset_session` — \
         o settle assenta o preview no frame 1 e o frame 2 desenha mundo × centro (translação \
         dobrada, o \"pula pro canto direito\"). Restaure o `.chain(self.vec_offset_session...)` \
         na lista `drawing`."
    );
}

/// **A janela de RETUNE está costurada no frame.** O gate de unidade
/// (`changing_the_join_after_release_retunes_the_committed_offset`) prova a máquina e o
/// `apply` — espelhando o frame; espelho não vê a `render_loop`. Este irmão cobra o SÍTIO:
/// o release abre a janela (`after_release`), o grab novo a limpa, e o braço `Retune`
/// chama o `apply`. Tirar o bloco deixa os unit gates verdes e os chips mortos de novo.
#[test]
fn the_render_loop_drives_the_retune_window() {
    let src = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/render_loop/mod.rs"
    ))
    .expect("render_loop/mod.rs");

    for needle in [
        "OffsetRetune::after_release",
        "RetuneStep::Retune",
        "win.apply(",
        "self.vec_offset_retune = None",
    ] {
        assert!(
            src.contains(needle),
            "a render_loop perdeu `{needle}` — o retune de Join/Side pós-release (o pedido \
             de 2026-07-20) não chega ao produto: o clique no chip volta a só armar o \
             PRÓXIMO arrasto."
        );
    }
}
