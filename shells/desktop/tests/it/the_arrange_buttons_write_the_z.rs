//! **Arch-gate: os botões Arrange escrevem o Z — e SÓ o Z.**
//!
//! # Porque é um arch-gate, e não um gate de unidade
//!
//! Os gates de `vec_entities::zorder::arrange_tests` provam a PORTA — que `reorder` move a forma na
//! pilha e deixa a Hierarquia intacta. Todos passariam com a **fiação arrancada**: o clique a cair
//! no `VecScene::reorder_path` (que mexe na ordem do VETOR da cena, e essa é reescrita a cada frame
//! pela projeção da árvore — ADR-0110) deixaria os quatro botões como estavam antes desta wave:
//! **acendem, mexem, e o frame seguinte desfaz**.
//!
//! Essa metade vive dentro do laço de frame, que exige janela — nenhum teste de unidade a alcança.
//! É a mesma classe do `a_placed_instance_lands_a_screen_step_from_its_main`.
//!
//! ⚠️ **O nome deste arquivo já foi `..._write_the_tree`.** A lei mudou (Enio, 2026-08-04: *"o
//! objeto não deve ser movido na hierarquia, apenas o Z muda"*), e um gate cujo nome afirma o
//! oposto do que ele julga é pior que gate nenhum.

use std::fs;

fn src(name: &str) -> String {
    fs::read_to_string(format!("{}/src/{name}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// O corpo de uma função — do `{` de abertura ao `}` que o fecha, **contando chaves**.
///
/// ⚠️ Ancorar na função SEGUINTE é o proxy que expira, e esta suíte já pagou por isso: a janela
/// anterior ia do `fn reorder(` até `\nfn sibling_move(`, e quando aquele vizinho mudou de nome o
/// gate perdeu o limite. Contar chaves não tem vizinho de que depender.
fn body_of(s: &str, sig: &str) -> String {
    let at = s
        .find(sig)
        .unwrap_or_else(|| panic!("`{sig}` mudou de forma — reancore este gate"));
    let open = at + s[at..].find('{').expect("uma fn tem corpo");
    let mut depth = 0i32;
    for (i, c) in s[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return s[open..=open + i].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("chaves desequilibradas a partir de `{sig}`");
}

/// **O clique cai na porta do Z.**
#[test]
fn the_reorder_click_lands_on_the_z_door() {
    let s = src("render_loop/mod.rs");
    let at = s
        .find("if let Some(order) = pending_vec_reorder")
        .expect("o sítio que honra o Arrange mudou de forma — reancore este gate");
    // A janela acaba na PRÓPRIA chamada que ela julga: ancorar num vizinho é o proxy que expira.
    let block = &s[at..];
    let end = block.find("zorder::reorder(").expect(
        "o Arrange deixou de passar pela porta do Z. Se ele voltou ao `VecScene::reorder_path`, os \
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

/// **O número que o painel MOSTRA e o que ele ESCREVE são a mesma porta.**
///
/// ⚠️ São duas travessias da mesma fiação e é fácil ligá-las a lugares diferentes — o campo lê o
/// autorado e o commit escreve noutro sítio. Aí ele mostra um número e edita outro, e nada falha.
#[test]
fn the_published_z_index_comes_from_the_same_module_as_the_write() {
    let s = src("render_loop/mod.rs");
    let at = s
        .find("ph2d_panel_vector::state::set_z_index(")
        .expect("a shell deixou de publicar o Z-index: o campo some do painel em silêncio");
    assert!(
        s[at..at + 400].contains("zorder::authored_z("),
        "o Z publicado nao vem da porta que o campo escreve"
    );
    assert!(
        s.contains("zorder::set_authored_z("),
        "o commit do campo Z nao chega a' porta de escrita — o artista digita e nada acontece"
    );
}

/// **O `reorder` escreve o Z e NÃO ESCREVE A ÁRVORE** (Enio, 2026-08-04).
///
/// ⚠️ O gate de comportamento (`every_verb_writes_the_z_and_leaves_the_hierarchy_alone`) mede a
/// consequência: a lista da Hierarquia sai intacta. Este mede a CAUSA no lugar exacto onde a
/// tentação volta — o `reorder` a chamar uma escrita de árvore como "plano B", que foi
/// literalmente o desenho anterior. As duas metades são defeitos diferentes: a primeira pega uma
/// escrita que MOVE a lista, esta pega a porta ser aberta de todo.
///
/// ⚠️ **O `restack` fica de fora de propósito** e continua a escrever `RootOrder`: ele não move
/// uma forma que o artista pôs, ele COLOCA um grupo recém-criado (o Blend). Por isso a janela é o
/// corpo do `reorder`, e não o arquivo.
#[test]
fn the_reorder_writes_the_z_and_never_the_tree() {
    let s = src("vec_zorder.rs");
    let body = body_of(&s, "pub(crate) fn reorder(");
    assert!(
        body.contains("set_authored_z("),
        "o `reorder` deixou de escrever o Z: os quatro botoes voltaram a nao fazer nada"
    );
    for door in ["RootOrder(", "reinsert_children_in_order", "Children>("] {
        assert!(
            !body.contains(door),
            "o `reorder` voltou a escrever na ARVORE (`{door}`) — a lei diz que o objeto nao se \
             move na hierarquia, so' o Z muda"
        );
    }
}

/// **Os ajudantes que escreviam a árvore SUMIRAM do módulo.**
///
/// ⚠️ Pelo mesmo motivo do `apply_vec_reorder` acima: enquanto compilassem, o próximo a mexer nos
/// botões teria duas funções plausíveis e uma delas proibida pela lei.
#[test]
fn the_sibling_writing_helpers_are_gone() {
    let s = src("vec_zorder.rs");
    for gone in ["fn sibling_move", "fn write_sibling_order", "fn siblings("] {
        assert!(
            !s.contains(gone),
            "`{gone}` voltou: a metade que re-arruma a Hierarquia do artista esta' de pe' outra vez"
        );
    }
}
