//! **Arch-gate: quem pergunta "isto é convertível?" usa a PORTA ÚNICA.**
//!
//! O `convertible` do "Convert to Curves" já apodreceu DUAS vezes por ENUMERAR as fontes de
//! geometria viva em vez de perguntar uma vez: ficou desligado num caminho só-efeitos, e depois
//! num caminho só-quinas — as duas em silêncio, com todo unit test do motor verde. A cura foi o
//! `vec_convert::is_convertible`; este gate cobra que os dois consumidores de fato o chamem.
//! [[feedback_a_condition_that_enumerates_its_readers_rots]]
//!
//! É um contador de símbolos e vale ZERO como auditoria — mas o que ele guarda é a omissão
//! MECÂNICA (alguém re-inline a pergunta e a resposta volta a divergir), que é exatamente o modo
//! como esta política falhou. A semântica está nos gates de `vec_convert::tests`, que rodam a
//! conversão de verdade sobre cada fonte.

/// O BOTÃO se oferece pela porta única — e não por uma lista de componentes re-enumerada no
/// `render_loop`. Um `convertible` escrito à mão ali é como as duas regressões nasceram.
#[test]
fn the_convert_button_asks_the_single_door() {
    // ⚠️⚠️ O QUADRO pela ordem em que corre (`frame_text::render_frame`), SEM COMENTÁRIOS, e a agulha é a CHAMADA. Até
    // à `line/render-bodies` (2026-09-13) este gate lia o `render_loop/mod.rs` inteiro, e a porta só lá estava num
    // COMENTÁRIO (`// … pela porta ÚNICA (`vec_convert::is_convertible`)`): a chamada tinha-se mudado para a
    // `fase_selection_mirror_convert_envelope` na OBRA 2, e o gate ficou verde sobre a prosa que explica a regra.
    let render_loop: String = crate::frame_text::render_frame()
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        render_loop.contains("vec_convert::is_convertible("),
        "o `convertible` do render_loop deixou de usar `vec_convert::is_convertible`. Se ele \
         voltou a enumerar as fontes (VecShape / effects / …), a resposta do BOTÃO e a do \
         CONVERSOR divergem outra vez — e o sintoma é um botão desligado sobre algo que o \
         conversor sabe congelar, sem erro nenhum."
    );
}

/// O GESTO de quina congela a receita de uma forma viva antes de escrever o raio. Sem esta
/// chamada, o Fillet/Chamfer volta a RECUSAR o vértice de uma Shape (o `has_derived_verts` a
/// barra) — que foi como o Enio o encontrou: *"nao funciona diretamente nos vertex das shapes"*.
#[test]
fn the_corner_gesture_freezes_a_live_shape_recipe() {
    let dispatch = crate::input_text::dispatch();
    assert!(
        dispatch.contains("vec_convert::freeze_shape_recipe"),
        "o press das ferramentas de quina deixou de congelar a receita da forma viva. Sem isso \
         o `has_derived_verts` recusa a Shape e a ferramenta fica inerte no vértice dela — \
         pintada, armada e sem efeito."
    );
}
