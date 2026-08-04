//! **Arch-gate: os botões Arrange escrevem na ÁRVORE, e o Z-index vem da mesma porta.**
//!
//! # Porque é um arch-gate, e não um gate de unidade
//!
//! Os gates de `vec_entities::zorder::arrange_tests` provam a PORTA — que `reorder` move a forma
//! na pilha dos irmãos, que o `z_index` conta a partir do fundo. Todos passariam com a **fiação
//! arrancada**: o clique a cair no `VecScene::reorder_path` (que mexe na ordem do VETOR da cena, e
//! essa é reescrita a cada frame pela projeção da árvore — ADR-0110) deixaria os quatro botões
//! exactamente como estavam antes desta wave: **acendem, mexem, e o frame seguinte desfaz**.
//!
//! Essa metade vive dentro do laço de frame, que exige janela — nenhum teste de unidade a alcança.
//! É a mesma classe do `a_placed_instance_lands_a_screen_step_from_its_main`.

use std::fs;

fn src(name: &str) -> String {
    fs::read_to_string(format!("{}/src/{name}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// **O clique cai na porta da ÁRVORE.**
#[test]
fn the_reorder_click_lands_on_the_tree_door() {
    let s = src("render_loop/mod.rs");
    let at = s
        .find("if let Some(order) = pending_vec_reorder")
        .expect("o sítio que honra o Arrange mudou de forma — reancore este gate");
    // A janela acaba na PRÓPRIA chamada que ela julga: ancorar num vizinho é o proxy que expira.
    let block = &s[at..];
    let end = block.find("zorder::reorder(").expect(
        "o Arrange deixou de escrever na ÁRVORE. Se ele voltou ao `VecScene::reorder_path`, os \
             quatro botões estão MORTOS outra vez: a projeção reescreve a ordem do vetor da cena a \
             cada frame, e o clique some sem erro nenhum",
    );
    assert!(
        !block[..end].contains("apply_vec_reorder"),
        "a porta antiga (que escreve na CENA) voltou ao caminho do clique"
    );
}

/// **A segunda porta MORREU** — e tem de continuar morta.
///
/// ⚠️ Enquanto ela existisse compilada, o próximo a ligar um botão de z-order teria duas funções
/// com o nome certo e uma delas errada. Uma porta sem chamador não é código morto silencioso: é
/// uma segunda resposta à espera de que alguém a chame.
#[test]
fn the_scene_order_door_is_gone_from_the_shell() {
    let s = src("input_dispatch.rs");
    assert!(
        !s.contains("fn apply_vec_reorder"),
        "o `apply_vec_reorder` (que escreve na ordem do VETOR da cena) voltou a existir"
    );
}

/// **O número que o painel mostra sai da MESMA porta que o botão move.**
///
/// ⚠️ Sem isto o readout pode ficar certo por acidente hoje e virar decoração amanhã — duas
/// travessias da árvore respondem o mesmo até o critério de desempate mudar num lado só.
#[test]
fn the_published_z_index_comes_from_the_same_module_as_the_move() {
    let s = src("render_loop/mod.rs");
    let at = s
        .find("ph2d_panel_vector::state::set_z_index(")
        .expect("a shell deixou de publicar o Z-index: a linha some do painel em silêncio");
    assert!(
        s[at..at + 400].contains("zorder::z_index("),
        "o Z-index publicado nao vem da porta que os botoes movem"
    );
}
