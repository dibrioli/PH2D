//! **O clique no checkbox chega ao MUNDO, e a publicação vem DEPOIS dele** — arch-gate sobre a
//! costura que nenhum unit test alcança (plano UI/UX W3b).
//!
//! A LEI é gateada onde ela mora: os testes do `vec_resize_box_edit` dirigem um `SimWorld` REAL
//! headless e provam o toggle, o destacamento no neutro e a recusa da seleção múltipla; o seam do
//! painel prova que o checkbox está vivo sob o ponteiro. O que nenhum dos dois toca é o
//! `render_loop`, que precisa de `App` + janela — e é lá que se decidem as duas metades que faltam:
//!
//! 1. **O clique é ROTEADO.** Sem o braço, o `PanelEvent::Click` cai no `_ => {}` e o checkbox
//!    acende sob o rato, chega ao barramento, e não faz nada — a forma exata em que os controlos
//!    deste repo apodrecem.
//! 2. **A publicação vem DEPOIS de honrar, e esta é a metade barata de errar.** Publicar primeiro
//!    deixa a caixa a mostrar o estado ANTERIOR por um frame: o artista clica, vê o visto não
//!    mudar, clica outra vez, e desfaz o que acabou de pedir.
//!
//! ⚠️ Nada aqui afirma distância em bytes — a lição de `the_dispatch_is_handed_the_live_geometry`
//! (2026-07-23) é que um proxy posicional expira na wave seguinte. O que se afirma é *qual porta é
//! chamada* e *em que ORDEM relativa*, que é a propriedade.

use std::fs;

fn source() -> String {
    fs::read_to_string("src/render_loop/mod.rs").expect("render_loop/mod.rs")
}

/// **O clique é roteado para a porta.**
#[test]
fn the_click_is_routed_to_the_one_door() {
    let src = source();
    assert!(
        src.contains("ph2d_editor::ids::VECTOR_TRANSFORM_RESIZE_BOX"),
        "o Click do checkbox nao e' reconhecido no roteador — ele acende sob o rato, chega ao \
         barramento e morre no `_ => {{}}`"
    );
    assert!(
        src.contains("crate::vec_resize_box_edit::toggle_resize_box("),
        "o roteador reconhece o clique e NAO chama a porta — o gesto vira um no-op mudo"
    );
}

/// **Honrar vem ANTES de publicar.**
///
/// ⚠️ É a mutação barata: trocar as duas linhas de lugar. Ela não muda um pixel do estado final e
/// faz o checkbox mostrar o valor anterior por um frame — o artista lê isso como *"o clique não
/// pegou"* e clica de novo, desfazendo o que pediu.
#[test]
fn the_world_is_written_before_the_panel_is_told() {
    let src = source();
    let honour = src
        .find("crate::vec_resize_box_edit::toggle_resize_box(")
        .expect("a porta que honra o clique");
    let publish = src
        .find("ph2d_panel_vector::state::set_resize_box(")
        .expect("a publicacao para o painel");
    assert!(
        honour < publish,
        "a publicacao corre ANTES de honrar o clique — a caixa mostra o estado anterior por um \
         frame, e o artista clica duas vezes"
    );
}

/// **A publicação pergunta à MESMA porta que o gizmo honra.**
///
/// ⚠️ Sem isto o painel poderia derivar o estado por conta própria (*"é moldura?"*), e o checkbox
/// mostraria uma resposta que a alça não usa: o artista veria a caixa marcada e os filhos a
/// esticar. A porta única é `ph2d_ecs::resizes_box`, e os dois lados a atravessam —
/// `selected_resize_box` para MOSTRAR, `resizable_frame` para HONRAR.
#[test]
fn both_sides_ask_the_same_door() {
    for (path, what) in [
        ("src/vec_resize_box_edit.rs", "o lado que MOSTRA"),
        ("src/vec_frame_resize.rs", "o lado que HONRA"),
    ] {
        let s = fs::read_to_string(path).unwrap_or_else(|_| panic!("{path}"));
        assert!(
            s.contains("ph2d_ecs::resizes_box("),
            "{what} nao atravessa a porta unica — uma segunda derivacao faria o checkbox e a alca \
             discordarem"
        );
    }
}
