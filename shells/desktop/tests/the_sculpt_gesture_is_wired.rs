//! **Arch-gates do gesto de escultura** (ADR-0150 W2).
//!
//! ⚠️ Por que a fonte e não o comportamento: a cena 3D só existe com um
//! `wgpu::Device` vivo (`AppGfx.sculpt3d`), então nenhum teste headless a
//! constrói e nenhum teste de unidade das crates alcança esta fiação. É o mesmo
//! motivo pelo qual o gizmo de âncora da `line/physics` e a decisão de upload do
//! Painter têm arch-gate: *um gate de unidade é cego à fiação do shell*.
//!
//! Cada asserção abaixo tem uma mutação que a derruba, listada no handoff.

use std::fs;

/// A fonte **sem comentários**.
///
/// ⚠️ Não é higiene: um arch-gate que varre o arquivo cru afirma coisas sobre a
/// PROSA. Este mesmo gate nasceu vermelho porque o doc-comment do `undo_stroke`
/// explica *por que* ele não usa `refresh_region` — a explicação continha a
/// palavra que a asserção proibia. Um gate que dispara em documentação ensina a
/// não documentar.
fn source(name: &str) -> String {
    let raw = fs::read_to_string(format!("{}/src/{name}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("não consegui ler src/{name}: {e}"));
    raw.lines()
        .map(|l| match l.find("//") {
            Some(at) => &l[..at],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// O corpo de `fn <name>` até a chave que o fecha, contando profundidade.
fn function_body(src: &str, name: &str) -> String {
    let at = src
        .find(&format!("fn {name}"))
        .unwrap_or_else(|| panic!("não achei `fn {name}`"));
    let open = src[at..].find('{').expect("corpo") + at;
    let mut depth = 0i32;
    for (i, c) in src[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return src[open..open + i + 1].to_string();
                }
            }
            _ => {}
        }
    }
    panic!("`fn {name}` não fecha");
}

#[test]
fn the_left_button_sculpts_where_it_hits_and_orbits_where_it_misses() {
    let body = function_body(&source("sculpt3d.rs"), "sculpt3d_pointer_down");
    assert!(
        body.contains("stroke.begin("),
        "o pen-down tem de CONGELAR o `pre` — sem isso a lei do traço não começa"
    );
    let hit = body
        .find("if scene.sculpt_at(")
        .expect("o Down decide pelo resultado do pick");
    let sculpt = body[hit..]
        .find("Drag::Sculpt")
        .expect("o ramo que ACERTA abre um traço");
    let orbit = body[hit..]
        .find("Drag::Orbit")
        .expect("o ramo que ERRA cai na órbita");
    // Arrastar no vazio é o gesto mais comum do mundo; se ele não fizesse nada,
    // o artista concluiria que a cena travou. É o que o SculptGL faz.
    assert!(
        sculpt < orbit,
        "os ramos do pick estão trocados: quem acerta é que esculpe"
    );
}

#[test]
fn the_modifiers_are_read_once_at_pen_down_and_hold_for_the_whole_stroke() {
    // Soltar o Shift no meio de uma pincelada faria METADE dela ser outra
    // ferramenta — e a lei do traço congela um `pre` só, então não há como
    // representar isso. Nenhum app de escultura permite, e aqui a garantia é
    // estrutural: quem lê os modificadores é o Down, e o Move não os consulta.
    let src = source("sculpt3d.rs");
    let down = function_body(&src, "sculpt3d_pointer_down");
    assert!(
        down.contains("scene.brush.invert = ctrl"),
        "o Ctrl (o `inverte` de todo app de escultura) tem de ser lido no Down"
    );
    assert!(
        down.contains("Verb::Smooth"),
        "o Shift tem de virar Smooth enquanto segurar — o atalho universal"
    );
    let mv = function_body(&src, "sculpt3d_pointer_move");
    assert!(
        !mv.contains("modifiers") && !mv.contains("shift") && !mv.contains("ctrl"),
        "o Move não pode reler modificador: o traço mudaria de ferramenta no meio"
    );
}

#[test]
fn the_gpu_is_handed_the_refreshed_window_not_the_moved_one() {
    // ⚠️ Este gate existe porque o defeito É este, e um gate de GPU o pegou: o
    // conjunto que muda de NORMAL é maior que o que se MOVE — um vizinho parado
    // ao lado de uma face que girou tem a normal mudada. Subir só os movidos
    // deixa a malha iluminada por normais velhas numa faixa de um anel de
    // largura, bem na BORDA do pincel. O arch-gate impede a regressão de entrar
    // pela porta do shell, onde nenhum gate de crate a veria.
    let body = function_body(&source("sculpt3d.rs"), "sculpt_at");
    assert!(
        body.contains("last_refreshed()"),
        "a janela do upload é `last_refreshed`, o superconjunto"
    );
    assert!(
        !body.contains("last_moved()"),
        "`last_moved` é a janela do UNDO, não a da GPU"
    );
}

#[test]
fn releasing_the_button_turns_the_stroke_into_an_undo_entry() {
    let src = source("sculpt3d.rs");
    let up = function_body(&src, "sculpt3d_pointer_up");
    assert!(
        up.contains("Drag::Sculpt") && up.contains("close_stroke()"),
        "soltar um traço tem de fechá-lo; sem isso o Ctrl+Z não tem o que desfazer"
    );
    let close = function_body(&src, "close_stroke");
    // O undo NÃO é um segundo sistema: a lei do traço já congela o `pre` por
    // vértice tocado, e a lista de tocados É a janela.
    assert!(
        close.contains("stroke.touched()") && close.contains("stroke.base_positions()"),
        "a entrada de undo sai do traço, não de uma segunda captura"
    );
}

#[test]
fn undoing_rebuilds_the_index_instead_of_refitting_the_way_back() {
    // Um refit sobre a VOLTA só cresce as caixas frouxas (ele nunca as encolhe
    // abaixo do que já viu), então cada Ctrl+Z deixaria a árvore um pouco mais
    // gorda e a consulta um pouco mais lenta, para sempre. Um undo é
    // user-paced — é o lugar certo para pagar a resposta exata.
    let body = function_body(&source("sculpt3d.rs"), "undo_stroke");
    assert!(
        body.contains("self.mesh.rebuild()"),
        "desfazer tem de reconstruir o índice"
    );
    assert!(
        !body.contains("refresh_region"),
        "o caminho incremental não serve para a volta"
    );
}

#[test]
fn every_3d_port_is_inert_without_a_scene() {
    // A promessa de removibilidade do `docs/3D/02.3` no nível do FRAME: num run
    // normal `sculpt3d` é `None`, cada porta devolve `false` no primeiro `if`, e
    // o dispatch 2D segue como se o módulo não existisse.
    let src = source("sculpt3d.rs");
    for port in [
        "sculpt3d_pointer_down",
        "sculpt3d_pointer_up",
        "sculpt3d_pointer_move",
        "sculpt3d_wheel",
        "sculpt3d_key",
    ] {
        let body = function_body(&src, port);
        assert!(
            body.contains("sculpt3d_scene_mut()") && body.contains("return false"),
            "`{port}` tem de recusar sem cena armada"
        );
    }
}

#[test]
fn the_shell_takes_the_3d_keys_before_the_widget_store_sees_them() {
    let body = function_body(&source("input_dispatch/keyboard.rs"), "key_input");
    let hook = body
        .find("sculpt3d_key(")
        .expect("as teclas da cena 3D têm de estar costuradas");
    let store = body
        .find("self.handler.on_key(")
        .expect("o store recebe as teclas");
    assert!(
        hook < store,
        "a cena 3D tem de ver a tecla ANTES do store, senão `1..9` viram outra coisa"
    );
}

#[test]
fn the_model_follows_the_hand() {
    // ⚠️ **Proxy deliberado.** O FATO — *arrastar para a direita vira o modelo
    // para a direita* — é definido e medido na crate, em
    // `dragging_right_turns_the_model_right_and_dragging_down_shows_its_top`,
    // que projeta um ponto do modelo NA TELA. Aqui só se afirma que a shell
    // entrega os sinais que aquele fato exige; dirigir a câmera de verdade
    // precisaria de um device.
    //
    // Os dois sinais estavam TROCADOS e o smoke os pegou: `yaw` positivo leva o
    // OLHO para `+X`, e a câmera indo para a direita faz o modelo parecer ir
    // para a esquerda.
    let body = function_body(&source("sculpt3d.rs"), "sculpt3d_pointer_move");
    assert!(
        body.contains(".orbit(-dx * ORBIT_RAD_PER_PX, dy * ORBIT_RAD_PER_PX)"),
        "a órbita da shell tem de negar o `dx` e NÃO o `dy`"
    );
}

#[test]
fn a_click_on_a_panel_is_not_a_click_on_the_model() {
    // Sem esta pergunta a cena 3D engolia TODO botão do app, inclusive os do
    // rail — ela devolvia `true` incondicionalmente e o dispatch 2D nunca via o
    // evento.
    let src = source("sculpt3d.rs");
    let down = function_body(&src, "sculpt3d_pointer_down");
    assert!(
        down.contains("cursor_over_hero_panel("),
        "o Down tem de recusar um clique sobre painel"
    );
    // ⚠️ E o Move/Up NÃO podem fazer a mesma pergunta: um arrasto em curso
    // continua sendo do gesto que o abriu, mesmo que o cursor passeie sobre um
    // painel. É a regra de captura que todo gizmo deste shell segue, e gateá-la
    // aqui impede que alguém "complete" a correção e quebre o traço longo.
    for port in ["sculpt3d_pointer_move", "sculpt3d_pointer_up"] {
        assert!(
            !function_body(&src, port).contains("cursor_over_hero_panel("),
            "`{port}` não pode largar um arrasto em curso ao cruzar um painel"
        );
    }
}

#[test]
fn the_mirror_is_off_until_the_artist_asks_for_it() {
    // Um default que só se descobre por acidente é pior que um default menos
    // ambicioso: com o espelho ligado o artista clicava de um lado e via uma
    // segunda protuberância do outro, sem nada na tela explicando por quê.
    let body = function_body(&source("sculpt3d.rs"), "new(");
    assert!(
        body.contains("symmetry: Symmetry::default()"),
        "a simetria tem de nascer desligada; o `X` a liga"
    );
}
