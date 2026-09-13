//! ⭐⭐ **O QUADRO PUBLICA O QUE O PINTOR MEDIU** — o chip e o rectângulo do menu de vistas.
//!
//! ⚠️ **Este gate mudou-se de `ph2d-app-sculpt3d` para cá em 2026-09-11 (W2/L3-B).** Ele mede
//! FONTE do `render_loop`, que é da shell, e estava na crate só porque a família ainda vivia
//! aqui dentro. *Um gate mora ao lado do que ele mede* — senão o caminho relativo dele é
//! corrigido a cada mudança de casa, e o modo de falha (o `include_str!` que não lê) só
//! aparece quando alguém compila os testes.

/// ⭐⭐ **O QUADRO PUBLICA O QUE O PINTOR MEDIU** — o chip e o rectângulo do menu.
///
/// ⛔ **Sem isto o botão é invisível ao ponteiro:** a largura do chip é a do
/// TEXTO e só o pintor a mede, então descartar o retorno do `paint_view_label`
/// deixa a lista de alvos vazia — o nome aparece na tela e o clique atravessa-o.
/// *É a forma exacta do report, e nada além de um censo a apanha: o gate de
/// registo mede ids, e aqui não há id nenhum.*
#[test]
fn o_quadro_publica_o_chip_e_o_rectangulo_que_o_pintor_mediu() {
    // ⚠️⚠️ **O caminho mudou DUAS vezes em dois dias, e é isso que o corrigiu de vez.** Na
    // Fase A ele virou `../render_loop/mod.rs` (o ficheiro desceu um nível ao entrar em
    // `src/sculpt3d/`); na Fase B a família saiu para uma crate e o `render_loop` ficou —
    // e um `include_str!` de uma crate para dentro de uma shell é uma seta que não devia
    // existir. ⇒ o gate mudou-se para o lado que MEDE: ele afirma uma propriedade do
    // `render_loop`, e agora vive ao lado dele.
    //
    // ⚠️ **E «o quadro» deixou de ser um ficheiro** (OBRA 2 da `line/render-loop`, 2026-09-13): o desenho do
    // modelador 3D mudou-se para a fase `fase_field3d_smoke_draw`, e o gate lê o texto EMENDADO do quadro
    // (`frame_text::render_frame`) — a ordem e a distância entre a chamada e a publicação são as de execução.
    let fonte = crate::frame_text::render_frame();
    for (chamada, porta) in [
        ("paint_view_label(", "note_view_labels("),
        ("paint_view_menu(", "note_view_menu_rect("),
    ] {
        let at = fonte
            .find(chamada)
            .unwrap_or_else(|| panic!("controlo positivo: `{chamada}` sumiu do quadro"));
        // A publicação tem de vir depois da chamada, e perto dela.
        let depois = &fonte[at..];
        assert!(
            depois.find(porta).is_some_and(|d| d < 1500),
            "o quadro chama `{chamada}` e nao publica o resultado por `{porta}` -- o alvo do \
             clique fica vazio e o nome aparece na tela com o clique a atravessa'-lo"
        );
    }
}
