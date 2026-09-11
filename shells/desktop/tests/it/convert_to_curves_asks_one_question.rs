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

use std::fs;

fn src(name: &str) -> String {
    fs::read_to_string(format!("{}/src/{name}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// O BOTÃO se oferece pela porta única — e não por uma lista de componentes re-enumerada no
/// `render_loop`. Um `convertible` escrito à mão ali é como as duas regressões nasceram.
#[test]
fn the_convert_button_asks_the_single_door() {
    let render_loop = src("render_loop/mod.rs");
    assert!(
        render_loop.contains("vec_convert::is_convertible"),
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
    let dispatch = src("input_dispatch.rs");
    assert!(
        dispatch.contains("vec_convert::freeze_shape_recipe"),
        "o press das ferramentas de quina deixou de congelar a receita da forma viva. Sem isso \
         o `has_derived_verts` recusa a Shape e a ferramenta fica inerte no vértice dela — \
         pintada, armada e sem efeito."
    );
}
