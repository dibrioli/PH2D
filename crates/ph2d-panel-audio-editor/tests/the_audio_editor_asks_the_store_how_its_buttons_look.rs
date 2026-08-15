//! **Os três pintores próprios deste painel PERGUNTAM ao store.**
//!
//! Este painel não usa `widget::Button` em sítio nenhum — ele tem os seus próprios `button`,
//! `toggle` e `icon_button`, e são eles que 50 chamadas atravessam. Por isso o gate global
//! [`every_button_wears_the_live_hover`] (que varre `Button::new(..).state(..)`) **é cego a ele**,
//! e foi exactamente isso que deixou o painel inteiro inerte sob o rato desde que nasceu, com os
//! ids registados como `InteractiveState::Button` no `populate` o tempo todo: *o store sabia, e
//! ninguém perguntava*.
//!
//! ⚠️ **Nenhum teste de unidade pode ocupar este lugar.** O `VectorScene` deste repo não expõe o
//! que foi desenhado, então a lei de cor é afirmável (`src/paint_tests.rs`) e a **chamada** não é:
//! um `button` que ignorasse o `action_bg` deixaria aqueles seis gates verdes e o produto
//! exactamente como estava.
//!
//! ⚠️ **O controlo positivo é metade do gate.** Um scanner que deixe de encontrar as funções
//! reporta zero ofensores — o mesmo que um produto correcto reporta. Ele exige ver as três.

use std::fs;

/// Os três pintores, com o ficheiro onde vivem e o nome da `fn`.
const PAINTERS: [(&str, &str); 3] = [
    ("src/paint.rs", "pub(crate) fn button("),
    ("src/paint.rs", "pub(crate) fn toggle("),
    ("src/paint_fx.rs", "fn icon_button("),
];

/// O corpo de `fn` que começa em `start`, até à chave que a fecha na coluna zero.
fn body_after(src: &str, start: usize) -> &str {
    let rest = &src[start..];
    match rest.find("\n}\n") {
        Some(end) => &rest[..end],
        None => rest,
    }
}

fn read(rel: &str) -> String {
    fs::read_to_string(format!("{}/{rel}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// **Cada pintor lê o par visual do id que está a pintar.**
///
/// *Mutação que deve sangrar:* trocar `hit_index.visual(id)` (ou `ctx.hit_index.visual(id)`) por
/// um par duro `(ButtonState::Normal, SETTLED)` em qualquer um dos três.
#[test]
fn the_three_painters_read_the_live_visual() {
    for (rel, sig) in PAINTERS {
        let src = read(rel);
        // ⚠️ `expect` e não `if let`: um pintor que se MUDE de ficheiro (ou que mude de nome) tem
        // de falhar alto — nunca varrer um sítio onde a coisa julgada deixou de existir e passar
        // por vácuo. É a cicatriz do gate do `keyboard.rs`.
        let at = src.find(sig).unwrap_or_else(|| {
            panic!("{rel}: `{sig}` desapareceu — o gate ficou a olhar para nada")
        });
        let body = body_after(&src, at);
        assert!(
            body.contains(".visual(id)"),
            "{rel}: `{sig}` pinta sem perguntar ao store — o botao volta a ser inerte sob o rato"
        );
    }
}

/// **A cor quente NÃO é escolhida aqui: ela é pedida à porta única do substrato.**
///
/// O que é deste painel é só o tom de REPOUSO (`Bg3`, que ele já pintava). A transição e os
/// tokens quentes saem de [`ph2d_editor_core::motion::hover_axis`] — a mesma função que o
/// `widget::Button`, o `IconButton` e o `Checkbox` já perguntam. Sem esta metade, o painel teria
/// uma **segunda** resposta a *«que cor tem um botão sob o rato?»*, e as duas divergiriam no dia
/// em que uma delas mudasse.
///
/// *Mutação que deve sangrar:* interpolar as cores à mão dentro do `action_bg`.
#[test]
fn the_hot_colour_comes_from_the_shared_axis() {
    let src = read("src/paint.rs");
    let at = src
        .find("fn action_bg(")
        .expect("`action_bg` desapareceu — o gate ficou a olhar para nada");
    let body = body_after(&src, at);
    assert!(
        body.contains("motion::hover_axis("),
        "a lei do hover foi re-escrita aqui em vez de pedida ao substrato"
    );
}

/// **E o painel continua sem uma segunda lista de widgets:** o `ClippedHits` que os pintores já
/// recebiam é quem carrega o store, e ele é construído **UMA** vez, do empréstimo conjunto.
///
/// ⚠️ **A metade da CONTAGEM é fraca de propósito, e o número mede-o:** um segundo
/// `ClippedHits::new` no mesmo escopo é **rejeitado pelo compilador** (`E0499`, o `&mut HitIndex`
/// já está emprestado) — o tipo já proíbe a cópia-e-cola, então esta linha não defende contra ela.
/// O que ela defende é uma **reestruturação** que separe os dois empréstimos e volte a construir
/// dois recortes, cada um a decidir por conta o que é visível. *Uma asserção cuja mutação óbvia
/// nem compila é fraca; dizê-lo é melhor que fingir que ela é forte.*
///
/// A metade do `store_and_hit_index_mut` é que é load-bearing: sem ela o corpo perde o store e
/// todo botão volta a ser inerte, com os seis gates da lei de cor **verdes**.
#[test]
fn the_body_builds_its_hit_handle_exactly_once() {
    let src = read("src/paint.rs");
    assert_eq!(
        src.matches("ClippedHits::new(").count(),
        1,
        "o corpo passou a ter mais de um handle de widgets"
    );

    // ⚠️ **O par é seguido pelos NOMES, não pela presença da chamada.** Uma versão anterior deste
    // gate só exigia que `store_and_hit_index_mut()` aparecesse no ficheiro, e a mutação que
    // entregava ao corpo um `WidgetStore::default()` — o painel inerte de volta, exactamente — ficou
    // **VERDE em toda a suíte**. É a terceira vez nesta wave que o oráculo declarava a resposta em
    // vez de seguir o dado.
    let at = src
        .find("= ctx.host.store_and_hit_index_mut();")
        .expect("o emprestimo conjunto desapareceu — o gate ficou a olhar para nada");
    let line_start = src[..at].rfind("let (").expect("a desestruturacao do par");
    let names: Vec<&str> = src[line_start + "let (".len()..at]
        .trim()
        .trim_end_matches(')')
        .split(',')
        .map(str::trim)
        .collect();
    assert_eq!(names.len(), 2, "o par deixou de ser um par: {names:?}");
    let call = format!("ClippedHits::new({}, {},", names[0], names[1]);
    assert!(
        src.contains(&call),
        "o handle nao recebe o par do HOST (esperava `{call}`) — um store qualquer aqui deixa \
         todo botao do painel inerte, com os gates da lei de cor verdes"
    );
}
